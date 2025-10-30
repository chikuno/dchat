//! Blockchain client for parallel chain architecture
//! Supports Chat Chain, Currency Chain, Cross-Chain Bridge, and Tokenomics

pub mod chat_chain;
pub mod client;
pub mod cross_chain;
pub mod currency_chain;
pub mod rpc;
pub mod tokenomics;

pub use chat_chain::{ChatChainClient, ChatChainConfig};
pub use client::BlockchainClient;
pub use cross_chain::{CrossChainBridge, CrossChainTransaction, CrossChainStatus};
pub use currency_chain::{CurrencyChainClient, CurrencyChainConfig};
pub use rpc::{RpcClient, RpcConfig};
pub use tokenomics::{
    TokenomicsManager, TokenSupplyConfig, MintEvent, MintReason, BurnEvent, BurnReason,
    LiquidityPool, DistributionSchedule, RecipientType, TokenomicsStats,
};
