#!/usr/bin/env pwsh
# Login to dchat as a user and perform interactive tasks

param(
    [string]$ServerIP = "4.222.211.71",
    [string]$ServerUser = "root",
    [ValidateSet("user1", "user2", "user3")]
    [string]$UserNode = "user1"
)

Write-Host "=== Logging into dchat testnet as $UserNode ===" -ForegroundColor Cyan
Write-Host ""

# Map user node to port
$ports = @{
    "user1" = @{ P2P = 7110; RPC = 7111; Metrics = 9110 }
    "user2" = @{ P2P = 7112; RPC = 7113; Metrics = 9111 }
    "user3" = @{ P2P = 7114; RPC = 7115; Metrics = 9112 }
}

$nodePort = $ports[$UserNode]

Write-Host "User Node: $UserNode" -ForegroundColor Green
Write-Host "P2P Port: $($nodePort.P2P)" -ForegroundColor Green
Write-Host "RPC Port: $($nodePort.RPC)" -ForegroundColor Green
Write-Host "Metrics Port: $($nodePort.Metrics)" -ForegroundColor Green
Write-Host ""

# Check if container is running
Write-Host "Checking container status..." -ForegroundColor Yellow
$containerStatus = ssh "$ServerUser@$ServerIP" "docker ps --filter name=dchat-$UserNode --format '{{.Status}}'"
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrEmpty($containerStatus)) {
    Write-Host "ERROR: Container dchat-$UserNode is not running!" -ForegroundColor Red
    Write-Host "Start it with: ssh $ServerUser@$ServerIP 'cd /opt/dchat && docker-compose -f docker-compose-testnet.yml up -d $UserNode'"
    exit 1
}
Write-Host "Container Status: $containerStatus" -ForegroundColor Green
Write-Host ""

# Check health endpoint
Write-Host "Checking health endpoint..." -ForegroundColor Yellow
$healthCheck = ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)/health"
if ($LASTEXITCODE -eq 0) {
    Write-Host "Health: $healthCheck" -ForegroundColor Green
} else {
    Write-Host "WARNING: Health endpoint not responding" -ForegroundColor Red
}
Write-Host ""

# Interactive menu
function Show-Menu {
    Write-Host "=== User Actions Menu ===" -ForegroundColor Cyan
    Write-Host "1. Check node status and info"
    Write-Host "2. View recent messages"
    Write-Host "3. Send a message"
    Write-Host "4. Create a channel"
    Write-Host "5. Join a channel"
    Write-Host "6. List channels"
    Write-Host "7. View node logs (tail)"
    Write-Host "8. Follow logs (real-time)"
    Write-Host "9. Check wallet balance"
    Write-Host "10. Get node peer info"
    Write-Host "11. Execute custom RPC command"
    Write-Host "12. Open interactive shell in container"
    Write-Host "0. Exit"
    Write-Host ""
}

function Execute-Action {
    param([string]$Choice)
    
    switch ($Choice) {
        "1" {
            Write-Host "Fetching node info..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)/status | jq"
        }
        "2" {
            Write-Host "Fetching recent messages..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)/messages?limit=10 | jq"
        }
        "3" {
            $recipient = Read-Host "Enter recipient address"
            $message = Read-Host "Enter message"
            Write-Host "Sending message..." -ForegroundColor Yellow
            $payload = @{
                recipient = $recipient
                message = $message
            } | ConvertTo-Json
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s -X POST http://localhost:$($nodePort.RPC)/send -H 'Content-Type: application/json' -d '$payload' | jq"
        }
        "4" {
            $channelName = Read-Host "Enter channel name"
            $description = Read-Host "Enter channel description"
            Write-Host "Creating channel..." -ForegroundColor Yellow
            $payload = @{
                name = $channelName
                description = $description
            } | ConvertTo-Json
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s -X POST http://localhost:$($nodePort.RPC)/channels/create -H 'Content-Type: application/json' -d '$payload' | jq"
        }
        "5" {
            $channelId = Read-Host "Enter channel ID to join"
            Write-Host "Joining channel..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s -X POST http://localhost:$($nodePort.RPC)/channels/$channelId/join | jq"
        }
        "6" {
            Write-Host "Fetching channel list..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)/channels | jq"
        }
        "7" {
            Write-Host "Recent logs (last 50 lines):" -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker logs dchat-$UserNode --tail=50"
        }
        "8" {
            Write-Host "Following logs (press Ctrl+C to stop)..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker logs -f dchat-$UserNode"
        }
        "9" {
            Write-Host "Checking wallet balance..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)/wallet/balance | jq"
        }
        "10" {
            Write-Host "Fetching peer info..." -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)/peers | jq"
        }
        "11" {
            $endpoint = Read-Host "Enter RPC endpoint (e.g., /status, /health)"
            Write-Host "Executing: GET $endpoint" -ForegroundColor Yellow
            ssh "$ServerUser@$ServerIP" "docker exec dchat-$UserNode curl -s http://localhost:$($nodePort.RPC)$endpoint | jq"
        }
        "12" {
            Write-Host "Opening interactive shell..." -ForegroundColor Yellow
            Write-Host "Type 'exit' to return to this menu" -ForegroundColor Cyan
            ssh -t "$ServerUser@$ServerIP" "docker exec -it dchat-$UserNode /bin/bash"
        }
        "0" {
            Write-Host "Exiting..." -ForegroundColor Green
            return $false
        }
        default {
            Write-Host "Invalid choice. Please try again." -ForegroundColor Red
        }
    }
    return $true
}

# Main loop
$continue = $true
while ($continue) {
    Show-Menu
    $choice = Read-Host "Enter your choice"
    Write-Host ""
    $continue = Execute-Action $choice
    Write-Host ""
    if ($continue) {
        Write-Host "Press Enter to continue..."
        Read-Host
    }
}

Write-Host "=== Session ended ===" -ForegroundColor Cyan
