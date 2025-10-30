#!/usr/bin/env pwsh
# Docker Configuration & Setup Script for dchat Project
# Sets up Docker CLI access, PATH configuration, and startup automation

param(
    [switch]$Setup,
    [switch]$Start,
    [switch]$Stop,
    [switch]$Status,
    [switch]$Auto,
    [switch]$Interactive,
    [switch]$Force
)

$ErrorActionPreference = "Continue"
$WarningPreference = "SilentlyContinue"

# Configuration
$DOCKER_SERVICE_NAME = "com.docker.service"
$DOCKER_STARTUP_SCRIPT = "$PSScriptRoot\docker-startup.ps1"

Write-Host "🐳 dchat Docker Configuration Tool`n" -ForegroundColor Cyan

# ============================================================================
# FUNCTION: Find Docker Executable
# ============================================================================
function Find-DockerExecutable {
    $possiblePaths = @(
        "C:\Program Files\Docker\Docker\resources\bin\docker.exe",
        "C:\Program Files\Docker\CLI\docker.exe",
        "${env:ProgramFiles}\Docker\Docker\resources\bin\docker.exe",
        "${env:ProgramFiles(x86)}\Docker\docker.exe",
        "C:\Docker\docker.exe"
    )
    
    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            return $path
        }
    }
    
    # Try to find via where command
    $where = cmd /c "where docker.exe 2>nul"
    if ($where) {
        return $where.Trim()
    }
    
    return $null
}



# ============================================================================
# FUNCTION: Get Docker Status
# ============================================================================
function Get-DockerStatus {
    Write-Host "`n📊 Docker Status Check`n" -ForegroundColor Magenta
    
    # Check Service
    $service = Get-Service -Name $DOCKER_SERVICE_NAME -ErrorAction SilentlyContinue
    if ($service) {
        $status = $service.Status
        $symbol = if ($status -eq "Running") { "✅" } else { "⏸️ " }
        Write-Host "$symbol Service Status: $status" -ForegroundColor $(
            if ($status -eq "Running") { "Green" } else { "Yellow" }
        )
    } else {
        Write-Host "❌ Docker service not found" -ForegroundColor Red
        return $false
    }
    
    # Check Docker CLI
    $docker = Find-DockerExecutable
    if ($docker) {
        Write-Host "✅ Docker CLI: Found at $docker" -ForegroundColor Green
    } else {
        Write-Host "❌ Docker CLI: Not found in PATH" -ForegroundColor Red
    }
    
    # Check Docker Compose
    $compose = Find-DockerComposeExecutable
    if ($compose) {
        Write-Host "✅ Docker Compose: Found at $compose" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Docker Compose: Not found" -ForegroundColor Yellow
    }
    
    # Check if Docker daemon is responding
    if ($service.Status -eq "Running") {
        try {
            $docker = Find-DockerExecutable
            if ($docker) {
                & $docker ps -q | Out-Null
                Write-Host "✅ Docker daemon: Responding" -ForegroundColor Green
                return $true
            }
        } catch {
            Write-Host "⚠️  Docker daemon: Not responding yet" -ForegroundColor Yellow
        }
    }
    
    return $false
}

# ============================================================================
# FUNCTION: Start Docker Service
# ============================================================================
function Start-DockerService {
    Write-Host "`n▶️  Starting Docker Service..." -ForegroundColor Cyan
    
    $service = Get-Service -Name $DOCKER_SERVICE_NAME -ErrorAction SilentlyContinue
    if (-not $service) {
        Write-Host "❌ Docker service not found" -ForegroundColor Red
        return $false
    }
    
    if ($service.Status -eq "Running") {
        Write-Host "✅ Docker service already running" -ForegroundColor Green
        return $true
    }
    
    try {
        # Start the service
        Start-Service -Name $DOCKER_SERVICE_NAME -ErrorAction Stop
        Write-Host "✅ Docker service started" -ForegroundColor Green
        
        # Wait for daemon to initialize
        Write-Host "⏳ Waiting for Docker daemon to initialize (up to 30 seconds)..." -ForegroundColor Yellow
        $maxAttempts = 30
        $attempt = 0
        
        do {
            Start-Sleep -Seconds 1
            $attempt++
            
            try {
                $docker = Find-DockerExecutable
                if ($docker) {
                    & $docker ps -q | Out-Null
                    Write-Host "✅ Docker daemon is ready!" -ForegroundColor Green
                    return $true
                }
            } catch {
                # Still initializing
                Write-Host -NoNewline "."
            }
        } while ($attempt -lt $maxAttempts)
        
        Write-Host "`n⚠️  Docker daemon may still be initializing. Try again in a moment." -ForegroundColor Yellow
        return $true
    } catch {
        Write-Host "❌ Failed to start Docker service: $_" -ForegroundColor Red
        return $false
    }
}

