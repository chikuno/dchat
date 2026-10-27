//! Balance tracker for currency chain
//!
//! Maintains account balances across transaction history, tracking
//! available funds, staked amounts, delegations, pending unstakes,
//! and unclaimed rewards.

use crate::currency_transaction_parser::{ParsedTransaction, TransactionData};
use crate::currency_transactions::*;
use dchat_core::types::UserId;
use dchat_core::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

fn checked_add(left: u128, right: u128, context: &str) -> Result<u128> {
    left.checked_add(right)
        .ok_or_else(|| Error::validation(format!("{} overflow", context)))
}

fn refresh_total(balance: &mut Balance) -> Result<()> {
    let total = checked_add(balance.available, balance.staked, "Balance total")?;
    let total = checked_add(total, balance.delegated, "Balance total")?;
    let total = checked_add(total, balance.unstaking, "Balance total")?;
    balance.total = checked_add(total, balance.unclaimed_rewards, "Balance total")?;
    Ok(())
}

fn empty_staking_info(account: UserId) -> StakingInfo {
    StakingInfo {
        account,
        total_staked: 0,
        validator_stake: 0,
        relay_stake: 0,
        reputation_stake: 0,
        delegations: Vec::new(),
        pending_unstakes: Vec::new(),
        pending_undelegations: Vec::new(),
    }
}

fn typed_stake_total(info: &StakingInfo) -> Result<u128> {
    let total = checked_add(info.validator_stake, info.relay_stake, "Typed stake total")?;
    checked_add(total, info.reputation_stake, "Typed stake total")
}

fn ensure_stake_consistency(account: UserId, balance: &Balance, info: &StakingInfo) -> Result<()> {
    let pending_unstakes = info
        .pending_unstakes
        .iter()
        .try_fold(0u128, |total, unstake| {
            checked_add(total, unstake.amount, "Pending unstake total")
        })?;
    let pending_undelegations = info
        .pending_undelegations
        .iter()
        .try_fold(0u128, |total, undelegation| {
            checked_add(total, undelegation.amount, "Pending undelegation total")
        })?;
    let pending_returns = checked_add(
        pending_unstakes,
        pending_undelegations,
        "Pending return total",
    )?;
    if info.account != account
        || info.total_staked != typed_stake_total(info)?
        || balance.staked != info.total_staked
        || balance.delegated != delegation_total(info)?
        || balance.unstaking != pending_returns
    {
        return Err(Error::validation(format!(
            "Stake records are inconsistent for account {}",
            info.account.0
        )));
    }
    Ok(())
}

fn delegation_total(info: &StakingInfo) -> Result<u128> {
    info.delegations
        .iter()
        .try_fold(0u128, |total, delegation| {
            checked_add(total, delegation.amount, "Delegation total")
        })
}

fn proportional_slash(amount: u128, total: u128, stakes: [u128; 3]) -> Result<[u128; 3]> {
    if amount > total {
        return Err(Error::validation("Slash amount exceeds staked balance"));
    }
    if total == 0 {
        return Ok([0; 3]);
    }

    let mut reductions = [0; 3];
    let mut allocated = 0;
    for (index, stake) in stakes.iter().enumerate() {
        reductions[index] = amount
            .checked_mul(*stake)
            .ok_or_else(|| Error::validation("Proportional slash calculation overflow"))?
            / total;
        allocated = checked_add(allocated, reductions[index], "Proportional slash total")?;
    }

    let mut remainder = amount - allocated;
    for (index, stake) in stakes.iter().enumerate() {
        let available = *stake - reductions[index];
        let extra = available.min(remainder);
        reductions[index] += extra;
        remainder -= extra;
    }
    if remainder != 0 {
        return Err(Error::validation("Unable to allocate slash amount"));
    }
    Ok(reductions)
}

/// Balance tracker maintaining account state
pub struct BalanceTracker {
    /// Account balances
    balances: Arc<RwLock<HashMap<UserId, Balance>>>,
    /// Staking information per account
    staking_info: Arc<RwLock<HashMap<UserId, StakingInfo>>>,
    /// Total supply in circulation
    total_supply: Arc<RwLock<u128>>,
    /// Total staked across all accounts
    total_staked: Arc<RwLock<u128>>,
}

impl BalanceTracker {
    /// Create new balance tracker
    pub fn new() -> Self {
        Self {
            balances: Arc::new(RwLock::new(HashMap::new())),
            staking_info: Arc::new(RwLock::new(HashMap::new())),
            total_supply: Arc::new(RwLock::new(0)),
            total_staked: Arc::new(RwLock::new(0)),
        }
    }

