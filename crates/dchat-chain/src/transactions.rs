//! On-chain transaction types for user operations
//!
//! This module defines blockchain transaction types for:
//! - User registration and identity management
//! - Direct messaging and channel communication
//! - Channel creation and access control
//! - Message confirmation and proof-of-delivery

use chrono::{DateTime, Utc};
use dchat_core::types::{ChannelId, MessageId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transaction type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    /// Register a new user identity on-chain
    RegisterUser,
    /// Send a direct message (stores hash/metadata)
    SendDirectMessage,
    /// Create a new channel
    CreateChannel,
    /// Post message to channel
    PostToChannel,
    /// Join a channel
    JoinChannel,
    /// Update user profile
    UpdateProfile,
    /// Submit delivery proof for relay reward
    SubmitDeliveryProof,
    /// Execute a smart contract instruction via WASM runtime
    SmartContractExecution,
    /// Submit a governance vote
    GovernanceVote,
    /// Create a governance proposal
    GovernanceProposal,
    /// Transfer currency-chain tokens
    CurrencyTransfer,
}

/// On-chain user registration transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserTx {
    /// Unique user identifier
    pub user_id: UserId,
    /// Username
    pub username: String,
    /// Ed25519 public key (hex encoded)
    pub public_key: String,
    /// Registration timestamp
    pub timestamp: DateTime<Utc>,
    /// Initial reputation score (default 0)
    pub initial_reputation: i64,
}

/// Replace the public profile fields for a registered user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileTx {
    /// User whose profile is being updated
    pub user_id: UserId,
    /// Unique username
    pub username: String,
    /// Optional display name
    pub display_name: Option<String>,
    /// Content hash of the profile image
    pub avatar_hash: Option<String>,
    /// Optional profile description
    pub bio: Option<String>,
}

/// On-chain direct message transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendDirectMessageTx {
    /// Unique message identifier
    pub message_id: MessageId,
    /// Sender user ID
    pub sender_id: UserId,
    /// Recipient user ID
    pub recipient_id: UserId,
    /// Message content hash (SHA-256, for integrity verification)
    pub content_hash: String,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Encrypted payload size (for relay reward calculation)
    pub payload_size: usize,
    /// Optional relay node that delivered
    pub relay_node_id: Option<String>,
}

/// On-chain channel creation transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelTx {
    /// Unique channel identifier
    pub channel_id: ChannelId,
    /// Channel name
    pub name: String,
    /// Channel description
    pub description: String,
    /// Channel creator user ID
    pub creator_id: UserId,
    /// Channel visibility (public/private/token-gated)
    pub visibility: ChannelVisibility,
    /// Creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Required stake for moderation (if any)
    pub stake_amount: Option<u64>,
}

/// On-chain delivery proof transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitDeliveryProofTx {
    /// Unique message identifier
    pub message_id: MessageId,
    /// Relay node that delivered
    pub relay_peer_id: String,
    /// Recipient user ID
    pub recipient_id: UserId,
    /// Recipient signature (hex-encoded Ed25519 signature)
    pub recipient_signature: String,
    /// Delivery timestamp
    pub timestamp: DateTime<Utc>,
    /// Message content hash (for verification)
    pub content_hash: String,
    /// Relay reward amount (calculated based on message size/priority)
    pub reward_amount: u64,
}

/// Cryptographic delivery evidence submitted by the relay that created the proof.
///
/// The relay's envelope signature authenticates the submitter. The signatures
/// here follow the network delivery-proof format and do not mint relay rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainDeliveryProofTx {
    /// Network-level 32-byte message identifier covered by both signatures
    pub message_id: [u8; 32],
    /// Recipient whose registered key signed the acknowledgment
    pub recipient_id: UserId,
    /// Recipient's registered Ed25519 public key
    pub recipient_public_key: [u8; 32],
    /// Recipient signature over message ID, relay key, and timestamp
    pub recipient_signature: Vec<u8>,
    /// Unix timestamp in seconds, covered by both signatures
    pub timestamp: u64,
    /// Relay signature over message ID, recipient key, recipient signature, and timestamp
    pub relay_signature: Vec<u8>,
}

