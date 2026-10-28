use crate::block_hierarchy::{BlockError, CurrencyWorldState, Hash, TxReceipt};
use dchat_chain::Transaction;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_CURRENCY_BLOCK_TRANSACTIONS: usize = 2_500;
const MAX_CURRENCY_BLOCK_PAYLOAD_BYTES: usize = 3 * 1024 * 1024;

/// Complete deterministic transfer block for the independent currency chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyBlock {
    pub height: u64,
    pub previous_hash: [u8; 32],
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<TxReceipt>,
    pub state_root: Hash,
    pub hash: [u8; 32],
}

impl CurrencyBlock {
    /// Execute a bounded transaction snapshot against a candidate copy of currency state.
    pub fn build(
        height: u64,
        previous_hash: [u8; 32],
        transactions: Vec<Transaction>,
        state: &CurrencyWorldState,
        chain_id: u32,
    ) -> Result<(Self, CurrencyWorldState), BlockError> {
        if height == 0 {
            return Err(BlockError::InvalidTransaction(
                "currency block height must be greater than zero".to_string(),
            ));
        }
        Self::validate_transactions(&transactions)?;
        let mut candidate = state.clone();
        let mut receipts = Vec::with_capacity(transactions.len());
        for (index, transaction) in transactions.iter().enumerate() {
            receipts.push(candidate.execute_transfer(
                transaction,
                chain_id,
                u32::try_from(index).map_err(|_| {
                    BlockError::InvalidTransaction(
                        "currency block transaction index overflow".into(),
                    )
                })?,
            )?);
        }
        let state_root = candidate.compute_state_root();
        let mut block = Self {
            height,
            previous_hash,
            transactions,
            receipts,
            state_root,
            hash: [0; 32],
        };
        block.hash = block.calculate_hash()?;
        Ok((block, candidate))
    }

    /// Re-execute and verify a peer-proposed block against the committed pre-state.
    pub fn verify_and_execute(
        &self,
        state: &CurrencyWorldState,
        chain_id: u32,
    ) -> Result<CurrencyWorldState, BlockError> {
        Self::validate_transactions(&self.transactions)?;
        if self.height == 0 || self.receipts.len() != self.transactions.len() {
            return Err(BlockError::InvalidTransaction(
                "invalid currency block height or receipt count".to_string(),
            ));
        }
        let mut candidate = state.clone();
        let mut receipts = Vec::with_capacity(self.transactions.len());
        for (index, transaction) in self.transactions.iter().enumerate() {
            receipts.push(candidate.execute_transfer(
                transaction,
                chain_id,
                u32::try_from(index).map_err(|_| {
                    BlockError::InvalidTransaction(
                        "currency block transaction index overflow".into(),
                    )
                })?,
            )?);
        }
        let state_root = candidate.compute_state_root();
        if receipts != self.receipts || state_root != self.state_root {
            return Err(BlockError::InvalidTransaction(
                "currency block execution commitment mismatch".to_string(),
            ));
        }
        if self.calculate_hash()? != self.hash {
            return Err(BlockError::InvalidTransaction(
                "currency block hash mismatch".to_string(),
            ));
        }
        Ok(candidate)
    }

    pub fn transaction_ids(&self) -> Vec<Uuid> {
        self.transactions
            .iter()
            .map(|transaction| transaction.tx_id)
            .collect()
    }

    fn validate_transactions(transactions: &[Transaction]) -> Result<(), BlockError> {
        if transactions.is_empty() || transactions.len() > MAX_CURRENCY_BLOCK_TRANSACTIONS {
            return Err(BlockError::InvalidTransaction(
                "currency block transaction count is outside allowed bounds".to_string(),
            ));
        }
        let payload_bytes = transactions.iter().try_fold(0usize, |total, transaction| {
            total.checked_add(transaction.payload.len()).ok_or_else(|| {
                BlockError::InvalidTransaction("currency block payload size overflow".to_string())
            })
        })?;
        if payload_bytes > MAX_CURRENCY_BLOCK_PAYLOAD_BYTES {
            return Err(BlockError::InvalidTransaction(
                "currency block payload exceeds size limit".to_string(),
            ));
        }
        Ok(())
    }

    fn calculate_hash(&self) -> Result<[u8; 32], BlockError> {
        let encoded = bincode::serialize(&(
            self.height,
            self.previous_hash,
            &self.transactions,
            &self.receipts,
            self.state_root,
        ))
        .map_err(|error| {
            BlockError::InvalidTransaction(format!(
                "currency block serialization failed: {}",
                error
            ))
        })?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"dchat/currency-block/v1");
        hasher.update(&encoded);
        Ok(*hasher.finalize().as_bytes())
    }
}
