# dchat Build Setup Verification Script
# This script verifies your build environment is ready for dchat development

Write-Host "dchat Build Environment Verification" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Check Rust installation
Write-Host "[1/3] Verifying Rust installation..." -ForegroundColor Yellow
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (Test-Path $cargoPath) {
    $rustVersion = & $cargoPath --version
    $rustupPath = "$env:USERPROFILE\.cargo\bin\rustup.exe"
    $rustupVersion = if (Test-Path $rustupPath) { & $rustupPath --version } else { "rustup not found" }
    Write-Host "  ✓ Rust is installed" -ForegroundColor Green
    Write-Host "    - $rustVersion" -ForegroundColor Gray
    Write-Host "    - $rustupVersion" -ForegroundColor Gray
    $rustInstalled = $true
} else {
    Write-Host "  ✗ Rust is NOT installed at: $cargoPath" -ForegroundColor Red
    Write-Host "  → Install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check Visual Studio Build Tools
Write-Host ""
Write-Host "[2/3] Verifying Visual Studio Build Tools..." -ForegroundColor Yellow
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $vsInstalls = & $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
    if ($vsInstalls.Count -gt 0) {
        Write-Host "  ✓ Visual Studio Build Tools with C++ workload detected" -ForegroundColor Green
        foreach ($install in $vsInstalls) {
            Write-Host "    - $($install.displayName) ($($install.installationVersion))" -ForegroundColor Gray
        }
    } else {
        Write-Host "  ⚠ Visual Studio installed but C++ workload may be missing" -ForegroundColor Yellow
        Write-Host "  → Ensure 'Desktop development with C++' workload is installed" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ⚠ Visual Studio Build Tools not found via vswhere" -ForegroundColor Yellow
    Write-Host "  → But build tools may still be available in PATH" -ForegroundColor Yellow
}

# Check for project structure
Write-Host ""
Write-Host "[3/3] Checking dchat project structure..." -ForegroundColor Yellow
$requiredCrates = @(
    "crates\dchat-core",
    "crates\dchat-crypto",
    "crates\dchat-identity",
    "crates\dchat-network",
    "crates\dchat-messaging",
    "crates\dchat-storage"
)

$allCratesExist = $true
foreach ($crate in $requiredCrates) {
    if (Test-Path $crate) {
        Write-Host "  ✓ Found $crate" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Missing $crate" -ForegroundColor Red
        $allCratesExist = $false
    }
}

Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

if ($allCratesExist -and $rustInstalled) {
    Write-Host "✅ SETUP VERIFIED - Ready to build!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Quick Start Commands:" -ForegroundColor Cyan
    Write-Host "  cargo build --all              # Build all crates" -ForegroundColor White
    Write-Host "  cargo build --release          # Build release binary" -ForegroundColor White
    Write-Host "  cargo test --all               # Run all tests" -ForegroundColor White
    Write-Host "  cargo run --release            # Run dchat" -ForegroundColor White
    Write-Host "  cargo check --all              # Quick syntax check" -ForegroundColor White
    Write-Host ""
    Write-Host "Rust Toolchain Location:" -ForegroundColor Cyan
    $rustHome = $env:RUSTUP_HOME
    if (-not $rustHome) {
        $rustHome = "$env:USERPROFILE\.rustup"
    }
    Write-Host "  $rustHome" -ForegroundColor Gray
    Write-Host ""
    exit 0
} else {
    Write-Host "❌ SETUP INCOMPLETE" -ForegroundColor Red
    if (-not $rustInstalled) {
        Write-Host "  - Rust is not installed" -ForegroundColor Red
    }
    if (-not $allCratesExist) {
        Write-Host "  - Some project crates are missing" -ForegroundColor Red
    }
    exit 1
}
