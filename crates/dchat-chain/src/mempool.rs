//! In-memory transaction mempool for block production and gossip ingress

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use crate::transactions::Transaction;
use dchat_core::error::{Error, Result};
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
        let mut queue = self.pending.write().map_err(|_| Error::internal("Mempool lock poisoned"))?;
        if queue.len() >= self.max_capacity {
            return Err(Error::validation("Mempool capacity exceeded"));
        }
        // Avoid duplicate transaction IDs
        if queue.iter().any(|t| t.tx_id == tx.tx_id) {
            return Ok(());
        }
        debug!("Adding transaction {} to mempool (queue len: {})", tx.tx_id, queue.len() + 1);
        queue.push_back(tx);
        Ok(())
    }

    /// Drain valid transactions up to max_gas or max_count for block production
    pub fn drain_valid_transactions(&self, max_gas: u64, max_count: usize) -> Vec<Transaction> {
        let mut queue = match self.pending.write() {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };

        let mut drained = Vec::new();
        let mut accumulated_gas: u64 = 0;

        while let Some(tx) = queue.pop_front() {
            let tx_gas = tx.fee_paid.max(1);
            if accumulated_gas.saturating_add(tx_gas) > max_gas && !drained.is_empty() {
                // Return transaction back to front of queue
                queue.push_front(tx);
                break;
            }

            accumulated_gas = accumulated_gas.saturating_add(tx_gas);
            drained.push(tx);

            if drained.len() >= max_count {
                break;
            }
        }

        if !drained.is_empty() {
            info!("Drained {} transactions from mempool (total gas: {})", drained.len(), accumulated_gas);
        }

        drained
    }

    /// Current number of transactions in mempool
    pub fn len(&self) -> usize {
        self.pending.read().map(|q| q.len()).unwrap_or(0)
    }

    /// Whether the mempool is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
