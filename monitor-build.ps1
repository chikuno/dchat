# Docker Build Progress Monitor
# Use this to track the dchat Docker build

Write-Host "`n╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DCHAT DOCKER BUILD MONITOR - BUILDING RELAY IMAGES      ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

$startTime = Get-Date
$maxWaitTime = 600  # 10 minutes

Write-Host "⏱️  Build started at: $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow
Write-Host "⏰ Timeout set to: $($maxWaitTime / 60) minutes`n" -ForegroundColor Yellow

Write-Host "Building components:" -ForegroundColor Magenta
Write-Host "  🔨 Compiling Rust binary (relay-node)" -ForegroundColor Gray
Write-Host "  📦 Building relay-1, relay-2, relay-3 images" -ForegroundColor Gray
Write-Host "  🗄️  Starting PostgreSQL, Prometheus, Grafana, Jaeger" -ForegroundColor Gray
Write-Host ""

$elapsedSeconds = 0
$checkInterval = 10  # Check every 10 seconds

while ($elapsedSeconds -lt $maxWaitTime) {
    $elapsed = New-TimeSpan -Seconds $elapsedSeconds
    Write-Host -NoNewline "`r⏳ Elapsed: $($elapsed.Minutes)m $($elapsed.Seconds)s  "
    
    # Check if containers are running
    $runningContainers = docker ps --format "{{.Names}}" 2>&1 | Measure-Object | Select-Object -ExpandProperty Count
    $builtImages = docker images | Select-String -Pattern "dchat|relay" | Measure-Object | Select-Object -ExpandProperty Count
    
    if ($runningContainers -gt 0) {
        Write-Host "`n✅ Containers are starting! Build successful!`n" -ForegroundColor Green
        break
    }
    
    if ($builtImages -gt 0) {
        Write-Host "`n📦 Images built! Containers starting...`n" -ForegroundColor Yellow
        Start-Sleep -Seconds 5
        continue
    }
    
    Start-Sleep -Seconds $checkInterval
    $elapsedSeconds += $checkInterval
}

Write-Host ""

# Final check
$containers = docker-compose ps 2>&1

if ($containers -match "Up") {
    Write-Host "✅ BUILD COMPLETE - All services are running!`n" -ForegroundColor Green
    Write-Host "Services Status:" -ForegroundColor Cyan
    docker-compose ps
    
    Write-Host "`n📊 Access your services:" -ForegroundColor Magenta
    Write-Host "  • Grafana Dashboard:     http://localhost:3000 (admin/admin)" -ForegroundColor Gray
    Write-Host "  • Prometheus Metrics:    http://localhost:9090" -ForegroundColor Gray
    Write-Host "  • Jaeger Tracing:        http://localhost:16686" -ForegroundColor Gray
    Write-Host "  • PostgreSQL Database:   localhost:5432" -ForegroundColor Gray
    Write-Host "  • Relay Nodes P2P:       localhost:7070-7072" -ForegroundColor Gray
    
    Write-Host "`n🔍 Useful Commands:" -ForegroundColor Magenta
    Write-Host "  docker-compose logs -f relay-node-1    # View relay-1 logs" -ForegroundColor Gray
    Write-Host "  docker-compose logs -f                 # View all logs" -ForegroundColor Gray
    Write-Host "  docker ps                              # Show running containers" -ForegroundColor Gray
    Write-Host "  docker-compose down                    # Stop all services" -ForegroundColor Gray
    
} elseif ($elapsedSeconds -ge $maxWaitTime) {
    Write-Host "`n⚠️  Build timed out after $($maxWaitTime / 60) minutes`n" -ForegroundColor Yellow
    Write-Host "This usually means:" -ForegroundColor Yellow
    Write-Host "  1. Rust compilation is taking longer (normal for first build)" -ForegroundColor Gray
    Write-Host "  2. Network issues downloading dependencies" -ForegroundColor Gray
    Write-Host "  3. Docker daemon needs restart" -ForegroundColor Gray
    
    Write-Host "`n🔧 Troubleshooting:" -ForegroundColor Magenta
    Write-Host "  • Check logs:  docker-compose logs" -ForegroundColor Gray
    Write-Host "  • Restart:     docker-compose down && docker-compose up -d" -ForegroundColor Gray
    Write-Host "  • Or rebuild:  docker-compose build --no-cache" -ForegroundColor Gray
} else {
    Write-Host "`n⏳ Build still in progress..." -ForegroundColor Yellow
    Write-Host "`nCurrent status:" -ForegroundColor Cyan
    docker-compose ps
}

Write-Host ""
