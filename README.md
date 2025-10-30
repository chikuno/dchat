# dchat: Decentralized Chat Application

A Rust-based decentralized chat application combining end-to-end encryption, sovereign identity, and blockchain-enforced message ordering.

## 🏗️ Implementation Status: Phase 1 Complete! ✅

### ✅ Completed Components (100%)

#### Core Infrastructure (`dchat-core`)
- **Error handling system**: Comprehensive error types for all subsystems
- **Type definitions**: Core types for users, channels, messages, reputation
- **Configuration management**: TOML-based configuration with validation
- **Event system**: Event bus with async handlers for system coordination

#### Cryptography (`dchat-crypto`)
- **Key management**: Ed25519 keypairs with secure memory handling
- **Noise Protocol**: End-to-end encrypted sessions with handshake management
- **Digital signatures**: Ed25519 signatures with batch verification
- **Key derivation**: HKDF-based key derivation for different purposes
- **Key rotation**: Automatic key rotation for forward secrecy
- **Post-quantum crypto**: Hybrid classical+PQ schemes (Kyber768 + Falcon)
- **Handshake management**: Multi-peer handshake coordination

#### Identity Management (`dchat-identity`)
- **Identity registry**: User identity creation and management
- **Multi-device support**: Device registration and synchronization
- **Hierarchical key derivation**: BIP-32/44 style key paths
- **Device synchronization**: Gossip-based multi-device sync
- **Guardian system**: M-of-N guardian account recovery with timelocks
- **Burner identities**: Temporary anonymous identities with limits
- **Verification system**: Badge awards and verification proofs

#### Network Layer (`dchat-network`)
- **libp2p integration**: Complete transport stack (TCP/WebSocket → DNS → Noise → Yamux)
- **Peer discovery**: Kademlia DHT for global discovery, mDNS for local networks
- **Channel communication**: Gossipsub pub/sub for scalable message propagation
- **NAT traversal**: UPnP port mapping, TURN fallback, DCUtR hole punching
- **Relay infrastructure**: Incentivized relay nodes with uptime tracking and proof-of-delivery
- **Message routing**: User-to-peer mapping, offline message queueing, circuit-based onion routing
- **Eclipse prevention**: ASN diversity tracking (max 30% from same autonomous system)

#### Messaging System (`dchat-messaging`)
- **Message types**: Direct messages, channel messages, system messages
- **Message ordering**: Blockchain-based sequence numbers for causal ordering
- **Delivery tracking**: Proof-of-delivery system with relay signatures
- **Offline messaging**: Delay-tolerant queue for unavailable recipients
- **Expiration policies**: Duration-based, view-count, after-read expiration
- **Out-of-order handling**: Gap detection and automatic reordering

#### Storage Layer (`dchat-storage`)
- **Database schema**: SQLite schema for users, messages, channels, devices, guardians
- **CRUD operations**: Complete database operations with async SQLx
- **Content deduplication**: BLAKE3-based content addressing with reference counting
- **Lifecycle management**: TTL-based expiration, hot/warm/cold tiering
- **Encrypted backups**: ChaCha20-Poly1305 encrypted backups with integrity checks
- **Automatic cleanup**: Scheduled cleanup of expired messages and old backups

#### Main Application (`src/main.rs`, `src/lib.rs`)
- **DchatApp**: Unified application integrating all crates
- **Prelude module**: Complete public API for easy consumption
- **Event loop**: Coordinated event processing across all subsystems
- **Capability tests**: Built-in demonstrations of end-to-end functionality
- **Graceful lifecycle**: Proper initialization, startup, and shutdown

#### Integration Tests (`tests/`)
- **20+ integration tests**: Cross-crate interaction testing
- **End-to-end tests**: Complete message flows, multi-device sync, guardian recovery
- **System tests**: Event bus, reputation tracking, storage lifecycle
- **Encryption tests**: Full Noise Protocol handshake and message encryption

### 📋 Architecture Overview

