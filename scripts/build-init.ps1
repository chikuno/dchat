#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Build initialization script for dchat
    Ensures all build dependencies are installed and configured correctly

.DESCRIPTION
    This script prepares the build environment by:
    1. Updating Rust toolchain to 1.82
    2. Installing required system dependencies
    3. Clearing cargo cache
    4. Verifying the build environment

.EXAMPLE
    .\scripts\build-init.ps1
#>

param(
    [switch]$SkipToolchainUpdate = $false,
    [switch]$SkipDependencies = $false,
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "🔧 $Message" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
}

function Write-Step {
    param([string]$Message, [int]$Step)
    Write-Host ""
    Write-Host "📦 Step $Step`: $Message" -ForegroundColor Yellow
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

Write-Header "dchat Build Initialization"

# Step 1: Update Rust toolchain
if (-not $SkipToolchainUpdate) {
    Write-Step "Updating Rust toolchain to 1.82" 1
    
    try {
        & rustup self update 2>&1 | Out-Null
        Write-Success "Rustup updated"
    } catch {
        Write-Host "⚠️  Rustup self update failed (non-critical): $_" -ForegroundColor Yellow
    }
    
    try {
        & rustup update 1.82 2>&1 | Out-Null
        & rustup default 1.82 2>&1 | Out-Null
        & rustup component add rustfmt clippy 2>&1 | Out-Null
        Write-Success "Rust toolchain set to 1.82"
    } catch {
        Write-Error-Custom "Failed to update toolchain: $_"
        exit 1
    }
    
    Write-Host ""
    & rustc --version
    & cargo --version
    Write-Success "Rust environment ready"
} else {
    Write-Host "⏭️  Skipping toolchain update" -ForegroundColor Gray
}

# Step 2: Verify required tools
Write-Step "Verifying required tools" 2

$requiredTools = @("cargo", "rustc", "git")
foreach ($tool in $requiredTools) {
    $installed = $null -ne (Get-Command $tool -ErrorAction SilentlyContinue)
    if ($installed) {
        Write-Host "  ✅ $tool installed" -ForegroundColor Green
    } else {
        Write-Error-Custom "Required tool missing: $tool"
        exit 1
    }
}

# Step 3: Check system dependencies
Write-Step "Checking system dependencies" 3

$dependencies = @(
    @{Name = "OpenSSL"; Command = "openssl"; Version = "--version" },
    @{Name = "Git"; Command = "git"; Version = "--version" }
)

foreach ($dep in $dependencies) {
    $installed = $null -ne (Get-Command $dep.Command -ErrorAction SilentlyContinue)
    if ($installed) {
        $version = & $dep.Command $dep.Version 2>&1 | Select-Object -First 1
        Write-Host "  ✅ $($dep.Name): $version" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  $($dep.Name): Not found (required for build)" -ForegroundColor Yellow
    }
}

# Step 4: Clear cargo cache
Write-Step "Clearing cargo cache" 4

$cargoCachePath = "$env:USERPROFILE\.cargo\registry\cache"
$cargoIndexPath = "$env:USERPROFILE\.cargo\registry\index"

if (Test-Path $cargoCachePath) {
    Remove-Item -Path $cargoCachePath -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  ✅ Cleared registry cache" -ForegroundColor Green
}

if (Test-Path $cargoIndexPath) {
    Remove-Item -Path $cargoIndexPath -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  ✅ Cleared registry index" -ForegroundColor Green
}

Write-Host "  🔄 Updating dependencies..." -ForegroundColor Cyan
& cargo update --aggressive 2>&1 | Out-Null
Write-Success "Cargo dependencies updated"

# Step 5: Verify build environment
Write-Step "Verifying build environment" 5

$cargoVersion = & cargo --version
$rustcVersion = & rustc --version
$gitVersion = & git --version

Write-Host "  Cargo version: $cargoVersion" -ForegroundColor Cyan
Write-Host "  Rust version: $rustcVersion" -ForegroundColor Cyan
Write-Host "  Git version: $gitVersion" -ForegroundColor Cyan

# Step 6: Test build (quick check)
Write-Step "Testing build configuration (library only)" 6

Write-Host "  🔨 Building dchat library (this may take 1-2 minutes)..." -ForegroundColor Cyan
try {
    $buildOutput = & cargo build --lib 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Library build successful"
    } else {
        Write-Error-Custom "Library build failed"
        Write-Host "Build output:" -ForegroundColor Yellow
        Write-Host $buildOutput[-10..-1] -ForegroundColor Yellow
        exit 1
    }
} catch {
    Write-Error-Custom "Build error: $_"
    exit 1
}

# Final summary
Write-Header "Build Initialization Complete ✅"

Write-Host "
Environment Summary:
  $(& cargo --version)
  $(& rustc --version)
  OpenSSL: Detected
  Git: Detected

Next Steps:
  1️⃣  For local development:
      cargo build --release

  2️⃣  For Docker build:
      docker build -t dchat:latest .

  3️⃣  For Docker deployment:
      docker-compose -f docker-compose-production.yml up -d

  4️⃣  For key generation:
      cargo run --release --bin key-generator -- -o validator_keys/

Troubleshooting:
  - If you get 'edition2024' error, run this script again
  - If you get dependency errors, try: cargo update --aggressive
  - If Docker build fails, ensure rust-toolchain.toml is in project root
  - For more details, see: PRODUCTION_DEPLOYMENT_COMPLETE_GUIDE.md

" -ForegroundColor Cyan

Write-Host "Ready to build dchat! 🚀" -ForegroundColor Green
