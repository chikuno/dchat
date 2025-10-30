# dchat Testnet: Message Propagation Test Script
# Purpose: Send messages from user1 to user2 and verify propagation through validators/relays

param(
    [string]$Action = "help",
    [string]$Message = "Hello from testnet!",
    [string]$FromUser = "user1",
    [string]$ToUser = "user2"
)

# Configuration
$DOCKER_COMPOSE_FILE = "docker-compose-testnet.yml"
$LOG_DIR = "testnet-logs"
$TIMESTAMP = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"

function New-TestnetDirectory {
    <#
    .SYNOPSIS
    Create necessary directories for testnet
    #>
    if (-not (Test-Path $LOG_DIR)) {
        New-Item -ItemType Directory -Path $LOG_DIR | Out-Null
        Write-Host "✅ Created log directory: $LOG_DIR" -ForegroundColor Green
    }
    
    if (-not (Test-Path "validator_keys")) {
        New-Item -ItemType Directory -Path "validator_keys" | Out-Null
        Write-Host "✅ Created validator_keys directory" -ForegroundColor Green
    }
}

function Initialize-ValidatorKeys {
    <#
    .SYNOPSIS
    Generate validator keys (placeholder - would use real key generation in production)
    #>
    Write-Host "🔐 Initializing validator keys..." -ForegroundColor Cyan
    
    $validators = @("validator1", "validator2", "validator3", "validator4")
    
    foreach ($validator in $validators) {
        $keyFile = "validator_keys/$validator.key"
        if (-not (Test-Path $keyFile)) {
            # Create placeholder key file
            @"
[validator_key]
node_id = "$validator"
public_key = "placeholder_public_key_$validator"
private_key = "placeholder_private_key_$validator"
stake = 1000
"@ | Out-File -FilePath $keyFile -Encoding UTF8
            Write-Host "✅ Created key for $validator" -ForegroundColor Green
        }
    }
}

function Start-Testnet {
    <#
    .SYNOPSIS
    Start the entire testnet (4 validators + 7 relays + 3 users)
    #>
    Write-Host "`n🚀 Starting dchat Testnet (4V + 7R + 3U)..." -ForegroundColor Cyan
    Write-Host "=" * 80
    
    # Initialize
    New-TestnetDirectory
    Initialize-ValidatorKeys
    
    # Build image
    Write-Host "`n📦 Building Docker image..." -ForegroundColor Yellow
    docker build -t dchat:latest . 2>&1 | Tee-Object -FilePath "$LOG_DIR/build_$TIMESTAMP.log"
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Docker build failed!" -ForegroundColor Red
        return
    }
    
    Write-Host "✅ Docker image built successfully" -ForegroundColor Green
    
    # Start services
    Write-Host "`n🔄 Starting services via docker-compose..." -ForegroundColor Yellow
    docker-compose -f $DOCKER_COMPOSE_FILE up -d 2>&1 | Tee-Object -FilePath "$LOG_DIR/startup_$TIMESTAMP.log"
    
    Write-Host "`n⏳ Waiting for services to be healthy (30 seconds)..." -ForegroundColor Yellow
    Start-Sleep -Seconds 30
    
    # Check health
    Write-Host "`n🏥 Checking node health..." -ForegroundColor Cyan
    Get-NodeHealth
}

function Stop-Testnet {
    <#
    .SYNOPSIS
    Stop the testnet
    #>
    Write-Host "`n🛑 Stopping dchat Testnet..." -ForegroundColor Yellow
    docker-compose -f $DOCKER_COMPOSE_FILE down -v
    Write-Host "✅ Testnet stopped" -ForegroundColor Green
}

function Get-NodeHealth {
    <#
    .SYNOPSIS
    Check health of all nodes
    #>
    Write-Host "`n📊 Node Health Status:" -ForegroundColor Cyan
    Write-Host "=" * 80
    
    $nodes = @(
        @{Name="Validator1"; Port=7071},
        @{Name="Validator2"; Port=7073},
        @{Name="Validator3"; Port=7075},
        @{Name="Validator4"; Port=7077},
        @{Name="Relay1"; Port=7081},
        @{Name="Relay2"; Port=7083},
        @{Name="Relay3"; Port=7085},
        @{Name="Relay4"; Port=7087},
        @{Name="Relay5"; Port=7089},
        @{Name="Relay6"; Port=7091},
        @{Name="Relay7"; Port=7093},
        @{Name="User1"; Port=7111},
        @{Name="User2"; Port=7113},
        @{Name="User3"; Port=7115}
    )
    
    foreach ($node in $nodes) {
        $response = $null
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:$($node.Port)/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                Write-Host "✅ $($node.Name) (port $($node.Port)): HEALTHY" -ForegroundColor Green
            } else {
                Write-Host "⚠️  $($node.Name) (port $($node.Port)): UNHEALTHY" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "❌ $($node.Name) (port $($node.Port)): UNREACHABLE" -ForegroundColor Red
        }
    }
}

function Get-NetworkStatus {
    <#
    .SYNOPSIS
    Get network topology and connectivity
    #>
    Write-Host "`n📡 Network Status:" -ForegroundColor Cyan
    Write-Host "=" * 80
    
    Write-Host "`n📋 Validators:" -ForegroundColor Yellow
    docker-compose -f $DOCKER_COMPOSE_FILE exec -T validator1 curl -s http://validator1:7071/status 2>/dev/null | ConvertFrom-Json | ForEach-Object {
        Write-Host "  Validator1: Height=$($_.height), Peers=$($_.peers)"
    }
    
    Write-Host "`n📋 Relays Connected:" -ForegroundColor Yellow
    docker-compose -f $DOCKER_COMPOSE_FILE logs relay1 2>/dev/null | Select-String "peer|connected" | Select-Object -Last 5 | ForEach-Object {
        Write-Host "  $_"
    }
    
    Write-Host "`n📋 Users Registered:" -ForegroundColor Yellow
    docker-compose -f $DOCKER_COMPOSE_FILE ps | Select-String "user" | ForEach-Object {
        Write-Host "  $_"
    }
}

