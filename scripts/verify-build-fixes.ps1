# dchat Build Verification Script (Windows PowerShell)
# Checks that all build fixes are properly applied

param(
    [switch]$Detailed = $false
)

Write-Host "🔍 dchat Build Verification (Windows)" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

$errors = 0
$warnings = 0

function Write-Check($message, $status, $details = "") {
    if ($status -eq "pass") {
        Write-Host "✅ $message" -ForegroundColor Green
    } elseif ($status -eq "fail") {
        Write-Host "❌ $message" -ForegroundColor Red
        $script:errors++
    } elseif ($status -eq "warn") {
        Write-Host "⚠️  $message" -ForegroundColor Yellow
        $script:warnings++
    }
    if ($details) {
        Write-Host "   $details" -ForegroundColor Gray
    }
}

# Check 1: rust-toolchain.toml exists
Write-Host "📋 Check 1: rust-toolchain.toml" -ForegroundColor Blue
if (Test-Path "rust-toolchain.toml") {
    $content = Get-Content "rust-toolchain.toml" -Raw
    if ($content -match 'channel = "1.82"') {
        Write-Check "rust-toolchain.toml exists with channel 1.82" "pass"
    } else {
        Write-Check "rust-toolchain.toml has wrong channel" "fail" "Expected: 1.82"
    }
} else {
    Write-Check "rust-toolchain.toml NOT FOUND" "fail"
}
Write-Host ""

# Check 2: Dockerfile uses rust:1.82
Write-Host "📋 Check 2: Dockerfile Rust version" -ForegroundColor Blue
if (Test-Path "Dockerfile") {
    $dockerfile = Get-Content "Dockerfile" -Raw
    if ($dockerfile -match "FROM rust:1.82-bookworm") {
        Write-Check "Dockerfile uses rust:1.82" "pass"
    } else {
        Write-Check "Dockerfile does NOT use rust:1.82" "fail"
    }
} else {
    Write-Check "Dockerfile NOT FOUND" "fail"
}
Write-Host ""

# Check 3: Dockerfile includes cargo update
Write-Host "📋 Check 3: Dockerfile cache clearing" -ForegroundColor Blue
if (Test-Path "Dockerfile") {
    if ($dockerfile -match "cargo update --aggressive") {
        Write-Check "Dockerfile includes 'cargo update --aggressive'" "pass"
    } else {
        Write-Check "Dockerfile missing 'cargo update --aggressive'" "warn"
    }
} else {
    Write-Check "Dockerfile NOT FOUND" "fail"
}
Write-Host ""

# Check 4: build-init.sh exists
Write-Host "📋 Check 4: build-init.sh (Linux/macOS)" -ForegroundColor Blue
if (Test-Path "scripts/build-init.sh") {
    Write-Check "scripts/build-init.sh exists" "pass"
} else {
    Write-Check "scripts/build-init.sh NOT FOUND" "fail"
}
Write-Host ""

# Check 5: build-init.ps1 exists
Write-Host "📋 Check 5: build-init.ps1 (Windows)" -ForegroundColor Blue
if (Test-Path "scripts/build-init.ps1") {
    Write-Check "scripts/build-init.ps1 exists" "pass"
} else {
    Write-Check "scripts/build-init.ps1 NOT FOUND" "fail"
}
Write-Host ""

# Check 6: Cargo.toml has compatible versions
Write-Host "📋 Check 6: Cargo.toml dependency versions" -ForegroundColor Blue
if (Test-Path "Cargo.toml") {
    $cargo = Get-Content "Cargo.toml" -Raw
    
    if ($cargo -match 'dirs = "4\.0"') {
        Write-Check "dirs = 4.0 (compatible)" "pass"
    } else {
        Write-Check "dirs version wrong" "fail" "Should be 4.0"
    }
    
    if ($cargo -match 'reqwest.*"0\.11"') {
        Write-Check "reqwest = 0.11 (compatible)" "pass"
    } else {
        Write-Check "reqwest version wrong" "fail" "Should be 0.11"
    }
    
    if ($cargo -match 'config = "0\.13"') {
        Write-Check "config = 0.13 (compatible)" "pass"
    } else {
        Write-Check "config version wrong" "fail" "Should be 0.13"
    }
} else {
    Write-Check "Cargo.toml NOT FOUND" "fail"
}
Write-Host ""

# Check 7: Current Rust version
Write-Host "📋 Check 7: Current Rust version" -ForegroundColor Blue
try {
    $rustc = rustc --version 2>$null
    if ($rustc -match "1\.82") {
        Write-Check "Rust 1.82 active: $rustc" "pass"
    } else {
        Write-Check "Rust version: $rustc" "warn" "Can update with: rustup update 1.82"
    }
} catch {
    Write-Check "Rust not installed" "fail" "Install from: https://rustup.rs/"
}

try {
    $cargo = cargo --version
    Write-Host "   Cargo: $cargo" -ForegroundColor Gray
} catch {
    Write-Check "Cargo not found" "fail"
}
Write-Host ""

# Check 8: Cargo.lock exists
Write-Host "📋 Check 8: Cargo.lock" -ForegroundColor Blue
if (Test-Path "Cargo.lock") {
    Write-Check "Cargo.lock exists" "pass"
} else {
    Write-Check "Cargo.lock NOT FOUND" "fail"
}
Write-Host ""

# Check 9: Documentation files
Write-Host "📋 Check 9: Documentation" -ForegroundColor Blue
$docCount = 0
if (Test-Path "BUILD_FIXES.md") {
    Write-Host "✅ BUILD_FIXES.md" -ForegroundColor Green
    $docCount++
}
if (Test-Path "BUILD_FIXES_SUMMARY.md") {
    Write-Host "✅ BUILD_FIXES_SUMMARY.md" -ForegroundColor Green
    $docCount++
}
if (Test-Path "QUICK_FIX_EDITION2024.md") {
    Write-Host "✅ QUICK_FIX_EDITION2024.md" -ForegroundColor Green
    $docCount++
}
if ($docCount -lt 3) {
    Write-Check "Some documentation files missing" "warn" "$docCount/3 found"
}
Write-Host ""

# Summary
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor White
Write-Host "📊 VERIFICATION SUMMARY" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor White
Write-Host ""

if ($errors -eq 0 -and $warnings -eq 0) {
    Write-Host "✅ ALL CHECKS PASSED!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your build environment is correctly configured." -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. cargo build --release" -ForegroundColor White
    Write-Host "  2. docker build -t dchat:latest ." -ForegroundColor White
    Write-Host "  3. docker-compose -f docker-compose-production.yml up -d" -ForegroundColor White
    Write-Host ""
    exit 0
} elseif ($errors -eq 0) {
    Write-Host "⚠️  $warnings warnings found" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Your build should work, but some optimizations could be applied:" -ForegroundColor Yellow
    Write-Host "  - Update Rust with: rustup update 1.82" -ForegroundColor White
    Write-Host ""
    exit 0
} else {
    Write-Host "❌ $errors errors found!" -ForegroundColor Red
    if ($warnings -gt 0) {
        Write-Host "⚠️  $warnings warnings" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "Please fix the errors above before building." -ForegroundColor Red
    Write-Host ""
    Write-Host "To fix automatically, run:" -ForegroundColor Cyan
    Write-Host "  .\scripts\build-init.ps1" -ForegroundColor White
    Write-Host ""
    exit 1
}
