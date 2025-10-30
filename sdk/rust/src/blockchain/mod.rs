//! Blockchain module

pub mod client;
pub mod transaction;
pub mod chat_chain;
pub mod currency_chain;
pub mod cross_chain;

pub use client::{BlockchainClient, BlockchainConfig, BlockchainError, Result};
pub use transaction::{
    ChannelVisibility, CreateChannelTx, PostToChannelTx, RegisterUserTx,
    SendDirectMessageTx, TransactionReceipt, TransactionStatus,
};
pub use chat_chain::{ChatChainClient, ChatChainTransaction, ChatChainTxType};
pub use currency_chain::{CurrencyChainClient, CurrencyChainTransaction, CurrencyChainTxType, Wallet};
pub use cross_chain::{CrossChainBridge, CrossChainTransaction, CrossChainStatus, CrossChainOperation};
