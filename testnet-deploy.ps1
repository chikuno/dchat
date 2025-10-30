# dchat Testnet Deployment Script
# Deploys a complete testnet with relay nodes, monitoring, and health checks

param(
    [switch]$Clean,
    [switch]$MonitoringOnly,
    [int]$RelayCount = 3
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 dchat Testnet Deployment" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan
Write-Host ""

# Clean previous deployment
if ($Clean) {
    Write-Host "🧹 Cleaning previous deployment..." -ForegroundColor Yellow
    if (Test-Path "dchat_testnet_data") {
        Remove-Item -Recurse -Force "dchat_testnet_data"
    }
    Write-Host "✓ Clean complete" -ForegroundColor Green
    Write-Host ""
}

# Check binary exists
if (-not (Test-Path "target\release\dchat.exe")) {
    Write-Host "❌ Binary not found. Building..." -ForegroundColor Red
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Build failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host "✓ Binary ready: target\release\dchat.exe" -ForegroundColor Green
Write-Host ""

# Start monitoring stack with Docker Compose
if ($MonitoringOnly -or (-not $MonitoringOnly)) {
    Write-Host "📊 Starting monitoring stack..." -ForegroundColor Cyan
    docker-compose up -d prometheus grafana jaeger
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Monitoring stack started" -ForegroundColor Green
        Write-Host "  - Prometheus: http://localhost:9093" -ForegroundColor Gray
        Write-Host "  - Grafana: http://localhost:3000 (admin/admin)" -ForegroundColor Gray
        Write-Host "  - Jaeger: http://localhost:16686" -ForegroundColor Gray
    } else {
        Write-Host "⚠️  Docker Compose not available, skipping monitoring" -ForegroundColor Yellow
    }
    Write-Host ""
}

if ($MonitoringOnly) {
    Write-Host "✓ Monitoring-only mode complete" -ForegroundColor Green
    exit 0
}

# Deploy relay nodes
Write-Host "🔗 Deploying $RelayCount relay nodes..." -ForegroundColor Cyan

$jobs = @()
$exePath = Resolve-Path "target\release\dchat.exe"
$configPath = Resolve-Path "testnet-config.toml"

for ($i = 1; $i -le $RelayCount; $i++) {
    $port = 7070 + (($i - 1) * 2)
    $rpcPort = $port + 1
    $metricsPort = 9090 + ($i - 1)
    $dataDir = ".\dchat_testnet_data\relay$i"
    
    # Create data directory if it doesn't exist
    if (-not (Test-Path $dataDir)) {
        New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
    }
    
    if ($i -eq 1) {
        # First node is bootstrap node
        Write-Host "  [Relay $i] Bootstrap node on port $port (metrics: $metricsPort)" -ForegroundColor Yellow
        
        $job = Start-Job -Name "relay$i" -ScriptBlock {
            param($exePath, $configPath, $port, $metricsPort, $dataDir)
            $env:RUST_LOG = "info,dchat=debug"
            $env:DCHAT_P2P_PORT = $port
            $env:DCHAT_METRICS_PORT = $metricsPort
            $env:DCHAT_DATA_DIR = $dataDir
            
            # Run relay without bootstrap peers (first node)
            & $exePath --config $configPath --log-level debug relay --listen "0.0.0.0:$port"
        } -ArgumentList $exePath, $configPath, $port, $metricsPort, $dataDir
        
        # Wait for bootstrap node to start and extract peer ID
        Write-Host "  ⏳ Waiting for bootstrap node to initialize..." -ForegroundColor Gray
        Start-Sleep -Seconds 8
        
        # Try to extract peer ID from logs
        $bootstrapLogs = Receive-Job -Name "relay$i" -Keep 2>&1
        $peerIdMatch = $bootstrapLogs | Select-String -Pattern "Local peer ID: (12D3[a-zA-Z0-9]+)"
        
        if ($peerIdMatch) {
            $bootstrapPeerId = $peerIdMatch.Matches[0].Groups[1].Value
            Write-Host "  ✓ Bootstrap node started (Peer ID: $bootstrapPeerId)" -ForegroundColor Green
        } else {
            Write-Host "  ⚠️  Bootstrap node started (Peer ID detection failed, using default)" -ForegroundColor Yellow
            $bootstrapPeerId = "12D3KooWBootstrapNode"
        }
        
        # Check if job is still running
        if ((Get-Job -Name "relay$i").State -ne "Running") {
            Write-Host "  ❌ Bootstrap node failed to start. Check logs:" -ForegroundColor Red
            Receive-Job -Name "relay$i" 2>&1 | Select-Object -Last 10 | ForEach-Object { Write-Host "     $_" -ForegroundColor Red }
            throw "Bootstrap node failed to start"
        }
    } else {
        # Subsequent nodes bootstrap from first node
        Write-Host "  [Relay $i] Node on port $port (metrics: $metricsPort)" -ForegroundColor Cyan
        
        $bootstrap = "/ip4/127.0.0.1/tcp/7070"
        
        $job = Start-Job -Name "relay$i" -ScriptBlock {
            param($exePath, $configPath, $port, $metricsPort, $dataDir, $bootstrap)
            $env:RUST_LOG = "info,dchat=debug"
            $env:DCHAT_P2P_PORT = $port
            $env:DCHAT_METRICS_PORT = $metricsPort
            $env:DCHAT_DATA_DIR = $dataDir
            $env:DCHAT_BOOTSTRAP_PEERS = $bootstrap
            
            # Run relay with bootstrap peer
            & $exePath --config $configPath --log-level debug relay --listen "0.0.0.0:$port" --bootstrap $bootstrap
        } -ArgumentList $exePath, $configPath, $port, $metricsPort, $dataDir, $bootstrap
        
        Start-Sleep -Seconds 3
        
        # Check if relay started successfully
        if ((Get-Job -Name "relay$i").State -eq "Running") {
            Write-Host "  ✓ Relay $i started and connecting to bootstrap" -ForegroundColor Green
        } else {
            Write-Host "  ❌ Relay $i failed to start. Check logs:" -ForegroundColor Red
            Receive-Job -Name "relay$i" 2>&1 | Select-Object -Last 10 | ForEach-Object { Write-Host "     $_" -ForegroundColor Red }
        }
    }
    
    $jobs += $job
}

# Give network time to establish connections
Write-Host ""
Write-Host "⏳ Allowing network to establish connections..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# Check final status of all nodes
Write-Host ""
Write-Host "🔍 Checking node status..." -ForegroundColor Cyan
$allRunning = $true
foreach ($job in $jobs) {
    $status = (Get-Job -Id $job.Id).State
    $relayNum = $job.Name -replace "relay", ""
    if ($status -eq "Running") {
        Write-Host "  ✓ Relay ${relayNum}: ${status}" -ForegroundColor Green
    } else {
        Write-Host "  ❌ Relay ${relayNum}: ${status}" -ForegroundColor Red
        $allRunning = $false
    }
}

if (-not $allRunning) {
    Write-Host ""
    Write-Host "⚠️  Some relays failed to start. Showing recent logs:" -ForegroundColor Yellow
    foreach ($job in $jobs) {
        if ((Get-Job -Id $job.Id).State -ne "Running") {
            $relayNum = $job.Name -replace "relay", ""
            Write-Host ""
            Write-Host "Relay ${relayNum} logs:" -ForegroundColor Red
            Receive-Job -Id $job.Id 2>&1 | Select-Object -Last 15 | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
        }
    }
}

Write-Host ""

if ($allRunning) {
    Write-Host "✅ Testnet Deployment Complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 Deployment Summary:" -ForegroundColor Cyan
    Write-Host "  Relay Nodes: $RelayCount active" -ForegroundColor White
    Write-Host "  Bootstrap Peer ID: $bootstrapPeerId" -ForegroundColor White
    Write-Host "  P2P Ports: 7070-$($7070 + ($RelayCount - 1) * 2)" -ForegroundColor White
    Write-Host "  RPC Ports: 7071-$($7071 + ($RelayCount - 1) * 2)" -ForegroundColor White
    Write-Host "  Metrics Ports: 9090-$(9090 + $RelayCount - 1)" -ForegroundColor White
    Write-Host ""
    Write-Host "🔍 Monitoring:" -ForegroundColor Cyan
    Write-Host "  Health Checks: http://localhost:8080/health" -ForegroundColor White
    Write-Host "  Metrics (Relay 1): http://localhost:9090/metrics" -ForegroundColor White
    Write-Host "  Metrics (Relay 2): http://localhost:9091/metrics" -ForegroundColor White
    Write-Host "  Metrics (Relay 3): http://localhost:9092/metrics" -ForegroundColor White
    Write-Host ""
    Write-Host "📊 View logs:" -ForegroundColor Cyan
    Write-Host "  All relays: Get-Job | Receive-Job -Keep" -ForegroundColor Gray
    for ($i = 1; $i -le $RelayCount; $i++) {
        Write-Host "  Relay ${i}: Receive-Job -Name relay${i} -Keep | Select-Object -Last 20" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "🔗 Connect to testnet:" -ForegroundColor Cyan
    Write-Host "  .\target\release\dchat.exe user --bootstrap `"/ip4/127.0.0.1/tcp/7070`"" -ForegroundColor Gray
    Write-Host ""
    Write-Host "🛑 Stop testnet:" -ForegroundColor Cyan
    Write-Host "  Get-Job | Stop-Job; Get-Job | Remove-Job" -ForegroundColor Gray
    Write-Host ""
    Write-Host "📄 Documentation:" -ForegroundColor Cyan
    Write-Host "  Testnet Status: cat TESTNET_STATUS.md" -ForegroundColor Gray
    Write-Host "  Operations Guide: cat OPERATIONAL_GUIDE.md" -ForegroundColor Gray
    Write-Host ""
    Write-Host "✅ Testnet is now running! Press Ctrl+C to stop monitoring." -ForegroundColor Green
    Write-Host ""
    
    # Auto-monitor logs (optional)
    if (-not $MonitoringOnly) {
        Write-Host "📡 Live log monitoring (Ctrl+C to exit):" -ForegroundColor Yellow
        Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
        
        try {
            # Monitor all relay logs in real-time
            while ($true) {
                Start-Sleep -Seconds 5
                $newLogs = Get-Job | Receive-Job -Keep 2>&1 | Select-Object -Last 10
                if ($newLogs) {
                    $newLogs | ForEach-Object { Write-Host $_ }
                }
            }
        } catch {
            Write-Host ""
            Write-Host "⏹️  Monitoring stopped" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "❌ Testnet Deployment Failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Some relay nodes failed to start. Please check the logs above." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "🔧 Troubleshooting:" -ForegroundColor Cyan
    Write-Host "  1. Ensure target\release\dchat.exe exists (run: cargo build --release)" -ForegroundColor Gray
    Write-Host "  2. Check if ports are already in use (7070-7074, 9090-9092)" -ForegroundColor Gray
    Write-Host "  3. View detailed logs: Receive-Job -Name relay1 2>&1" -ForegroundColor Gray
    Write-Host "  4. Clean and retry: .\testnet-deploy.ps1 -Clean" -ForegroundColor Gray
    Write-Host ""
    
    # Clean up failed jobs
    Write-Host "🧹 Cleaning up failed jobs..." -ForegroundColor Yellow
    Get-Job | Stop-Job
    Get-Job | Remove-Job
    
    exit 1
}

# Give network time to establish connections
Write-Host ""
Write-Host "⏳ Allowing network to establish connections..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# Check final status of all nodes
Write-Host ""
Write-Host "🔍 Checking node status..." -ForegroundColor Cyan
$allRunning = $true
foreach ($job in $jobs) {
    $status = (Get-Job -Id $job.Id).State
    $relayNum = $job.Name -replace "relay", ""
    if ($status -eq "Running") {
        Write-Host "  ✓ Relay ${relayNum}: ${status}" -ForegroundColor Green
    } else {
        Write-Host "  ❌ Relay ${relayNum}: ${status}" -ForegroundColor Red
        $allRunning = $false
    }
}

if (-not $allRunning) {
    Write-Host ""
    Write-Host "⚠️  Some relays failed to start. Showing recent logs:" -ForegroundColor Yellow
    foreach ($job in $jobs) {
        if ((Get-Job -Id $job.Id).State -ne "Running") {
            $relayNum = $job.Name -replace "relay", ""
            Write-Host ""
            Write-Host "Relay ${relayNum} logs:" -ForegroundColor Red
            Receive-Job -Id $job.Id 2>&1 | Select-Object -Last 15 | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
        }
    }
}

Write-Host ""
Write-Host "✅ Testnet Deployment Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Deployment Summary:" -ForegroundColor Cyan
Write-Host "  Relay Nodes: $RelayCount" -ForegroundColor White
Write-Host "  P2P Ports: 7070-$($7070 + ($RelayCount - 1) * 2)" -ForegroundColor White
Write-Host "  RPC Ports: 7071-$($7071 + ($RelayCount - 1) * 2)" -ForegroundColor White
Write-Host "  Metrics Ports: 9090-$(9090 + $RelayCount - 1)" -ForegroundColor White
Write-Host ""
Write-Host "🔍 Monitoring URLs:" -ForegroundColor Cyan
Write-Host "  Prometheus: http://localhost:9093" -ForegroundColor White
Write-Host "  Grafana: http://localhost:3000 (admin/admin)" -ForegroundColor White
Write-Host "  Jaeger: http://localhost:16686" -ForegroundColor White
Write-Host ""
Write-Host "🏥 Health Checks:" -ForegroundColor Cyan
Write-Host "  Relay 1: http://localhost:7071/health" -ForegroundColor White
Write-Host "  Relay 2: http://localhost:7073/health" -ForegroundColor White
Write-Host "  Relay 3: http://localhost:7075/health" -ForegroundColor White
Write-Host ""
Write-Host "📊 View logs:" -ForegroundColor Cyan
for ($i = 0; $i -lt $jobs.Count; $i++) {
    Write-Host "  Relay $($i + 1): Receive-Job -Id $($jobs[$i].Id) -Keep" -ForegroundColor Gray
}
Write-Host ""
Write-Host "🛑 Stop testnet:" -ForegroundColor Cyan
Write-Host "  Get-Job | Stop-Job; Get-Job | Remove-Job" -ForegroundColor Gray
Write-Host "  docker-compose down" -ForegroundColor Gray
Write-Host ""
Write-Host "✅ Testnet is now running!" -ForegroundColor Green
