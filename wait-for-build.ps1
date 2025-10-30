#!/usr/bin/env pwsh
# Wait for Docker build to complete and verify services are running

param(
    [int]$TimeoutSeconds = 1800  # 30 minutes
)

$startTime = Get-Date
$checkInterval = 10

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DCHAT BUILD WAITER - Monitoring Rust Compilation       ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "⏱️  Started at: $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow
Write-Host "⏰ Timeout: $([Math]::Floor($TimeoutSeconds / 60)) minutes" -ForegroundColor Yellow
Write-Host "🔍 Checking every $checkInterval seconds" -ForegroundColor Yellow
Write-Host ""

$spinnerChars = @('|', '/', '-', '\')
$spinnerIndex = 0

while ($true) {
    $elapsed = ((Get-Date) - $startTime).TotalSeconds
    
    if ($elapsed -gt $TimeoutSeconds) {
        Write-Host ""
        Write-Host "❌ Timeout reached ($([Math]::Floor($TimeoutSeconds / 60)) minutes)" -ForegroundColor Red
        Write-Host ""
        Write-Host "Build may have failed. Check logs with:" -ForegroundColor Yellow
        Write-Host "  docker-compose logs relay1" -ForegroundColor Gray
        Write-Host "  docker-compose logs relay2" -ForegroundColor Gray
        Write-Host "  docker-compose logs relay3" -ForegroundColor Gray
        exit 1
    }
    
    # Check for running containers
    $runningContainers = @(docker-compose ps --services --filter "status=running" 2>$null)
    
    # Check for built images
    $builtImages = @(docker images --filter "reference=dchat*:*" --format "{{.Repository}}" 2>$null | Where-Object {$_})
    
    if ($runningContainers.Count -ge 7) {
        Write-Host ""
        Write-Host "✅ All 7 containers are running!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Container Status:" -ForegroundColor Cyan
        docker-compose ps
        Write-Host ""
        Write-Host "Time taken: $([Math]::Floor($elapsed))s ($([Math]::Floor($elapsed/60))m $([Math]::Floor($elapsed%60))s)" -ForegroundColor Green
        Write-Host ""
        Write-Host "🌐 Access your services:" -ForegroundColor Cyan
        Write-Host "  • Grafana Dashboard: http://localhost:3000 (admin/admin)" -ForegroundColor White
        Write-Host "  • Prometheus Metrics: http://localhost:9090" -ForegroundColor White
        Write-Host "  • Jaeger Traces: http://localhost:16686" -ForegroundColor White
        Write-Host "  • relay-1 RPC: http://localhost:7071" -ForegroundColor White
        Write-Host "  • relay-2 RPC: http://localhost:7073" -ForegroundColor White
        Write-Host "  • relay-3 RPC: http://localhost:7075" -ForegroundColor White
        Write-Host ""
        exit 0
    }
    
    if ($builtImages.Count -gt 0) {
        Write-Host "✅ Relay images built! Containers starting..." -ForegroundColor Green
    } else {
        # Show spinner
        $spinner = $spinnerChars[$spinnerIndex % 4]
        $spinnerIndex++
        
        $elapsedMin = [Math]::Floor($elapsed / 60)
        $elapsedSec = [Math]::Floor($elapsed % 60)
        
        Write-Host -NoNewline "`r⏳ Elapsed: ${elapsedMin}m ${elapsedSec}s | Compiling Rust binary $spinner  "
    }
    
    Start-Sleep -Seconds $checkInterval
}