impl OnChainDeliveryProofTx {
    /// Verify the signatures using the submitting relay's envelope key.
    pub fn verify(&self, relay_public_key: &[u8; 32]) -> Result<(), String> {
        use ed25519_dalek::{Signature, VerifyingKey, Verifier};

        if crate::signed_envelope::address_from_public_key(&self.recipient_public_key)
            != self.recipient_id.0
        {
            return Err("delivery proof recipient does not match its public key".to_string());
        }

        let recipient_key = VerifyingKey::from_bytes(&self.recipient_public_key)
            .map_err(|error| format!("invalid recipient public key: {}", error))?;
        let relay_key = VerifyingKey::from_bytes(relay_public_key)
            .map_err(|error| format!("invalid relay public key: {}", error))?;
        let recipient_signature: [u8; 64] = self
            .recipient_signature
            .as_slice()
            .try_into()
            .map_err(|_| "invalid recipient signature length".to_string())?;
        let relay_signature: [u8; 64] = self
            .relay_signature
            .as_slice()
            .try_into()
            .map_err(|_| "invalid relay signature length".to_string())?;

        let mut recipient_message = Vec::with_capacity(72);
        recipient_message.extend_from_slice(&self.message_id);
        recipient_message.extend_from_slice(relay_public_key);
        recipient_message.extend_from_slice(&self.timestamp.to_le_bytes());
        recipient_key
            .verify(
                &recipient_message,
                &Signature::from_bytes(&recipient_signature),
            )
            .map_err(|error| format!("invalid recipient delivery signature: {}", error))?;

        let mut relay_message = Vec::with_capacity(136);
        relay_message.extend_from_slice(&self.message_id);
        relay_message.extend_from_slice(&self.recipient_public_key);
        relay_message.extend_from_slice(&recipient_signature);
        relay_message.extend_from_slice(&self.timestamp.to_le_bytes());
        relay_key
            .verify(&relay_message, &Signature::from_bytes(&relay_signature))
            .map_err(|error| format!("invalid relay delivery signature: {}", error))
    }
}

/// On-chain channel message transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToChannelTx {
    /// Unique message identifier
    pub message_id: MessageId,
    /// Channel identifier
    pub channel_id: ChannelId,
    /// Sender user ID
    pub sender_id: UserId,
    /// Message content hash (SHA-256)
    pub content_hash: String,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Payload size
    pub payload_size: usize,
}

/// On-chain channel join transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinChannelTx {
    /// Channel identifier
    pub channel_id: ChannelId,
    /// User joining the channel
    pub user_id: UserId,
    /// Join timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional access token (for token-gated channels)
    pub access_token: Option<String>,
}

/// Channel visibility types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelVisibility {
    /// Anyone can join
    Public,
    /// Invite-only
    Private,
    /// Requires specific token/NFT
    TokenGated { token_id: String },
}

/// Transaction status on blockchain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction submitted to mempool
    Pending,
    /// Transaction included in block
    Confirmed {
        block_height: u64,
        block_hash: String,
    },
    /// Transaction failed validation
    Failed { reason: String },
    /// Transaction timed out
    TimedOut,
}

/// Generic blockchain transaction wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction identifier
    pub tx_id: Uuid,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Transaction payload (serialized specific transaction)
    pub payload: Vec<u8>,
    /// Transaction hash (SHA-256 of payload)
    pub tx_hash: String,
    /// Current status
    pub status: TransactionStatus,
    /// Submission timestamp
    pub submitted_at: DateTime<Utc>,
    /// Confirmation timestamp (if confirmed)
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Gas/fee paid
    pub fee_paid: u64,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(tx_type: TransactionType, payload: Vec<u8>) -> Self {
        use sha2::{Digest, Sha256};

        let tx_hash = format!("{:x}", Sha256::digest(&payload));

        Self {
            tx_id: Uuid::new_v4(),
            tx_type,
            payload,
            tx_hash,
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: 0,
        }
    }

    /// Check if transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TransactionStatus::Confirmed { .. })
    }

    /// Check if transaction is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status, TransactionStatus::Pending)
    }

    /// Check if transaction failed
    pub fn is_failed(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Failed { .. } | TransactionStatus::TimedOut
        )
    }
}

/// Transaction receipt after confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction ID
    pub tx_id: Uuid,
    /// Block height where confirmed
    pub block_height: u64,
    /// Block hash
    pub block_hash: String,
    /// Transaction index in block
    pub tx_index: u32,
    /// Gas used
    pub gas_used: u64,
    /// Confirmation timestamp
    pub confirmed_at: DateTime<Utc>,
    /// Success or failure
    pub success: bool,
    /// Optional error message
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_register_user_tx() {
        let user_id = UserId::new();
        let tx = RegisterUserTx {
            user_id: user_id.clone(),
            username: "alice".to_string(),
            public_key: "deadbeef".to_string(),
            timestamp: Utc::now(),
            initial_reputation: 0,
        };

        assert_eq!(tx.user_id, user_id);
        assert_eq!(tx.username, "alice");
    }

    #[test]
    fn test_transaction_status() {
        let payload = b"test_payload".to_vec();
        let tx = Transaction::new(TransactionType::RegisterUser, payload);

        assert!(tx.is_pending());
        assert!(!tx.is_confirmed());
        assert!(!tx.is_failed());
    }

    #[test]
    fn test_channel_visibility() {
        let public_channel = CreateChannelTx {
            channel_id: ChannelId::new(),
            name: "general".to_string(),
            description: "Public channel".to_string(),
            creator_id: UserId::new(),
            visibility: ChannelVisibility::Public,
            timestamp: Utc::now(),
            stake_amount: None,
        };

        assert_eq!(public_channel.visibility, ChannelVisibility::Public);
    }
}
