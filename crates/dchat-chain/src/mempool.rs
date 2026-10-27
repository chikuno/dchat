//! In-memory transaction mempool for block production and gossip ingress

use crate::transactions::Transaction;
use dchat_core::error::{Error, Result};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// In-memory queue of pending transactions awaiting block inclusion
#[derive(Debug, Clone)]
pub struct Mempool {
    pending: Arc<RwLock<VecDeque<Transaction>>>,
    max_capacity: usize,
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl Mempool {
    /// Create a new mempool with specified capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            pending: Arc::new(RwLock::new(VecDeque::with_capacity(max_capacity))),
            max_capacity,
        }
    }

    /// Add a verified transaction to the mempool
    pub fn add_transaction(&self, tx: Transaction) -> Result<()> {
        let mut queue = self
            .pending
            .write()
            .map_err(|_| Error::internal("Mempool lock poisoned"))?;
        // Avoid duplicate transaction IDs
        if queue.iter().any(|t| t.tx_id == tx.tx_id) {
            return Ok(());
        }
        if queue.len() >= self.max_capacity {
            return Err(Error::validation("Mempool capacity exceeded"));
        }
        debug!(
            "Adding transaction {} to mempool (queue len: {})",
            tx.tx_id,
            queue.len() + 1
        );
        queue.push_back(tx);
        Ok(())
    }

    /// Drain transactions up to the supplied gas and transaction limits.
    ///
    /// The caller must provide the runtime's gas estimate; fees are not a gas
    /// measurement. A transaction that exceeds the remaining budget stays queued.
    pub fn drain_valid_transactions<F>(
        &self,
        max_gas: u64,
        max_count: usize,
        mut estimate_gas: F,
    ) -> Result<Vec<Transaction>>
    where
        F: FnMut(&Transaction) -> Option<u64>,
    {
        if max_count == 0 {
            return Ok(Vec::new());
        }

        let mut queue = self
            .pending
            .write()
            .map_err(|_| Error::internal("Mempool lock poisoned"))?;

        let mut drained = Vec::new();
        let mut accumulated_gas: u64 = 0;

        while let Some(tx) = queue.front() {
            let tx_gas = estimate_gas(tx)
                .ok_or_else(|| Error::validation("Unable to estimate transaction gas"))?;
            if tx_gas > max_gas.saturating_sub(accumulated_gas) {
                break;
            }

            let tx = queue
                .pop_front()
                .ok_or_else(|| Error::internal("Mempool queue changed while locked"))?;
            accumulated_gas = accumulated_gas.saturating_add(tx_gas);
            drained.push(tx);

            if drained.len() >= max_count {
                break;
            }
        }

        if !drained.is_empty() {
            info!(
                "Drained {} transactions from mempool (total gas: {})",
                drained.len(),
                accumulated_gas
            );
        }

        Ok(drained)
    }

    /// Return a stable snapshot of pending transactions without reserving or removing them.
    ///
    /// Block producers should use this when a transaction must remain available until
    /// consensus finality; failed proposal attempts must not lose queued transactions.
    pub fn pending_transactions(&self) -> Result<Vec<Transaction>> {
        self.pending
            .read()
            .map(|queue| queue.iter().cloned().collect())
            .map_err(|_| Error::internal("Mempool lock poisoned"))
    }

    /// Return a bounded snapshot suitable for constructing one candidate block.
    pub fn pending_transactions_limited(
        &self,
        max_count: usize,
        max_payload_bytes: usize,
    ) -> Result<Vec<Transaction>> {
        let queue = self
            .pending
            .read()
            .map_err(|_| Error::internal("Mempool lock poisoned"))?;
        let mut snapshot = Vec::new();
        let mut payload_bytes = 0usize;
        for tx in queue.iter() {
            if snapshot.len() >= max_count {
                break;
            }
            let next_size = payload_bytes.saturating_add(tx.payload.len());
            if next_size > max_payload_bytes {
                break;
            }
            payload_bytes = next_size;
            snapshot.push(tx.clone());
        }
        Ok(snapshot)
    }

    /// Remove transactions only after the block containing them has finalized.
    pub fn remove_transactions(&self, tx_ids: &[uuid::Uuid]) -> Result<usize> {
        if tx_ids.is_empty() {
            return Ok(0);
        }

        let tx_ids: std::collections::HashSet<_> = tx_ids.iter().copied().collect();
        let mut queue = self
            .pending
            .write()
            .map_err(|_| Error::internal("Mempool lock poisoned"))?;
        let original_len = queue.len();
        queue.retain(|tx| !tx_ids.contains(&tx.tx_id));
        Ok(original_len - queue.len())
    }

    /// Current number of transactions in mempool
    pub fn len(&self) -> Result<usize> {
        self.pending
            .read()
            .map(|queue| queue.len())
            .map_err(|_| Error::internal("Mempool lock poisoned"))
    }

    /// Whether the mempool is empty
    pub fn is_empty(&self) -> Result<bool> {
        self.len().map(|len| len == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction(payload: &[u8], fee_paid: u64) -> Transaction {
        let mut tx = Transaction::new(
            crate::transactions::TransactionType::SendDirectMessage,
            payload.to_vec(),
        );
        tx.fee_paid = fee_paid;
        tx
    }

    #[test]
    fn zero_transaction_limit_does_not_drain() {
        let mempool = Mempool::default();
        mempool.add_transaction(transaction(b"tx", 1)).unwrap();

        let drained = mempool
            .drain_valid_transactions(10, 0, |_| Some(1))
            .unwrap();

        assert!(drained.is_empty());
        assert_eq!(mempool.len().unwrap(), 1);
    }

    #[test]
    fn over_budget_transaction_stays_queued_and_fee_is_not_used_as_gas() {
        let mempool = Mempool::default();
        mempool
            .add_transaction(transaction(b"tx", 100_000))
            .unwrap();

        let drained = mempool
            .drain_valid_transactions(4, 10, |_| Some(5))
            .unwrap();
        assert!(drained.is_empty());
        assert_eq!(mempool.len().unwrap(), 1);

        let drained = mempool
            .drain_valid_transactions(5, 10, |_| Some(5))
            .unwrap();
        assert_eq!(drained.len(), 1);
        assert!(mempool.is_empty().unwrap());
    }

    #[test]
    fn missing_gas_estimate_preserves_transaction_and_returns_error() {
        let mempool = Mempool::default();
        mempool.add_transaction(transaction(b"tx", 1)).unwrap();

        assert!(mempool.drain_valid_transactions(10, 10, |_| None).is_err());
        assert_eq!(mempool.len().unwrap(), 1);
    }

    #[test]
    fn snapshot_keeps_transactions_until_finalized_ids_are_removed() {
        let mempool = Mempool::default();
        let pending = transaction(b"tx", 1);
        let tx_id = pending.tx_id;
        mempool.add_transaction(pending).unwrap();

        assert_eq!(mempool.pending_transactions().unwrap().len(), 1);
        assert_eq!(mempool.len().unwrap(), 1);

        assert_eq!(mempool.remove_transactions(&[tx_id]).unwrap(), 1);
        assert!(mempool.is_empty().unwrap());
    }

    #[test]
    fn bounded_snapshot_respects_count_and_payload_limits() {
        let mempool = Mempool::default();
        mempool.add_transaction(transaction(b"one", 1)).unwrap();
        mempool.add_transaction(transaction(b"two", 1)).unwrap();

        assert_eq!(mempool.pending_transactions_limited(1, 100).unwrap().len(), 1);
        assert!(mempool.pending_transactions_limited(10, 2).unwrap().is_empty());
        assert_eq!(mempool.len().unwrap(), 2);
    }
}
