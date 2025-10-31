#!/usr/bin/env pwsh
# Check dchat testnet container logs on remote server

param(
    [string]$ServerIP = "4.222.211.71",
    [string]$ServerUser = "root",
    [int]$TailLines = 50
)

Write-Host "=== Checking dchat testnet container logs on $ServerIP ===" -ForegroundColor Cyan
Write-Host ""

# Function to check container logs
function Check-ContainerLogs {
    param(
        [string]$ContainerName,
        [string]$Type
    )
    
    Write-Host "--- $Type: $ContainerName ---" -ForegroundColor Yellow
    ssh "$ServerUser@$ServerIP" "docker logs $ContainerName --tail=$TailLines 2>&1"
    Write-Host ""
}

# Check if we can connect
Write-Host "Testing SSH connection..." -ForegroundColor Green
$testConnection = ssh -o ConnectTimeout=5 "$ServerUser@$ServerIP" "echo 'Connected'"
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Cannot connect to $ServerIP" -ForegroundColor Red
    Write-Host "Please check:"
    Write-Host "  1. Server is running"
    Write-Host "  2. SSH key is configured"
    Write-Host "  3. Firewall allows SSH (port 22)"
    exit 1
}
Write-Host "Connection successful!" -ForegroundColor Green
Write-Host ""

# Check container status first
Write-Host "=== Container Status ===" -ForegroundColor Cyan
ssh "$ServerUser@$ServerIP" "docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}' | grep dchat"
Write-Host ""

# Check validators
Write-Host "=== VALIDATOR LOGS ===" -ForegroundColor Magenta
Check-ContainerLogs "dchat-validator1" "Validator"
Check-ContainerLogs "dchat-validator2" "Validator"
Check-ContainerLogs "dchat-validator3" "Validator"
Check-ContainerLogs "dchat-validator4" "Validator"

# Check relays
Write-Host "=== RELAY LOGS ===" -ForegroundColor Magenta
Check-ContainerLogs "dchat-relay1" "Relay"
Check-ContainerLogs "dchat-relay2" "Relay"
Check-ContainerLogs "dchat-relay3" "Relay"

# Check users
Write-Host "=== USER NODE LOGS ===" -ForegroundColor Magenta
Check-ContainerLogs "dchat-user1" "User"
Check-ContainerLogs "dchat-user2" "User"
Check-ContainerLogs "dchat-user3" "User"

# Check monitoring
Write-Host "=== MONITORING LOGS ===" -ForegroundColor Magenta
Check-ContainerLogs "dchat-prometheus" "Prometheus"
Check-ContainerLogs "dchat-grafana" "Grafana"
Check-ContainerLogs "dchat-jaeger" "Jaeger"

Write-Host "=== Log check complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "To follow logs in real-time, use:" -ForegroundColor Green
Write-Host "  ssh $ServerUser@$ServerIP 'docker logs -f dchat-user1'"
Write-Host ""
Write-Host "To check health endpoints:" -ForegroundColor Green
Write-Host "  ssh $ServerUser@$ServerIP 'docker exec dchat-user1 curl -s http://localhost:7111/health | jq'"