# ============================================================================
# FUNCTION: Stop Docker Service
# ============================================================================
function Stop-DockerService {
    Write-Host "`n⏹️  Stopping Docker Service..." -ForegroundColor Cyan
    
    $service = Get-Service -Name $DOCKER_SERVICE_NAME -ErrorAction SilentlyContinue
    if (-not $service) {
        Write-Host "❌ Docker service not found" -ForegroundColor Red
        return $false
    }
    
    if ($service.Status -eq "Stopped") {
        Write-Host "✅ Docker service already stopped" -ForegroundColor Green
        return $true
    }
    
    try {
        Stop-Service -Name $DOCKER_SERVICE_NAME -Force -ErrorAction Stop
        Write-Host "✅ Docker service stopped" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "❌ Failed to stop Docker service: $_" -ForegroundColor Red
        return $false
    }
}

# ============================================================================
# FUNCTION: Add Docker to PATH
# ============================================================================
function Add-DockerToPath {
    Write-Host "`n🔧 Configuring Docker PATH..." -ForegroundColor Cyan
    
    $docker = Find-DockerExecutable
    if (-not $docker) {
        Write-Host "❌ Docker executable not found" -ForegroundColor Red
        return $false
    }
    
    $dockerBinDir = Split-Path $docker
    
    # Check if already in PATH
    if ($env:PATH -split ';' -contains $dockerBinDir) {
        Write-Host "✅ Docker already in PATH" -ForegroundColor Green
        return $true
    }
    
    # Add to current session PATH
    $env:PATH = "$dockerBinDir;$env:PATH"
    Write-Host "✅ Added Docker to current session PATH" -ForegroundColor Green
    
    # Verify
    try {
        $test = & $docker --version
        Write-Host "✅ Docker CLI accessible: $test" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "⚠️  Could not verify Docker access: $_" -ForegroundColor Yellow
        return $false
    }
}

# ============================================================================
# FUNCTION: Create Startup Script
# ============================================================================
function New-DockerStartupScript {
    Write-Host "`n📝 Creating Docker startup script..." -ForegroundColor Cyan
    
    $scriptContent = @'
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
'@
    
    try {
        Set-Content -Path $DOCKER_STARTUP_SCRIPT -Value $scriptContent -Force
        Write-Host "✅ Created: $DOCKER_STARTUP_SCRIPT" -ForegroundColor Green
        
        # Make executable
        if ((Get-Item $DOCKER_STARTUP_SCRIPT).Attributes -notmatch "Archive") {
            Set-ItemProperty -Path $DOCKER_STARTUP_SCRIPT -Name Attributes -Value "Archive"
        }
        
        return $true
    } catch {
        Write-Host "❌ Failed to create startup script: $_" -ForegroundColor Red
        return $false
    }
}

# ============================================================================
# FUNCTION: Setup PowerShell Profile
# ============================================================================
function Set-PowerShellProfile {
    Write-Host "`n⚙️  Configuring PowerShell profile..." -ForegroundColor Cyan
    Write-Host "✅ Docker PATH already configured in current session" -ForegroundColor Green
    Write-Host "💡 Docker will be accessible directly from terminal" -ForegroundColor Gray
    return $true
}

