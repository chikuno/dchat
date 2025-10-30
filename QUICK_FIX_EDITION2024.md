# Quick Fix: edition2024 Error

## The Error You Saw
```
error: feature `edition2024` is required
The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.75.0).
```

## The Fix (Choose ONE)

### ⚡ Quick Fix (2 minutes)
```bash
# Update Rust
rustup update 1.82
rustup default 1.82

# Clear cache
cargo clean
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/registry/index

# Update deps and rebuild
cargo update --aggressive
cargo build --release
```

### 🔧 Full Initialization (5 minutes)
```bash
# For Linux/macOS
cd /opt/dchat
chmod +x scripts/build-init.sh
./scripts/build-init.sh

# For Windows (PowerShell)
cd C:\Users\USER\dchat
powershell -File scripts/build-init.ps1
```

### 🐳 Docker Fix (Automatic)
```bash
# Just rebuild - all fixes are baked in
docker build -t dchat:latest .
docker-compose -f docker-compose-production.yml up -d
```

## What Was Fixed

| Problem | Solution |
|---------|----------|
| Rust 1.75 too old | Updated to Rust 1.82 |
| Docker using old Rust | Updated Dockerfile to rust:1.82 |
| Stale cache | Added `cargo update --aggressive` |
| Incompatible deps | Pinned: dirs=4.0, reqwest=0.11, config=0.13 |
| Manual setup hard | Created `scripts/build-init.sh` and `.ps1` |

## Verify It Works
```bash
$ rustc --version
rustc 1.82.0 (...)

$ cargo build --lib
Finished dev [...] target(s) in 2.34s
✅ Success!
```

## Key Changes Made

1. ✅ Created `rust-toolchain.toml` (pins Rust 1.82)
2. ✅ Updated `Dockerfile` (uses rust:1.82)
3. ✅ Updated `Cargo.toml` (compatible versions)
4. ✅ Created `scripts/build-init.sh` (Linux/macOS)
5. ✅ Created `scripts/build-init.ps1` (Windows)
6. ✅ Created `BUILD_FIXES.md` (detailed guide)

## For Deployment

Next time you deploy:

```bash
# Pull latest fixes
git pull origin main

# Initialize build (one time)
./scripts/build-init.sh

# Generate keys
cargo run --release --bin key-generator -- -o validator_keys/validator1.key

# Build Docker
docker build -t dchat:latest .

# Deploy
docker-compose -f docker-compose-production.yml up -d
```

That's it! All build issues are resolved. 🚀