function Send-TestMessage {
    <#
    .SYNOPSIS
    Send a test message from one user to another
    #>
    param(
        [string]$From = "user1",
        [string]$To = "user2",
        [string]$Msg = "Test message"
    )
    
    Write-Host "`n💬 Sending Message Test..." -ForegroundColor Cyan
    Write-Host "=" * 80
    Write-Host "From: $From → To: $To"
    Write-Host "Message: $Msg"
    Write-Host ""
    
    # Determine ports
    $fromPort = if ($From -eq "user1") { 7111 } elseif ($From -eq "user2") { 7113 } else { 7115 }
    $toPort = if ($To -eq "user1") { 7111 } elseif ($To -eq "user2") { 7113 } else { 7115 }
    
    Write-Host "📤 Step 1: $From sends message via RPC (port $fromPort)..." -ForegroundColor Yellow
    try {
        $response = Invoke-WebRequest `
            -Uri "http://localhost:$fromPort/send-message" `
            -Method POST `
            -ContentType "application/json" `
            -Body (ConvertTo-Json @{
                to = $To
                content = $Msg
                encrypted = $true
            }) `
            -TimeoutSec 5
        
        $msgId = ($response.Content | ConvertFrom-Json).message_id
        Write-Host "✅ Message sent. ID: $msgId" -ForegroundColor Green
    } catch {
        Write-Host "❌ Send failed: $_" -ForegroundColor Red
        return
    }
    
    Write-Host "`n⏳ Step 2: Message propagates through relays (wait 2 seconds)..." -ForegroundColor Yellow
    Start-Sleep -Seconds 2
    
    Write-Host "`n📥 Step 3: $To checks for messages (port $toPort)..." -ForegroundColor Yellow
    try {
        $response = Invoke-WebRequest `
            -Uri "http://localhost:$toPort/messages?from=$From" `
            -TimeoutSec 5
        
        $messages = $response.Content | ConvertFrom-Json
        if ($messages.Count -gt 0) {
            Write-Host "✅ Message received!" -ForegroundColor Green
            Write-Host "   Content: $($messages[-1].content)" -ForegroundColor Green
            Write-Host "   Timestamp: $($messages[-1].timestamp)" -ForegroundColor Green
            Write-Host "   Hops: $($messages[-1].hops)" -ForegroundColor Green
        } else {
            Write-Host "⚠️  No messages received yet..." -ForegroundColor Yellow
        }
    } catch {
        Write-Host "❌ Retrieve failed: $_" -ForegroundColor Red
    }
}

function Get-Logs {
    <#
    .SYNOPSIS
    Collect logs from all nodes
    #>
    Write-Host "`n📋 Collecting logs..." -ForegroundColor Cyan
    
    docker-compose -f $DOCKER_COMPOSE_FILE logs --tail=100 > "$LOG_DIR/all_nodes_$TIMESTAMP.log"
    Write-Host "✅ Logs saved to: $LOG_DIR/all_nodes_$TIMESTAMP.log" -ForegroundColor Green
    
    # Also save individual node logs
    $nodes = @("validator1", "validator2", "validator3", "validator4", "relay1", "relay2", "relay3", "relay4", "relay5", "relay6", "relay7", "user1", "user2", "user3")
    foreach ($node in $nodes) {
        docker-compose -f $DOCKER_COMPOSE_FILE logs --tail=50 "dchat-$node" > "$LOG_DIR/${node}_$TIMESTAMP.log" 2>&1
    }
    
    Write-Host "✅ Individual node logs saved to: $LOG_DIR/" -ForegroundColor Green
}

function Show-Help {
    <#
    .SYNOPSIS
    Display help information
    #>
    Write-Host @"
dchat Testnet Manager
=====================

USAGE:
    .\testnet-message-propagation.ps1 -Action <action> [options]

ACTIONS:
    start           - Start the testnet (4 validators + 7 relays + 3 users)
    stop            - Stop the testnet
    health          - Check health of all nodes
    status          - Get network status and topology
    send-message    - Send a test message and verify propagation
    logs            - Collect logs from all nodes
    help            - Show this help message

EXAMPLES:
    # Start testnet
    .\testnet-message-propagation.ps1 -Action start
    
    # Check health
    .\testnet-message-propagation.ps1 -Action health
    
    # Send test message from user1 to user2
    .\testnet-message-propagation.ps1 -Action send-message `
        -FromUser user1 -ToUser user2 -Message "Hello World"
    
    # Collect logs
    .\testnet-message-propagation.ps1 -Action logs

MONITORING:
    - Prometheus:   http://localhost:9090
    - Grafana:      http://localhost:3000 (admin/admin)
    - Jaeger:       http://localhost:16686

NETWORK ARCHITECTURE:
    Validators: validator1, validator2, validator3, validator4 (consensus layer)
    Relays:     relay1-relay7 (message delivery)
    Users:      user1, user2, user3 (end-points)

MESSAGE PROPAGATION FLOW:
    user1 → relay (user1 connected) → validator (records) → relay (user2 connected) → user2

"@
}

# Main execution
switch ($Action) {
    "start" { Start-Testnet }
    "stop" { Stop-Testnet }
    "health" { Get-NodeHealth }
    "status" { Get-NetworkStatus }
    "send-message" { Send-TestMessage -From $FromUser -To $ToUser -Msg $Message }
    "logs" { Get-Logs }
    default { Show-Help }
}

Write-Host "`n" -ForegroundColor Cyan
