# Docker Cleanup Script - Remove everything except dchat image
# WARNING: This will destroy all containers, volumes, and non-dchat images!

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Docker Cleanup Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This will remove:" -ForegroundColor Yellow
Write-Host "  ✗ All containers (running and stopped)" -ForegroundColor Red
Write-Host "  ✗ All volumes" -ForegroundColor Red
Write-Host "  ✗ All networks (except default)" -ForegroundColor Red
Write-Host "  ✗ All images (except dchat:*)" -ForegroundColor Red
Write-Host ""
Write-Host "This will preserve:" -ForegroundColor Yellow
Write-Host "  ✓ dchat:latest and dchat:* images" -ForegroundColor Green
Write-Host ""

$confirm = Read-Host "Are you sure? (yes/no)"

if ($confirm -ne "yes") {
    Write-Host "Aborted." -ForegroundColor Yellow
    exit 0
}

Write-Host ""
Write-Host "Starting cleanup..." -ForegroundColor Cyan
Write-Host ""

# Step 1: Stop all running containers
Write-Host "1. Stopping all running containers..." -ForegroundColor Yellow
$runningContainers = docker ps -q
if ($runningContainers) {
    docker stop $runningContainers
    Write-Host "   ✓ Stopped all containers" -ForegroundColor Green
} else {
    Write-Host "   ℹ No running containers" -ForegroundColor Gray
}

# Step 2: Remove all containers
Write-Host ""
Write-Host "2. Removing all containers..." -ForegroundColor Yellow
$allContainers = docker ps -aq
if ($allContainers) {
    docker rm -f $allContainers
    Write-Host "   ✓ Removed all containers" -ForegroundColor Green
} else {
    Write-Host "   ℹ No containers to remove" -ForegroundColor Gray
}

# Step 3: Remove all volumes
Write-Host ""
Write-Host "3. Removing all volumes..." -ForegroundColor Yellow
$allVolumes = docker volume ls -q
if ($allVolumes) {
    docker volume rm $allVolumes 2>$null
    Write-Host "   ✓ Removed volumes" -ForegroundColor Green
} else {
    Write-Host "   ℹ No volumes to remove" -ForegroundColor Gray
}

# Step 4: Remove all networks (except default ones)
Write-Host ""
Write-Host "4. Removing custom networks..." -ForegroundColor Yellow
$customNetworks = docker network ls --filter type=custom -q
if ($customNetworks) {
    docker network rm $customNetworks 2>$null
    Write-Host "   ✓ Removed custom networks" -ForegroundColor Green
} else {
    Write-Host "   ℹ No custom networks to remove" -ForegroundColor Gray
}

# Step 5: Remove all images except dchat
Write-Host ""
Write-Host "5. Removing images (preserving dchat:*)..." -ForegroundColor Yellow
$allImages = docker images --format "{{.Repository}}:{{.Tag}}"
$imagesToRemove = $allImages | Where-Object { $_ -notmatch "^dchat:" }
if ($imagesToRemove) {
    $imagesToRemove | ForEach-Object { docker rmi -f $_ 2>$null }
    Write-Host "   ✓ Removed non-dchat images" -ForegroundColor Green
} else {
    Write-Host "   ℹ No images to remove" -ForegroundColor Gray
}

# Step 6: Prune system
Write-Host ""
Write-Host "6. Pruning Docker system..." -ForegroundColor Yellow
docker system prune -f --volumes
Write-Host "   ✓ System pruned" -ForegroundColor Green

# Step 7: Show what's left
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Cleanup Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Remaining Docker resources:" -ForegroundColor Yellow
Write-Host ""
Write-Host "Images:" -ForegroundColor Cyan
docker images
Write-Host ""
Write-Host "Containers:" -ForegroundColor Cyan
docker ps -a
Write-Host ""
Write-Host "Volumes:" -ForegroundColor Cyan
docker volume ls
Write-Host ""
Write-Host "Networks:" -ForegroundColor Cyan
docker network ls
Write-Host ""
Write-Host "✅ Cleanup finished successfully!" -ForegroundColor Green
