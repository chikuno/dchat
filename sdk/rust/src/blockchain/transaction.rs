//! Transaction types for blockchain operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Transaction status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction timed out
    TimedOut,
}

/// Channel visibility enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelVisibility {
    /// Public channel
    Public,
    /// Private channel
    Private,
    /// Token-gated channel
    #[serde(rename = "token_gated")]
    TokenGated,
}

/// Transaction receipt with confirmation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction ID
    pub tx_id: String,
    /// Transaction hash
    pub tx_hash: String,
    /// Whether transaction was successful
    pub success: bool,
    /// Block height (if confirmed)
    pub block_height: Option<u64>,
    /// Block hash (if confirmed)
    pub block_hash: Option<String>,
    /// Timestamp
    pub timestamp: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Register user transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserTx {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Ed25519 public key (hex-encoded)
    pub public_key: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Initial reputation
    #[serde(default = "default_reputation")]
    pub initial_reputation: u32,
}

/// Send direct message transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendDirectMessageTx {
    /// Message ID
    pub message_id: String,
    /// Sender ID
    pub sender_id: String,
    /// Recipient ID
    pub recipient_id: String,
    /// SHA-256 content hash
    pub content_hash: String,
    /// Payload size in bytes
    pub payload_size: usize,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional relay node ID
    pub relay_node_id: Option<String>,
}

/// Create channel transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelTx {
    /// Channel ID
    pub channel_id: String,
    /// Channel name
    pub name: String,
    /// Channel description
    pub description: String,
    /// Creator ID
    pub creator_id: String,
    /// Channel visibility
    pub visibility: ChannelVisibility,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional token requirement
    pub token_requirement: Option<String>,
}

/// Post to channel transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToChannelTx {
    /// Message ID
    pub message_id: String,
    /// Channel ID
    pub channel_id: String,
    /// Sender ID
    pub sender_id: String,
    /// SHA-256 content hash
    pub content_hash: String,
    /// Payload size in bytes
    pub payload_size: usize,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

fn default_reputation() -> u32 {
    100
}
