#!/usr/bin/env pwsh
# Urgent Fix Deployment Script
# Fixes: 1) Health check endpoint, 2) Gossipsub mesh configuration

Write-Host "🚀 dchat Urgent Fix Deployment" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build Docker image locally
Write-Host "📦 Step 1: Building Docker image..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful" -ForegroundColor Green
Write-Host ""

# Step 2: Build Docker image
Write-Host "🐳 Step 2: Building Docker image..." -ForegroundColor Yellow
docker build -t dchat:latest .
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Docker build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Docker image built" -ForegroundColor Green
Write-Host ""

# Step 3: Save Docker image
Write-Host "💾 Step 3: Saving Docker image..." -ForegroundColor Yellow
docker save dchat:latest | gzip > dchat-latest.tar.gz
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Docker save failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Image saved to dchat-latest.tar.gz" -ForegroundColor Green
Write-Host ""

# Step 4: Upload to server
Write-Host "⬆️  Step 4: Uploading to server..." -ForegroundColor Yellow
scp -i "C:\Users\USER\Downloads\anacreon.pem" `
    dchat-latest.tar.gz `
    docker-compose-testnet.yml `
    azureuser@4.221.211.71:~/chain/dchat/
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Upload failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Files uploaded" -ForegroundColor Green
Write-Host ""

# Step 5: Deploy on server
Write-Host "🚀 Step 5: Deploying on server..." -ForegroundColor Yellow
ssh -i "C:\Users\USER\Downloads\anacreon.pem" azureuser@4.221.211.71 @"
cd ~/chain/dchat
echo '📥 Loading Docker image...'
gunzip -c dchat-latest.tar.gz | sudo docker load
echo '🛑 Stopping existing containers...'
sudo docker-compose -f docker-compose-testnet.yml down
echo '🧹 Cleaning up old containers...'
sudo docker system prune -f
echo '🚀 Starting updated stack...'
sudo docker-compose -f docker-compose-testnet.yml up -d
echo '✅ Deployment complete!'
echo ''
echo '📊 Container status:'
sudo docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}' | grep dchat
"@

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Deployment failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Urgent fixes deployed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "🔍 Fixes applied:" -ForegroundColor Cyan
Write-Host "  1. Health server now binds to 0.0.0.0:8080 (accessible from Docker)" -ForegroundColor White
Write-Host "  2. All health checks updated to port 8080" -ForegroundColor White
Write-Host "  3. Gossipsub flood_publish enabled for user nodes" -ForegroundColor White
Write-Host ""
Write-Host "📝 To verify:" -ForegroundColor Yellow
Write-Host "  ssh -i 'C:\Users\USER\Downloads\anacreon.pem' azureuser@4.221.211.71 'sudo docker ps'" -ForegroundColor Gray
Write-Host ""
Write-Host "🔬 To test user messaging:" -ForegroundColor Yellow
Write-Host "  ssh -i 'C:\Users\USER\Downloads\anacreon.pem' azureuser@4.221.211.71 'sudo docker logs dchat-user1-test --tail 20'" -ForegroundColor Gray
Write-Host ""

# Cleanup
Write-Host "🧹 Cleaning up local tar.gz..." -ForegroundColor Yellow
Remove-Item dchat-latest.tar.gz -ErrorAction SilentlyContinue
Write-Host "✅ Cleanup complete" -ForegroundColor Green
