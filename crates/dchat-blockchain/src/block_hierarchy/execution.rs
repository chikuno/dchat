//! Transaction Execution Engine
//!
//! Handles transaction execution within miniblocks with proper state tracking,
//! receipts generation, and deterministic state root computation.

use super::core_types::{
    Address, ExecutionTransaction, Hash, LaneId, Miniblock, MiniblockBody, TxReceipt,
};
use super::lane_sharding::BodyCommitments;
use super::BlockError;
use crate::canonical;
use bincode::Options;
use dchat_chain::chain::{CurrencyGenesisBlock, GenesisBalance};
use dchat_chain::{
    ChannelVisibility, CreateChannelTx, CurrencyTransactionType, EnvelopeDomain,
    EnvelopeVerifier, GovernanceProposalTx, GovernanceTransaction, JoinChannelTx,
    OnChainDeliveryProofTx, PostToChannelTx, RegisterUserTx, SendDirectMessageTx,
    SignedTransactionEnvelope, Transaction as ChainTransaction, TransactionType, TransferTx,
    UnifiedTransactionType, UpdateProfileTx,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum gas per miniblock
pub const MAX_GAS_PER_MINIBLOCK: u64 = 15_000_000;
/// Base gas cost per transaction (re-export from ExecutionTransaction)
pub const BASE_TX_GAS: u64 = ExecutionTransaction::BASE_GAS;
/// Gas cost per byte of tx payload (re-export from ExecutionTransaction)
pub const GAS_PER_BYTE: u64 = ExecutionTransaction::GAS_PER_BYTE;

// ─────────────────────────────────────────────────────────────────────────────
// Account State
// ─────────────────────────────────────────────────────────────────────────────

/// Account state in world state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountState {
    /// Account balance
    pub balance: u64,
    /// Account nonce (for replay protection)
    pub nonce: u64,
    /// Optional code hash (for smart accounts)
    pub code_hash: Option<Hash>,
    /// Storage root (for accounts with storage)
    pub storage_root: Option<Hash>,
}

impl AccountState {
    /// Create new empty account
    pub fn new() -> Self {
        Self::default()
    }

    /// Create account with balance
    pub fn with_balance(balance: u64) -> Self {
        Self {
            balance,
            ..Default::default()
        }
    }