The system is designed as a **dual-chain architecture**:
- **Chat Chain**: Identity, messaging, channels, governance, reputation
- **Currency Chain**: Payments, staking, rewards, economics

#### Key Features Implemented

1. **End-to-End Encryption**
   - Noise Protocol (XX pattern) for mutual authentication
   - Rotating ephemeral keys for forward secrecy
   - Post-quantum resistant hybrid schemes

2. **Hierarchical Key Management**
   - BIP-32 style key derivation
   - Per-device, per-conversation, and per-purpose keys
   - Secure memory handling with automatic zeroization

3. **Identity System**
   - Sovereign identity with Ed25519 public keys
   - Multi-device support with sync capabilities
   - Burner identities for privacy

4. **Reputation & Trust**
   - Cryptographically provable reputation scores
   - Separate scores for messaging, governance, and relay activities
   - No linkage to personal data

## 🚀 Getting Started

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **Visual Studio Build Tools** (Windows):
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```
   **Important**: After installation, you need to install the C++ workload:
   - Run "Visual Studio Installer"
   - Modify your Build Tools installation
   - Check "Desktop development with C++"
   - Install the selected components
   
   Alternatively, use the Developer Command Prompt or PowerShell:
   ```powershell
   # Open "Developer PowerShell for VS 2022" from Start Menu
   # Then run cargo commands from there
   ```

### Building

```bash
# Clone the repository
git clone <repository-url>
cd dchat

# Build the project
cargo build --release

# Run tests
cargo test

