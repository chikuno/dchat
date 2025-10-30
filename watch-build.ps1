#!/usr/bin/env pwsh
# Build progress monitor for dchat

$startTime = Get-Date
$timeout = 1800  # 30 minutes

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DCHAT DOCKER BUILD MONITOR - RELAY IMAGES               ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "⏱️  Build started at: $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow
Write-Host "⏰ Timeout set to: $($timeout/60) minutes" -ForegroundColor Yellow
Write-Host ""

while ($true) {
    $elapsed = (Get-Date) - $startTime
    $seconds = $elapsed.TotalSeconds
    
    # Check container status
    $containers = docker-compose ps --format "table {{.Service}} {{.Status}}" 2>$null
    $upCount = ($containers | Select-String "Up" | Measure-Object).Count
    
    # Check for any running builds
    $buildStatus = docker ps --filter "ancestor=test-dchat" --format "table {{.Status}}" 2>$null
    
    if ($upCount -ge 7) {
        Write-Host "✅ Build complete! All 7 containers running." -ForegroundColor Green
        Write-Host ""
        Write-Host "Container Status:" -ForegroundColor Cyan
        docker-compose ps
        Write-Host ""
        Write-Host "Time taken: $([Math]::Floor($seconds))s ($([Math]::Floor($seconds/60))m $([Math]::Floor($seconds%60))s)" -ForegroundColor Green
        break
    }
    
    if ($seconds -gt $timeout) {
        Write-Host "⚠️  Build timed out after $([Math]::Floor($timeout/60)) minutes" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Current container status:" -ForegroundColor Cyan
        docker-compose ps
        Write-Host ""
        Write-Host "To troubleshoot:" -ForegroundColor Yellow
        Write-Host "  docker-compose logs relay1" -ForegroundColor Gray
        Write-Host "  docker-compose logs relay2" -ForegroundColor Gray
        Write-Host "  docker-compose logs relay3" -ForegroundColor Gray
        Write-Host ""
        break
    }
    
    Write-Host "⏳ Elapsed: $([Math]::Floor($seconds))s | Status: Building Rust binaries..." -NoNewline
    Write-Host " " -NoNewline
    
    Start-Sleep -Seconds 5
    
    # Update status line
    [Console]::CursorLeft = 0
    [Console]::CursorTop = [Console]::CursorTop - 1
    
    # Show spinner
    $spinner = @('|', '/', '-', '\')
    $index = [Math]::Floor(($seconds * 2) % 4)
    Write-Host "⏳ Elapsed: $([Math]::Floor($seconds))s | Status: Building... $($spinner[$index])" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Expected services:" -ForegroundColor Cyan
Write-Host "  • relay1: Bootstrap node (port 7070)" -ForegroundColor Gray
Write-Host "  • relay2: Peer node (port 7072)" -ForegroundColor Gray
Write-Host "  • relay3: Peer node (port 7074)" -ForegroundColor Gray
Write-Host "  • postgres: Database (port 5432)" -ForegroundColor Gray
Write-Host "  • prometheus: Metrics (port 9090)" -ForegroundColor Gray
Write-Host "  • grafana: Dashboard (port 3000)" -ForegroundColor Gray
Write-Host "  • jaeger: Tracing (port 16686)" -ForegroundColor Gray
Write-Host ""
Write-Host "Access points:" -ForegroundColor Cyan
Write-Host "  • Grafana: http://localhost:3000" -ForegroundColor Gray
Write-Host "  • Prometheus: http://localhost:9090" -ForegroundColor Gray
Write-Host "  • Jaeger: http://localhost:16686" -ForegroundColor Gray
