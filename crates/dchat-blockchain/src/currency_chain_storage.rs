use crate::{block_hierarchy::CurrencyWorldState, CurrencyBlock};
use bincode::Options;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use tokio::{
    fs::{self, File, OpenOptions},
    io::AsyncWriteExt,
};

const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;
const JOURNAL_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrencyFinalityVote {
    pub validator_id: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum JournalRecord {
    Header {
        genesis_hash: [u8; 32],
        chain_id: u32,
        validator_id: [u8; 32],
    },
    Prepared {
        block: CurrencyBlock,
    },
    VoteLock {
        height: u64,
        hash: [u8; 32],
    },
    Finalized {
        block: CurrencyBlock,
        votes: Vec<CurrencyFinalityVote>,
    },
}

pub struct CurrencyChainRecovery {
    pub state: CurrencyWorldState,
    pub height: u64,
    pub last_hash: [u8; 32],
    pub pending_block: Option<CurrencyBlock>,
}

/// Append-only local journal for currency finality and the validator's vote lock.
pub struct CurrencyChainJournal {
    file: File,
    chain_id: u32,
    authorized_validators: HashSet<[u8; 32]>,
    quorum: usize,
    height: u64,
    last_hash: [u8; 32],
    state: CurrencyWorldState,
    pending_block: Option<CurrencyBlock>,
    votes: HashMap<u64, [u8; 32]>,
    poisoned: bool,
}

impl CurrencyChainJournal {
    pub async fn open(
        path: impl AsRef<Path>,
        genesis_hash: [u8; 32],
        chain_id: u32,
        validator_id: [u8; 32],
        authorized_validators: HashSet<[u8; 32]>,
        quorum: usize,
        genesis_state: CurrencyWorldState,
    ) -> Result<(Self, CurrencyChainRecovery), String> {
        if quorum == 0
            || quorum > authorized_validators.len()
            || !authorized_validators.contains(&validator_id)
        {
            return Err("invalid currency journal quorum or validator identity".to_string());
        }
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("failed to create currency journal directory: {error}"))?;
        }
        let contents = match fs::read(&path).await {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(format!("failed to read currency journal: {error}")),
        };
        let mut state = genesis_state;
        let mut height = 0u64;
        let mut last_hash = genesis_hash;
        let mut pending_block = None;
        let mut votes = HashMap::new();

        if contents.is_empty() {
            let header = JournalRecord::Header {
                genesis_hash,
                chain_id,
                validator_id,
            };
            let frame = encode_frame(&header)?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
                .map_err(|error| format!("failed to create currency journal: {error}"))?;
            file.write_all(&frame)
                .await
                .map_err(|error| format!("failed to write currency journal header: {error}"))?;
            file.sync_all()
                .await
                .map_err(|error| format!("failed to sync currency journal header: {error}"))?;
        } else {
            let mut offset = 0usize;
            let mut record_index = 0usize;
            while offset < contents.len() {
                let Some((record, next_offset)) = decode_frame(&contents, offset)? else {
                    break;
                };
                if record_index == 0 && !matches!(&record, JournalRecord::Header { .. }) {
                    return Err("currency journal does not begin with a header".to_string());
                }
                match record {
                    JournalRecord::Header {
                        genesis_hash: stored_genesis_hash,
                        chain_id: stored_chain_id,
                        validator_id: stored_validator_id,
                    } if record_index == 0 => {
                        if stored_genesis_hash != genesis_hash
                            || stored_chain_id != chain_id
                            || stored_validator_id != validator_id
                        {
                            return Err(
                                "currency journal belongs to a different genesis, chain, or validator"
                                    .to_string(),
                            );
                        }
                    }
                    JournalRecord::Prepared { block } => {
                        if pending_block.is_some()
                            || height.checked_add(1) != Some(block.height)
                            || block.previous_hash != last_hash
                        {
                            return Err("currency journal has an invalid prepared block sequence"
                                .to_string());
                        }
                        block
                            .verify_and_execute(&state, chain_id)
                            .map_err(|error| format!("invalid prepared currency block: {error}"))?;
                        pending_block = Some(block);
                    }
                    JournalRecord::VoteLock {
                        height: vote_height,
                        hash,
                    } => {
                        if height.checked_add(1) != Some(vote_height)
                            || !pending_block.as_ref().is_some_and(|block| {
                                block.height == vote_height && block.hash == hash
                            })
                        {
                            return Err(
                                "currency journal contains a vote without its prepared proposal"
                                    .to_string(),
                            );
                        }
                        if votes
                            .insert(vote_height, hash)
                            .is_some_and(|prior_hash| prior_hash != hash)
                        {
                            return Err(
                                "currency journal contains conflicting local votes".to_string()
                            );
                        }
                    }
                    JournalRecord::Finalized {
                        block,
                        votes: certificate,
                    } => {
                        if height.checked_add(1) != Some(block.height)
                            || block.previous_hash != last_hash
                            || pending_block.as_ref().map(|prepared| prepared.hash)
                                != Some(block.hash)
                        {
                            return Err(
                                "currency journal has a conflicting finalized block".to_string()
                            );
                        }
                        verify_certificate(&block, &certificate, &authorized_validators, quorum)?;
                        state = block
                            .verify_and_execute(&state, chain_id)
                            .map_err(|error| {
                                format!("invalid finalized currency block: {error}")
                            })?;
                        height = block.height;
                        last_hash = block.hash;
                        pending_block = None;
                        votes.remove(&height);
                    }
                    JournalRecord::Header { .. } => {
                        return Err("currency journal contains an unexpected header".to_string());
                    }
                }
                offset = next_offset;
                record_index += 1;
            }
            if record_index == 0 {
                let file = OpenOptions::new()
                    .write(true)
                    .open(&path)
                    .await
                    .map_err(|error| {
                        format!("failed to repair incomplete currency journal header: {error}")
                    })?;
                file.set_len(0).await.map_err(|error| {
                    format!("failed to reset incomplete currency journal header: {error}")
                })?;
                file.sync_all()
                    .await
                    .map_err(|error| format!("failed to sync currency journal reset: {error}"))?;
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(&path)
                    .await
                    .map_err(|error| format!("failed to reopen currency journal: {error}"))?;
                file.write_all(&encode_frame(&JournalRecord::Header {
                    genesis_hash,
                    chain_id,
                    validator_id,
                })?)
                .await
                .map_err(|error| format!("failed to rewrite currency journal header: {error}"))?;
                file.sync_all()
                    .await
                    .map_err(|error| format!("failed to sync currency journal header: {error}"))?;
            } else if offset < contents.len() {
                let file = OpenOptions::new()
                    .write(true)
                    .open(&path)
                    .await
                    .map_err(|error| format!("failed to repair currency journal tail: {error}"))?;
                file.set_len(offset as u64).await.map_err(|error| {
                    format!("failed to truncate incomplete currency journal tail: {error}")
                })?;
                file.sync_all().await.map_err(|error| {
                    format!("failed to sync repaired currency journal: {error}")
                })?;
            }
        }

        let file = OpenOptions::new()
            .append(true)
            .open(&path)
            .await
            .map_err(|error| format!("failed to open currency journal for append: {error}"))?;
        let recovery = CurrencyChainRecovery {
            state: state.clone(),
            height,
            last_hash,
            pending_block: pending_block.clone(),
        };
        Ok((
            Self {
                file,
                chain_id,
                authorized_validators,
                quorum,
                height,
                last_hash,
                state,
                pending_block,
                votes,
                poisoned: false,
            },
            recovery,
        ))
    }

    pub async fn prepare_block(&mut self, block: &CurrencyBlock) -> Result<(), String> {
        if self.height.checked_add(1) != Some(block.height) || block.previous_hash != self.last_hash
        {
            return Err("currency block is not the next journal height".to_string());
        }
        if let Some(existing) = &self.pending_block {
            return if existing.hash == block.hash {
                Ok(())
            } else {
                Err("a conflicting currency proposal is already persisted for this height".into())
            };
        }
        block
            .verify_and_execute(&self.state, self.chain_id)
            .map_err(|error| format!("currency block replay failed before prepare: {error}"))?;
        self.append(&JournalRecord::Prepared {
            block: block.clone(),
        })
        .await?;
        self.pending_block = Some(block.clone());
        Ok(())
    }

    /// Persist the one-hash-per-height vote lock before producing a signature.
    pub async fn record_vote(&mut self, height: u64, hash: [u8; 32]) -> Result<(), String> {
        if self.height.checked_add(1) != Some(height)
            || !self
                .pending_block
                .as_ref()
                .is_some_and(|block| block.height == height && block.hash == hash)
        {
            return Err("currency vote does not match the persisted next-height proposal".into());
        }
        if let Some(existing_hash) = self.votes.get(&height) {
            return if existing_hash == &hash {
                Ok(())
            } else {
                Err(format!(
                    "refusing to double-vote on currency block height {height}"
                ))
            };
        }
        self.append(&JournalRecord::VoteLock { height, hash })
            .await?;
        self.votes.insert(height, hash);
        Ok(())
    }

    pub async fn finalize_pending(
        &mut self,
        expected_hash: [u8; 32],
        votes: Vec<CurrencyFinalityVote>,
    ) -> Result<CurrencyWorldState, String> {
        let block = self
            .pending_block
            .as_ref()
            .filter(|block| block.hash == expected_hash)
            .cloned()
            .ok_or_else(|| "currency finality does not match the persisted proposal".to_string())?;
        if self.height.checked_add(1) != Some(block.height) || block.previous_hash != self.last_hash
        {
            return Err("currency finality does not match the persisted next proposal".into());
        }
        verify_certificate(&block, &votes, &self.authorized_validators, self.quorum)?;
        let candidate_state = block
            .verify_and_execute(&self.state, self.chain_id)
            .map_err(|error| format!("currency block replay failed before finality: {error}"))?;
        self.append(&JournalRecord::Finalized {
            block: block.clone(),
            votes,
        })
        .await?;
        self.height = block.height;
        self.last_hash = block.hash;
        self.state = candidate_state.clone();
        self.pending_block = None;
        self.votes.remove(&block.height);
        Ok(candidate_state)
    }

    async fn append(&mut self, record: &JournalRecord) -> Result<(), String> {
        if self.poisoned {
            return Err("currency journal is unavailable after a prior write failure".into());
        }
        let frame = encode_frame(record)?;
        if let Err(error) = self.file.write_all(&frame).await {
            self.poisoned = true;
            return Err(format!("failed to append currency journal: {error}"));
        }
        if let Err(error) = self.file.sync_all().await {
            self.poisoned = true;
            return Err(format!("failed to sync currency journal: {error}"));
        }
        Ok(())
    }

    pub fn pending_block(&self) -> Option<CurrencyBlock> {
        self.pending_block.clone()
    }

    pub fn local_vote_hash(&self, height: u64) -> Option<[u8; 32]> {
        self.votes.get(&height).copied()
    }
}