    /// Process a transaction and update balances
    pub async fn process_transaction(&self, transaction: &ParsedTransaction) -> Result<()> {
        match &transaction.data {
            TransactionData::Transfer(tx) => self.process_transfer(tx).await,
            TransactionData::Stake(tx) => self.process_stake(tx).await,
            TransactionData::Unstake(tx) => self.process_unstake(tx).await,
            TransactionData::Delegate(tx) => self.process_delegate(tx).await,
            TransactionData::Undelegate(tx) => self.process_undelegate(tx).await,
            TransactionData::ClaimRewards(tx) => self.process_claim_rewards(tx).await,
            TransactionData::Slash(tx) => self.process_slash(tx).await,
            TransactionData::BlockReward(tx) => self.process_block_reward(tx).await,
            TransactionData::RelayPayment(tx) => self.process_relay_payment(tx).await,
            TransactionData::ChannelAccess(tx) => self.process_channel_access(tx).await,
        }
    }

    /// Process transfer transaction
    async fn process_transfer(&self, tx: &TransferTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let debit = checked_add(tx.amount, u128::from(tx.fee), "Transfer debit")?;
        let mut sender_balance = balances.get(&tx.from).cloned().unwrap_or_default();
        if sender_balance.available < debit {
            return Err(Error::validation("Insufficient balance for transfer"));
        }

        if tx.from == tx.to {
            sender_balance.available -= u128::from(tx.fee);
            refresh_total(&mut sender_balance)?;
            balances.insert(tx.from, sender_balance);
        } else {
            let mut recipient_balance = balances.get(&tx.to).cloned().unwrap_or_default();
            sender_balance.available -= debit;
            recipient_balance.available =
                checked_add(recipient_balance.available, tx.amount, "Recipient balance")?;
            refresh_total(&mut sender_balance)?;
            refresh_total(&mut recipient_balance)?;
            balances.insert(tx.from, sender_balance);
            balances.insert(tx.to, recipient_balance);
        }

        debug!(
            "Transfer: {} -> {} amount: {} fee: {}",
            tx.from.0, tx.to.0, tx.amount, tx.fee
        );

        Ok(())
    }

    /// Process stake transaction
    async fn process_stake(&self, tx: &StakeTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut staking_info = self.staking_info.write().await;
        let mut total_staked = self.total_staked.write().await;

        let mut balance = balances.get(&tx.staker).cloned().unwrap_or_default();
        let mut info = staking_info
            .get(&tx.staker)
            .cloned()
            .unwrap_or_else(|| empty_staking_info(tx.staker));
        ensure_stake_consistency(tx.staker, &balance, &info)?;
        if balance.available < tx.amount {
            return Err(Error::validation("Insufficient balance for staking"));
        }
        balance.available -= tx.amount;
        balance.staked = checked_add(balance.staked, tx.amount, "Staked balance")?;

        let typed_stake = match tx.stake_type {
            StakeType::Validator => &mut info.validator_stake,
            StakeType::Relay => &mut info.relay_stake,
            StakeType::Reputation => &mut info.reputation_stake,
        };
        *typed_stake = checked_add(*typed_stake, tx.amount, "Stake type balance")?;
        info.total_staked = checked_add(info.total_staked, tx.amount, "Account stake total")?;
        let new_total_staked = checked_add(*total_staked, tx.amount, "Network stake total")?;
        refresh_total(&mut balance)?;

        balances.insert(tx.staker, balance);
        staking_info.insert(tx.staker, info);
        *total_staked = new_total_staked;

        info!(
            "Stake: {} amount: {} type: {:?}",
            tx.staker.0, tx.amount, tx.stake_type
        );

        Ok(())
    }

    /// Process unstake transaction
    async fn process_unstake(&self, tx: &UnstakeTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut staking_info = self.staking_info.write().await;
        let mut total_staked = self.total_staked.write().await;

        let mut balance = balances.get(&tx.unstaker).cloned().unwrap_or_default();
        let mut info = staking_info
            .get(&tx.unstaker)
            .cloned()
            .unwrap_or_else(|| empty_staking_info(tx.unstaker));
        ensure_stake_consistency(tx.unstaker, &balance, &info)?;
        if balance.staked < tx.amount {
            return Err(Error::validation("Insufficient staked balance"));
        }
        let typed_stake = match tx.stake_type {
            StakeType::Validator => &mut info.validator_stake,
            StakeType::Relay => &mut info.relay_stake,
            StakeType::Reputation => &mut info.reputation_stake,
        };
        if *typed_stake < tx.amount {
            return Err(Error::validation(
                "Insufficient balance for the requested stake type",
            ));
        }
        let new_total_staked = total_staked
            .checked_sub(tx.amount)
            .ok_or_else(|| Error::validation("Network stake total is inconsistent"))?;
        balance.staked -= tx.amount;
        balance.unstaking = checked_add(balance.unstaking, tx.amount, "Unstaking balance")?;
        *typed_stake -= tx.amount;
        info.total_staked -= tx.amount;

        info.pending_unstakes.push(PendingUnstake {
            amount: tx.amount,
            stake_type: tx.stake_type,
            unlock_at: tx.unlock_at,
            staked_at: tx.timestamp,
        });
        refresh_total(&mut balance)?;
        balances.insert(tx.unstaker, balance);
        staking_info.insert(tx.unstaker, info);
        *total_staked = new_total_staked;

        info!(
            "Unstake: {} amount: {} type: {:?} unlock_at: {}",
            tx.unstaker.0, tx.amount, tx.stake_type, tx.unlock_at
        );

        Ok(())
    }

