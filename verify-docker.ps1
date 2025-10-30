#!/usr/bin/env pwsh
# Docker verification script for dchat testnet
# Validates installation, configuration, and service health

param(
    [switch]$Verbose,
    [switch]$Fix
)

$ErrorActionPreference = "Continue"
$checks_passed = 0
$checks_failed = 0
$checks_warnings = 0

function Write-Check {
    param(
        [string]$Check,
        [string]$Status,
        [string]$Message = ""
    )
    
    $symbol = switch ($Status) {
        "PASS" { "✅"; $checks_passed++ }
        "FAIL" { "❌"; $checks_failed++ }
        "WARN" { "⚠️ "; $checks_warnings++ }
        "INFO" { "ℹ️ " }
        default { "?" }
    }
    
    Write-Host "$symbol $Check" -ForegroundColor $(
        @{PASS="Green"; FAIL="Red"; WARN="Yellow"; INFO="Cyan"}[$Status]
    ) -NoNewline
    
    if ($Message) {
        Write-Host " - $Message" -ForegroundColor Gray
    } else {
        Write-Host ""
    }
}

Write-Host "🐳 dchat Docker Verification Script`n" -ForegroundColor Cyan

# 1. Docker Installation
Write-Host "1️⃣  Checking Docker Installation..." -ForegroundColor Magenta
try {
    $docker_version = docker --version 2>&1
    Write-Check "Docker installed" "PASS" $docker_version
} catch {
    Write-Check "Docker installed" "FAIL" "Docker not found in PATH. Install Docker Desktop."
}

# 2. Docker Daemon
Write-Host "`n2️⃣  Checking Docker Daemon..." -ForegroundColor Magenta
try {
    $docker_ps = docker ps 2>&1
    if ($?) {
        Write-Check "Docker daemon running" "PASS"
    }
} catch {
    Write-Check "Docker daemon running" "FAIL" "Docker Desktop may not be started. Start Docker and try again."
}

# 3. Docker Compose
Write-Host "`n3️⃣  Checking Docker Compose..." -ForegroundColor Magenta
try {
    $compose_version = docker-compose --version 2>&1
    Write-Check "Docker Compose installed" "PASS" $compose_version
} catch {
    Write-Check "Docker Compose installed" "FAIL" "Docker Compose not found. Install via Docker Desktop."
}

# 4. Configuration Files
Write-Host "`n4️⃣  Checking Configuration Files..." -ForegroundColor Magenta

$required_files = @(
    "docker-compose.yml",
    "Dockerfile",
    "testnet-config.toml",
    "monitoring/prometheus.yml"
)

foreach ($file in $required_files) {
    if (Test-Path $file) {
        Write-Check "File: $file" "PASS"
    } else {
        Write-Check "File: $file" "FAIL"
    }
}

# 5. Config Directory
Write-Host "`n5️⃣  Checking Config Directories..." -ForegroundColor Magenta

$config_files = @(
    "config/relay1.toml",
    "config/relay2.toml",
    "config/relay3.toml"
)

$config_missing = @()
foreach ($file in $config_files) {
    if (Test-Path $file) {
        Write-Check "Config: $file" "PASS"
    } else {
        Write-Check "Config: $file" "FAIL"
        $config_missing += $file
    }
}

if ($config_missing.Count -gt 0 -and $Fix) {
    Write-Host "`n🔧 Auto-fixing missing config files..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path "config" -Force | Out-Null
    Copy-Item "testnet-config.toml" "config/relay1.toml" -Force
    Copy-Item "testnet-config.toml" "config/relay2.toml" -Force
    Copy-Item "testnet-config.toml" "config/relay3.toml" -Force
    Write-Host "✅ Config files created" -ForegroundColor Green
}

# 6. Running Services
Write-Host "`n6️⃣  Checking Running Services..." -ForegroundColor Magenta

try {
    $services = docker-compose ps --format "json" 2>&1 | ConvertFrom-Json
    $service_count = $services | Measure-Object | Select-Object -ExpandProperty Count
    
    if ($service_count -gt 0) {
        Write-Check "Services running" "PASS" "$service_count services"
        $services | ForEach-Object {
            $status = if ($_.Status -match "Up") { "PASS" } else { "WARN" }
            Write-Host "  - $($_.Service): $($_.Status)" -ForegroundColor Gray
        }
    } else {
        Write-Check "Services running" "WARN" "No services currently running"
        Write-Host "  Run: docker-compose up -d" -ForegroundColor Yellow
    }
} catch {
    Write-Check "Services running" "WARN" "Unable to check (daemon may not be running)"
}