# Run the application
cargo run
```

### Configuration

The application creates a default configuration file `dchat.toml` on first run:

```toml
[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/udp/0/quic-v1"]
bootstrap_peers = []
max_connections = 100
connection_timeout_ms = 10000
enable_mdns = true
enable_upnp = true

[storage]
data_dir = "./dchat_data"
max_message_cache_size = 10000
message_retention_days = 30
enable_backup = true
backup_interval_hours = 24

[crypto]
key_rotation_interval_hours = 168  # 1 week
max_messages_per_key = 10000
enable_post_quantum = false
noise_protocol_pattern = "Noise_XX_25519_ChaChaPoly_BLAKE2s"

[governance]
voting_period_hours = 168  # 1 week
minimum_stake_for_proposal = 1000
quorum_threshold = 0.1  # 10%
enable_anonymous_voting = true

[relay]
enable_relay = false
max_relay_connections = 50
relay_reward_threshold = 100
uptime_reporting_interval_minutes = 15
stake_amount = 1000
```

## 🧪 Testing

### Cryptography Tests
```bash
# Test core crypto functionality
cargo test -p dchat-crypto

# Test Noise Protocol handshakes
cargo test -p dchat-crypto noise::tests

# Test key rotation
cargo test -p dchat-crypto rotation::tests

# Test post-quantum schemes
cargo test -p dchat-crypto post_quantum::tests
```

### Core System Tests
```bash
# Test core types and error handling
cargo test -p dchat-core

```
dchat/
├── src/
│   └── main.rs                 # Main application entry point
├── crates/
│   ├── dchat-core/             # Core types and utilities
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs        # Error handling
│   │   │   ├── types.rs        # Core type definitions
│   │   │   ├── config.rs       # Configuration management
│   │   │   └── events.rs       # Event system
│   │   └── Cargo.toml
│   ├── dchat-crypto/           # Cryptographic primitives
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── keys.rs         # Key management
│   │   │   ├── noise.rs        # Noise Protocol
│   │   │   ├── signatures.rs   # Digital signatures
│   │   │   ├── kdf.rs          # Key derivation
│   │   │   ├── rotation.rs     # Key rotation
│   │   │   ├── handshake.rs    # Handshake management
│   │   │   └── post_quantum.rs # Post-quantum crypto
│   │   └── Cargo.toml
│   └── dchat-identity/         # Identity management
│       ├── src/
│       │   ├── lib.rs
│       │   ├── identity.rs     # Identity registry
│       │   ├── device.rs       # Multi-device support
│       │   ├── derivation.rs   # Key derivation paths
│       │   ├── sync.rs         # Device synchronization
│       │   ├── guardian.rs     # Account recovery
│       │   ├── verification.rs # Badge system
│       │   └── burner.rs       # Burner identities
│       └── Cargo.toml
├── ARCHITECTURE.md             # Complete system architecture
├── Cargo.toml                  # Workspace configuration
└── README.md                   # This file
```     │   ├── signatures.rs   # Digital signatures
│       │   ├── kdf.rs          # Key derivation
│       │   ├── rotation.rs     # Key rotation
│       │   ├── handshake.rs    # Handshake management
│       │   └── post_quantum.rs # Post-quantum crypto
│       └── Cargo.toml
├── ARCHITECTURE.md             # Complete system architecture
├── Cargo.toml                  # Workspace configuration
└── README.md                   # This file
```

## 🔑 Cryptographic Features

### Noise Protocol Implementation
- **Pattern**: XX (mutual authentication)
- **Curve**: Curve25519 for key exchange
- **Cipher**: ChaCha20-Poly1305 for encryption
- **Hash**: BLAKE2s for hashing

### Key Management
- **Identity Keys**: Long-term Ed25519 keys
- **Ephemeral Keys**: Per-session rotating keys
- **Device Keys**: Per-device derived keys
- **Conversation Keys**: Per-peer derived keys

### Post-Quantum Readiness
- **KEM**: Kyber768 for key encapsulation
- **Signatures**: Falcon512 for post-quantum signatures
- **Hybrid Mode**: Classical + PQ for security transition

## 🛠️ Development

### Adding New Features

1. **Core Types**: Add to `dchat-core/src/types.rs`
2. **Error Handling**: Update `dchat-core/src/error.rs`
3. **Events**: Add to `dchat-core/src/events.rs`
4. **Cryptography**: Extend `dchat-crypto/src/`

### Code Conventions

- **Error Handling**: Use `Result<T>` for all fallible operations
- **Async**: Use `async/await` for I/O operations
- **Logging**: Use `tracing` for structured logging
- **Security**: Never log plaintext keys or sensitive data
- **Testing**: Include comprehensive unit and integration tests
### Phase 1 (Current - 75% Complete)
- ✅ Core cryptographic primitives
- ✅ Key management and rotation
- ✅ Basic type system and error handling
- ✅ Identity management system
- ✅ Multi-device support
- ✅ Guardian-based recovery
- ✅ Burner identities

### Phase 2 (Next)
- [ ] Network layer with libp2p
- [ ] Message handling and ordering
- [ ] Local storage with SQLite
- [ ] Channel management
- [ ] Relay node implementationnversation
- Old keys securely deleted after rotation
- Noise Protocol provides built-in forward secrecy

### Post-Quantum Security
- Hybrid schemes protect against future quantum computers
- Gradual migration path from classical to post-quantum
- Maintains backward compatibility during transition

## 🗺️ Roadmap

### Phase 1 (Current)
- ✅ Core cryptographic primitives
- ✅ Key management and rotation
- ✅ Basic type system and error handling

### Phase 2 (Next)
- [ ] Identity management and registration
- [ ] Network layer with libp2p
- [ ] Message handling and ordering
- [ ] Local storage with SQLite

### Phase 3 (Future)
- [ ] Blockchain integration (chat and currency chains)
- [ ] Governance and voting systems
- [ ] Relay network and incentives
- [ ] User interface (CLI/TUI/GUI)

### Phase 4 (Advanced)
- [ ] Zero-knowledge privacy features
- [ ] Cross-chain bridge
- [ ] Mobile applications
- [ ] Plugin ecosystem

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines and code of conduct.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 📞 Support

For questions, issues, or contributions:
- Create an issue on GitHub
- Join our community discussions
- Read the [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design information

---

**Note**: This is an early-stage implementation. The cryptographic primitives and core infrastructure are functional, but the full messaging system, blockchain integration, and user interface are still under development. See the [ARCHITECTURE.md](ARCHITECTURE.md) file for the complete vision and implementation roadmap.