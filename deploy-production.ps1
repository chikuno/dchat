# dchat Production Deployment Script
# Deploys testnet to remote server: rpc.webnetcore.top:8080
# Run this script on the remote server after SSH connection

param(
    [string]$ServerUrl = "rpc.webnetcore.top",
    [int]$RpcPort = 8080,
    [int]$Validators = 4,
    [string]$ValidatorStake = "10000",
    [string]$RelayStake = "1000"
)

# ============================================================================
# CONFIGURATION
# ============================================================================
$DeployDir = "/opt/dchat"
$ValidatorKeysDir = "$DeployDir/validator_keys"
$MonitoringDir = "$DeployDir/monitoring"
$LogsDir = "$DeployDir/logs"
$BackupDir = "$DeployDir/backups"

# Colors for output
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $Color = switch ($Level) {
        "ERROR" { $Red }
        "WARN" { $Yellow }
        "SUCCESS" { $Green }
        default { $Blue }
    }
    Write-Host "$Color[$Timestamp][$Level]$Reset $Message"
}

function Check-Prerequisites {
    Write-Log "Checking prerequisites..."
    
    $tools = @("docker", "docker-compose", "git", "curl")
    foreach ($tool in $tools) {
        try {
            $version = & $tool --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Log "✓ $tool installed: $version" "SUCCESS"
            }
        }
        catch {
            Write-Log "✗ $tool not found. Please install it." "ERROR"
            exit 1
        }
    }
}

function Create-Directories {
    Write-Log "Creating deployment directories..."
    
    $dirs = @($DeployDir, $ValidatorKeysDir, $MonitoringDir, $LogsDir, $BackupDir)
    foreach ($dir in $dirs) {
        if (!(Test-Path -Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Log "✓ Created $dir" "SUCCESS"
        }
    }
}

function Clone-Repository {
    Write-Log "Cloning dchat repository..."
    
    if (Test-Path -Path "$DeployDir/.git") {
        Write-Log "Repository already cloned. Pulling latest changes..."
        Push-Location $DeployDir
        & git pull
        Pop-Location
    }
    else {
        & git clone https://github.com/chikuno/dchat.git $DeployDir
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Failed to clone repository" "ERROR"
        exit 1
    }
    Write-Log "✓ Repository ready" "SUCCESS"
}

function Generate-ValidatorKeys {
    Write-Log "Generating $Validators validator keys..."
    
    Push-Location $DeployDir
    
    # Build the key generation tool
    & cargo build --release --bin key-generator 2>&1 | Out-Null
    
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Failed to build key generator" "ERROR"
        Pop-Location
        exit 1
    }
    
    # Generate keys
    for ($i = 1; $i -le $Validators; $i++) {
        $KeyFile = "$ValidatorKeysDir/validator$i.key"
        if (!(Test-Path -Path $KeyFile)) {
            Write-Log "Generating key for validator$i..."
            & ./target/release/key-generator -o $KeyFile -t ed25519 2>&1 | Out-Null
            
            if ($LASTEXITCODE -ne 0) {
                Write-Log "Failed to generate key for validator$i" "ERROR"
                Pop-Location
                exit 1
            }
            
            # Secure permissions
            & chmod 400 $KeyFile
            Write-Log "✓ Generated $KeyFile" "SUCCESS"
        }
        else {
            Write-Log "Key already exists: $KeyFile" "WARN"
        }
    }
    
    Pop-Location
}

function Create-MonitoringConfig {
    Write-Log "Creating monitoring configuration..."
    
    # Create Prometheus config
    $PrometheusConfig = @"
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'dchat-testnet'

scrape_configs:
  # Validators
  - job_name: 'validators'
    static_configs:
      - targets:
          - 'validator1:9090'
          - 'validator2:9090'
          - 'validator3:9090'
          - 'validator4:9090'
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Relays
  - job_name: 'relays'
    static_configs:
      - targets:
          - 'relay1:9100'
          - 'relay2:9100'
          - 'relay3:9100'
          - 'relay4:9100'
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Prometheus self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

alerting:
  alertmanagers:
    - static_configs:
        - targets: []
"@

    $PrometheusConfig | Out-File -FilePath "$MonitoringDir/prometheus.yml" -Force
    Write-Log "✓ Created prometheus.yml" "SUCCESS"

    # Create Grafana datasource config
    $GrafanaDir = "$MonitoringDir/grafana/datasources"
    New-Item -ItemType Directory -Path $GrafanaDir -Force | Out-Null

    $DatasourceConfig = @"
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
"@

    $DatasourceConfig | Out-File -FilePath "$GrafanaDir/prometheus.yml" -Force
    Write-Log "✓ Created Grafana datasource config" "SUCCESS"
}

