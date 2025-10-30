# dchat Rust SDK

Official Rust SDK for dchat - a decentralized chat application with blockchain-enforced message ordering and end-to-end encryption.

## Features

- ✅ **Blockchain Integration**: All operations create on-chain transactions
- ✅ **User Management**: Create users with Ed25519 keypairs
- ✅ **Direct Messaging**: Send encrypted direct messages with SHA-256 content hashing
- ✅ **Channel Operations**: Create and post to channels
- ✅ **Transaction Confirmation**: Wait for blockchain confirmations with configurable blocks
- ✅ **Real-time Updates**: WebSocket support for transaction updates
- ✅ **Full Async**: Complete async/await support with Tokio
- ✅ **Type-Safe**: Comprehensive type safety and error handling
- ✅ **Production-Ready**: Error handling, timeouts, retries, caching

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dchat-sdk = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
```

## Quick Start

```rust
use dchat_sdk::blockchain::BlockchainClient;
use dchat_sdk::user::UserManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize blockchain client
    let blockchain = BlockchainClient::local();
    
    // Create user manager
    let user_manager = UserManager::new(blockchain, "http://localhost:8080".to_string());
    
    // Create user
    let user = user_manager.create_user("alice").await?;
    println!("User created: {}", user.user_id);
    println!("On-chain confirmed: {}", user.on_chain_confirmed);
    println!("Transaction ID: {:?}", user.tx_id);
    
    Ok(())
}
```

## Documentation

### BlockchainClient

```rust
use dchat_sdk::blockchain::{BlockchainClient, BlockchainConfig};

// Local development
let blockchain = BlockchainClient::local();

// Custom configuration
let config = BlockchainConfig {
    rpc_url: "https://rpc.dchat.io".to_string(),
    ws_url: Some("wss://ws.dchat.io".to_string()),
    confirmation_blocks: 6,
    confirmation_timeout: 300,
    max_retries: 3,
};
let blockchain = BlockchainClient::new(config);

// Register user
let tx_id = blockchain.register_user(
    "user-id".to_string(),
    "alice".to_string(),
    "ed25519-public-key-hex".to_string(),
).await?;

// Wait for confirmation
let receipt = blockchain.wait_for_confirmation(&tx_id).await?;
println!("Confirmed: {}", receipt.success);
```

### UserManager

```rust
use dchat_sdk::user::UserManager;

let manager = UserManager::new(blockchain, "http://localhost:8080".to_string());

// Create user
let user = manager.create_user("alice").await?;

// Send direct message
let message = manager.send_direct_message(
    &user.user_id,
    "recipient-id",
    "Hello!",
    None,
).await?;

// Create channel
let channel = manager.create_channel(
    &user.user_id,
    "General",
    Some("General discussion"),
).await?;

// Post to channel
let post = manager.post_to_channel(
    &user.user_id,
    &channel.channel_id,
    "Welcome!",
).await?;
```

### Cryptographic Utilities

```rust
use dchat_sdk::crypto::{KeyPair, hash_content};

// Generate key pair
let keypair = KeyPair::generate();
println!("Public: {}", keypair.public_key_hex());

// Sign message
let signature = keypair.sign(b"message");

// Verify signature
keypair.verify(b"message", &signature)?;

// Hash content
let hash = hash_content("message content");
```

## Examples

Run examples with:

```bash
# Complete workflow
cargo run --example complete_workflow

# Blockchain client
cargo run --example blockchain_client

# Crypto operations
cargo run --example crypto_operations
```

## Requirements

- Rust 1.70+
- Tokio runtime for async operations

## Dependencies

- `tokio`: Async runtime
- `serde`/`serde_json`: Serialization
- `reqwest`: HTTP client
- `uuid`: Unique identifiers
- `ed25519-dalek`: Ed25519 cryptography
- `sha2`: SHA-256 hashing
- `chrono`: Date/time handling

## Project Structure

```
sdk/rust/
├── src/
│   ├── lib.rs                     # Main library
│   ├── blockchain/
│   │   ├── mod.rs
│   │   ├── client.rs              # BlockchainClient
│   │   └── transaction.rs         # Transaction types
│   ├── user/
│   │   ├── mod.rs
│   │   ├── manager.rs             # UserManager
│   │   └── models.rs              # Models
│   └── crypto/
│       └── mod.rs                 # Cryptographic utilities
├── examples/
│   ├── complete_workflow.rs
│   ├── blockchain_client.rs
│   └── crypto_operations.rs
├── Cargo.toml
└── README.md
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run clippy
cargo clippy

# Format
cargo fmt
```

## License

See [LICENSE](../../LICENSE) in the main repository.

## Support

- Documentation: See [ARCHITECTURE.md](../../ARCHITECTURE.md)
- Issues: [GitHub Issues](https://github.com/dchat/dchat/issues)
- Discussions: [GitHub Discussions](https://github.com/dchat/dchat/discussions)
