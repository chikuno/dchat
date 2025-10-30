#!/usr/bin/env pwsh
# Docker Startup Script for dchat Project
# Auto-starts Docker daemon if not running

$DOCKER_SERVICE_NAME = "com.docker.service"
$MAX_WAIT = 30  # seconds

Write-Host "🐳 Checking Docker status..." -ForegroundColor Cyan

$service = Get-Service -Name $DOCKER_SERVICE_NAME -ErrorAction SilentlyContinue
if (-not $service) {
    Write-Host "⚠️  Docker service not found" -ForegroundColor Yellow
    exit 1
}

if ($service.Status -eq "Running") {
    Write-Host "✅ Docker already running" -ForegroundColor Green
    exit 0
}

Write-Host "▶️  Starting Docker service..." -ForegroundColor Yellow
try {
    Start-Service -Name $DOCKER_SERVICE_NAME -ErrorAction Stop
    Write-Host "✅ Docker service started" -ForegroundColor Green
    
    # Wait for daemon
    $attempt = 0
    Write-Host "⏳ Waiting for daemon initialization..." -NoNewline
    while ($attempt -lt $MAX_WAIT) {
        Start-Sleep -Seconds 1
        $attempt++
        Write-Host -NoNewline "."
    }
    Write-Host " Done!" -ForegroundColor Green
    exit 0
} catch {
    Write-Host "❌ Failed to start Docker: $_" -ForegroundColor Red
    exit 1
}