function Build-DockerImage {
    Write-Log "Building Docker image..."
    
    Push-Location $DeployDir
    
    & docker build -t dchat:latest -f Dockerfile . 2>&1 | Tee-Object -FilePath "$LogsDir/docker-build.log"
    
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Docker build failed. Check $LogsDir/docker-build.log" "ERROR"
        Pop-Location
        exit 1
    }
    
    Write-Log "✓ Docker image built successfully" "SUCCESS"
    Pop-Location
}

function Start-Testnet {
    Write-Log "Starting dchat testnet..."
    
    Push-Location $DeployDir
    
    & docker-compose -f docker-compose-production.yml up -d 2>&1 | Tee-Object -FilePath "$LogsDir/compose-up.log"
    
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Docker Compose startup failed. Check $LogsDir/compose-up.log" "ERROR"
        Pop-Location
        exit 1
    }
    
    Write-Log "✓ Services started" "SUCCESS"
    Pop-Location
    
    # Wait for services to be ready
    Write-Log "Waiting for services to be healthy..."
    Start-Sleep -Seconds 30
    
    Check-ServiceHealth
}

function Check-ServiceHealth {
    Write-Log "Checking service health..."
    
    $services = @(
        @{ Name = "validator1"; Port = 7071 },
        @{ Name = "validator2"; Port = 7073 },
        @{ Name = "validator3"; Port = 7075 },
        @{ Name = "validator4"; Port = 7077 },
        @{ Name = "relay1"; Port = 7081 },
        @{ Name = "prometheus"; Port = 9090 },
        @{ Name = "grafana"; Port = 3000 }
    )
    
    foreach ($service in $services) {
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:$($service.Port)/health" -TimeoutSec 5 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                Write-Log "✓ $($service.Name) is healthy" "SUCCESS"
            }
        }
        catch {
            Write-Log "✗ $($service.Name) health check failed" "WARN"
        }
    }
}

function Configure-NginxReverseProxy {
    Write-Log "Creating Nginx reverse proxy configuration..."
    
    $NginxConfig = @"
upstream dchat_validators {
    least_conn;
    server validator1:7071 weight=1 max_fails=2 fail_timeout=30s;
    server validator2:7071 weight=1 max_fails=2 fail_timeout=30s;
    server validator3:7071 weight=1 max_fails=2 fail_timeout=30s;
    server validator4:7071 weight=1 max_fails=2 fail_timeout=30s;
    keepalive 64;
}

upstream dchat_relays {
    least_conn;
    server relay1:7081 weight=2 max_fails=2 fail_timeout=30s;
    server relay2:7081 weight=1 max_fails=2 fail_timeout=30s;
    server relay3:7081 weight=1 max_fails=2 fail_timeout=30s;
    server relay4:7081 weight=1 max_fails=2 fail_timeout=30s;
    keepalive 64;
}

# Rate limiting
limit_req_zone `$binary_remote_addr zone=general:10m rate=100r/s;
limit_req_zone `$binary_remote_addr zone=websocket:10m rate=50r/s;
limit_req_zone `$binary_remote_addr zone=rpc:10m rate=1000r/s;

server {
    listen 8080;
    server_name $ServerUrl;
    
    # Logging
    access_log /var/log/nginx/dchat_access.log combined buffer=32k flush=5s;
    error_log /var/log/nginx/dchat_error.log warn;
    
    # Timeouts
    proxy_connect_timeout 60s;
    proxy_send_timeout 60s;
    proxy_read_timeout 60s;
    send_timeout 60s;
    
    # Buffering
    proxy_buffering on;
    proxy_buffer_size 4k;
    proxy_buffers 8 4k;
    
    # Health check endpoint
    location /health {
        access_log off;
        proxy_pass http://dchat_validators;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header Host `$host;
    }
    
    # RPC endpoints (high rate limit)
    location ~ ^/(rpc|api) {
        limit_req zone=rpc burst=100 nodelay;
        proxy_pass http://dchat_validators;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header Host `$host;
        proxy_set_header X-Forwarded-Proto `$scheme;
        
        # Cache GET requests
        proxy_cache_methods GET HEAD;
        proxy_cache_valid 200 10s;
        add_header X-Cache-Status `$upstream_cache_status;
    }
    
    # WebSocket endpoints
    location ~ ^/(ws|websocket) {
        limit_req zone=websocket burst=20 nodelay;
        proxy_pass http://dchat_validators;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header Host `$host;
    }
    
    # Relay endpoints
    location /relay {
        limit_req zone=general burst=50 nodelay;
        proxy_pass http://dchat_relays;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
    }
    
    # Default to validator
    location / {
        limit_req zone=general burst=50 nodelay;
        proxy_pass http://dchat_validators;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header Host `$host;
    }
}

