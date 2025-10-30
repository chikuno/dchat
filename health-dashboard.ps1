#!/usr/bin/env powershell
# dchat Network Health Monitoring Dashboard
# Real-time monitoring of all validators, relays, and system metrics
# Usage: ./health-dashboard.ps1

param(
    [switch]$Continuous = $false,
    [int]$Interval = 10,
    [switch]$Detailed = $false,
    [string]$ExportFormat = ""
)

# Configuration
$ValidatorPorts = @(7071, 7073, 7075, 7077)
$RelayPorts = @(7081, 7083, 7085, 7087)
$MonitoringBaseUrl = "http://localhost"
$TimestampFormat = "yyyy-MM-dd HH:mm:ss"

# Colors
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Cyan = "`e[36m"
$Reset = "`e[0m"

class NodeStatus {
    [string]$Name
    [string]$Type
    [int]$Port
    [bool]$IsHealthy
    [string]$Status
    [int]$PeerCount
    [string]$LatestBlock
    [string]$ConsensusState
    [float]$ResponseTime
    [string]$CPU
    [string]$Memory
    [datetime]$LastCheck
}

function Get-NodeHealth {
    param(
        [string]$NodeName,
        [string]$NodeType,
        [int]$Port
    )
    
    $startTime = Get-Date
    $node = [NodeStatus]::new()
    $node.Name = $NodeName
    $node.Type = $NodeType
    $node.Port = $Port
    $node.LastCheck = Get-Date
    
    try {
        # Health check
        $healthResponse = Invoke-WebRequest -Uri "$MonitoringBaseUrl`:$Port/health" `
            -TimeoutSec 5 -ErrorAction SilentlyContinue
        
        if ($healthResponse.StatusCode -eq 200) {
            $node.IsHealthy = $true
            $node.Status = "✓ HEALTHY"
        }
        else {
            $node.IsHealthy = $false
            $node.Status = "✗ UNHEALTHY"
        }
    }
    catch {
        $node.IsHealthy = $false
        $node.Status = "✗ OFFLINE"
    }
    
    $node.ResponseTime = (Get-Date - $startTime).TotalMilliseconds
    
    # Get detailed metrics if healthy
    if ($node.IsHealthy) {
        try {
            # Peer count
            $peersResponse = Invoke-WebRequest -Uri "$MonitoringBaseUrl`:$Port/peers" `
                -TimeoutSec 5 -ErrorAction SilentlyContinue
            if ($peersResponse) {
                $peersJson = $peersResponse.Content | ConvertFrom-Json
                $node.PeerCount = $peersJson.peer_count ?? 0
            }
            
            # Latest block
            $blockResponse = Invoke-WebRequest -Uri "$MonitoringBaseUrl`:$Port/chain/latest-block" `
                -TimeoutSec 5 -ErrorAction SilentlyContinue
            if ($blockResponse) {
                $blockJson = $blockResponse.Content | ConvertFrom-Json
                $node.LatestBlock = $blockJson.block_number ?? "N/A"
            }
            
            # Docker stats
            $containerId = & docker ps --filter "name=dchat-$($NodeName.ToLower())" -q
            if ($containerId) {
                $statsJson = & docker stats $containerId --no-stream --format='json' 2>$null | ConvertFrom-Json
                $node.CPU = "$([math]::Round([double]$statsJson.CPUPerc -replace '%', ''))"
                $node.Memory = $statsJson.MemUsage
            }
        }
        catch {
            # Silently fail on detailed metrics
        }
    }
    
    return $node
}

function Format-NodeRow {
    param([NodeStatus]$Node)
    
    $statusColor = if ($Node.IsHealthy) { $Green } else { $Red }
    $statusSymbol = if ($Node.IsHealthy) { "✓" } else { "✗" }
    
    $line = "$($Node.Name.PadRight(15)) | "
    $line += "$($Node.Type.PadRight(10)) | "
    $line += "$statusColor$($statusSymbol) $($Node.Status.PadRight(15))$Reset | "
    $line += "$($Node.PeerCount.ToString().PadRight(5)) | "
    $line += "$($Node.LatestBlock.ToString().PadRight(8)) | "
    $line += "$("$($Node.ResponseTime)ms".PadRight(10)) | "
    $line += "$($Node.CPU.PadRight(6)) | "
    $line += "$($Node.Memory.PadRight(15))"
    
    return $line
}

function Display-Dashboard {
    Clear-Host
    
    $timestamp = (Get-Date).ToString($TimestampFormat)
    Write-Host "$Blue╔════════════════════════════════════════════════════════════════════════════════╗$Reset"
    Write-Host "$Blue║$Reset $Green DCHAT NETWORK HEALTH DASHBOARD$Reset $Blue║$Reset"
    Write-Host "$Blue║$Reset Timestamp: $timestamp $Blue║$Reset"
    Write-Host "$Blue╚════════════════════════════════════════════════════════════════════════════════╝$Reset"
    Write-Host ""
    
    # Validators Section
    Write-Host "$Cyan━━━ VALIDATORS ━━━$Reset"
    Write-Host "Name            | Type       | Status          | Peers | Block    | Response | CPU   | Memory"
    Write-Host "-" * 95
    
    $validatorStatus = @()
    $healthyValidators = 0
    
    for ($i = 0; $i -lt $ValidatorPorts.Count; $i++) {
        $port = $ValidatorPorts[$i]
        $name = "validator$($i+1)"
        $node = Get-NodeHealth -NodeName $name -NodeType "Validator" -Port $port
        $validatorStatus += $node
        
        if ($node.IsHealthy) { $healthyValidators++ }
        Write-Host (Format-NodeRow -Node $node)
    }
    
    Write-Host ""
    Write-Host "$Cyan━━━ RELAYS ━━━$Reset"
    Write-Host "Name            | Type       | Status          | Peers | Block    | Response | CPU   | Memory"
    Write-Host "-" * 95
    
    $relayStatus = @()
    $healthyRelays = 0
    
    for ($i = 0; $i -lt $RelayPorts.Count; $i++) {
        $port = $RelayPorts[$i]
        $name = "relay$($i+1)"
        $node = Get-NodeHealth -NodeName $name -NodeType "Relay" -Port $port
        $relayStatus += $node
        
        if ($node.IsHealthy) { $healthyRelays++ }
        Write-Host (Format-NodeRow -Node $node)
    }
    
    Write-Host ""
    
    # Summary Section
    Write-Host "$Cyan━━━ SUMMARY ━━━$Reset"
    
    $validatorHealthPercent = [math]::Round(($healthyValidators / $ValidatorPorts.Count) * 100)
    $relayHealthPercent = [math]::Round(($healthyRelays / $RelayPorts.Count) * 100)
    $totalHealthPercent = [math]::Round((($healthyValidators + $healthyRelays) / ($ValidatorPorts.Count + $RelayPorts.Count)) * 100)
    
    $validatorColor = if ($validatorHealthPercent -eq 100) { $Green } elseif ($validatorHealthPercent -ge 75) { $Yellow } else { $Red }
    $relayColor = if ($relayHealthPercent -eq 100) { $Green } elseif ($relayHealthPercent -ge 75) { $Yellow } else { $Red }
    $totalColor = if ($totalHealthPercent -eq 100) { $Green } elseif ($totalHealthPercent -ge 75) { $Yellow } else { $Red }
    
    Write-Host "Validator Health: $validatorColor$validatorHealthPercent%$Reset ($healthyValidators/$($ValidatorPorts.Count))"
    Write-Host "Relay Health:     $relayColor$relayHealthPercent%$Reset ($healthyRelays/$($RelayPorts.Count))"
    Write-Host "Overall Health:   $totalColor$totalHealthPercent%$Reset"
    
    Write-Host ""
    
    # Consensus Status
    try {
        $consensusResponse = Invoke-WebRequest -Uri "$MonitoringBaseUrl`:7071/chain/consensus-status" `
            -TimeoutSec 5 -ErrorAction SilentlyContinue
        if ($consensusResponse) {
            $consensus = $consensusResponse.Content | ConvertFrom-Json
            Write-Host "$Cyan━━━ CONSENSUS ━━━$Reset"
            Write-Host "Voting Validators: $($consensus.voting_count ?? 'N/A')/4"
            Write-Host "Block Number:      $($consensus.block_number ?? 'N/A')"
            Write-Host "Finalized:         $($consensus.finalized ?? 'N/A')"
            Write-Host ""
        }
    }
    catch {
        # Silently skip if unavailable
    }
    
    # System Resources
    Write-Host "$Cyan━━━ SYSTEM RESOURCES ━━━$Reset"
    try {
        $statsOutput = & docker stats --no-stream --format='table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}' 2>$null
        $dchatStats = $statsOutput | Select-String "dchat"
        
        $totalCpu = 0
        $totalMem = 0
        
        foreach ($stat in $dchatStats) {
            $parts = $stat -split '\s+'
            if ($parts.Count -gt 2) {
                $cpu = [double]($parts[1] -replace '%', '')
                $totalCpu += $cpu
            }
        }
        
        Write-Host "Total CPU Usage: $($totalCpu)%"
        Write-Host "Active Containers: $($dchatStats.Count)"
    }
    catch {
        Write-Host "Unable to retrieve system resources"
    }
    
    Write-Host ""
    Write-Host "$Cyan━━━ MONITORING ENDPOINTS ━━━$Reset"
    Write-Host "Prometheus:  http://localhost:9090"
    Write-Host "Grafana:     http://localhost:3000 (admin/admin)"
    Write-Host "Jaeger:      http://localhost:16686"
    Write-Host "RPC:         http://rpc.webnetcore.top:8080"
    Write-Host ""
    
    if ($Continuous) {
        Write-Host "$Yellow[Auto-refresh every $Interval seconds - Press Ctrl+C to exit]$Reset"
        Write-Host "Last updated: $(Get-Date -Format 'HH:mm:ss')"
    }
}