    /// Process delegate transaction
    async fn process_delegate(&self, tx: &DelegateTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut staking_info = self.staking_info.write().await;

        let mut balance = balances.get(&tx.delegator).cloned().unwrap_or_default();
        let mut info = staking_info
            .get(&tx.delegator)
            .cloned()
            .unwrap_or_else(|| empty_staking_info(tx.delegator));
        ensure_stake_consistency(tx.delegator, &balance, &info)?;
        if delegation_total(&info)? != balance.delegated {
            return Err(Error::validation(format!(
                "Delegation records are inconsistent for account {}",
                tx.delegator.0
            )));
        }
        if balance.available < tx.amount {
            return Err(Error::validation("Insufficient balance for delegation"));
        }
        balance.available -= tx.amount;
        balance.delegated = checked_add(balance.delegated, tx.amount, "Delegated balance")?;

        info.delegations.push(Delegation {
            validator: tx.validator,
            amount: tx.amount,
            delegated_at: tx.timestamp,
        });
        refresh_total(&mut balance)?;
        balances.insert(tx.delegator, balance);
        staking_info.insert(tx.delegator, info);

        info!(
            "Delegate: {} -> {} amount: {}",
            tx.delegator.0, tx.validator.0, tx.amount
        );

        Ok(())
    }

    /// Process undelegate transaction
    async fn process_undelegate(&self, tx: &UndelegateTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut staking_info = self.staking_info.write().await;

        let mut balance = balances.get(&tx.delegator).cloned().unwrap_or_default();
        let mut info = staking_info
            .get(&tx.delegator)
            .cloned()
            .unwrap_or_else(|| empty_staking_info(tx.delegator));
        ensure_stake_consistency(tx.delegator, &balance, &info)?;
        if delegation_total(&info)? != balance.delegated {
            return Err(Error::validation(format!(
                "Delegation records are inconsistent for account {}",
                tx.delegator.0
            )));
        }
        if balance.delegated < tx.amount {
            return Err(Error::validation("Insufficient delegated balance"));
        }
        let validator_delegation = info
            .delegations
            .iter()
            .filter(|delegation| delegation.validator == tx.validator)
            .try_fold(0u128, |total, delegation| {
                checked_add(total, delegation.amount, "Validator delegation total")
            })?;
        if validator_delegation < tx.amount {
            return Err(Error::validation(
                "Insufficient delegation to the requested validator",
            ));
        }

        balance.delegated -= tx.amount;
        balance.unstaking = checked_add(balance.unstaking, tx.amount, "Unstaking balance")?;
        let mut remaining = tx.amount;
        for delegation in info
            .delegations
            .iter_mut()
            .filter(|delegation| delegation.validator == tx.validator)
        {
            let reduction = delegation.amount.min(remaining);
            delegation.amount -= reduction;
            remaining -= reduction;
            if remaining == 0 {
                break;
            }
        }
        info.delegations.retain(|delegation| delegation.amount > 0);
        info.pending_undelegations.push(PendingUndelegation {
            validator: tx.validator,
            amount: tx.amount,
            unlock_at: tx.unlock_at,
            delegated_at: tx.timestamp,
        });
        refresh_total(&mut balance)?;
        balances.insert(tx.delegator, balance);
        staking_info.insert(tx.delegator, info);

        info!(
            "Undelegate: {} <- {} amount: {} unlock_at: {}",
            tx.delegator.0, tx.validator.0, tx.amount, tx.unlock_at
        );

        Ok(())
    }

    /// Process claim rewards transaction
    async fn process_claim_rewards(&self, tx: &ClaimRewardsTx) -> Result<()> {
        let mut balances = self.balances.write().await;

        let mut balance = balances.get(&tx.claimer).cloned().unwrap_or_default();
        if balance.unclaimed_rewards < tx.amount {
            return Err(Error::validation("Insufficient unclaimed rewards"));
        }
        balance.unclaimed_rewards -= tx.amount;
        balance.available = checked_add(balance.available, tx.amount, "Available balance")?;
        refresh_total(&mut balance)?;
        balances.insert(tx.claimer, balance);

        info!(
            "ClaimRewards: {} amount: {} type: {:?} blocks: {}-{}",
            tx.claimer.0, tx.amount, tx.reward_type, tx.from_height, tx.to_height
        );

        Ok(())
    }