fn verify_certificate(
    block: &CurrencyBlock,
    votes: &[CurrencyFinalityVote],
    authorized_validators: &HashSet<[u8; 32]>,
    quorum: usize,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for vote in votes {
        if !seen.insert(vote.validator_id) || !authorized_validators.contains(&vote.validator_id) {
            return Err(
                "currency finality certificate has duplicate or unauthorized voters".into(),
            );
        }
        let key = VerifyingKey::from_bytes(&vote.validator_id)
            .map_err(|error| format!("invalid finality voter key: {error}"))?;
        let signature_bytes: [u8; 64] = vote
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| "currency finality signature must be 64 bytes".to_string())?;
        key.verify(&block.hash, &Signature::from_bytes(&signature_bytes))
            .map_err(|_| "invalid currency finality signature".to_string())?;
    }
    if seen.len() < quorum {
        return Err(format!(
            "currency finality certificate has {} votes; {} required",
            seen.len(),
            quorum
        ));
    }
    Ok(())
}

fn encode_frame(record: &JournalRecord) -> Result<Vec<u8>, String> {
    let payload = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .serialize(&(JOURNAL_VERSION, record))
        .map_err(|error| format!("failed to serialize currency journal record: {error}"))?;
    if payload.len() > MAX_RECORD_BYTES {
        return Err("currency journal record exceeds size limit".to_string());
    }
    let mut frame = Vec::with_capacity(4 + payload.len() + 32);
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&payload);
    frame.extend_from_slice(blake3::hash(&payload).as_bytes());
    Ok(frame)
}

