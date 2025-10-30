# ============================================================================
# Setup Docker Auto-Start via Windows Startup Folder (No Admin Needed!)
# ============================================================================
# This creates a shortcut in the Windows Startup folder that runs when user logs in
# No admin elevation required - works for current user
# ============================================================================

$ErrorActionPreference = "Continue"

Write-Host "`n🚀 Setting up Docker auto-start via Startup folder..." -ForegroundColor Cyan
Write-Host "   (No admin elevation required!)" -ForegroundColor Gray

# Paths
$startupFolder = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs\Startup")
$shortcutPath = Join-Path $startupFolder "Docker-Startup.lnk"
$scriptPath = "C:\Users\USER\dchat\docker-startup.ps1"

# Verify startup folder exists
if (-not (Test-Path $startupFolder)) {
    Write-Host "📁 Creating startup folder..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $startupFolder -Force | Out-Null
}

# Create shortcut
Write-Host "`n📍 Creating startup shortcut..." -ForegroundColor Cyan
Write-Host "   Source: $scriptPath" -ForegroundColor Gray
Write-Host "   Target: $shortcutPath" -ForegroundColor Gray

try {
    # Use COM to create shortcut (works without PowerShell modules)
    $WshShell = New-Object -ComObject WScript.Shell
    $shortcut = $WshShell.CreateShortcut($shortcutPath)
    
    # Configure shortcut to run PowerShell script
    $shortcut.TargetPath = "powershell.exe"
    $shortcut.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`" -WindowStyle Hidden"
    $shortcut.WorkingDirectory = "C:\Users\USER\dchat"
    $shortcut.Description = "Auto-start Docker daemon for dchat"
    $shortcut.IconLocation = "C:\Program Files\Docker\Docker\Docker.ico"
    
    # Save shortcut
    $shortcut.Save()
    
    Write-Host "`n✅ Created startup shortcut: $shortcutPath" -ForegroundColor Green
    Write-Host "`n🔄 Docker will now auto-start when you log in!" -ForegroundColor Green
    Write-Host "`n💡 Details:" -ForegroundColor Cyan
    Write-Host "   • Runs when you log in (not system startup)" -ForegroundColor Gray
    Write-Host "   • Runs silently in background (-WindowStyle Hidden)" -ForegroundColor Gray
    Write-Host "   • No admin elevation required" -ForegroundColor Gray
    Write-Host "   • Can be removed by deleting shortcut from Startup folder" -ForegroundColor Gray
    
    Write-Host "`n📋 To verify it's working:" -ForegroundColor Cyan
    Write-Host '   1. Log out and log back in' -ForegroundColor Gray
    Write-Host '   2. Or run manually: C:\Users\USER\dchat\docker-startup.ps1' -ForegroundColor Gray
    Write-Host '   3. Check Docker status: docker ps' -ForegroundColor Gray
    
} catch {
    Write-Host "`n❌ Failed to create shortcut: $_" -ForegroundColor Red
    Write-Host "`n💡 Alternative: Run docker-startup.ps1 manually" -ForegroundColor Yellow
    Write-Host "   C:\Users\USER\dchat\docker-startup.ps1" -ForegroundColor Gray
}

# List what's in the startup folder
Write-Host "`n📂 Startup folder contents:" -ForegroundColor Cyan
Get-ChildItem $startupFolder -ErrorAction SilentlyContinue | Select-Object Name | ForEach-Object {
    Write-Host "   • $($_.Name)" -ForegroundColor Gray
}

Write-Host "`n✨ Setup complete!" -ForegroundColor Green