    /// Process slash transaction
    async fn process_slash(&self, tx: &SlashTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut staking_info = self.staking_info.write().await;
        let mut total_staked = self.total_staked.write().await;

        let mut balance = balances
            .get(&tx.validator)
            .cloned()
            .ok_or_else(|| Error::validation("Validator has no balance to slash"))?;
        let mut info = staking_info
            .get(&tx.validator)
            .cloned()
            .ok_or_else(|| Error::validation("Validator has no staking record to slash"))?;
        ensure_stake_consistency(tx.validator, &balance, &info)?;
        let stake_total = typed_stake_total(&info)?;
        let expected_remaining = balance
            .staked
            .checked_sub(tx.slash_amount)
            .ok_or_else(|| Error::validation("Slash amount exceeds staked balance"))?;
        if tx.remaining_stake != expected_remaining {
            return Err(Error::validation(
                "Slash remaining-stake value is inconsistent",
            ));
        }
        let reductions = proportional_slash(
            tx.slash_amount,
            stake_total,
            [
                info.validator_stake,
                info.relay_stake,
                info.reputation_stake,
            ],
        )?;
        let new_total_staked = total_staked
            .checked_sub(tx.slash_amount)
            .ok_or_else(|| Error::validation("Network stake total is inconsistent"))?;

        balance.staked = expected_remaining;
        info.validator_stake -= reductions[0];
        info.relay_stake -= reductions[1];
        info.reputation_stake -= reductions[2];
        info.total_staked = expected_remaining;
        refresh_total(&mut balance)?;
        balances.insert(tx.validator, balance);
        staking_info.insert(tx.validator, info);
        *total_staked = new_total_staked;

        warn!(
            "Slash: {} amount: {} reason: {:?} remaining: {} authorized_by: {}",
            tx.validator.0, tx.slash_amount, tx.reason, tx.remaining_stake, tx.authorized_by
        );

        Ok(())
    }

    /// Process block reward transaction
    async fn process_block_reward(&self, tx: &BlockRewardTx) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut total_supply = self.total_supply.write().await;

        let mut rewards = HashMap::new();
        rewards.insert(tx.proposer, tx.proposer_reward);
        let mut distributed = tx.proposer_reward;
        for (validator, reward) in &tx.validator_rewards {
            let current = rewards.get(validator).copied().unwrap_or_default();
            rewards.insert(
                *validator,
                checked_add(current, *reward, "Account block reward")?,
            );
            distributed = checked_add(distributed, *reward, "Distributed block reward")?;
        }
        if distributed != tx.total_reward {
            return Err(Error::validation(
                "Block reward distribution does not match total reward",
            ));
        }
        let new_total_supply = checked_add(*total_supply, tx.total_reward, "Total supply")?;
        let mut updated_balances = Vec::with_capacity(rewards.len());
        for (account, reward) in rewards {
            let mut balance = balances.get(&account).cloned().unwrap_or_default();
            balance.unclaimed_rewards =
                checked_add(balance.unclaimed_rewards, reward, "Unclaimed rewards")?;
            refresh_total(&mut balance)?;
            updated_balances.push((account, balance));
        }
        for (account, balance) in updated_balances {
            balances.insert(account, balance);
        }

        *total_supply = new_total_supply;

        info!(
            "BlockReward: height {} proposer: {} total: {} proposer_share: {}",
            tx.block_height, tx.proposer.0, tx.total_reward, tx.proposer_reward
        );

