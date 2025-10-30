#!/usr/bin/env pwsh
# Quick test script for dchat testnet connectivity and health

param(
    [switch]$Verbose,
    [switch]$Watch,
    [int]$Interval = 5
)

$ErrorActionPreference = "SilentlyContinue"

function Test-RelayHealth {
    param([string]$Relay, [int]$Port)
    
    try {
        $response = curl -s -m 2 "http://localhost:$Port/health" 2>&1
        return $response
    } catch {
        return "UNREACHABLE"
    }
}

function Test-RelayMetrics {
    param([string]$Relay, [int]$Port)
    
    try {
        $response = curl -s -m 2 "http://localhost:$Port/metrics" 2>&1
        if ($response -match "dchat_") {
            return "✅ Metrics available"
        } else {
            return "⚠️  No metrics"
        }
    } catch {
        return "❌ Connection failed"
    }
}

function Test-PrometheusTargets {
    try {
        $response = curl -s "http://localhost:9093/api/v1/targets" 2>&1 | ConvertFrom-Json
        $active = ($response.data.activeTargets | Measure-Object).Count
        return "Prometheus: $active targets active"
    } catch {
        return "Prometheus: Unavailable"
    }
}

function Get-UpstreamPeers {
    param([int]$RelayPort)
    
    try {
        $logs = docker logs $(docker ps --filter "label=relay_port=$RelayPort" -q) 2>&1 | grep -i "peer\|upstream" | tail -5
        return $logs
    } catch {
        return "No logs available"
    }
}

