#!/bin/bash
# Build initialization script for dchat
# Ensures all build dependencies are installed and configured correctly

set -e

echo "🔧 dchat Build Initialization"
echo "=============================="

# Step 1: Update Rust toolchain
echo ""
echo "📦 Step 1: Updating Rust toolchain..."
rustup self update
rustup update 1.82
rustup default 1.82
rustup component add rustfmt clippy

echo "✅ Rust toolchain updated:"
rustc --version
cargo --version

# Step 2: Install system dependencies
echo ""
echo "📦 Step 2: Installing system dependencies..."
if command -v apt-get &> /dev/null; then
    echo "Detected Debian/Ubuntu system"
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        libssl-dev \
        libsqlite3-dev \
        pkg-config \
        curl \
        ca-certificates \
        git
elif command -v yum &> /dev/null; then
    echo "Detected RHEL/CentOS system"
    sudo yum install -y \
        gcc \
        openssl-devel \
        sqlite-devel \
        pkg-config \
        curl \
        ca-certificates \
        git
elif command -v brew &> /dev/null; then
    echo "Detected macOS system"
    brew install openssl sqlite pkg-config
else
    echo "⚠️  Unknown OS. Please install system dependencies manually:"
    echo "  - build-essential/gcc"
    echo "  - libssl-dev/openssl-devel"
    echo "  - libsqlite3-dev/sqlite-devel"
    echo "  - pkg-config"
fi

# Step 3: Clear cargo cache
echo ""
echo "📦 Step 3: Clearing cargo cache..."
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/registry/index
cargo update --aggressive

# Step 4: Verify build environment
echo ""
echo "📦 Step 4: Verifying build environment..."
echo "Cargo version: $(cargo --version)"
echo "Rust version: $(rustc --version)"
echo "OpenSSL version: $(pkg-config --modversion openssl 2>/dev/null || echo 'Not found - will download')"

# Step 5: Build in debug mode first (faster to catch errors)
echo ""
echo "📦 Step 5: Building dchat (debug mode for verification)..."
cargo build --lib 2>&1 | tail -20

echo ""
echo "✅ Build initialization complete!"
echo ""
echo "Next steps:"
echo "  1. For local development: cargo build --release"
echo "  2. For Docker build: docker build -t dchat:latest ."
echo "  3. For deployment: docker-compose -f docker-compose-production.yml up -d"
