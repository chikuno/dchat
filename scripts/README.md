# 📝 Build & Deployment Scripts

This directory contains automated scripts for building, initializing, and verifying the dchat build environment.

## 📋 Scripts Overview

### Build Initialization Scripts

#### `build-init.sh` (Linux/macOS)
Automated build environment initialization.

**Features:**
- Updates Rust to 1.82
- Installs system dependencies
- Clears stale cargo cache
- Validates build configuration

**Usage:**
```bash
./scripts/build-init.sh
```

**What it does:**
1. Checks if Rust is installed
2. Updates Rust to 1.82 (if older)
3. Installs build dependencies (on Linux)
4. Clears cargo cache
5. Runs `cargo build --lib` to verify
6. Reports success/failure

**Time:** ~5 minutes

#### `build-init.ps1` (Windows)
Windows PowerShell version of build initialization.

**Features:**
- Same as build-init.sh but for Windows
- Color-coded output (Cyan/Green/Red)
- Clear progress messages

**Usage:**
```powershell
.\scripts\build-init.ps1
```

**Prerequisites:**
- PowerShell 5.0+
- Git installed
- Rustup installed

**Time:** ~5 minutes

---

### Verification Scripts

#### `verify-build-fixes.sh` (Linux/macOS)
Verifies all build fixes are properly applied.

**Checks:**
1. ✅ rust-toolchain.toml exists with channel 1.82
2. ✅ Dockerfile uses rust:1.82
3. ✅ Dockerfile includes cache clearing
4. ✅ build-init.sh is executable
5. ✅ build-init.ps1 exists
6. ✅ Cargo.toml has compatible versions
7. ✅ Current Rust version is 1.82
8. ✅ Cargo.lock exists
9. ✅ Documentation files present

**Usage:**
```bash
./scripts/verify-build-fixes.sh
```

**Output:**
```
✅ ALL CHECKS PASSED!

Your build environment is correctly configured.

Next steps:
  1. cargo build --release
  2. docker build -t dchat:latest .
  3. docker-compose -f docker-compose-production.yml up -d
```

**Time:** ~1 minute

#### `verify-build-fixes.ps1` (Windows)
Windows PowerShell version of verification.

**Same checks as .sh version**

**Usage:**
```powershell
.\scripts\verify-build-fixes.ps1
```

**Time:** ~1 minute

---

## 🚀 Quick Start Guide

### Option 1: Quick Fix (2 minutes)
```bash
rustup update 1.82
cargo clean
cargo update --aggressive
cargo build --release
```

### Option 2: Full Initialization (5 minutes) - RECOMMENDED
```bash
# Linux/macOS
./scripts/build-init.sh
cargo run --release --bin key-generator

# Windows
.\scripts\build-init.ps1
cargo run --release --bin key-generator
```

### Option 3: Docker (Automatic)
```bash
docker build -t dchat:latest .
docker-compose -f docker-compose-production.yml up -d
```

---

## ✅ Verification Workflow

### Before Deployment
```bash
# 1. Verify all fixes are in place
./scripts/verify-build-fixes.sh

# 2. Check Rust version
rustc --version

# 3. Test clean build
cargo build --lib

# 4. Generate keys
cargo run --release --bin key-generator

# 5. Test Docker
docker build -t dchat:latest .
```

### After Each Step
Look for:
- ✅ No errors in output
- ✅ Completion messages
- ✅ No "edition2024" errors
- ✅ Expected file generation

---

## 🛠️ Common Issues & Solutions

