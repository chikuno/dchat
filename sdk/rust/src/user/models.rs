//! User management models and responses

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response for user creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Ed25519 public key (hex-encoded)
    pub public_key: String,
    /// Ed25519 private key (hex-encoded)
    pub private_key: String,
    /// When user was created
    pub created_at: DateTime<Utc>,
    /// Whether confirmed on-chain
    pub on_chain_confirmed: bool,
    /// Transaction ID
    pub tx_id: Option<String>,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Ed25519 public key (hex-encoded)
    pub public_key: String,
    /// When user was created
    pub created_at: DateTime<Utc>,
    /// User reputation score
    pub reputation: u32,
    /// Whether confirmed on-chain
    pub on_chain_confirmed: bool,
}

/// Direct message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessageResponse {
    /// Message ID
    pub message_id: String,
    /// Sender ID
    pub sender_id: String,
    /// Recipient ID
    pub recipient_id: String,
    /// SHA-256 content hash
    pub content_hash: String,
    /// When message was created
    pub created_at: DateTime<Utc>,
    /// Whether confirmed on-chain
    pub on_chain_confirmed: bool,
    /// Transaction ID
    pub tx_id: Option<String>,
}

/// Channel creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelResponse {
    /// Channel ID
    pub channel_id: String,
    /// Channel name
    pub name: String,
    /// Channel description
    pub description: Option<String>,
    /// Creator ID
    pub creator_id: String,
    /// When channel was created
    pub created_at: DateTime<Utc>,
    /// Whether confirmed on-chain
    pub on_chain_confirmed: bool,
    /// Transaction ID
    pub tx_id: Option<String>,
}

/// Channel message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    /// Message ID
    pub message_id: String,
    /// Channel ID
    pub channel_id: String,
    /// Sender ID
    pub sender_id: String,
    /// Message content
    pub content: String,
    /// SHA-256 content hash
    pub content_hash: String,
    /// When message was created
    pub created_at: DateTime<Utc>,
    /// Whether confirmed on-chain
    pub on_chain_confirmed: bool,
}

/// Direct message with decrypted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    /// Message ID
    pub message_id: String,
    /// Sender ID
    pub sender_id: String,
    /// Recipient ID
    pub recipient_id: String,
    /// Message content
    pub content: String,
    /// SHA-256 content hash
    pub content_hash: String,
    /// When message was created
    pub created_at: DateTime<Utc>,
    /// Whether confirmed on-chain
    pub on_chain_confirmed: bool,
}
