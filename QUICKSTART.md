# Quick Start Guide

## 🚀 Getting dchat Running

### Step 1: Verify Your Setup
```powershell
# Run the setup verification script
.\check-setup.ps1
```

This will check:
- ✅ Rust installation
- ✅ Visual Studio Build Tools
- ✅ C++ workload (required for MSVC linker)
- ✅ Project structure

### Step 2: Fix Build Tools (If Needed)

If the check script reports missing C++ workload:

**Option A: Visual Studio Installer (Recommended)**
1. Open "Visual Studio Installer" from Start Menu
2. Click "Modify" on "Build Tools 2022"
3. Check "Desktop development with C++"
4. Click "Install" (will download ~1-2GB)

**Option B: Command Line**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive"
```

### Step 3: Build the Project

**If link.exe is in PATH:**
```powershell
cargo build --all
```

**If not, use Developer PowerShell:**
1. Open "Developer PowerShell for VS 2022" from Start Menu
2. Navigate to dchat directory:
   ```powershell
   cd C:\Users\USER\dchat
   ```
3. Build:
   ```powershell
   cargo build --all
   ```

### Step 4: Run Tests
```powershell
# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p dchat-crypto
cargo test -p dchat-identity
cargo test -p dchat-network
```

### Step 5: Start dchat
```powershell
# Start a user node (interactive)
cargo run --release

# Or with configuration
cargo run --release -- --config ./config.toml
```

---

## 📝 Configuration

Create a `config.toml` file:

```toml
[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/4001"]
bootstrap_nodes = []
enable_mdns = true
enable_upnp = true

[storage]
data_dir = "./dchat_data"
max_message_cache_size = 10000
message_retention_days = 30
enable_backup = true

[crypto]
key_rotation_interval_hours = 168  # 1 week
enable_post_quantum = false

[relay]
enable_relay = false
max_relay_connections = 50
```

---

## 🧪 Testing Individual Components

### Cryptography Tests
```powershell
# Test Noise Protocol
cargo test -p dchat-crypto noise::tests --nocapture

# Test key rotation
cargo test -p dchat-crypto rotation::tests --nocapture

# Test post-quantum schemes
cargo test -p dchat-crypto post_quantum::tests --nocapture
```

### Identity Tests
```powershell
# Test multi-device
cargo test -p dchat-identity device::tests --nocapture

# Test guardian recovery
cargo test -p dchat-identity guardian::tests --nocapture

# Test burner identities
cargo test -p dchat-identity burner::tests --nocapture
```

### Network Tests
```powershell
# Test peer discovery
cargo test -p dchat-network discovery::tests --nocapture

# Test relay infrastructure
cargo test -p dchat-network relay::tests --nocapture
```

### Messaging Tests
```powershell
# Test message ordering
cargo test -p dchat-messaging ordering::tests --nocapture

# Test delivery tracking
cargo test -p dchat-messaging delivery::tests --nocapture
```

### Storage Tests
```powershell
# Test database operations
cargo test -p dchat-storage database::tests --nocapture

# Test deduplication
cargo test -p dchat-storage deduplication::tests --nocapture

# Test encrypted backups
cargo test -p dchat-storage backup::tests --nocapture
```

---

## 🐛 Troubleshooting

### Issue: `link.exe` not found
**Solution:** Use Developer PowerShell for VS 2022 (not regular PowerShell)

### Issue: Compilation takes forever
**Cause:** First build compiles ~225 dependencies
**Solution:** Be patient! Subsequent builds are much faster due to caching.

### Issue: Tests failing
**Check:** 
1. Is SQLite available? (should be auto-installed by sqlx)
2. Are you running from project root?
3. Try: `cargo clean` then rebuild

### Issue: Network tests timing out
**Cause:** Firewall blocking localhost connections
**Solution:** Allow Rust test binaries in Windows Firewall

---

## 📚 Project Structure

```
dchat/
├── crates/
│   ├── dchat-core/         # ✅ Complete - Error handling, types, config
│   ├── dchat-crypto/       # ✅ Complete - Cryptography primitives
│   ├── dchat-identity/     # ✅ Complete - Identity management
│   ├── dchat-network/      # ✅ Complete - P2P networking
│   ├── dchat-messaging/    # ✅ Complete - Message handling
│   └── dchat-storage/      # ✅ Complete - Persistence layer
├── src/
│   ├── main.rs            # Application entry point
│   └── lib.rs             # Library exports
├── Cargo.toml             # Workspace configuration
├── README.md              # Project documentation
├── ARCHITECTURE.md        # System design (34 components)
├── BACKEND_SUMMARY.md     # Implementation summary
├── check-setup.ps1        # Setup verification script
└── QUICKSTART.md          # This file
```

---

## 🎯 What's Implemented

### ✅ Phase 1: Foundation (95% Complete)
- [x] Core infrastructure (errors, types, config, events)
- [x] Cryptography (Noise, Ed25519, key rotation, post-quantum)
- [x] Identity management (multi-device, guardians, burners)
- [x] P2P networking (libp2p, DHT, NAT traversal, relay)
- [x] Messaging (ordering, delivery proofs, offline queues)
- [x] Storage (SQLite, deduplication, encrypted backup)

### 🚧 Phase 2: Integration (Next Steps)
- [ ] Blockchain integration (message ordering on-chain)
- [ ] Relay node rewards (staking, uptime proofs)
- [ ] Governance (DAO voting, moderation)
- [ ] UI (terminal interface, later GUI)

---

## 🔗 Useful Commands

```powershell
# Check project
cargo check --all

# Build release version
cargo build --release --all

# Run with logging
$env:RUST_LOG="debug"; cargo run

# Format code
cargo fmt --all

# Lint code
cargo clippy --all

# Update dependencies
cargo update

# Clean build artifacts
cargo clean

# Build documentation
cargo doc --all --no-deps --open
```

---

## 📖 Next Steps

1. **Complete Visual Studio Build Tools Setup**
2. **Build and test the project**
3. **Read ARCHITECTURE.md** for system design details
4. **Explore individual crate documentation**
5. **Run integration tests**
6. **Start contributing to Phase 2!**

---

**Status**: Backend infrastructure complete and ready to run! 🎉