fn decode_frame(contents: &[u8], offset: usize) -> Result<Option<(JournalRecord, usize)>, String> {
    if contents.len().saturating_sub(offset) < 4 {
        return Ok(None);
    }
    let length = u32::from_le_bytes(
        contents[offset..offset + 4]
            .try_into()
            .map_err(|_| "invalid currency journal frame length".to_string())?,
    ) as usize;
    if length > MAX_RECORD_BYTES {
        return Err("currency journal record exceeds size limit".to_string());
    }
    let payload_start = offset + 4;
    let payload_end = payload_start
        .checked_add(length)
        .ok_or_else(|| "currency journal frame length overflow".to_string())?;
    let frame_end = payload_end
        .checked_add(32)
        .ok_or_else(|| "currency journal frame length overflow".to_string())?;
    if frame_end > contents.len() {
        return Ok(None);
    }
    let payload = &contents[payload_start..payload_end];
    if blake3::hash(payload).as_bytes() != &contents[payload_end..frame_end] {
        return Err("currency journal checksum mismatch".to_string());
    }
    let (version, record): (u8, JournalRecord) = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(MAX_RECORD_BYTES as u64)
        .deserialize(payload)
        .map_err(|error| format!("invalid currency journal record: {error}"))?;
    if version != JOURNAL_VERSION {
        return Err(format!("unsupported currency journal version {version}"));
    }
    Ok(Some((record, frame_end)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CurrencyBlock;
    use chrono::Utc;
    use dchat_chain::{
        chain::genesis::{CurrencyGenesisConfig, GenesisBalance, GenesisBuilder, GenesisValidator},
        currency_transactions::{CurrencyTransactionType, TransferTx},
        signed_envelope::{address_from_public_key, EnvelopeBuilder},
        transactions::{Transaction, TransactionStatus, TransactionType},
    };
    use dchat_core::types::UserId;
    use ed25519_dalek::{Signer, SigningKey};
    use std::io::Write;
    use uuid::Uuid;

    fn test_genesis(signing_key: &SigningKey) -> (CurrencyWorldState, [u8; 32], [u8; 32], Uuid) {
        let public_key = signing_key.verifying_key().to_bytes();
        let account = address_from_public_key(&public_key);
        let builder = GenesisBuilder::new(signing_key.clone());
        let genesis = builder
            .create_currency_genesis_with_balances(
                vec![GenesisValidator {
                    public_key: hex::encode(public_key),
                    stake_amount: 100,
                    voting_power: 1,
                    address: "validator".to_string(),
                }],
                CurrencyGenesisConfig {
                    chain_id: "journal-test-currency".to_string(),
                    block_time_secs: 5,
                    token_name: "Test Token".to_string(),
                    token_symbol: "TEST".to_string(),
                    token_decimals: 8,
                },
                2_000,
                vec![
                    GenesisBalance {
                        account: UserId(account),
                        available: 1_000,
                        staked: 100,
                    },
                    GenesisBalance {
                        account: UserId(Uuid::from_bytes([7; 16])),
                        available: 0,
                        staked: 0,
                    },
                ],
            )
            .unwrap();
        let state = CurrencyWorldState::from_genesis(&genesis).unwrap();
        let genesis_hash = [9; 32];
        (state, genesis_hash, public_key, account)
    }

    fn test_transfer_block(
        signing_key: &SigningKey,
        state: &CurrencyWorldState,
        previous_hash: [u8; 32],
        amount: u128,
    ) -> CurrencyBlock {
        let public_key = signing_key.verifying_key().to_bytes();
        let sender = address_from_public_key(&public_key);
        let recipient = Uuid::from_bytes([7; 16]);
        let transfer = TransferTx {
            tx_id: Uuid::new_v4(),
            from: UserId(sender),
            to: UserId(recipient),
            amount,
            memo: None,
            fee: 1,
            timestamp: Utc::now(),
            nonce: 1,
            signature: Vec::new(),
        };
        let envelope = EnvelopeBuilder::new(1338, public_key)
            .nonce(1)
            .currency_tx(CurrencyTransactionType::Transfer)
            .payload(bincode::serialize(&transfer).unwrap())
            .build_and_sign(signing_key)
            .unwrap();
        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let transaction_hash = blake3::hash(&hash_input);
        let mut tx_id = [0; 16];
        tx_id.copy_from_slice(&transaction_hash.as_bytes()[..16]);
        let transaction = Transaction {
            tx_id: Uuid::from_bytes(tx_id),
            tx_type: TransactionType::CurrencyTransfer,
            payload: envelope.to_bytes().unwrap(),
            tx_hash: format!("0x{}", hex::encode(transaction_hash.as_bytes())),
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: 1,
        };
        CurrencyBlock::build(1, previous_hash, vec![transaction], state, 1338)
            .unwrap()
            .0
    }

    #[test]
    fn journal_frame_round_trips_and_checksums_payload() {
        let record = JournalRecord::Header {
            genesis_hash: [1; 32],
            chain_id: 1338,
            validator_id: [2; 32],
        };
        let frame = encode_frame(&record).unwrap();
        let (decoded, next_offset) = decode_frame(&frame, 0).unwrap().unwrap();
        assert_eq!(next_offset, frame.len());
        match decoded {
            JournalRecord::Header {
                genesis_hash,
                chain_id,
                validator_id,
            } => {
                assert_eq!(genesis_hash, [1; 32]);
                assert_eq!(chain_id, 1338);
                assert_eq!(validator_id, [2; 32]);
            }
            _ => panic!("decoded unexpected journal record"),
        }

        let mut corrupted = frame;
        corrupted[5] ^= 1;
        assert!(decode_frame(&corrupted, 0).is_err());
    }

    #[test]
    fn incomplete_journal_frame_is_detected_as_a_torn_tail() {
        let frame = encode_frame(&JournalRecord::Header {
            genesis_hash: [3; 32],
            chain_id: 1338,
            validator_id: [4; 32],
        })
        .unwrap();
        assert!(decode_frame(&frame[..frame.len() - 1], 0)
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn journal_recovers_finalized_state_and_pending_vote_lock() {
        let signing_key = SigningKey::from_bytes(&[21; 32]);
        let (genesis_state, genesis_hash, validator_id, _) = test_genesis(&signing_key);
        let authorized_validators = HashSet::from([validator_id]);
        let path = std::env::temp_dir()
            .join(format!("dchat-currency-journal-{}", Uuid::new_v4()))
            .join("validator.journal");
        let block = test_transfer_block(&signing_key, &genesis_state, genesis_hash, 100);
        let conflicting_block =
            test_transfer_block(&signing_key, &genesis_state, genesis_hash, 101);

        let (mut journal, recovery) = CurrencyChainJournal::open(
            &path,
            genesis_hash,
            1338,
            validator_id,
            authorized_validators.clone(),
            1,
            genesis_state.clone(),
        )
        .await
        .unwrap();
        assert_eq!(recovery.height, 0);
        journal.prepare_block(&block).await.unwrap();
        journal.record_vote(1, block.hash).await.unwrap();
        assert!(journal
            .record_vote(1, conflicting_block.hash)
            .await
            .is_err());
        drop(journal);

        let (mut journal, recovery) = CurrencyChainJournal::open(
            &path,
            genesis_hash,
            1338,
            validator_id,
            authorized_validators.clone(),
            1,
            genesis_state.clone(),
        )
        .await
        .unwrap();
        assert_eq!(recovery.height, 0);
        assert_eq!(recovery.pending_block.as_ref().unwrap().hash, block.hash);
        assert_eq!(journal.local_vote_hash(1), Some(block.hash));
        journal.record_vote(1, block.hash).await.unwrap();

        let signature = signing_key.sign(&block.hash).to_bytes().to_vec();
        let finalized_state = journal
            .finalize_pending(
                block.hash,
                vec![CurrencyFinalityVote {
                    validator_id,
                    signature,
                }],
            )
            .await
            .unwrap();
        let finalized_root = finalized_state.clone().compute_state_root();
        drop(journal);

        let valid_journal_length = std::fs::metadata(&path).unwrap().len();
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(&[0x01, 0x02])
            .unwrap();
        let (_journal, recovery) = CurrencyChainJournal::open(
            &path,
            genesis_hash,
            1338,
            validator_id,
            authorized_validators,
            1,
            genesis_state,
        )
        .await
        .unwrap();
        assert_eq!(recovery.height, 1);
        assert_eq!(recovery.last_hash, block.hash);
        assert!(recovery.pending_block.is_none());
        assert_eq!(recovery.state.clone().compute_state_root(), finalized_root);
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            valid_journal_length
        );
        drop(_journal);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