        Ok(())
    }

    /// Process relay payment transaction
    async fn process_relay_payment(&self, tx: &RelayPaymentTx) -> Result<()> {
        let mut balances = self.balances.write().await;

        let mut payer_balance = balances.get(&tx.payer).cloned().unwrap_or_default();
        if payer_balance.available < tx.amount {
            return Err(Error::validation("Insufficient balance for relay payment"));
        }
        payer_balance.available -= tx.amount;

        if tx.payer == tx.relay {
            refresh_total(&mut payer_balance)?;
            balances.insert(tx.payer, payer_balance);
        } else {
            let mut relay_balance = balances.get(&tx.relay).cloned().unwrap_or_default();
            relay_balance.available =
                checked_add(relay_balance.available, tx.amount, "Relay balance")?;
            refresh_total(&mut payer_balance)?;
            refresh_total(&mut relay_balance)?;
            balances.insert(tx.payer, payer_balance);
            balances.insert(tx.relay, relay_balance);
        }

        info!(
            "RelayPayment: {} -> {} amount: {} messages: {} bytes: {}",
            tx.payer.0, tx.relay.0, tx.amount, tx.message_count, tx.bytes_relayed
        );

        Ok(())
    }

    /// Process channel access payment transaction
    async fn process_channel_access(&self, tx: &ChannelAccessTx) -> Result<()> {
        let mut balances = self.balances.write().await;

        let mut user_balance = balances.get(&tx.user).cloned().unwrap_or_default();
        if user_balance.available < tx.amount {
            return Err(Error::validation("Insufficient balance for channel access"));
        }
        user_balance.available -= tx.amount;

        if tx.user == tx.creator {
            refresh_total(&mut user_balance)?;
            balances.insert(tx.user, user_balance);
        } else {
            let mut creator_balance = balances.get(&tx.creator).cloned().unwrap_or_default();
            creator_balance.available = checked_add(
                creator_balance.available,
                tx.amount,
                "Channel creator balance",
            )?;
            refresh_total(&mut user_balance)?;
            refresh_total(&mut creator_balance)?;
            balances.insert(tx.user, user_balance);
            balances.insert(tx.creator, creator_balance);
        }

        info!(
            "ChannelAccess: {} -> {} amount: {} channel: {} duration: {} days",
            tx.user.0, tx.creator.0, tx.amount, tx.channel_id, tx.duration_days
        );

        Ok(())
    }

    /// Get balance for an account
    pub async fn get_balance(&self, user_id: UserId) -> Balance {
        self.balances
            .read()
            .await
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get staking info for an account
    pub async fn get_staking_info(&self, user_id: UserId) -> Option<StakingInfo> {
        self.staking_info.read().await.get(&user_id).cloned()
    }

    /// Get total supply
    pub async fn get_total_supply(&self) -> u128 {
        *self.total_supply.read().await
    }

    /// Get total staked
    pub async fn get_total_staked(&self) -> u128 {
        *self.total_staked.read().await
    }

    /// Process completed timelocks (move from unstaking to available)
    pub async fn process_completed_timelocks(
        &self,
        current_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let mut balances = self.balances.write().await;
        let mut staking_info = self.staking_info.write().await;

        let mut updated_balances = Vec::new();
        let mut updated_staking_info = Vec::new();
        for (user_id, existing_info) in staking_info.iter() {
            let mut info = existing_info.clone();
            let mut completed_amount = 0u128;
            let mut pending_unstakes = Vec::with_capacity(info.pending_unstakes.len());
            for unstake in info.pending_unstakes.drain(..) {
                if unstake.unlock_at <= current_time {
                    completed_amount =
                        checked_add(completed_amount, unstake.amount, "Completed timelocks")?;
                } else {
                    pending_unstakes.push(unstake);
                }
            }
            info.pending_unstakes = pending_unstakes;
            let mut pending_undelegations = Vec::with_capacity(info.pending_undelegations.len());
            for undelegation in info.pending_undelegations.drain(..) {
                if undelegation.unlock_at <= current_time {
                    completed_amount =
                        checked_add(completed_amount, undelegation.amount, "Completed timelocks")?;
                } else {
                    pending_undelegations.push(undelegation);
                }
            }
            info.pending_undelegations = pending_undelegations;

            if completed_amount == 0 {
                continue;
            }

            let mut balance = balances.get(user_id).cloned().ok_or_else(|| {
                Error::validation(format!(
                    "Missing balance for completed timelock account {}",
                    user_id.0
                ))
            })?;
            if balance.unstaking < completed_amount {
                return Err(Error::validation(format!(
                    "Unstaking balance is inconsistent for account {}",
                    user_id.0
                )));
            }
            balance.unstaking -= completed_amount;
            balance.available = checked_add(
                balance.available,
                completed_amount,
                "Available balance after timelock",
            )?;
            refresh_total(&mut balance)?;
            updated_balances.push((*user_id, balance));
            updated_staking_info.push((*user_id, info));
        }
        for (user_id, balance) in updated_balances {
            balances.insert(user_id, balance);
        }
        for (user_id, info) in updated_staking_info {
            staking_info.insert(user_id, info);
        }

        Ok(())
    }
}

impl BalanceTracker {
    /// Apply atomic balance transitions for a parsed transaction
    pub async fn apply_transaction(&self, transaction: &ParsedTransaction) -> Result<()> {
        self.process_transaction(transaction).await
    }

    /// Apply the balance transition supported by a chat-chain transaction.
    ///
    /// Currency transfers, staking, rewards, and slashing use `ParsedTransaction`
    /// and `apply_transaction`; they must not be treated as successful no-ops here.
    pub async fn apply_chain_transaction(
        &self,
        tx: &crate::transactions::Transaction,
    ) -> Result<()> {
        use crate::transactions::TransactionType;

        let sender_id = match tx.tx_type {
            TransactionType::SendDirectMessage => {
                let message =
                    serde_json::from_slice::<crate::transactions::SendDirectMessageTx>(&tx.payload)
                        .map_err(|e| {
                            Error::validation(format!(
                                "Invalid direct-message transaction payload: {}",
                                e
                            ))
                        })?;
                message.sender_id
            }
            unsupported => {
                return Err(Error::validation(format!(
                    "Balance application is not supported for transaction type {:?}",
                    unsupported
                )));
            }
        };

        let fee = u128::from(tx.fee_paid);
        if fee == 0 {
            return Ok(());
        }

        let mut balances = self.balances.write().await;
        let available = balances
            .get(&sender_id)
            .map(|balance| balance.available)
            .unwrap_or_default();
        if available < fee {
            return Err(Error::validation(
                "Insufficient balance for transaction fee",
            ));
        }

        let mut sender_balance = balances.get(&sender_id).cloned().unwrap_or_default();
        sender_balance.available -= fee;
        refresh_total(&mut sender_balance)?;
        balances.insert(sender_id, sender_balance);
        Ok(())
    }

