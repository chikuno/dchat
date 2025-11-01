#!/usr/bin/env pwsh
# Test user node messaging after urgent fix

param(
    [string]$Server = "4.221.211.71",
    [string]$Key = "C:\Users\USER\Downloads\anacreon.pem"
)

Write-Host "🧪 Testing dchat User Node Messaging" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

$sshCmd = "ssh -i `"$Key`" azureuser@$Server"

# Test 1: Check health endpoints
Write-Host "🏥 Test 1: Checking health endpoints..." -ForegroundColor Yellow
Write-Host "Validators:" -ForegroundColor White
Invoke-Expression "$sshCmd 'for i in 1 2 3 4; do echo -n `"validator`$i: `"; curl -s http://localhost:909`$((`$i-1))/health 2>&1 | head -1; done'"
Write-Host ""
Write-Host "Relays:" -ForegroundColor White
Invoke-Expression "$sshCmd 'for i in 1 2 3 4 5 6 7; do echo -n `"relay`$i: `"; curl -s http://localhost:910`$((`$i-1))/health 2>&1 | head -1; done'"
Write-Host ""

# Test 2: Check container health status
Write-Host "🩺 Test 2: Docker health status..." -ForegroundColor Yellow
Invoke-Expression "$sshCmd 'sudo docker ps --format `"table {{.Names}}\t{{.Status}}`" | grep dchat | head -15'"
Write-Host ""

# Test 3: Check user node logs
Write-Host "📋 Test 3: User node logs (last 20 lines)..." -ForegroundColor Yellow
Write-Host "--- User1 ---" -ForegroundColor Cyan
Invoke-Expression "$sshCmd 'sudo docker logs dchat-user1-test --tail 20 2>&1'"
Write-Host ""
Write-Host "--- User2 ---" -ForegroundColor Cyan
Invoke-Expression "$sshCmd 'sudo docker logs dchat-user2-test --tail 20 2>&1'"
Write-Host ""

# Test 4: Check relay message stats
Write-Host "📊 Test 4: Relay message statistics..." -ForegroundColor Yellow
Invoke-Expression "$sshCmd 'sudo docker logs dchat-relay1 2>&1 | grep bandwidth | tail -1'"
Write-Host ""

# Test 5: Check validator block production
Write-Host "⛓️  Test 5: Validator block production..." -ForegroundColor Yellow
Invoke-Expression "$sshCmd 'sudo docker logs dchat-validator1 --tail 5 2>&1 | grep block'"
Write-Host ""

Write-Host "✅ Tests complete!" -ForegroundColor Green
Write-Host ""
Write-Host "🔍 Key indicators to check:" -ForegroundColor Cyan
Write-Host "  ✓ Health endpoints should return status" -ForegroundColor White
Write-Host "  ✓ Container health should be 'healthy' (not 'unhealthy')" -ForegroundColor White
Write-Host "  ✓ User nodes should NOT show 'InsufficientPeers' error" -ForegroundColor White
Write-Host "  ✓ Validators should be producing blocks" -ForegroundColor White
Write-Host ""