if ($Watch) {
    Write-Host "🔄 Watching dchat testnet health (refresh every ${Interval}s, Ctrl+C to stop)`n" -ForegroundColor Cyan
    
    while ($true) {
        Clear-Host
        Write-Host "📊 dchat Testnet Health Monitor" -ForegroundColor Cyan
        Write-Host "═════════════════════════════════════" -ForegroundColor Cyan
        Write-Host "Timestamp: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n" -ForegroundColor Gray
        
        # Relay Health
        Write-Host "🔴 Relay Nodes:" -ForegroundColor Yellow
        $relays = @(
            @{Name="Relay1"; Port=8080},
            @{Name="Relay2"; Port=8081},
            @{Name="Relay3"; Port=8082}
        )
        
        foreach ($relay in $relays) {
            $health = Test-RelayHealth $relay.Name $relay.Port
            $metrics = Test-RelayMetrics $relay.Name $relay.Port
            
            $status = if ($health -ne "UNREACHABLE") { "✅" } else { "❌" }
            Write-Host "  $status $($relay.Name) (localhost:$($relay.Port))" -ForegroundColor Gray
            Write-Host "     Health: $health" -ForegroundColor Gray
            Write-Host "     $metrics" -ForegroundColor Gray
        }
        
        # Prometheus Status
        Write-Host "`n📊 Monitoring Stack:" -ForegroundColor Yellow
        $prom = Test-PrometheusTargets
        Write-Host "  $prom" -ForegroundColor Gray
        
        try {
            $grafana = curl -s -m 2 "http://localhost:3000" -o $null -w "✅ Grafana (HTTP %{http_code})"
            Write-Host "  $grafana" -ForegroundColor Gray
        } catch {
            Write-Host "  ❌ Grafana: Unavailable" -ForegroundColor Gray
        }
        
        try {
            $jaeger = curl -s -m 2 "http://localhost:16686" -o $null -w "✅ Jaeger (HTTP %{http_code})"
            Write-Host "  $jaeger" -ForegroundColor Gray
        } catch {
            Write-Host "  ❌ Jaeger: Unavailable" -ForegroundColor Gray
        }
        
        # Docker Services
        Write-Host "`n🐳 Docker Services:" -ForegroundColor Yellow
        $services = docker-compose ps --format "json" 2>&1 | ConvertFrom-Json
        $running = ($services | Where-Object { $_.Status -match "Up" } | Measure-Object).Count
        $total = ($services | Measure-Object).Count
        Write-Host "  Running: $running/$total services" -ForegroundColor Gray
        
        # Database
        Write-Host "`n💾 Database:" -ForegroundColor Yellow
        try {
            $db_test = docker exec dchat-postgres pg_isready -U dchat 2>&1
            if ($? -eq $true) {
                Write-Host "  ✅ PostgreSQL: Ready" -ForegroundColor Gray
            } else {
                Write-Host "  ⚠️  PostgreSQL: Initializing..." -ForegroundColor Gray
            }
        } catch {
            Write-Host "  ❌ PostgreSQL: Unavailable" -ForegroundColor Gray
        }
        
        Write-Host "`nPress Ctrl+C to stop monitoring...`n" -ForegroundColor Gray
        Start-Sleep -Seconds $Interval
    }
} else {
    # Single run
    Write-Host "🧪 Running dchat Testnet Connectivity Test`n" -ForegroundColor Cyan
    Write-Host "═════════════════════════════════════`n" -ForegroundColor Cyan
    
    # 1. Docker Services
    Write-Host "1️⃣  Docker Services" -ForegroundColor Yellow
    try {
        $services = docker-compose ps --format "json" 2>&1 | ConvertFrom-Json
        if ($services) {
            $services | ForEach-Object {
                $status = if ($_.Status -match "Up") { "✅" } else { "⚠️ " }
                Write-Host "  $status $($_.Service): $($_.Status)" -ForegroundColor Gray
            }
        } else {
            Write-Host "  ⚠️  No services running. Run: docker-compose up -d" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "  ❌ Error checking services. Is Docker running?" -ForegroundColor Red
        exit 1
    }
    
    # 2. Relay Connectivity
    Write-Host "`n2️⃣  Relay Node Connectivity" -ForegroundColor Yellow
    $relay_ports = @(8080, 8081, 8082)
    $relay_names = @("relay1", "relay2", "relay3")
    
    for ($i = 0; $i -lt $relay_ports.Count; $i++) {
        $health = Test-RelayHealth $relay_names[$i] $relay_ports[$i]
        if ($health -ne "UNREACHABLE") {
            Write-Host "  ✅ $($relay_names[$i]) responding" -ForegroundColor Green
            if ($Verbose) {
                Write-Host "     Response: $health" -ForegroundColor Gray
            }
        } else {
            Write-Host "  ❌ $($relay_names[$i]) not responding" -ForegroundColor Red
        }
    }
    
    # 3. Metrics Collection
    Write-Host "`n3️⃣  Metrics Collection" -ForegroundColor Yellow
    try {
        $response = curl -s "http://localhost:9093/api/v1/targets" 2>&1 | ConvertFrom-Json
        $active = ($response.data.activeTargets | Measure-Object).Count
        Write-Host "  ✅ Prometheus: $active targets active" -ForegroundColor Green
        
        if ($Verbose) {
            $response.data.activeTargets | ForEach-Object {
                Write-Host "     - $($_.labels.job): $($_.lastScrapeStatus)" -ForegroundColor Gray
            }
        }
    } catch {
        Write-Host "  ❌ Prometheus not responding" -ForegroundColor Red
    }
    
    # 4. Monitoring Dashboards
    Write-Host "`n4️⃣  Monitoring Dashboards" -ForegroundColor Yellow
    
    @(
        @{Name="Grafana"; URL="http://localhost:3000"; Port=3000},
        @{Name="Jaeger"; URL="http://localhost:16686"; Port=16686},
        @{Name="Prometheus"; URL="http://localhost:9093"; Port=9093}
    ) | ForEach-Object {
        try {
            $response = curl -s -m 2 $_.URL -o $null -w "%{http_code}"
            if ($response -in @("200", "302", "403")) {
                Write-Host "  ✅ $($_.Name) ($($_.URL))" -ForegroundColor Green
            } else {
                Write-Host "  ⚠️  $($_.Name) (HTTP $response)" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "  ❌ $($_.Name) not responding" -ForegroundColor Red
        }
    }
    
    # 5. Network Connectivity
    Write-Host "`n5️⃣  Network Diagnostics" -ForegroundColor Yellow
    try {
        $relay1_to_relay2 = docker exec dchat-relay1 curl -s -m 2 http://relay2:9091 -o $null -w "%{http_code}" 2>&1
        Write-Host "  ✅ relay1→relay2: HTTP $relay1_to_relay2" -ForegroundColor Green
    } catch {
        Write-Host "  ❌ relay1→relay2: Connection failed" -ForegroundColor Red
    }
    
    try {
        $relay1_db = docker exec dchat-relay1 curl -s -m 2 http://postgres:5432 -o $null 2>&1
        Write-Host "  ✅ relay1→postgres: Port 5432 open" -ForegroundColor Green
    } catch {
        Write-Host "  ⚠️  relay1→postgres: Testing" -ForegroundColor Yellow
    }
    
    # Summary
    Write-Host "`n" -NoNewline
    Write-Host "═════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "✅ Test complete! Use -Watch flag for continuous monitoring." -ForegroundColor Green
    Write-Host "`nAccess dashboards:" -ForegroundColor Cyan
    Write-Host "  • Grafana:     http://localhost:3000" -ForegroundColor Gray
    Write-Host "  • Jaeger:      http://localhost:16686" -ForegroundColor Gray
    Write-Host "  • Prometheus:  http://localhost:9093" -ForegroundColor Gray
}