    /// Verify balance tracker internal consistency and invariants before committing state root
    pub async fn verify_integrity(&self) -> Result<()> {
        let balances = self.balances.read().await;
        let staking_info = self.staking_info.read().await;
        let total_staked = *self.total_staked.read().await;
        let mut computed_staked: u128 = 0;

        for (user, balance) in balances.iter() {
            let calculated_total = balance
                .available
                .checked_add(balance.staked)
                .and_then(|total| total.checked_add(balance.delegated))
                .and_then(|total| total.checked_add(balance.unstaking))
                .and_then(|total| total.checked_add(balance.unclaimed_rewards))
                .ok_or_else(|| {
                    Error::validation(format!("Balance total overflow for user {}", user.0))
                })?;
            if balance.total != calculated_total {
                return Err(Error::validation(format!(
                    "Balance total invariant violated for user {}: recorded {}, calculated {}",
                    user.0, balance.total, calculated_total
                )));
            }
            computed_staked = computed_staked
                .checked_add(balance.staked)
                .ok_or_else(|| Error::validation("Total staked balance overflow"))?;

            match staking_info.get(user) {
                Some(info) => {
                    if info.account != *user
                        || info.total_staked != typed_stake_total(info)?
                        || balance.staked != info.total_staked
                        || balance.delegated != delegation_total(info)?
                    {
                        return Err(Error::validation(format!(
                            "Staking metadata invariant violated for user {}",
                            user.0
                        )));
                    }

                    let pending_unstakes =
                        info.pending_unstakes
                            .iter()
                            .try_fold(0u128, |total, unstake| {
                                checked_add(total, unstake.amount, "Pending unstake total")
                            })?;
                    let pending_undelegations = info.pending_undelegations.iter().try_fold(
                        0u128,
                        |total, undelegation| {
                            checked_add(total, undelegation.amount, "Pending undelegation total")
                        },
                    )?;
                    if checked_add(
                        pending_unstakes,
                        pending_undelegations,
                        "Pending return total",
                    )? != balance.unstaking
                    {
                        return Err(Error::validation(format!(
                            "Pending return invariant violated for user {}",
                            user.0
                        )));
                    }
                }
                None if balance.staked != 0 || balance.delegated != 0 || balance.unstaking != 0 => {
                    return Err(Error::validation(format!(
                        "Missing staking metadata for user {}",
                        user.0
                    )));
                }
                None => {}
            }
        }

        for (user, info) in staking_info.iter() {
            if !balances.contains_key(user)
                && (info.total_staked != 0
                    || typed_stake_total(info)? != 0
                    || delegation_total(info)? != 0
                    || !info.pending_unstakes.is_empty()
                    || !info.pending_undelegations.is_empty())
            {
                return Err(Error::validation(format!(
                    "Staking metadata has no balance for user {}",
                    user.0
                )));
            }
        }

        if total_staked != computed_staked {
            return Err(Error::validation(format!(
                "Total staked invariant violated: tracker total {}, sum of accounts {}",
                total_staked, computed_staked
            )));
        }

        debug!(
            "✓ Balance tracker state integrity verified: {} accounts checked",
            balances.len()
        );
        Ok(())
    }
}

impl Default for BalanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_transfer() {
        let tracker = BalanceTracker::new();

        let from = UserId(Uuid::new_v4());
        let to = UserId(Uuid::new_v4());

        // Set initial balance
        tracker.balances.write().await.insert(
            from,
            Balance {
                available: 1000,
                staked: 0,
                delegated: 0,
                unstaking: 0,
                unclaimed_rewards: 0,
                total: 1000,
            },
        );

        // Create transfer
        let tx = TransferTx {
            tx_id: Uuid::new_v4(),
            from,
            to,
            amount: 500,
            memo: None,
            fee: 10,
            timestamp: chrono::Utc::now(),
            nonce: 1,
            signature: vec![],
        };

        tracker.process_transfer(&tx).await.unwrap();

        let from_balance = tracker.get_balance(from).await;
        let to_balance = tracker.get_balance(to).await;