function Export-Dashboard {
    param([string]$Format)
    
    $timestamp = (Get-Date).ToString("yyyyMMdd_HHmmss")
    $filename = "dchat-health-$timestamp"
    
    switch ($Format.ToLower()) {
        "json" {
            $data = @{
                timestamp = Get-Date
                validators = @()
                relays = @()
            }
            
            foreach ($port in $ValidatorPorts) {
                $data.validators += (Get-NodeHealth -NodeName "validator$($ValidatorPorts.IndexOf($port)+1)" `
                    -NodeType "Validator" -Port $port)
            }
            
            foreach ($port in $RelayPorts) {
                $data.relays += (Get-NodeHealth -NodeName "relay$($RelayPorts.IndexOf($port)+1)" `
                    -NodeType "Relay" -Port $port)
            }
            
            $data | ConvertTo-Json | Out-File -FilePath "$filename.json"
            Write-Host "Exported to $filename.json"
        }
        
        "csv" {
            $output = "Timestamp,Name,Type,Healthy,Status,Peers,Block,ResponseTime,CPU,Memory"
            $output | Out-File -FilePath "$filename.csv"
            
            foreach ($port in $ValidatorPorts) {
                $node = Get-NodeHealth -NodeName "validator$($ValidatorPorts.IndexOf($port)+1)" `
                    -NodeType "Validator" -Port $port
                $line = "$($node.LastCheck),$($node.Name),$($node.Type),$($node.IsHealthy),$($node.Status),$($node.PeerCount),$($node.LatestBlock),$($node.ResponseTime),$($node.CPU),$($node.Memory)"
                $line | Out-File -FilePath "$filename.csv" -Append
            }
            
            Write-Host "Exported to $filename.csv"
        }
    }
}

# Main execution
if ($Continuous) {
    while ($true) {
        Display-Dashboard
        if ($ExportFormat) {
            Export-Dashboard -Format $ExportFormat
        }
        Start-Sleep -Seconds $Interval
    }
}
else {
    Display-Dashboard
    if ($ExportFormat) {
        Export-Dashboard -Format $ExportFormat
    }
}
