#!/usr/bin/env bash
# dchat Build Verification Script
# Checks that all build fixes are properly applied

set -e

echo "🔍 dchat Build Verification"
echo "==========================="
echo ""

ERRORS=0
WARNINGS=0

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check 1: rust-toolchain.toml exists
echo "📋 Check 1: rust-toolchain.toml"
if [ -f "rust-toolchain.toml" ]; then
    CHANNEL=$(grep -o 'channel = "[^"]*"' rust-toolchain.toml | cut -d'"' -f2)
    if [ "$CHANNEL" == "1.82" ]; then
        echo -e "${GREEN}✅${NC} rust-toolchain.toml exists with channel 1.82"
    else
        echo -e "${RED}❌${NC} rust-toolchain.toml has wrong channel: $CHANNEL (need 1.82)"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${RED}❌${NC} rust-toolchain.toml NOT FOUND"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Check 2: Dockerfile uses rust:1.82
echo "📋 Check 2: Dockerfile Rust version"
if grep -q "FROM rust:1.82-bookworm" Dockerfile; then
    echo -e "${GREEN}✅${NC} Dockerfile uses rust:1.82"
else
    echo -e "${RED}❌${NC} Dockerfile does NOT use rust:1.82"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Check 3: Dockerfile includes cargo update
echo "📋 Check 3: Dockerfile cache clearing"
if grep -q "cargo update --aggressive" Dockerfile; then
    echo -e "${GREEN}✅${NC} Dockerfile includes 'cargo update --aggressive'"
else
    echo -e "${YELLOW}⚠️${NC} Dockerfile missing 'cargo update --aggressive'"
    WARNINGS=$((WARNINGS + 1))
fi
echo ""

# Check 4: build-init.sh exists and is executable
echo "📋 Check 4: build-init.sh (Linux/macOS)"
if [ -f "scripts/build-init.sh" ]; then
    if [ -x "scripts/build-init.sh" ]; then
        echo -e "${GREEN}✅${NC} scripts/build-init.sh exists and is executable"
    else
        echo -e "${YELLOW}⚠️${NC} scripts/build-init.sh exists but is NOT executable"
        echo "   Run: chmod +x scripts/build-init.sh"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo -e "${RED}❌${NC} scripts/build-init.sh NOT FOUND"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Check 5: build-init.ps1 exists
echo "📋 Check 5: build-init.ps1 (Windows)"
if [ -f "scripts/build-init.ps1" ]; then
    echo -e "${GREEN}✅${NC} scripts/build-init.ps1 exists"
else
    echo -e "${RED}❌${NC} scripts/build-init.ps1 NOT FOUND"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Check 6: Cargo.toml has compatible versions
echo "📋 Check 6: Cargo.toml dependency versions"
DEPS_OK=0
if grep -q 'dirs = "4.0"' Cargo.toml; then
    echo -e "${GREEN}✅${NC} dirs = 4.0 (compatible)"
    DEPS_OK=$((DEPS_OK + 1))
else
    echo -e "${RED}❌${NC} dirs version wrong (should be 4.0)"
    ERRORS=$((ERRORS + 1))
fi

if grep -q 'reqwest = { version = "0.11"' Cargo.toml; then
    echo -e "${GREEN}✅${NC} reqwest = 0.11 (compatible)"
    DEPS_OK=$((DEPS_OK + 1))
else
    echo -e "${RED}❌${NC} reqwest version wrong (should be 0.11)"
    ERRORS=$((ERRORS + 1))
fi

if grep -q 'config = "0.13"' Cargo.toml; then
    echo -e "${GREEN}✅${NC} config = 0.13 (compatible)"
    DEPS_OK=$((DEPS_OK + 1))
else
    echo -e "${RED}❌${NC} config version wrong (should be 0.13)"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Check 7: Current Rust version
echo "📋 Check 7: Current Rust version"
RUSTC_VERSION=$(rustc --version 2>/dev/null || echo "not installed")
CARGO_VERSION=$(cargo --version 2>/dev/null || echo "not installed")

if echo "$RUSTC_VERSION" | grep -q "1.82"; then
    echo -e "${GREEN}✅${NC} Rust 1.82 active: $RUSTC_VERSION"
else
    echo -e "${YELLOW}⚠️${NC} Rust version: $RUSTC_VERSION"
    echo "   (Can be updated with: rustup update 1.82 && rustup default 1.82)"
    WARNINGS=$((WARNINGS + 1))
fi
echo "   Cargo: $CARGO_VERSION"
echo ""

# Check 8: Cargo.lock exists
echo "📋 Check 8: Cargo.lock"
if [ -f "Cargo.lock" ]; then
    echo -e "${GREEN}✅${NC} Cargo.lock exists"
else
    echo -e "${RED}❌${NC} Cargo.lock NOT FOUND"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Check 9: Documentation files
echo "📋 Check 9: Documentation"
DOCS=0
if [ -f "BUILD_FIXES.md" ]; then
    echo -e "${GREEN}✅${NC} BUILD_FIXES.md"
    DOCS=$((DOCS + 1))
fi
if [ -f "BUILD_FIXES_SUMMARY.md" ]; then
    echo -e "${GREEN}✅${NC} BUILD_FIXES_SUMMARY.md"
    DOCS=$((DOCS + 1))
fi
if [ -f "QUICK_FIX_EDITION2024.md" ]; then
    echo -e "${GREEN}✅${NC} QUICK_FIX_EDITION2024.md"
    DOCS=$((DOCS + 1))
fi
if [ $DOCS -lt 3 ]; then
    echo -e "${YELLOW}⚠️${NC} Some documentation files missing"
    WARNINGS=$((WARNINGS + 1))
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 VERIFICATION SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✅ ALL CHECKS PASSED!${NC}"
    echo ""
    echo "Your build environment is correctly configured."
    echo ""
    echo "Next steps:"
    echo "  1. cargo build --release"
    echo "  2. docker build -t dchat:latest ."
    echo "  3. docker-compose -f docker-compose-production.yml up -d"
    echo ""
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  $WARNINGS warnings found${NC}"
    echo ""
    echo "Your build should work, but some optimizations could be applied:"
    echo "  - Make scripts executable with: chmod +x scripts/*.sh"
    echo "  - Update Rust with: rustup update 1.82"
    echo ""
    exit 0
else
    echo -e "${RED}❌ $ERRORS errors found!${NC}"
    echo -e "${YELLOW}⚠️  $WARNINGS warnings${NC}"
    echo ""
    echo "Please fix the errors above before building."
    echo ""
    echo "To fix automatically, run:"
    echo "  ./scripts/build-init.sh         (Linux/macOS)"
    echo "  ./scripts/build-init.ps1        (Windows)"
    echo ""
    exit 1
fi