# Redirect HTTP to HTTPS (when SSL is configured)
server {
    listen 80;
    server_name $ServerUrl;
    return 301 https://`$server_name:8080`$request_uri;
}
"@

    $NginxPath = "$DeployDir/nginx/nginx.conf"
    New-Item -ItemType Directory -Path (Split-Path -Parent $NginxPath) -Force | Out-Null
    $NginxConfig | Out-File -FilePath $NginxPath -Force
    Write-Log "✓ Created nginx.conf" "SUCCESS"
    Write-Log "Deploy nginx with: docker run -d -p 8080:8080 -v $NginxPath:/etc/nginx/conf.d/default.conf nginx" "INFO"
}

function Create-BackupScript {
    Write-Log "Creating backup and recovery scripts..."
    
    $BackupScript = @"
#!/bin/bash
# Daily backup script

BACKUP_DIR="/opt/dchat/backups"
TIMESTAMP=\$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="\$BACKUP_DIR/dchat-backup-\$TIMESTAMP.tar.gz"

echo "Creating backup: \$BACKUP_FILE"
docker-compose -f /opt/dchat/docker-compose-production.yml exec -T validator1 tar czf /var/lib/dchat/backup.tar.gz /var/lib/dchat/data/
docker cp dchat-validator1:/var/lib/dchat/backup.tar.gz \$BACKUP_FILE

# Keep only last 7 backups
find \$BACKUP_DIR -name "dchat-backup-*.tar.gz" -mtime +7 -delete

echo "Backup completed: \$BACKUP_FILE"
du -h \$BACKUP_FILE
"@

    $RecoveryScript = @"
#!/bin/bash
# Recovery script

BACKUP_FILE="\$1"
if [ -z "\$BACKUP_FILE" ]; then
    echo "Usage: ./recover.sh <backup_file>"
    exit 1
fi

echo "Recovering from: \$BACKUP_FILE"

# Stop services
docker-compose -f /opt/dchat/docker-compose-production.yml stop

# Restore data
docker cp \$BACKUP_FILE dchat-validator1:/tmp/backup.tar.gz
docker-compose -f /opt/dchat/docker-compose-production.yml exec -T validator1 tar xzf /tmp/backup.tar.gz -C /

# Restart services
docker-compose -f /opt/dchat/docker-compose-production.yml up -d

echo "Recovery completed"
"@

    $BackupScript | Out-File -FilePath "$BackupDir/backup.sh" -Force
    $RecoveryScript | Out-File -FilePath "$BackupDir/recover.sh" -Force
    
    Write-Log "✓ Created backup scripts" "SUCCESS"
}

function Display-ConnectionInfo {
    Write-Log "Deployment complete! Connection information:" "SUCCESS"
    
    Write-Host ""
    Write-Host "$Green=== DCHAT TESTNET ENDPOINTS ===$Reset"
    Write-Host "RPC Endpoint:        http://$ServerUrl:$RpcPort"
    Write-Host "Validator1 RPC:      http://localhost:7071/rpc"
    Write-Host "Validator2 RPC:      http://localhost:7073/rpc"
    Write-Host "Validator3 RPC:      http://localhost:7075/rpc"
    Write-Host "Validator4 RPC:      http://localhost:7077/rpc"
    Write-Host ""
    Write-Host "$Green=== MONITORING ===$Reset"
    Write-Host "Prometheus:          http://localhost:9090"
    Write-Host "Grafana:             http://localhost:3000 (admin/admin)"
    Write-Host "Jaeger:              http://localhost:16686"
    Write-Host ""
    Write-Host "$Green=== LOGS ===$Reset"
    Write-Host "All logs stored in:  $LogsDir"
    Write-Host "View Docker logs:    docker-compose -f docker-compose-production.yml logs -f"
    Write-Host ""
    Write-Host "$Green=== VERIFICATION ===$Reset"
    Write-Host "Check health:        curl http://localhost:7071/health"
    Write-Host "View services:       docker ps | grep dchat"
    Write-Host "View metrics:        curl http://localhost:9090/metrics"
    Write-Host ""
}

# ============================================================================
# MAIN DEPLOYMENT FLOW
# ============================================================================
function main {
    Write-Log "Starting dchat production deployment..." "INFO"
    Write-Log "Server: $ServerUrl:$RpcPort" "INFO"
    Write-Log "Validators: $Validators" "INFO"
    Write-Log ""
    
    Check-Prerequisites
    Create-Directories
    Clone-Repository
    Generate-ValidatorKeys
    Create-MonitoringConfig
    Build-DockerImage
    Start-Testnet
    Configure-NginxReverseProxy
    Create-BackupScript
    Display-ConnectionInfo
    
    Write-Log "Deployment finished successfully!" "SUCCESS"
}

# Run main deployment
main