### Issue: "rustup: command not found"
**Solution:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
./scripts/build-init.sh
```

### Issue: Permission denied on .sh scripts
**Solution:**
```bash
chmod +x scripts/*.sh
./scripts/build-init.sh
```

### Issue: Docker not found
**Solution:**
```bash
# Install Docker
docker run hello-world  # Verify installation
docker build -t dchat:latest .
```

### Issue: Cargo lock conflicts
**Solution:**
```bash
rm Cargo.lock
cargo update
./scripts/verify-build-fixes.sh
cargo build --lib
```

---

## 📊 Environment Requirements

### Minimum Versions
- **Rust**: 1.82 (pinned in rust-toolchain.toml)
- **Cargo**: Latest (auto-updated with Rust)
- **Docker**: 20.10+ (for containerization)
- **Docker Compose**: 2.0+ (for orchestration)

### Supported Platforms
- ✅ Linux (Ubuntu, Debian, CentOS)
- ✅ macOS (10.15+)
- ✅ Windows (10/11 with WSL2 or PowerShell)

---

## 📈 Build Performance

**Expected Times:**
- `cargo build --lib`: ~2 minutes (first time)
- `cargo build --lib`: ~30 seconds (incremental)
- `cargo build --release`: ~5 minutes (first time)
- `docker build`: ~10 minutes (first time)
- `docker build`: ~2 minutes (cached)

---

## 🔍 Detailed Script Documentation

### build-init.sh Details

**Steps executed:**
```
1. Check Rust installation
2. Update Rust to 1.82
3. Install dependencies (linux-headers, build-essential, etc.)
4. Clear cargo registry cache
5. Clear cargo index cache
6. Run cargo update --aggressive
7. Test build with cargo build --lib
8. Report final status
```

**Exit codes:**
- 0 = Success
- 1 = Error during setup
- 2 = Build verification failed

### build-init.ps1 Details

**Same steps as .sh but with Windows equivalents:**
```
1. Check Rust installation
2. Update Rust to 1.82
3. Check Docker availability
4. Clear cargo caches
5. Run cargo update --aggressive
6. Test build
7. Report final status
```

**Uses:**
- Admin Check (optional, informational only)
- Color-coded console output
- Proper path handling for Windows

---

## 🎯 Deployment Workflow

### Step 1: Initialize
```bash
./scripts/build-init.sh
```

### Step 2: Verify
```bash
./scripts/verify-build-fixes.sh
```

### Step 3: Build
```bash
cargo build --release
```

### Step 4: Test
```bash
cargo run --release --bin key-generator
```

### Step 5: Containerize
```bash
docker build -t dchat:latest .
```

### Step 6: Deploy
```bash
docker-compose -f docker-compose-production.yml up -d
```

---

## 📚 Related Documentation

- `QUICK_FIX_EDITION2024.md` — Quick reference (2 min)
- `BUILD_FIXES_SUMMARY.md` — Executive summary (5 min)
- `BUILD_FIXES.md` — Technical guide (15 min)
- `BUILD_FIX_COMPLETE_CHECKLIST.md` — Full details (10 min)
- `PRODUCTION_DEPLOYMENT_COMPLETE_GUIDE.md` — Deployment guide (20 min)

---

## 💡 Tips & Best Practices

### For Local Development
```bash
# Use build-init.sh once during setup
./scripts/build-init.sh

# Then for daily work
cargo build --lib           # Fast incremental builds
cargo run --bin key-generator
cargo test                  # Run tests
```

### For CI/CD Pipelines
```bash
# In your CI pipeline, use:
./scripts/build-init.sh     # One-time setup
cargo build --release       # Full release build
docker build -t dchat:latest .
```

### For Docker
```bash
# The Dockerfile automatically applies all fixes
docker build -t dchat:latest .
# All fixes baked in: Rust 1.82, cache clearing, deps pinned
```

---

## ⚙️ Configuration

### Rust Toolchain
Edit `rust-toolchain.toml` to change Rust version (not recommended):
```toml
[toolchain]
channel = "1.82"
components = ["rustfmt", "clippy"]
```

### Build Settings
Modify `Cargo.toml` for custom dependencies (use with caution):
```toml
# Keep these versions to avoid edition2024 errors:
dirs = "4.0"
reqwest = { version = "0.11", ... }
config = "0.13"
```

---

## 🐛 Troubleshooting

### Check logs
```bash
# For detailed build output
RUST_BACKTRACE=1 cargo build --release 2>&1 | tee build.log

# For detailed script output
bash -x scripts/build-init.sh
```

### Clean cache
```bash
# Remove all build artifacts
cargo clean

# Clear cargo registry cache
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/registry/index

# Rebuild from scratch
cargo build --lib
```

### Docker troubleshooting
```bash
# Build without cache
docker build --no-cache -t dchat:latest .

# Check build output
docker build -t dchat:latest . 2>&1 | grep -i error

# Inspect Dockerfile
cat Dockerfile | grep -E "FROM|RUN|COPY"
```

---

## ✨ Support

All scripts include error checking and descriptive messages.

**Getting help:**
1. Run verification script first: `./scripts/verify-build-fixes.sh`
2. Check error messages carefully
3. Review BUILD_FIXES.md troubleshooting section
4. Consult PRODUCTION_DEPLOYMENT_COMPLETE_GUIDE.md

---

**Status**: ✅ All scripts ready for use  
**Last Updated**: 2024  
**Rust Version**: 1.82 (pinned)
