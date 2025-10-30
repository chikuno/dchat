//! dchat Rust SDK
//!
//! Official Rust SDK for building decentralized chat applications with blockchain integration.
//!
//! # Features
//!
//! - Blockchain integration with transaction submission and confirmation
//! - User management with Ed25519 keypairs
//! - Direct messaging with SHA-256 content hashing
//! - Channel creation and posting
//! - Full async/await support
//! - Type-safe API
//!
//! # Quick Start
//!
//! ```no_run
//! use dchat_sdk::blockchain::BlockchainClient;
//! use dchat_sdk::user::UserManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize blockchain client
//!     let blockchain = BlockchainClient::local();
//!     
//!     // Create user manager
//!     let user_manager = UserManager::new(blockchain, "http://localhost:8080".to_string());
//!     
//!     // Create user
//!     let user = user_manager.create_user("alice").await?;
//!     println!("User created: {}", user.user_id);
//!     println!("On-chain confirmed: {}", user.on_chain_confirmed);
//!     
//!     Ok(())
//! }
//! ```

pub mod blockchain;
pub mod crypto;
pub mod user;

pub use blockchain::{BlockchainClient, BlockchainConfig, BlockchainError};
pub use user::UserManager;
pub use crypto::{KeyPair, hash_content};

/// SDK version
pub const VERSION: &str = "0.1.0";