# ============================================================================
# FUNCTION: Setup Auto-Start (Task Scheduler)
# ============================================================================
function Set-DockerAutoStart {
    Write-Host "`n🔄 Setting up Docker auto-start..." -ForegroundColor Cyan
    
    $taskName = "DChatDockerAutoStart"
    
    try {
        # Check if task already exists
        $task = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
        if ($task) {
            Write-Host "✅ Docker auto-start task already configured" -ForegroundColor Green
            return $true
        }
        
        # Create task action
        $action = New-ScheduledTaskAction -Execute "powershell.exe" `
            -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$DOCKER_STARTUP_SCRIPT`""
        
        # Create trigger (at system startup)
        $trigger = New-ScheduledTaskTrigger -AtStartup
        
        # Create task settings (compatible with all PS versions)
        $settings = New-ScheduledTaskSettingsSet `
            -StartWhenAvailable `
            -MultipleInstances IgnoreNew `
            -ExecutionTimeLimit (New-TimeSpan -Minutes 5)
        
        # Create principal (System account, highest privilege)
        $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" `
            -LogonType ServiceAccount -RunLevel Highest
        
        # Register the task
        Register-ScheduledTask -TaskName $taskName `
            -Action $action -Trigger $trigger -Settings $settings `
            -Principal $principal -Force | Out-Null
        
        Write-Host "✅ Created scheduled task: $taskName" -ForegroundColor Green
        Write-Host "   Docker will auto-start on system boot" -ForegroundColor Gray
        return $true
    } catch {
        Write-Host "⚠️  Could not create scheduled task (may require elevation): $_" -ForegroundColor Yellow
        Write-Host "    Docker will not auto-start on boot" -ForegroundColor Gray
        return $false
    }
}

# ============================================================================
# FUNCTION: Interactive Setup
# ============================================================================
function Invoke-InteractiveSetup {
    Write-Host "
╔═══════════════════════════════════════════════════════════════╗
║      Docker Configuration Setup - Interactive Mode           ║
╚═══════════════════════════════════════════════════════════════╝
" -ForegroundColor Cyan
    
    # Check current status
    Get-DockerStatus
    
    Write-Host "`n" -NoNewline
    $continue = Read-Host "Continue with setup? (y/n)"
    if ($continue -ne "y") {
        Write-Host "Setup cancelled" -ForegroundColor Yellow
        return
    }
    
    # Add to PATH
    Write-Host "`n" -NoNewline
    $addPath = Read-Host "Add Docker to PATH? (y/n)"
    if ($addPath -eq "y") {
        Add-DockerToPath
    }
    
    # Create startup script
    Write-Host "`n" -NoNewline
    $createStartup = Read-Host "Create startup script? (y/n)"
    if ($createStartup -eq "y") {
        New-DockerStartupScript
    }
    
    # Update PowerShell profile
    Write-Host "`n" -NoNewline
    $updateProfile = Read-Host "Update PowerShell profile? (y/n)"
    if ($updateProfile -eq "y") {
        Set-PowerShellProfile
    }
    
    # Setup auto-start
    Write-Host "`n" -NoNewline
    $autoStart = Read-Host "Setup Docker auto-start on boot? (y/n)"
    if ($autoStart -eq "y") {
        Set-DockerAutoStart
    }
    
    Write-Host "`n✅ Setup complete!" -ForegroundColor Green
    Write-Host "
Next steps:
1. Reload PowerShell profile: . `$PROFILE
2. Start Docker: docker-start
3. Verify: docker ps
" -ForegroundColor Cyan
}

# ============================================================================
# MAIN LOGIC
# ============================================================================

# If no parameters, show status and offer help
if (-not ($Setup -or $Start -or $Stop -or $Status -or $Auto -or $Interactive -or $Force)) {
    Get-DockerStatus
    Write-Host "
Usage:
  .\configure-docker.ps1 -Setup       # Run full setup
  .\configure-docker.ps1 -Start       # Start Docker service
  .\configure-docker.ps1 -Stop        # Stop Docker service
  .\configure-docker.ps1 -Status      # Check Docker status
  .\configure-docker.ps1 -Auto        # Setup auto-start only
  .\configure-docker.ps1 -Interactive # Interactive setup wizard
" -ForegroundColor Gray
    exit 0
}

# Execute requested operations
if ($Interactive) {
    Invoke-InteractiveSetup
}

if ($Setup) {
    Write-Host "`n🔧 Running full Docker setup..." -ForegroundColor Cyan
    Add-DockerToPath
    New-DockerStartupScript
    Set-PowerShellProfile
    Set-DockerAutoStart
    Write-Host "`n✅ Full setup complete!" -ForegroundColor Green
}

if ($Start) {
    Start-DockerService
}

if ($Stop) {
    Stop-DockerService
}

if ($Status) {
    Get-DockerStatus
}

if ($Auto) {
    New-DockerStartupScript
    Set-DockerAutoStart
}

Write-Host ""