        assert_eq!(from_balance.available, 490); // 1000 - 500 - 10
        assert_eq!(to_balance.available, 500);
    }

    #[tokio::test]
    async fn test_stake() {
        let tracker = BalanceTracker::new();

        let staker = UserId(Uuid::new_v4());

        // Set initial balance
        tracker.balances.write().await.insert(
            staker,
            Balance {
                available: 1000,
                staked: 0,
                delegated: 0,
                unstaking: 0,
                unclaimed_rewards: 0,
                total: 1000,
            },
        );

        // Create stake
        let tx = StakeTx {
            tx_id: Uuid::new_v4(),
            staker,
            amount: 500,
            stake_type: StakeType::Validator,
            lock_duration_days: Some(30),
            timestamp: chrono::Utc::now(),
            nonce: 1,
            signature: vec![],
        };

        tracker.process_stake(&tx).await.unwrap();

        let balance = tracker.get_balance(staker).await;
        assert_eq!(balance.available, 500);
        assert_eq!(balance.staked, 500);
        assert_eq!(tracker.get_total_staked().await, 500);
    }

    #[tokio::test]
    async fn transfer_overflow_does_not_mutate_either_account() {
        let tracker = BalanceTracker::new();
        let from = UserId(Uuid::new_v4());
        let to = UserId(Uuid::new_v4());
        tracker.balances.write().await.insert(
            from,
            Balance {
                available: 100,
                total: 100,
                ..Balance::default()
            },
        );
        tracker.balances.write().await.insert(
            to,
            Balance {
                available: u128::MAX,
                total: u128::MAX,
                ..Balance::default()
            },
        );

        let tx = TransferTx {
            tx_id: Uuid::new_v4(),
            from,
            to,
            amount: 1,
            memo: None,
            fee: 0,
            timestamp: chrono::Utc::now(),
            nonce: 1,
            signature: vec![],
        };
        assert!(tracker.process_transfer(&tx).await.is_err());
        assert_eq!(tracker.get_balance(from).await.available, 100);
        assert_eq!(tracker.get_balance(to).await.available, u128::MAX);
    }

    #[tokio::test]
    async fn unstake_rejects_a_different_stake_type_without_mutation() {
        let tracker = BalanceTracker::new();
        let user = UserId(Uuid::new_v4());
        let mut info = empty_staking_info(user);
        info.total_staked = 100;
        info.validator_stake = 100;
        tracker.staking_info.write().await.insert(user, info);
        tracker.balances.write().await.insert(
            user,
            Balance {
                staked: 100,
                total: 100,
                ..Balance::default()
            },
        );
        *tracker.total_staked.write().await = 100;

        let tx = UnstakeTx {
            tx_id: Uuid::new_v4(),
            unstaker: user,
            amount: 10,
            stake_type: StakeType::Relay,
            unlock_at: chrono::Utc::now(),
            timestamp: chrono::Utc::now(),
            nonce: 1,
            signature: vec![],
        };
        assert!(tracker.process_unstake(&tx).await.is_err());
        assert_eq!(tracker.get_balance(user).await.staked, 100);
        assert_eq!(tracker.get_total_staked().await, 100);
        assert_eq!(
            tracker
                .get_staking_info(user)
                .await
                .unwrap()
                .validator_stake,
            100
        );
    }

    #[tokio::test]
    async fn undelegation_tracks_validator_and_unlocks_after_timelock() {
        let tracker = BalanceTracker::new();
        let delegator = UserId(Uuid::new_v4());
        let validator = UserId(Uuid::new_v4());
        let delegated_at = chrono::Utc::now();
        let unlock_at = delegated_at + chrono::Duration::minutes(1);
        let mut info = empty_staking_info(delegator);
        info.delegations.push(Delegation {
            validator,
            amount: 100,
            delegated_at,
        });
        tracker.staking_info.write().await.insert(delegator, info);
        tracker.balances.write().await.insert(
            delegator,
            Balance {
                delegated: 100,
                total: 100,
                ..Balance::default()
            },
        );

        let tx = UndelegateTx {
            tx_id: Uuid::new_v4(),
            delegator,
            validator,
            amount: 60,
            unlock_at,
            timestamp: delegated_at,
            nonce: 1,
            signature: vec![],
        };
        tracker.process_undelegate(&tx).await.unwrap();
        let balance = tracker.get_balance(delegator).await;
        assert_eq!(balance.delegated, 40);
        assert_eq!(balance.unstaking, 60);
        assert_eq!(balance.available, 0);

        tracker
            .process_completed_timelocks(delegated_at)
            .await
            .unwrap();
        assert_eq!(tracker.get_balance(delegator).await.available, 0);
        tracker
            .process_completed_timelocks(unlock_at)
            .await
            .unwrap();
        let balance = tracker.get_balance(delegator).await;
        assert_eq!(balance.delegated, 40);
        assert_eq!(balance.unstaking, 0);
        assert_eq!(balance.available, 60);
        tracker.verify_integrity().await.unwrap();
    }

    #[tokio::test]
    async fn claiming_more_than_unclaimed_rewards_does_not_mint_balance() {
        let tracker = BalanceTracker::new();
        let claimer = UserId(Uuid::new_v4());
        tracker.balances.write().await.insert(
            claimer,
            Balance {
                unclaimed_rewards: 10,
                total: 10,
                ..Balance::default()
            },
        );
        let tx = ClaimRewardsTx {
            tx_id: Uuid::new_v4(),
            claimer,
            reward_type: RewardType::BlockProduction,
            amount: 11,
            from_height: 1,
            to_height: 1,
            timestamp: chrono::Utc::now(),
            nonce: 1,
            signature: vec![],
        };

        assert!(tracker.process_claim_rewards(&tx).await.is_err());
        let balance = tracker.get_balance(claimer).await;
        assert_eq!(balance.available, 0);
        assert_eq!(balance.unclaimed_rewards, 10);
        assert_eq!(balance.total, 10);
    }

    #[tokio::test]
    async fn slash_updates_stake_categories_exactly_and_checks_remaining_amount() {
        let tracker = BalanceTracker::new();
        let validator = UserId(Uuid::new_v4());
        let mut info = empty_staking_info(validator);
        info.total_staked = 100;
        info.validator_stake = 60;
        info.relay_stake = 30;
        info.reputation_stake = 10;
        tracker.staking_info.write().await.insert(validator, info);
        tracker.balances.write().await.insert(
            validator,
            Balance {
                staked: 100,
                total: 100,
                ..Balance::default()
            },
        );
        *tracker.total_staked.write().await = 100;
        let tx = SlashTx {
            validator,
            reason: SlashReason::DoubleSign,
            slash_amount: 10,
            remaining_stake: 89,
            evidence_hash: "evidence".to_string(),
            timestamp: chrono::Utc::now(),
            authorized_by: Uuid::new_v4(),
        };

        assert!(tracker.process_slash(&tx).await.is_err());
        assert_eq!(tracker.get_balance(validator).await.staked, 100);

        let tx = SlashTx {
            remaining_stake: 90,
            ..tx
        };
        tracker.process_slash(&tx).await.unwrap();
        let info = tracker.get_staking_info(validator).await.unwrap();
        assert_eq!(info.validator_stake, 54);
        assert_eq!(info.relay_stake, 27);
        assert_eq!(info.reputation_stake, 9);
        assert_eq!(info.total_staked, 90);
        assert_eq!(tracker.get_total_staked().await, 90);
        tracker.verify_integrity().await.unwrap();
    }

    #[tokio::test]
    async fn block_reward_distribution_must_equal_total_before_minting() {
        let tracker = BalanceTracker::new();
        let proposer = UserId(Uuid::new_v4());
        let tx = BlockRewardTx {
            tx_id: Uuid::new_v4(),
            block_height: 1,
            proposer,
            total_reward: 10,
            proposer_reward: 6,
            validator_rewards: vec![(UserId(Uuid::new_v4()), 3)],
            timestamp: chrono::Utc::now(),
        };

        assert!(tracker.process_block_reward(&tx).await.is_err());
        assert_eq!(tracker.get_total_supply().await, 0);
        assert_eq!(tracker.get_balance(proposer).await.total, 0);
    }

    #[tokio::test]
    async fn chain_transaction_rejects_malformed_and_unsupported_payloads() {
        let tracker = BalanceTracker::new();
        let malformed = crate::transactions::Transaction::new(
            crate::transactions::TransactionType::SendDirectMessage,
            b"not-json".to_vec(),
        );
        assert!(tracker.apply_chain_transaction(&malformed).await.is_err());

        let unsupported = crate::transactions::Transaction::new(
            crate::transactions::TransactionType::CreateChannel,
            b"{}".to_vec(),
        );
        assert!(tracker.apply_chain_transaction(&unsupported).await.is_err());
    }

    #[tokio::test]
    async fn chain_message_fee_is_deducted_atomically() {
        let tracker = BalanceTracker::new();
        let sender = UserId(Uuid::new_v4());
        let message = crate::transactions::SendDirectMessageTx {
            message_id: MessageId(Uuid::new_v4()),
            sender_id: sender,
            recipient_id: UserId(Uuid::new_v4()),
            content_hash: "hash".to_string(),
            timestamp: chrono::Utc::now(),
            payload_size: 1,
            relay_node_id: None,
        };
        let payload = serde_json::to_vec(&message).unwrap();
        let mut tx = crate::transactions::Transaction::new(
            crate::transactions::TransactionType::SendDirectMessage,
            payload,
        );
        tx.fee_paid = 25;

        tracker.balances.write().await.insert(
            sender,
            Balance {
                available: 100,
                total: 100,
                ..Balance::default()
            },
        );
        tracker.apply_chain_transaction(&tx).await.unwrap();

        let balance = tracker.get_balance(sender).await;
        assert_eq!(balance.available, 75);
        assert_eq!(balance.total, 75);
    }

    #[tokio::test]
    async fn chain_message_fee_failure_does_not_create_or_change_balance() {
        let tracker = BalanceTracker::new();
        let sender = UserId(Uuid::new_v4());
        let message = crate::transactions::SendDirectMessageTx {
            message_id: MessageId(Uuid::new_v4()),
            sender_id: sender,
            recipient_id: UserId(Uuid::new_v4()),
            content_hash: "hash".to_string(),
            timestamp: chrono::Utc::now(),
            payload_size: 1,
            relay_node_id: None,
        };
        let mut tx = crate::transactions::Transaction::new(
            crate::transactions::TransactionType::SendDirectMessage,
            serde_json::to_vec(&message).unwrap(),
        );
        tx.fee_paid = 25;

        assert!(tracker.apply_chain_transaction(&tx).await.is_err());
        let balance = tracker.get_balance(sender).await;
        assert_eq!(balance.available, 0);
        assert_eq!(balance.total, 0);
    }
}