# 7. Port Availability
Write-Host "`n7️⃣  Checking Port Availability..." -ForegroundColor Magenta

$ports = @(
    @{Name="Relay1 P2P"; Port=7070},
    @{Name="Relay2 P2P"; Port=7072},
    @{Name="Relay3 P2P"; Port=7074},
    @{Name="Prometheus"; Port=9093},
    @{Name="Grafana"; Port=3000},
    @{Name="Jaeger"; Port=16686}
)

foreach ($port_info in $ports) {
    $conn = Test-NetConnection localhost -Port $port_info.Port -WarningAction SilentlyContinue
    if ($conn.TcpTestSucceeded) {
        Write-Check "$($port_info.Name) (:$($port_info.Port))" "PASS"
    } else {
        Write-Check "$($port_info.Name) (:$($port_info.Port))" "INFO" "Port available (service not running)"
    }
}

# 8. Health Checks
Write-Host "`n8️⃣  Checking Service Health..." -ForegroundColor Magenta

$health_endpoints = @(
    @{Name="Relay1 Health"; URL="http://localhost:8080/health"},
    @{Name="Prometheus API"; URL="http://localhost:9093/api/v1/query?query=up"},
    @{Name="Grafana"; URL="http://localhost:3000"}
)

foreach ($endpoint in $health_endpoints) {
    try {
        $response = curl -s -m 2 $endpoint.URL -o $null -w "%{http_code}"
        if ($response -eq "200" -or $response -eq "302" -or $response -eq "403") {
            Write-Check "$($endpoint.Name)" "PASS" "HTTP $response"
        } else {
            Write-Check "$($endpoint.Name)" "INFO" "HTTP $response"
        }
    } catch {
        Write-Check "$($endpoint.Name)" "INFO" "Service not responding"
    }
}

# 9. Docker Network
Write-Host "`n9️⃣  Checking Docker Network..." -ForegroundColor Magenta

try {
    $networks = docker network ls --format "json" 2>&1 | ConvertFrom-Json
    $dchat_net = $networks | Where-Object { $_.Name -eq "dchat-network" }
    
    if ($dchat_net) {
        Write-Check "dchat-network exists" "PASS"
    } else {
        Write-Check "dchat-network exists" "INFO" "Will be created on docker-compose up"
    }
} catch {
    Write-Check "Docker network check" "INFO" "Unable to verify"
}

# 10. Disk Space
Write-Host "`n🔟 Checking Disk Space..." -ForegroundColor Magenta

try {
    $drive = Get-PSDrive C
    $free_gb = [math]::Round($drive.Free / 1GB, 2)
    $total_gb = [math]::Round($drive.Used + $drive.Free / 1GB, 2)
    
    if ($free_gb -gt 20) {
        Write-Check "Disk space (C:)" "PASS" "$free_gb GB free"
    } elseif ($free_gb -gt 5) {
        Write-Check "Disk space (C:)" "WARN" "$free_gb GB free (recommend 20GB+)"
    } else {
        Write-Check "Disk space (C:)" "FAIL" "$free_gb GB free (minimum 5GB required)"
    }
} catch {
    Write-Check "Disk space" "INFO" "Unable to check"
}

# Summary
Write-Host "`n" -NoNewline
Write-Host "═════════════════════════════════════" -ForegroundColor Cyan
Write-Host "📊 Verification Summary" -ForegroundColor Cyan
Write-Host "═════════════════════════════════════" -ForegroundColor Cyan

$total = $checks_passed + $checks_failed + $checks_warnings
Write-Host "✅ PASSED : $checks_passed" -ForegroundColor Green
Write-Host "❌ FAILED : $checks_failed" -ForegroundColor Red
Write-Host "⚠️  WARNINGS: $checks_warnings" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "📈 Total  : $total checks" -ForegroundColor Cyan

if ($checks_failed -eq 0) {
    Write-Host "`n✅ All critical checks passed! Ready to deploy." -ForegroundColor Green
    Write-Host "`nNext steps:" -ForegroundColor Cyan
    Write-Host "1. docker-compose up -d" -ForegroundColor Gray
    Write-Host "2. docker-compose ps" -ForegroundColor Gray
    Write-Host "3. Open http://localhost:3000 (Grafana)" -ForegroundColor Gray
    exit 0
} else {
    Write-Host "`n❌ Some checks failed. Please fix issues above." -ForegroundColor Red
    if ($config_missing.Count -gt 0) {
        Write-Host "`n💡 Tip: Run with -Fix flag to auto-fix config files" -ForegroundColor Yellow
        Write-Host "   .\verify-docker.ps1 -Fix" -ForegroundColor Yellow
    }
    exit 1
}