    /// Compute hash of account state
    pub fn hash(&self) -> Hash {
        let bytes = canonical::canonical_serialize(self);
        canonical::domain_hash(b"dchat/account_state/v1", &bytes)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// World State
// ─────────────────────────────────────────────────────────────────────────────

/// World state representing all accounts
#[derive(Debug, Clone)]
pub struct WorldState {
    /// Account states indexed by address
    accounts: BTreeMap<Address, AccountState>,
    /// Deterministic application records for executed chat transactions
    chat_records: BTreeMap<Vec<u8>, Vec<u8>>,
    /// State root (cached)
    state_root: Option<Hash>,
    /// Dirty accounts (modified since last root computation)
    dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatUserRecord {
    public_key: [u8; 32],
    username: String,
    display_name: Option<String>,
    avatar_hash: Option<String>,
    bio: Option<String>,
}

impl WorldState {
    /// Create new empty world state
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
            chat_records: BTreeMap::new(),
            state_root: None,
            dirty: false,
        }
    }

    /// Create from existing accounts
    pub fn from_accounts(accounts: BTreeMap<Address, AccountState>) -> Self {
        Self {
            accounts,
            chat_records: BTreeMap::new(),
            state_root: None,
            dirty: true,
        }
    }

    /// Get account state
    pub fn get_account(&self, address: &Address) -> AccountState {
        self.accounts.get(address).cloned().unwrap_or_default()
    }

    /// Set account state
    pub fn set_account(&mut self, address: Address, state: AccountState) {
        self.accounts.insert(address, state);
        self.dirty = true;
        self.state_root = None;
    }

    /// Get mutable account (creates if missing)
    pub fn get_account_mut(&mut self, address: &Address) -> &mut AccountState {
        self.dirty = true;
        self.state_root = None;
        self.accounts.entry(*address).or_default()
    }

    /// Execute a signed chat transaction and produce its deterministic receipt.
    ///
    /// Invalid envelopes reject the proposed block. Valid envelopes whose
    /// application rules fail produce a failed receipt and still consume nonce.
    pub fn execute_chat_transaction(
        &mut self,
        tx: &ChainTransaction,
        chain_id: u32,
        block_height: u64,
        tx_index: u32,
    ) -> Result<TxReceipt, BlockError> {
        const MAX_ENVELOPE_BYTES: usize = 1024 * 1024;
        const CHAT_BASE_GAS: u64 = 21_000;
        const CHAT_GAS_PER_BYTE: u64 = 16;

        if tx.payload.len() > MAX_ENVELOPE_BYTES {
            return Err(BlockError::InvalidTransaction(
                "signed envelope exceeds the execution size limit".to_string(),
            ));
        }
        let envelope: SignedTransactionEnvelope = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(MAX_ENVELOPE_BYTES as u64)
            .deserialize(&tx.payload)
            .map_err(|error| {
                BlockError::InvalidTransaction(format!("invalid signed envelope: {}", error))
            })?;
        if envelope.domain != EnvelopeDomain::Chat
            || envelope.tx_type != UnifiedTransactionType::Chat(tx.tx_type)
        {
            return Err(BlockError::InvalidTransaction(
                "envelope domain or transaction type does not match".to_string(),
            ));
        }

        let sender = Uuid::from_bytes(envelope.sender);
        let sender_address = chat_user_address(&envelope.sender);
        let previous_nonce = self.get_account(&sender_address).nonce;
        let verifier = EnvelopeVerifier::with_nonces(
            chain_id,
            HashMap::from([(sender, previous_nonce)]),
        );
        verifier.verify(&envelope).map_err(|error| {
            BlockError::InvalidTransaction(format!("signed envelope verification failed: {}", error))
        })?;
        if envelope.nonce != previous_nonce.saturating_add(1) {
            return Err(BlockError::InvalidTransaction(format!(
                "unexpected sender nonce: expected {}, got {}",
                previous_nonce.saturating_add(1),
                envelope.nonce
            )));
        }

        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let transaction_hash = blake3::hash(&hash_input);
        let mut expected_tx_id = [0; 16];
        expected_tx_id.copy_from_slice(&transaction_hash.as_bytes()[..16]);
        if tx.tx_id.as_bytes() != &expected_tx_id
            || tx.tx_hash != format!("0x{}", hex::encode(transaction_hash.as_bytes()))
        {
            return Err(BlockError::InvalidTransaction(
                "transaction ID or hash does not match its signed envelope".to_string(),
            ));
        }

        let gas_used = CHAT_BASE_GAS
            .checked_add((envelope.payload.len() as u64).saturating_mul(CHAT_GAS_PER_BYTE))
            .ok_or_else(|| BlockError::InvalidTransaction("chat transaction gas overflow".into()))?;

        let application_result = self.apply_chat_operation(&envelope, block_height);
        self.get_account_mut(&sender_address).nonce = envelope.nonce;
        let post_state_root = self.compute_state_root();

        let mut receipt = match application_result {
            Ok(()) => TxReceipt::success(tx.tx_id, tx_index, gas_used, post_state_root),
            Err(reason) => {
                let mut receipt = TxReceipt::failure(tx.tx_id, tx_index, &reason);
                receipt.gas_used = gas_used;
                receipt.post_state_root = post_state_root;
                receipt
            }
        };
        receipt.post_state_root = post_state_root;
        Ok(receipt)
    }

    fn apply_chat_operation(
        &mut self,
        envelope: &SignedTransactionEnvelope,
        block_height: u64,
    ) -> Result<(), String> {
        let sender = Uuid::from_bytes(envelope.sender);
        match envelope.tx_type {
            UnifiedTransactionType::Chat(TransactionType::RegisterUser) => {
                let tx: RegisterUserTx = decode_chat_payload(&envelope.payload)?;
                let key = chat_record_key(b"user", tx.user_id.0.as_bytes());
                if tx.user_id.0 != sender
                    || tx.public_key != hex::encode(envelope.public_key)
                    || tx.username.trim().is_empty()
                    || tx.username.len() > 64
                    || tx.initial_reputation != 0
                {
                    return Err("invalid registration fields".to_string());
                }
                let user = ChatUserRecord {
                    public_key: envelope.public_key,
                    username: tx.username,
                    display_name: None,
                    avatar_hash: None,
                    bio: None,
                };
                let value = bincode::serialize(&user).map_err(|error| error.to_string())?;
                self.insert_chat_record(key, value)
            }
            UnifiedTransactionType::Chat(TransactionType::UpdateProfile) => {
                let tx: UpdateProfileTx = decode_chat_payload(&envelope.payload)?;
                if tx.user_id.0 != sender
                    || tx.username.trim().is_empty()
                    || tx.username.len() > 64
                    || tx
                        .display_name
                        .as_ref()
                        .is_some_and(|name| name.len() > 128)
                    || tx.avatar_hash.as_ref().is_some_and(|hash| hash.len() > 128)
                    || tx.bio.as_ref().is_some_and(|bio| bio.len() > 512)
                {
                    return Err("invalid profile update fields".to_string());
                }
                let key = chat_record_key(b"user", tx.user_id.0.as_bytes());
                let mut user: ChatUserRecord = self
                    .chat_records
                    .get(&key)
                    .ok_or_else(|| "profile owner is not registered".to_string())
                    .and_then(|bytes| {
                        bincode::deserialize(bytes)
                            .map_err(|error| format!("invalid user state: {}", error))
                    })?;
                user.username = tx.username;
                user.display_name = tx.display_name;
                user.avatar_hash = tx.avatar_hash;
                user.bio = tx.bio;
                let value = bincode::serialize(&user).map_err(|error| error.to_string())?;
                self.set_chat_record(key, value);
                Ok(())
            }
            UnifiedTransactionType::Chat(TransactionType::SubmitDeliveryProof) => {
                let tx: OnChainDeliveryProofTx = decode_chat_payload(&envelope.payload)?;
                tx.verify(&envelope.public_key)?;
                let recipient_key = chat_record_key(b"user", tx.recipient_id.0.as_bytes());
                let recipient: ChatUserRecord = self
                    .chat_records
                    .get(&recipient_key)
                    .ok_or_else(|| "delivery recipient is not registered".to_string())
                    .and_then(|bytes| {
                        bincode::deserialize(bytes)
                            .map_err(|error| format!("invalid recipient state: {}", error))
                    })?;
                if recipient.public_key != tx.recipient_public_key {
                    return Err("delivery proof key does not match registered recipient".to_string());
                }
                self.insert_chat_record(
                    chat_record_key(b"delivery-proof", &tx.message_id),
                    payload_commitment(&envelope.payload).to_vec(),
                )
            }
            UnifiedTransactionType::Chat(TransactionType::SendDirectMessage) => {
                let tx: SendDirectMessageTx = decode_chat_payload(&envelope.payload)?;
                if tx.sender_id.0 != sender
                    || !self.has_chat_record(b"user", tx.sender_id.0.as_bytes())
                    || !self.has_chat_record(b"user", tx.recipient_id.0.as_bytes())
                {
                    return Err("sender or recipient is not registered".to_string());
                }
                let key = chat_record_key(b"message", tx.message_id.0.as_bytes());
                self.insert_chat_record(key, payload_commitment(&envelope.payload).to_vec())
            }
            UnifiedTransactionType::Chat(TransactionType::CreateChannel) => {
                let tx: CreateChannelTx = decode_chat_payload(&envelope.payload)?;
                if tx.creator_id.0 != sender
                    || !self.has_chat_record(b"user", tx.creator_id.0.as_bytes())
                    || tx.name.trim().is_empty()
                    || tx.name.len() > 128
                    || tx.stake_amount.is_some_and(|stake| stake != 0)
                {
                    return Err("invalid channel creation or unsupported stake".to_string());
                }
                let channel_key = chat_record_key(b"channel", tx.channel_id.0.as_bytes());
                if self.chat_records.contains_key(&channel_key) {
                    return Err("channel already exists".to_string());
                }
                let visibility = bincode::serialize(&(
                    tx.visibility,
                    payload_commitment(&envelope.payload),
                ))
                .map_err(|error| error.to_string())?;
                self.insert_chat_record(channel_key, visibility)?;
                self.insert_chat_record(
                    chat_membership_key(tx.channel_id.0.as_bytes(), tx.creator_id.0.as_bytes()),
                    payload_commitment(&envelope.payload).to_vec(),
                )
            }
            UnifiedTransactionType::Chat(TransactionType::JoinChannel) => {
                let tx: JoinChannelTx = decode_chat_payload(&envelope.payload)?;
                if tx.user_id.0 != sender
                    || !self.has_chat_record(b"user", tx.user_id.0.as_bytes())
                {
                    return Err("joining user is not registered".to_string());
                }
                let channel_key = chat_record_key(b"channel", tx.channel_id.0.as_bytes());
                let (visibility, _channel_commitment): (ChannelVisibility, [u8; 32]) = self
                    .chat_records
                    .get(&channel_key)
                    .ok_or_else(|| "channel does not exist".to_string())
                    .and_then(|bytes| {
                        bincode::deserialize(bytes)
                            .map_err(|error| format!("invalid channel state: {}", error))
                    })?;
                if visibility != ChannelVisibility::Public || tx.access_token.is_some() {
                    return Err("private and token-gated joins are not supported".to_string());
                }
                self.insert_chat_record(
                    chat_membership_key(tx.channel_id.0.as_bytes(), tx.user_id.0.as_bytes()),
                    payload_commitment(&envelope.payload).to_vec(),
                )
            }
            UnifiedTransactionType::Chat(TransactionType::PostToChannel) => {
                let tx: PostToChannelTx = decode_chat_payload(&envelope.payload)?;
                if tx.sender_id.0 != sender
                    || !self.has_chat_record(b"user", tx.sender_id.0.as_bytes())
                {
                    return Err("posting user is not registered".to_string());
                }
                let channel_key = chat_record_key(b"channel", tx.channel_id.0.as_bytes());
                let (visibility, _channel_commitment): (ChannelVisibility, [u8; 32]) = self
                    .chat_records
                    .get(&channel_key)
                    .ok_or_else(|| "channel does not exist".to_string())
                    .and_then(|bytes| {
                        bincode::deserialize(bytes)
                            .map_err(|error| format!("invalid channel state: {}", error))
                    })?;
                if matches!(visibility, ChannelVisibility::TokenGated { .. }) {
                    return Err("token-gated channel posting is not supported".to_string());
                }
                if !self.chat_records.contains_key(&chat_membership_key(
                    tx.channel_id.0.as_bytes(),
                    tx.sender_id.0.as_bytes(),
                )) {
                    return Err("posting user is not a channel member".to_string());
                }
                self.insert_chat_record(
                    chat_record_key(b"message", tx.message_id.0.as_bytes()),
                    payload_commitment(&envelope.payload).to_vec(),
                )
            }
            UnifiedTransactionType::Chat(TransactionType::GovernanceProposal) => {
                let tx: GovernanceTransaction = decode_chat_payload(&envelope.payload)?;
                let GovernanceTransaction::SubmitProposal(proposal) = tx else {
                    return Err("governance proposal envelope must submit a proposal".to_string());
                };
                self.apply_governance_proposal(&proposal, sender)
            }
            UnifiedTransactionType::Chat(TransactionType::GovernanceVote) => {
                let tx: GovernanceTransaction = decode_chat_payload(&envelope.payload)?;
                self.apply_governance_vote(tx, sender, block_height)
            }
            UnifiedTransactionType::Chat(unsupported) => Err(format!(
                "chat transaction type {:?} has no deterministic handler",
                unsupported
            )),
            UnifiedTransactionType::Currency(_) => {
                Err("currency transaction cannot execute on the chat chain".to_string())
            }
        }
    }

    fn apply_governance_proposal(
        &mut self,
        proposal: &GovernanceProposalTx,
        sender: Uuid,
    ) -> Result<(), String> {
        if proposal.proposer.0 != sender
            || !self.has_chat_record(b"user", proposal.proposer.0.as_bytes())
            || proposal.title.trim().is_empty()
            || proposal.voting_end_height <= proposal.voting_start_height
            || proposal.quorum_required_bps > 10_000
            || proposal.approval_required_bps > 10_000
        {
            return Err("invalid governance proposal".to_string());
        }
        self.insert_chat_record(
            chat_record_key(b"proposal", proposal.proposal_id.as_bytes()),
            bincode::serialize(proposal).map_err(|error| error.to_string())?,
        )
    }

    fn apply_governance_vote(
        &mut self,
        tx: GovernanceTransaction,
        sender: Uuid,
        block_height: u64,
    ) -> Result<(), String> {
        match tx {
            GovernanceTransaction::CommitVote(vote) => {
                if vote.voter.0 != sender
                    || !self.has_chat_record(b"user", vote.voter.0.as_bytes())
                {
                    return Err("governance voter is not registered".to_string());
                }
                let proposal: GovernanceProposalTx = self.governance_proposal(vote.proposal_id)?;
                if block_height < proposal.voting_start_height
                    || block_height > proposal.voting_end_height
                {
                    return Err("vote commitment is outside the voting window".to_string());
                }
                self.insert_chat_record(
                    governance_vote_key(b"commit", vote.proposal_id, vote.voter.0.as_bytes()),
                    bincode::serialize(&vote).map_err(|error| error.to_string())?,
                )
            }
            GovernanceTransaction::RevealVote(vote) => {
                if vote.voter.0 != sender
                    || !self.has_chat_record(b"user", vote.voter.0.as_bytes())
                {
                    return Err("governance voter is not registered".to_string());
                }
                let proposal: GovernanceProposalTx = self.governance_proposal(vote.proposal_id)?;
                if block_height <= proposal.voting_end_height || vote.vote_choice > 2 {
                    return Err("vote reveal is invalid or premature".to_string());
                }
                let commit_key =
                    governance_vote_key(b"commit", vote.proposal_id, vote.voter.0.as_bytes());
                let commitment: dchat_chain::GovernanceVoteCommitTx = self
                    .chat_records
                    .get(&commit_key)
                    .ok_or_else(|| "vote reveal has no commitment".to_string())
                    .and_then(|bytes| {
                        bincode::deserialize(bytes)
                            .map_err(|error| format!("invalid vote commitment: {}", error))
                    })?;
                if !dchat_chain::verify_vote_reveal(
                    &commitment.commitment,
                    vote.vote_choice,
                    &vote.salt,
                ) {
                    return Err("vote reveal does not match its commitment".to_string());
                }
                self.insert_chat_record(
                    governance_vote_key(b"reveal", vote.proposal_id, vote.voter.0.as_bytes()),
                    payload_commitment(
                        &bincode::serialize(&vote).map_err(|error| error.to_string())?,
                    )
                    .to_vec(),
                )
            }
            _ => Err("unsupported governance vote operation".to_string()),
        }
    }

    fn governance_proposal(&self, proposal_id: Uuid) -> Result<GovernanceProposalTx, String> {
        self.chat_records
            .get(&chat_record_key(b"proposal", proposal_id.as_bytes()))
            .ok_or_else(|| "governance proposal does not exist".to_string())
            .and_then(|bytes| {
                bincode::deserialize(bytes)
                    .map_err(|error| format!("invalid governance proposal: {}", error))
            })
    }

    fn has_chat_record(&self, namespace: &[u8], id: &[u8]) -> bool {
        self.chat_records.contains_key(&chat_record_key(namespace, id))
    }

    fn insert_chat_record(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String> {
        if self.chat_records.contains_key(&key) {
            return Err("chat record already exists".to_string());
        }
        self.set_chat_record(key, value);
        Ok(())
    }

    fn set_chat_record(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.chat_records.insert(key, value);
        self.dirty = true;
        self.state_root = None;
    }

    /// Compute state root (Merkle root of accounts and chat application records)
    pub fn compute_state_root(&mut self) -> Hash {
        if !self.dirty {
            if let Some(root) = self.state_root {
                return root;
            }
        }

        // Build sorted Merkle tree of account hashes
        let mut leaves: Vec<Hash> = self
            .accounts
            .iter()
            .map(|(addr, state)| {
                let mut data = addr.to_vec();
                data.extend_from_slice(state.hash().as_bytes());
                canonical::domain_hash(b"dchat/account_leaf/v1", &data)
            })
            .collect();

        leaves.extend(self.chat_records.iter().map(|(key, value)| {
            canonical::domain_hash_parts(b"dchat/chat_record/v1", &[key, value])
        }));

        let root = compute_merkle_root(&mut leaves);
        self.state_root = Some(root);
        self.dirty = false;
        root
    }

    fn decode_chat_payload<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(1024 * 1024)
            .deserialize(bytes)
            .map_err(|error| format!("invalid chat transaction payload: {}", error))
    }

    fn chat_user_address(user_id: &[u8; 16]) -> Address {
        *canonical::domain_hash(b"dchat/chat/user_address/v1", user_id).as_bytes()
    }

    fn chat_record_key(namespace: &[u8], id: &[u8]) -> Vec<u8> {
        let mut key = Vec::with_capacity(namespace.len() + id.len() + 1);
        key.extend_from_slice(namespace);
        key.push(b'/');
        key.extend_from_slice(id);
        key
    }

    fn chat_membership_key(channel_id: &[u8], user_id: &[u8]) -> Vec<u8> {
        let mut key = chat_record_key(b"membership", channel_id);
        key.extend_from_slice(user_id);
        key
    }

    fn governance_vote_key(kind: &[u8], proposal_id: Uuid, voter_id: &[u8]) -> Vec<u8> {
        let mut key = chat_record_key(kind, proposal_id.as_bytes());
        key.extend_from_slice(voter_id);
        key
    }

    fn payload_commitment(payload: &[u8]) -> [u8; 32] {
        *blake3::hash(payload).as_bytes()
    }

    /// Get current state root (may be stale)
    pub fn state_root(&self) -> Option<Hash> {
        self.state_root
    }

    /// Account count
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Create snapshot for parallel execution
    pub fn snapshot(&self) -> WorldStateSnapshot {
        WorldStateSnapshot {
            accounts: self.accounts.clone(),
        }
    }

    /// Apply changes from snapshot
    pub fn apply_changes(&mut self, changes: &[(Address, AccountState)]) {
        for (addr, state) in changes {
            self.set_account(*addr, state.clone());
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// Currency account state used by the independent currency-chain state machine.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CurrencyAccountState {
    pub available: u128,
    pub staked: u128,
    pub nonce: u64,
}

/// Deterministic currency state with an explicit non-spendable reserve.
#[derive(Debug, Clone)]
pub struct CurrencyWorldState {
    accounts: BTreeMap<Uuid, CurrencyAccountState>,
    total_supply: u128,
    treasury_reserve: u128,
    state_root: Option<Hash>,
    dirty: bool,
}

impl CurrencyWorldState {
    pub fn from_genesis(genesis: &CurrencyGenesisBlock) -> Result<Self, BlockError> {
        let initial_balances = genesis
            .effective_initial_balances()
            .map_err(|error| BlockError::InvalidTransaction(error.to_string()))?;
        let treasury_reserve = genesis
            .treasury_reserve()
            .map_err(|error| BlockError::InvalidTransaction(error.to_string()))?;
        let accounts = initial_balances
            .into_iter()
            .map(|balance: GenesisBalance| {
                (
                    balance.account.0,
                    CurrencyAccountState {
                        available: u128::from(balance.available),
                        staked: u128::from(balance.staked),
                        nonce: 0,
                    },
                )
            })
            .collect();

        Ok(Self {
            accounts,
            total_supply: u128::from(genesis.initial_supply),
            treasury_reserve: u128::from(treasury_reserve),
            state_root: None,
            dirty: true,
        })
    }

    pub fn get_account(&self, account: &Uuid) -> CurrencyAccountState {
        self.accounts.get(account).cloned().unwrap_or_default()
    }

    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    pub fn treasury_reserve(&self) -> u128 {
        self.treasury_reserve
    }

    /// Execute a signed transfer. Valid-but-unfunded transfers consume their nonce and fail.
    pub fn execute_transfer(
        &mut self,
        tx: &ChainTransaction,
        chain_id: u32,
        tx_index: u32,
    ) -> Result<TxReceipt, BlockError> {
        const MAX_ENVELOPE_BYTES: usize = 1024 * 1024;
        const TRANSFER_GAS: u64 = 21_000;
        const GAS_PER_BYTE: u64 = 16;

        if tx.tx_type != TransactionType::CurrencyTransfer || tx.payload.len() > MAX_ENVELOPE_BYTES
        {
            return Err(BlockError::InvalidTransaction(
                "invalid currency transaction type or oversized envelope".to_string(),
            ));
        }
        let envelope: SignedTransactionEnvelope = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(MAX_ENVELOPE_BYTES as u64)
            .deserialize(&tx.payload)
            .map_err(|error| {
                BlockError::InvalidTransaction(format!("invalid signed envelope: {}", error))
            })?;
        if envelope.domain != EnvelopeDomain::Currency
            || envelope.tx_type
                != UnifiedTransactionType::Currency(CurrencyTransactionType::Transfer)
        {
            return Err(BlockError::InvalidTransaction(
                "currency transfer envelope type mismatch".to_string(),
            ));
        }
        let transfer: TransferTx = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(MAX_ENVELOPE_BYTES as u64)
            .deserialize(&envelope.payload)
            .map_err(|error| {
                BlockError::InvalidTransaction(format!(
                    "invalid currency transfer payload: {}",
                    error
                ))
            })?;

        let sender = Uuid::from_bytes(envelope.sender);
        let sender_state = self.get_account(&sender);
        let verifier = EnvelopeVerifier::with_nonces(
            chain_id,
            HashMap::from([(sender, sender_state.nonce)]),
        );
        verifier.verify(&envelope).map_err(|error| {
            BlockError::InvalidTransaction(format!(
                "signed envelope verification failed: {}",
                error
            ))
        })?;
        if envelope.nonce != sender_state.nonce.saturating_add(1)
            || transfer.from.0 != sender
            || transfer.nonce != envelope.nonce
            || transfer.amount == 0
            || tx.fee_paid != transfer.fee
            || tx.confirmed_at.is_some()
            || !matches!(&tx.status, dchat_chain::TransactionStatus::Pending)
        {
            return Err(BlockError::InvalidTransaction(
                "currency transfer metadata, sender, nonce, or amount is invalid".to_string(),
            ));
        }

        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let transaction_hash = blake3::hash(&hash_input);
        let mut expected_tx_id = [0; 16];
        expected_tx_id.copy_from_slice(&transaction_hash.as_bytes()[..16]);
        if tx.tx_id.as_bytes() != &expected_tx_id
            || tx.tx_hash != format!("0x{}", hex::encode(transaction_hash.as_bytes()))
        {
            return Err(BlockError::InvalidTransaction(
                "transaction ID or hash does not match signed envelope".to_string(),
            ));
        }

        let gas_used = TRANSFER_GAS
            .saturating_add((envelope.payload.len() as u64).saturating_mul(GAS_PER_BYTE));
        let original_available = sender_state.available;
        let mut next_sender = sender_state;
        next_sender.nonce = envelope.nonce;
        let mut next_recipient = None;
        let mut failure = None;
        if let Some(debit) = transfer.amount.checked_add(u128::from(transfer.fee)) {
            if next_sender.available < debit {
                failure = Some("insufficient currency balance".to_string());
            } else if transfer.from.0 == transfer.to.0 {
                next_sender.available -= u128::from(transfer.fee);
            } else {
                let recipient = self.get_account(&transfer.to.0);
                match recipient.available.checked_add(transfer.amount) {
                    Some(recipient_available) => {
                        next_sender.available -= debit;
                        next_recipient = Some((
                            transfer.to.0,
                            CurrencyAccountState {
                                available: recipient_available,
                                ..recipient
                            },
                        ));
                    }
                    None => failure = Some("currency recipient balance overflow".to_string()),
                }
            }
        } else {
            failure = Some("currency transfer debit overflow".to_string());
        }

        let next_supply = if failure.is_none() {
            match self.total_supply.checked_sub(u128::from(transfer.fee)) {
                Some(supply) => Some(supply),
                None => {
                    failure = Some("currency supply underflow".to_string());
                    None
                }
            }
        } else {
            None
        };

        if failure.is_some() {
            next_sender.available = original_available;
        }
        self.accounts.insert(sender, next_sender);
        if let Some((recipient, state)) = next_recipient.filter(|_| failure.is_none()) {
            self.accounts.insert(recipient, state);
        }
        if let Some(supply) = next_supply {
            self.total_supply = supply;
        }
        self.dirty = true;
        self.state_root = None;

        let post_state_root = self.compute_state_root();
        let mut receipt = if let Some(reason) = failure {
            TxReceipt::failure(tx.tx_id, tx_index, &reason)
        } else {
            TxReceipt::success(tx.tx_id, tx_index, gas_used, post_state_root)
        };
        receipt.gas_used = gas_used;
        receipt.post_state_root = post_state_root;
        Ok(receipt)
    }

    pub fn compute_state_root(&mut self) -> Hash {
        if !self.dirty {
            if let Some(root) = self.state_root {
                return root;
            }
        }
        let encoded = canonical::canonical_serialize(&(
            self.total_supply,
            self.treasury_reserve,
            &self.accounts,
        ));
        let root = canonical::domain_hash(b"dchat/currency_world_state/v1", &encoded);
        self.state_root = Some(root);
        self.dirty = false;
        root
    }
}

/// Immutable snapshot for parallel reads
#[derive(Debug, Clone)]
pub struct WorldStateSnapshot {
    accounts: BTreeMap<Address, AccountState>,
}

impl WorldStateSnapshot {
    pub fn get_account(&self, address: &Address) -> AccountState {
        self.accounts.get(address).cloned().unwrap_or_default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transaction Type Alias
// ─────────────────────────────────────────────────────────────────────────────

/// Transaction type for execution (alias to core_types::ExecutionTransaction)
pub type Transaction = ExecutionTransaction;

// ─────────────────────────────────────────────────────────────────────────────
// Execution Context
// ─────────────────────────────────────────────────────────────────────────────

/// Context for executing transactions
pub struct ExecutionContext {
    /// Block height
    pub block_height: u64,
    /// Block timestamp
    pub timestamp: u64,
    /// Gas limit for this execution
    pub gas_limit: u64,
    /// Gas used so far
    pub gas_used: u64,
    /// Fees collected
    pub fees_collected: u64,
}

impl ExecutionContext {
    pub fn new(block_height: u64, timestamp: u64, gas_limit: u64) -> Self {
        Self {
            block_height,
            timestamp,
            gas_limit,
            gas_used: 0,
            fees_collected: 0,
        }
    }

    pub fn remaining_gas(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }

    pub fn use_gas(&mut self, gas: u64) -> bool {
        if self.gas_used + gas > self.gas_limit {
            return false;
        }
        self.gas_used += gas;
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Execution Engine
// ─────────────────────────────────────────────────────────────────────────────

/// Transaction execution engine
pub struct ExecutionEngine {
    /// Whether to execute in parallel
    parallel: bool,
}

impl ExecutionEngine {
    pub fn new(parallel: bool) -> Self {
        Self { parallel }
    }

    /// Execute a miniblock
    pub fn execute_miniblock(
        &self,
        miniblock: &Miniblock,
        state: &mut WorldState,
        ctx: &mut ExecutionContext,
    ) -> Result<Vec<TxReceipt>, BlockError> {
        // Validate miniblock bounds
        miniblock.header.validate_bounds()?;

        // Get body, return empty receipts if body not present
        let body = match &miniblock.body {
            Some(b) => b,
            None => return Ok(Vec::new()),
        };

        // Get transactions from body
        let txs = self.extract_transactions(body)?;

        // Execute based on mode
        if self.parallel {
            self.execute_parallel(&txs, state, ctx)
        } else {
            self.execute_sequential(&txs, state, ctx)
        }
    }

    /// Extract transactions from body
    fn extract_transactions(&self, body: &MiniblockBody) -> Result<Vec<Transaction>, BlockError> {
        // MiniblockBody.transactions contains dchat_chain::Transaction (high-level envelope)
        // Convert to ExecutionTransaction using the from_chain_transaction method
        body.transactions
            .iter()
            .enumerate()
            .map(|(idx, chain_tx)| {
                ExecutionTransaction::from_chain_transaction(chain_tx).ok_or_else(|| {
                    BlockError::InvalidTransaction(format!(
                        "failed to parse execution data from chain transaction at index {}",
                        idx
                    ))
                })
            })
            .collect()
    }

    /// Execute transactions sequentially
    fn execute_sequential(
        &self,
        txs: &[Transaction],
        state: &mut WorldState,
        ctx: &mut ExecutionContext,
    ) -> Result<Vec<TxReceipt>, BlockError> {
        let mut receipts = Vec::with_capacity(txs.len());

        for (idx, tx) in txs.iter().enumerate() {
            let receipt = self.execute_single_tx(tx, state, ctx, idx)?;
            receipts.push(receipt);
        }

        Ok(receipts)
    }

    /// Execute transactions in parallel by lane
    fn execute_parallel(
        &self,
        txs: &[Transaction],
        state: &mut WorldState,
        ctx: &mut ExecutionContext,
    ) -> Result<Vec<TxReceipt>, BlockError> {
        // Group by lane
        let mut lanes: HashMap<LaneId, Vec<(usize, &Transaction)>> = HashMap::new();
        for (idx, tx) in txs.iter().enumerate() {
            lanes.entry(tx.lane()).or_default().push((idx, tx));
        }

        // Take snapshot
        let snapshot = state.snapshot();
        let results = Arc::new(RwLock::new(vec![None; txs.len()]));

        // Execute each lane in parallel
        let lane_changes: Vec<_> = lanes
            .par_iter()
            .map(|(_lane, lane_txs)| {
                let mut lane_changes: Vec<(Address, AccountState)> = Vec::new();

                for &(idx, tx) in lane_txs {
                    // Simple state update for this tx
                    let sender = tx.sender;
                    let mut sender_state = snapshot.get_account(&sender);

                    // Check nonce
                    if sender_state.nonce != tx.nonce {
                        let receipt = TxReceipt::failure(tx.tx_id, idx as u32, "nonce mismatch");
                        results.write().unwrap()[idx] = Some(receipt);
                        continue;
                    }

                    // Check balance
                    let total_cost = tx.value + tx.gas_limit * tx.gas_price;
                    if sender_state.balance < total_cost {
                        let receipt =
                            TxReceipt::failure(tx.tx_id, idx as u32, "insufficient balance");
                        results.write().unwrap()[idx] = Some(receipt);
                        continue;
                    }

                    // Deduct from sender
                    let gas_cost = tx.gas_cost();
                    let fee = gas_cost * tx.gas_price;
                    sender_state.balance -= tx.value + fee;
                    sender_state.nonce += 1;
                    lane_changes.push((sender, sender_state));

                    // Credit recipient
                    if let Some(recipient) = tx.recipient {
                        let mut recipient_state = snapshot.get_account(&recipient);
                        recipient_state.balance += tx.value;
                        lane_changes.push((recipient, recipient_state));
                    }

                    // Success receipt with post-state root (computed per-lane)
                    let receipt = TxReceipt::success(
                        tx.tx_id,
                        idx as u32,
                        gas_cost,
                        Hash::ZERO, // Post-state root computed after all lanes merge
                    );
                    results.write().unwrap()[idx] = Some(receipt);
                }

                lane_changes
            })
            .collect();

        // Apply all lane changes
        for changes in lane_changes {
            state.apply_changes(&changes);
        }

        // Collect receipts in order
        let receipts: Vec<TxReceipt> = results
            .read()
            .unwrap()
            .iter()
            .filter_map(|r| r.clone())
            .collect();

        // Update context - track both gas used and fees collected
        // Fees are computed as gas_cost * gas_price for each successful tx
        let (total_gas, total_fees): (u64, u64) = txs
            .iter()
            .zip(receipts.iter())
            .filter(|(_, r)| r.success)
            .map(|(tx, r)| {
                let fee_motes = r.gas_used.saturating_mul(tx.gas_price);
                (r.gas_used, fee_motes)
            })
            .fold((0u64, 0u64), |(g, f), (gas, fee)| {
                (g.saturating_add(gas), f.saturating_add(fee))
            });
        ctx.gas_used = ctx.gas_used.saturating_add(total_gas);
        ctx.fees_collected = ctx.fees_collected.saturating_add(total_fees);

        Ok(receipts)
    }

    /// Execute single transaction
    fn execute_single_tx(
        &self,
        tx: &Transaction,
        state: &mut WorldState,
        ctx: &mut ExecutionContext,
        idx: usize,
    ) -> Result<TxReceipt, BlockError> {
        let gas_cost = tx.gas_cost();

        // Check gas limit
        if !ctx.use_gas(gas_cost) {
            return Ok(TxReceipt::failure(tx.tx_id, idx as u32, "out of gas"));
        }

        // Get sender account
        let sender_account = state.get_account_mut(&tx.sender);

        // Verify nonce
        if sender_account.nonce != tx.nonce {
            return Ok(TxReceipt::failure(tx.tx_id, idx as u32, "nonce mismatch"));
        }

        // Calculate total cost
        let fee = gas_cost * tx.gas_price;
        let total_cost = tx.value + fee;

        // Check balance
        if sender_account.balance < total_cost {
            return Ok(TxReceipt::failure(
                tx.tx_id,
                idx as u32,
                "insufficient balance",
            ));
        }

        // Deduct from sender
        sender_account.balance -= total_cost;
        sender_account.nonce += 1;

        // Credit recipient
        if let Some(recipient) = tx.recipient {
            let recipient_account = state.get_account_mut(&recipient);
            recipient_account.balance += tx.value;
        }

        // Collect fee
        ctx.fees_collected += fee;

        // Compute post-state root after this transaction
        let post_state_root = state.compute_state_root();

        Ok(TxReceipt::success(
            tx.tx_id,
            idx as u32,
            gas_cost,
            post_state_root,
        ))
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// State Transition
// ─────────────────────────────────────────────────────────────────────────────

/// State transition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Pre-state root
    pub pre_state_root: Hash,
    /// Post-state root
    pub post_state_root: Hash,
    /// Transactions root
    pub tx_root: Hash,
    /// Receipts root
    pub receipts_root: Hash,
    /// Total gas used
    pub gas_used: u64,
    /// Fees collected
    pub fees_collected: u64,
}

impl StateTransition {
    /// Compute from execution
    pub fn compute(
        pre_root: Hash,
        post_root: Hash,
        commitments: &BodyCommitments,
        ctx: &ExecutionContext,
    ) -> Self {
        Self {
            pre_state_root: pre_root,
            post_state_root: post_root,
            tx_root: commitments.tx_root,
            receipts_root: commitments.receipts_root,
            gas_used: ctx.gas_used,
            fees_collected: ctx.fees_collected,
        }
    }

    /// Hash of state transition
    pub fn hash(&self) -> Hash {
        let bytes = canonical::canonical_serialize(self);
        canonical::domain_hash(b"dchat/state_transition/v1", &bytes)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Compute Merkle root from leaves
fn compute_merkle_root(leaves: &mut Vec<Hash>) -> Hash {
    if leaves.is_empty() {
        return Hash::ZERO;
    }

    // Pad to power of 2
    while leaves.len() & (leaves.len() - 1) != 0 {
        leaves.push(Hash::ZERO);
    }

    while leaves.len() > 1 {
        let mut next = Vec::with_capacity(leaves.len() / 2);
        for chunk in leaves.chunks(2) {
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(chunk[0].as_bytes());
            combined.extend_from_slice(chunk.get(1).unwrap_or(&Hash::ZERO).as_bytes());
            next.push(canonical::domain_hash(b"dchat/merkle_node/v1", &combined));
        }
        *leaves = next;
    }

    leaves[0]
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use dchat_chain::chain::{
        CurrencyGenesisBlock, CurrencyGenesisConfig, GenesisBalance, GenesisBuilder,
        GenesisValidator,
    };
    use dchat_chain::{
        address_from_public_key, chain_ids, EnvelopeBuilder, RegisterUserTx, TransactionStatus,
        TransferTx,
    };
    use dchat_core::types::UserId;
    use ed25519_dalek::SigningKey;

    fn signed_registration(nonce: u64) -> ChainTransaction {
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let user_id = UserId(address_from_public_key(&public_key));
        let payload = bincode::serialize(&RegisterUserTx {
            user_id,
            username: "alice".to_string(),
            public_key: hex::encode(public_key),
            timestamp: Utc::now(),
            initial_reputation: 0,
        })
        .unwrap();
        let envelope = EnvelopeBuilder::new(chain_ids::TESTNET_CHAT, public_key)
            .nonce(nonce)
            .chat_tx(TransactionType::RegisterUser)
            .payload(payload)
            .build_and_sign(&signing_key)
            .unwrap();
        let bytes = envelope.to_bytes().unwrap();
        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let hash = blake3::hash(&hash_input);
        let mut tx_id = [0; 16];
        tx_id.copy_from_slice(&hash.as_bytes()[..16]);
        ChainTransaction {
            tx_id: Uuid::from_bytes(tx_id),
            tx_type: TransactionType::RegisterUser,
            payload: bytes,
            tx_hash: format!("0x{}", hex::encode(hash.as_bytes())),
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: 0,
        }
    }

    fn signed_currency_transfer(
        signing_key: &SigningKey,
        recipient: Uuid,
        nonce: u64,
        amount: u128,
        fee: u64,
    ) -> ChainTransaction {
        let public_key = signing_key.verifying_key().to_bytes();
        let transfer = TransferTx {
            tx_id: Uuid::new_v4(),
            from: UserId(address_from_public_key(&public_key)),
            to: UserId(recipient),
            amount,
            memo: None,
            fee,
            timestamp: Utc::now(),
            nonce,
            signature: Vec::new(),
        };
        let envelope = EnvelopeBuilder::new(chain_ids::TESTNET_CURRENCY, public_key)
            .nonce(nonce)
            .currency_tx(CurrencyTransactionType::Transfer)
            .payload(bincode::serialize(&transfer).unwrap())
            .build_and_sign(signing_key)
            .unwrap();
        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let hash = blake3::hash(&hash_input);
        let mut tx_id = [0; 16];
        tx_id.copy_from_slice(&hash.as_bytes()[..16]);
        ChainTransaction {
            tx_id: Uuid::from_bytes(tx_id),
            tx_type: TransactionType::CurrencyTransfer,
            payload: envelope.to_bytes().unwrap(),
            tx_hash: format!("0x{}", hex::encode(hash.as_bytes())),
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: fee,
        }
    }

    fn currency_genesis(
        sender: Uuid,
        recipient: Uuid,
    ) -> CurrencyGenesisBlock {
        let signing_key = SigningKey::from_bytes(&[11; 32]);
        let builder = GenesisBuilder::new(signing_key);
        let genesis = builder
            .create_currency_genesis_with_balances(
                Vec::<GenesisValidator>::new(),
                CurrencyGenesisConfig {
                    chain_id: "test-currency".to_string(),
                    block_time_secs: 5,
                    token_name: "Test Token".to_string(),
                    token_symbol: "TEST".to_string(),
                    token_decimals: 8,
                },
                2_000,
                vec![
                    GenesisBalance {
                        account: UserId(sender),
                        available: 1_000,
                        staked: 0,
                    },
                    GenesisBalance {
                        account: UserId(recipient),
                        available: 0,
                        staked: 0,
                    },
                ],
            )
            .unwrap();
        genesis
    }

    #[test]
    fn test_account_state() {
        let state = AccountState::with_balance(1000);
        assert_eq!(state.balance, 1000);
        assert_eq!(state.nonce, 0);

        let hash = state.hash();
        assert_ne!(hash, Hash::ZERO);
    }

    #[test]
    fn test_world_state() {
        let mut state = WorldState::new();
        let addr = [1u8; 32];

        state.set_account(addr, AccountState::with_balance(1000));
        let account = state.get_account(&addr);
        assert_eq!(account.balance, 1000);

        let root = state.compute_state_root();
        assert_ne!(root, Hash::ZERO);
    }

    #[test]
    fn currency_transfer_replays_with_burned_fee_and_conserved_state() {
        let sender_key = SigningKey::from_bytes(&[12; 32]);
        let recipient_key = SigningKey::from_bytes(&[13; 32]);
        let sender = address_from_public_key(&sender_key.verifying_key().to_bytes());
        let recipient = address_from_public_key(&recipient_key.verifying_key().to_bytes());
        let genesis = currency_genesis(sender, recipient);
        let tx = signed_currency_transfer(&sender_key, recipient, 1, 100, 2);

        let mut producer_state = CurrencyWorldState::from_genesis(&genesis).unwrap();
        let producer_receipt = producer_state
            .execute_transfer(&tx, chain_ids::TESTNET_CURRENCY, 0)
            .unwrap();
        let mut peer_state = CurrencyWorldState::from_genesis(&genesis).unwrap();
        let peer_receipt = peer_state
            .execute_transfer(&tx, chain_ids::TESTNET_CURRENCY, 0)
            .unwrap();

        assert!(producer_receipt.success);
        assert_eq!(producer_receipt, peer_receipt);
        assert_eq!(producer_state.get_account(&sender).available, 898);
        assert_eq!(producer_state.get_account(&recipient).available, 100);
        assert_eq!(producer_state.total_supply(), 1_998);
        assert_eq!(producer_state.treasury_reserve(), 1_000);
        assert_eq!(
            producer_state.compute_state_root(),
            peer_state.compute_state_root()
        );
    }

    #[test]
    fn failed_currency_transfer_consumes_nonce_without_changing_balances_or_supply() {
        let sender_key = SigningKey::from_bytes(&[14; 32]);
        let recipient_key = SigningKey::from_bytes(&[15; 32]);
        let sender = address_from_public_key(&sender_key.verifying_key().to_bytes());
        let recipient = address_from_public_key(&recipient_key.verifying_key().to_bytes());
        let genesis = currency_genesis(sender, recipient);
        let tx = signed_currency_transfer(&sender_key, recipient, 1, 2_000, 2);
        let mut state = CurrencyWorldState::from_genesis(&genesis).unwrap();

        let receipt = state
            .execute_transfer(&tx, chain_ids::TESTNET_CURRENCY, 0)
            .unwrap();

        assert!(!receipt.success);
        assert_eq!(state.get_account(&sender).nonce, 1);
        assert_eq!(state.get_account(&sender).available, 1_000);
        assert_eq!(state.get_account(&recipient).available, 0);
        assert_eq!(state.total_supply(), 2_000);
    }

    #[test]
    fn currency_block_replay_rejects_receipt_tampering() {
        let sender_key = SigningKey::from_bytes(&[16; 32]);
        let recipient_key = SigningKey::from_bytes(&[17; 32]);
        let sender = address_from_public_key(&sender_key.verifying_key().to_bytes());
        let recipient = address_from_public_key(&recipient_key.verifying_key().to_bytes());
        let genesis = currency_genesis(sender, recipient);
        let tx = signed_currency_transfer(&sender_key, recipient, 1, 100, 2);
        let initial_state = CurrencyWorldState::from_genesis(&genesis).unwrap();
        let (block, producer_state) = crate::CurrencyBlock::build(
            1,
            [0; 32],
            vec![tx],
            &initial_state,
            chain_ids::TESTNET_CURRENCY,
        )
        .unwrap();
        let replayed_state = block
            .verify_and_execute(&initial_state, chain_ids::TESTNET_CURRENCY)
            .unwrap();
        assert_eq!(
            producer_state.clone().compute_state_root(),
            replayed_state.clone().compute_state_root()
        );

        let mut tampered = block;
        tampered.receipts[0].success = false;
        assert!(tampered
            .verify_and_execute(&initial_state, chain_ids::TESTNET_CURRENCY)
            .is_err());
    }

    #[test]
    fn signed_chat_execution_is_replayable_and_nonce_protected() {
        let tx = signed_registration(1);
        let mut producer_state = WorldState::new();
        let producer_receipt = producer_state
            .execute_chat_transaction(&tx, dchat_chain::chain_ids::TESTNET_CHAT, 1, 0)
            .unwrap();
        assert!(producer_receipt.success);

        let mut peer_state = WorldState::new();
        let peer_receipt = peer_state
            .execute_chat_transaction(&tx, dchat_chain::chain_ids::TESTNET_CHAT, 1, 0)
            .unwrap();
        assert_eq!(producer_receipt, peer_receipt);
        assert_eq!(
            producer_state.compute_state_root(),
            peer_state.compute_state_root()
        );
        assert!(producer_state
            .execute_chat_transaction(&tx, dchat_chain::chain_ids::TESTNET_CHAT, 2, 0)
            .is_err());
    }

    #[test]
    fn signed_profile_update_is_replayable_and_changes_user_state() {
        let registration = signed_registration(1);
        let mut producer_state = WorldState::new();
        assert!(
            producer_state
                .execute_chat_transaction(&registration, chain_ids::TESTNET_CHAT, 1, 0)
                .unwrap()
                .success
        );

        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let user_id = UserId(address_from_public_key(&public_key));
        let payload = bincode::serialize(&UpdateProfileTx {
            user_id,
            username: "alice-updated".to_string(),
            display_name: Some("Alice".to_string()),
            avatar_hash: None,
            bio: Some("Updated bio".to_string()),
        })
        .unwrap();
        let envelope = EnvelopeBuilder::new(chain_ids::TESTNET_CHAT, public_key)
            .nonce(2)
            .chat_tx(TransactionType::UpdateProfile)
            .payload(payload)
            .build_and_sign(&signing_key)
            .unwrap();
        let bytes = envelope.to_bytes().unwrap();
        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let hash = blake3::hash(&hash_input);
        let mut tx_id = [0; 16];
        tx_id.copy_from_slice(&hash.as_bytes()[..16]);
        let tx = ChainTransaction {
            tx_id: Uuid::from_bytes(tx_id),
            tx_type: TransactionType::UpdateProfile,
            payload: bytes,
            tx_hash: format!("0x{}", hex::encode(hash.as_bytes())),
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: 0,
        };

        let producer_receipt = producer_state
            .execute_chat_transaction(&tx, chain_ids::TESTNET_CHAT, 2, 0)
            .unwrap();
        assert!(producer_receipt.success);
        let user_key = WorldState::chat_record_key(b"user", user_id.0.as_bytes());
        let user: ChatUserRecord =
            bincode::deserialize(producer_state.chat_records.get(&user_key).unwrap()).unwrap();
        assert_eq!(user.username, "alice-updated");
        assert_eq!(user.display_name.as_deref(), Some("Alice"));
        assert_eq!(user.bio.as_deref(), Some("Updated bio"));

        let mut peer_state = WorldState::new();
        assert!(
            peer_state
                .execute_chat_transaction(&registration, chain_ids::TESTNET_CHAT, 1, 0)
                .unwrap()
                .success
        );
        let peer_receipt = peer_state
            .execute_chat_transaction(&tx, chain_ids::TESTNET_CHAT, 2, 0)
            .unwrap();
        assert_eq!(producer_receipt, peer_receipt);
        assert_eq!(
            producer_state.compute_state_root(),
            peer_state.compute_state_root()
        );
    }

    #[test]
    fn delivery_proof_is_replayed_and_cannot_be_included_twice() {
        use dchat_network::relay::proof::{
            create_recipient_acknowledgment, MessageId as ProofMessageId,
        };
        use ed25519_dalek::Signer;

        let recipient_key = SigningKey::from_bytes(&[7; 32]);
        let recipient_public_key = recipient_key.verifying_key().to_bytes();
        let recipient_id = UserId(address_from_public_key(&recipient_public_key));
        let registration = signed_registration(1);
        let relay_key = SigningKey::from_bytes(&[8; 32]);
        let message_id = ProofMessageId::new([3; 32]);
        let timestamp = 1_000;
        let recipient_signature = create_recipient_acknowledgment(
            message_id,
            &relay_key.verifying_key(),
            timestamp,
            &recipient_key,
        );
        let mut relay_message = Vec::new();
        relay_message.extend_from_slice(message_id.as_bytes());
        relay_message.extend_from_slice(recipient_key.verifying_key().as_bytes());
        relay_message.extend_from_slice(&recipient_signature.to_bytes());
        relay_message.extend_from_slice(&timestamp.to_le_bytes());
        let delivery = OnChainDeliveryProofTx {
            message_id: message_id.0,
            recipient_id,
            recipient_public_key,
            recipient_signature: recipient_signature.to_bytes().to_vec(),
            timestamp,
            relay_signature: relay_key.sign(&relay_message).to_bytes().to_vec(),
        };
        let make_delivery_transaction = |nonce| {
            let envelope = EnvelopeBuilder::new(
                chain_ids::TESTNET_CHAT,
                relay_key.verifying_key().to_bytes(),
            )
            .nonce(nonce)
            .chat_tx(TransactionType::SubmitDeliveryProof)
            .payload(bincode::serialize(&delivery).unwrap())
            .build_and_sign(&relay_key)
            .unwrap();
            let bytes = envelope.to_bytes().unwrap();
            let mut hash_input = envelope.signing_bytes();
            hash_input.extend_from_slice(&envelope.signature);
            let hash = blake3::hash(&hash_input);
            let mut tx_id = [0; 16];
            tx_id.copy_from_slice(&hash.as_bytes()[..16]);
            ChainTransaction {
                tx_id: Uuid::from_bytes(tx_id),
                tx_type: TransactionType::SubmitDeliveryProof,
                payload: bytes,
                tx_hash: format!("0x{}", hex::encode(hash.as_bytes())),
                status: TransactionStatus::Pending,
                submitted_at: Utc::now(),
                confirmed_at: None,
                fee_paid: 0,
            }
        };
        let tx = make_delivery_transaction(1);
        let mut producer_state = WorldState::new();
        assert!(producer_state
            .execute_chat_transaction(&registration, chain_ids::TESTNET_CHAT, 1, 0)
            .unwrap()
            .success);
        let producer_receipt = producer_state
            .execute_chat_transaction(&tx, chain_ids::TESTNET_CHAT, 2, 0)
            .unwrap();
        assert!(producer_receipt.success);
        assert!(!producer_state
            .execute_chat_transaction(
                &make_delivery_transaction(2),
                chain_ids::TESTNET_CHAT,
                3,
                0,
            )
            .unwrap()
            .success);

        let mut peer_state = WorldState::new();
        assert!(peer_state
            .execute_chat_transaction(&registration, chain_ids::TESTNET_CHAT, 1, 0)
            .unwrap()
            .success);
        let peer_receipt = peer_state
            .execute_chat_transaction(&tx, chain_ids::TESTNET_CHAT, 2, 0)
            .unwrap();
        assert_eq!(producer_receipt, peer_receipt);
        assert_eq!(
            producer_state.compute_state_root(),
            peer_state.compute_state_root()
        );
    }

    #[test]
    fn test_transaction() {
        let tx = Transaction {
            tx_id: uuid::Uuid::new_v4(),
            hash: Hash::ZERO,
            sender: [1u8; 32],
            recipient: Some([2u8; 32]),
            value: 100,
            payload: vec![0; 100],
            nonce: 0,
            gas_limit: 100_000,
            gas_price: 1,
            signature: vec![],
        };

        let hash = tx.compute_hash();
        assert_ne!(hash, Hash::ZERO);

        let gas = tx.gas_cost();
        assert_eq!(gas, BASE_TX_GAS + 100 * GAS_PER_BYTE);
    }

    #[test]
    fn test_execution_context() {
        let mut ctx = ExecutionContext::new(1, 0, 100_000);

        assert!(ctx.use_gas(50_000));
        assert_eq!(ctx.remaining_gas(), 50_000);

        assert!(ctx.use_gas(50_000));
        assert_eq!(ctx.remaining_gas(), 0);

        assert!(!ctx.use_gas(1));
    }

    #[test]
    fn test_merkle_root() {
        let mut leaves = vec![
            Hash::from([1u8; 32]),
            Hash::from([2u8; 32]),
            Hash::from([3u8; 32]),
            Hash::from([4u8; 32]),
        ];

        let root = compute_merkle_root(&mut leaves);
        assert_ne!(root, Hash::ZERO);
    }
}
