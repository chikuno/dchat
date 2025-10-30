//! Blockchain components for dchat
//!
//! This crate provides on-chain functionality including:
//! - On-chain transaction types for user operations
//! - Channel sharding and state partitioning
//! - Cryptographic dispute resolution
//! - Fork arbitration and consensus recovery
//! - Message consensus pruning with Merkle checkpoints
//! - Insurance fund for economic security

pub mod transactions;
pub mod sharding;
pub mod dispute_resolution;
pub mod pruning;
pub mod insurance_fund;

pub use transactions::{
    Transaction, TransactionType, TransactionStatus, TransactionReceipt,
    RegisterUserTx, SendDirectMessageTx, CreateChannelTx, PostToChannelTx,
    JoinChannelTx, ChannelVisibility,
};
pub use sharding::{ShardManager, ShardId, ShardConfig};
pub use dispute_resolution::{DisputeResolver, DisputeClaim, DisputeStatus};
pub use pruning::{PruningManager, PruningPolicy, MerkleCheckpoint, MerkleProof, NodeType};
pub use insurance_fund::{
    InsuranceFund, InsuranceClaim, ClaimType, ClaimStatus, FundConfiguration,
    FundStatistics, FundTransaction, TransactionType as FundTransactionType,
};
