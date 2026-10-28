//! Validates signed chat transactions before they enter the mempool.

use crate::mempool::Mempool;
use crate::signed_envelope::{
    EnvelopeDomain, EnvelopeVerifier, SignedTransactionEnvelope, UnifiedTransactionType,
};
use crate::transactions::{Transaction, TransactionStatus, TransactionType};
use bincode::Options;
use chrono::Utc;
use dchat_core::error::{Error, Result};
use dchat_core::types::UserId;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use uuid::Uuid;

const MAX_RECENTLY_ACCEPTED: usize = 10_000;
const MAX_ENVELOPE_BYTES: usize = 512 * 1024 - 256;

struct IngressState {
    verifier: EnvelopeVerifier,
    accepted: HashMap<[u8; 32], String>,
    accepted_order: VecDeque<[u8; 32]>,
}

/// Signed-envelope ingress and the pending transaction queue it feeds.
pub struct TransactionIngress {
    mempool: Mempool,
    state: Mutex<IngressState>,
}

impl TransactionIngress {
    /// Create an ingress for one chat-chain ID.
    pub fn new(chain_id: u32, mempool: Mempool) -> Self {
        Self {
            mempool,
            state: Mutex::new(IngressState {
                verifier: EnvelopeVerifier::new(chain_id),
                accepted: HashMap::new(),
                accepted_order: VecDeque::new(),
            }),
        }
    }

    /// Verify a serialized signed envelope and enqueue it once.
    ///
    /// The queued payload retains the complete envelope so downstream block
    /// validation can verify the sender's signature again.
    pub fn submit_envelope(&self, envelope_bytes: &[u8]) -> Result<String> {
        let envelope: SignedTransactionEnvelope = deserialize_bounded(envelope_bytes)
            .map_err(|error| Error::validation(format!("Invalid signed envelope: {}", error)))?;
        if envelope.domain != EnvelopeDomain::Chat {
            return Err(Error::validation(
                "Only chat-chain transaction envelopes are accepted",
            ));
        }

        let tx_type = match envelope.tx_type {
            UnifiedTransactionType::Chat(tx_type) => tx_type,
            UnifiedTransactionType::Currency(_) => {
                return Err(Error::validation(
                    "Currency transaction submitted to chat-chain ingress",
                ));
            }
        };
        validate_payload(&envelope, tx_type)?;

        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let hash = *blake3::hash(&hash_input).as_bytes();
        let tx_hash = format!("0x{}", hex::encode(hash));

        let mut state = self
            .state
            .lock()
            .map_err(|_| Error::internal("Transaction ingress lock poisoned"))?;
        if let Some(accepted_hash) = state.accepted.get(&hash) {
            return Ok(accepted_hash.clone());
        }
        let sender = envelope.sender_uuid();
        let expected_nonce = state.verifier.next_nonce(&sender);
        if envelope.nonce != expected_nonce {
            return Err(Error::validation(format!(
                "Unexpected transaction nonce: expected {}, got {}",
                expected_nonce, envelope.nonce
            )));
        }
        state.verifier.verify(&envelope).map_err(|error| {
            Error::validation(format!("Envelope verification failed: {}", error))
        })?;

        let mut tx_id_bytes = [0; 16];
        tx_id_bytes.copy_from_slice(&hash[..16]);
        let tx = Transaction {
            tx_id: Uuid::from_bytes(tx_id_bytes),
            tx_type,
            payload: envelope_bytes.to_vec(),
            tx_hash: tx_hash.clone(),
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: 0,
        };
        self.mempool.add_transaction(tx)?;

        state
            .verifier
            .verify_and_accept(&envelope)
            .map_err(|error| {
                Error::internal(format!(
                    "Envelope verification changed during acceptance: {}",
                    error
                ))
            })?;
        if state.accepted_order.len() == MAX_RECENTLY_ACCEPTED {
            if let Some(expired_hash) = state.accepted_order.pop_front() {
                state.accepted.remove(&expired_hash);
            }
        }
        state.accepted_order.push_back(hash);
        state.accepted.insert(hash, tx_hash.clone());
        Ok(tx_hash)
    }

    /// Return a handle to the mempool populated by this ingress.
    pub fn mempool(&self) -> Mempool {
        self.mempool.clone()
    }
}

fn validate_payload(envelope: &SignedTransactionEnvelope, tx_type: TransactionType) -> Result<()> {
    let sender = UserId(Uuid::from_bytes(envelope.sender));
    match tx_type {
        TransactionType::RegisterUser => {
            let tx: crate::transactions::RegisterUserTx = decode_payload(envelope)?;
            if tx.user_id != sender || tx.public_key != hex::encode(envelope.public_key) {
                return Err(Error::validation(
                    "Registration payload identity does not match signed envelope",
                ));
            }
        }
        TransactionType::SendDirectMessage => {
            let tx: crate::transactions::SendDirectMessageTx = decode_payload(envelope)?;
            if tx.sender_id != sender {
                return Err(Error::validation(
                    "Direct-message sender does not match signed envelope",
                ));
            }
        }
        TransactionType::CreateChannel => {
            let tx: crate::transactions::CreateChannelTx = decode_payload(envelope)?;
            if tx.creator_id != sender {
                return Err(Error::validation(
                    "Channel creator does not match signed envelope",
                ));
            }
        }
        TransactionType::PostToChannel => {
            let tx: crate::transactions::PostToChannelTx = decode_payload(envelope)?;
            if tx.sender_id != sender {
                return Err(Error::validation(
                    "Channel-message sender does not match signed envelope",
                ));
            }
        }
        TransactionType::JoinChannel => {
            let tx: crate::transactions::JoinChannelTx = decode_payload(envelope)?;
            if tx.user_id != sender {
                return Err(Error::validation(
                    "Channel-join user does not match signed envelope",
                ));
            }
        }
        TransactionType::GovernanceProposal => {
            let tx: crate::governance_transactions::GovernanceTransaction =
                decode_payload(envelope)?;
            let crate::governance_transactions::GovernanceTransaction::SubmitProposal(proposal) =
                tx
            else {
                return Err(Error::validation(
                    "Governance proposal envelope must submit a proposal",
                ));
            };
            if proposal.proposer != sender {
                return Err(Error::validation(
                    "Governance proposer does not match signed envelope",
                ));
            }
        }
        TransactionType::GovernanceVote => {
            let tx: crate::governance_transactions::GovernanceTransaction =
                decode_payload(envelope)?;
            let signer = match tx {
                crate::governance_transactions::GovernanceTransaction::CommitVote(vote) => {
                    vote.voter
                }
                crate::governance_transactions::GovernanceTransaction::RevealVote(vote) => {
                    vote.voter
                }
                _ => {
                    return Err(Error::validation(
                        "Governance vote envelope must commit or reveal a vote",
                    ));
                }
            };
            if signer != sender {
                return Err(Error::validation(
                    "Governance voter does not match signed envelope",
                ));
            }
        }
        unsupported => {
            return Err(Error::validation(format!(
                "Transaction type {:?} is not supported by chat-chain ingress",
                unsupported
            )));
        }
    }
    Ok(())
}

fn decode_payload<T: serde::de::DeserializeOwned>(
    envelope: &SignedTransactionEnvelope,
) -> Result<T> {
    deserialize_bounded(&envelope.payload)
        .map_err(|error| Error::validation(format!("Invalid transaction payload: {}", error)))
}

fn deserialize_bounded<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(Box::new(bincode::ErrorKind::SizeLimit));
    }

    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(MAX_ENVELOPE_BYTES as u64)
        .deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signed_envelope::{address_from_public_key, chain_ids, EnvelopeBuilder};
    use crate::transactions::RegisterUserTx;
    use ed25519_dalek::SigningKey;

    fn signed_registration(nonce: u64) -> (SigningKey, Vec<u8>) {
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
        (signing_key, envelope.to_bytes().unwrap())
    }

    #[test]
    fn accepts_and_deduplicates_a_valid_signed_envelope() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let (_, bytes) = signed_registration(1);

        let first_hash = ingress.submit_envelope(&bytes).unwrap();
        let retry_hash = ingress.submit_envelope(&bytes).unwrap();

        assert_eq!(first_hash, retry_hash);
        assert!(first_hash.starts_with("0x"));
        assert_eq!(ingress.mempool().len().unwrap(), 1);
    }

    #[test]
    fn rejects_invalid_signatures_without_consuming_the_nonce() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let (_, bytes) = signed_registration(1);
        let mut envelope = SignedTransactionEnvelope::from_bytes(&bytes).unwrap();
        envelope.signature[0] ^= 0x80;
        let invalid_bytes = envelope.to_bytes().unwrap();

        assert!(ingress.submit_envelope(&invalid_bytes).is_err());
        assert_eq!(ingress.mempool().len().unwrap(), 0);

        let (_, valid_bytes) = signed_registration(1);
        assert!(ingress.submit_envelope(&valid_bytes).is_ok());
        assert_eq!(ingress.mempool().len().unwrap(), 1);
    }

    #[test]
    fn rejects_payload_sender_mismatch_and_wrong_chain() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let (_, bytes) = signed_registration(1);
        let mut envelope = SignedTransactionEnvelope::from_bytes(&bytes).unwrap();
        let mut registration: RegisterUserTx = bincode::deserialize(&envelope.payload).unwrap();
        registration.user_id = UserId::new();
        envelope.payload = bincode::serialize(&registration).unwrap();
        envelope.payload_hash = *blake3::hash(&envelope.payload).as_bytes();
        envelope.sign(&SigningKey::from_bytes(&[7; 32]));
        assert!(ingress
            .submit_envelope(&envelope.to_bytes().unwrap())
            .is_err());

        let (_, bytes) = signed_registration(1);
        let mut envelope = SignedTransactionEnvelope::from_bytes(&bytes).unwrap();
        envelope.chain_id = chain_ids::MAINNET_CHAT;
        envelope.sign(&SigningKey::from_bytes(&[7; 32]));
        assert!(ingress
            .submit_envelope(&envelope.to_bytes().unwrap())
            .is_err());
        assert_eq!(ingress.mempool().len().unwrap(), 0);
    }

    #[test]
    fn rejects_malformed_envelopes_wrong_domains_and_replayed_nonces() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        assert!(ingress.submit_envelope(&[]).is_err());

        let (_, bytes) = signed_registration(1);
        let mut wrong_domain = SignedTransactionEnvelope::from_bytes(&bytes).unwrap();
        wrong_domain.domain = EnvelopeDomain::Currency;
        wrong_domain.sign(&SigningKey::from_bytes(&[7; 32]));
        assert!(ingress
            .submit_envelope(&wrong_domain.to_bytes().unwrap())
            .is_err());

        assert!(ingress.submit_envelope(&bytes).is_ok());
        let (_, replayed_nonce) = signed_registration(1);
        assert!(ingress.submit_envelope(&replayed_nonce).is_err());
        assert_eq!(ingress.mempool().len().unwrap(), 1);
    }

    #[test]
    fn requires_contiguous_sender_nonces() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let (_, nonce_two) = signed_registration(2);
        let (_, nonce_one) = signed_registration(1);

        assert!(ingress.submit_envelope(&nonce_two).is_err());
        assert!(ingress.submit_envelope(&nonce_one).is_ok());
        assert!(ingress.submit_envelope(&nonce_two).is_ok());
        assert_eq!(ingress.mempool().len().unwrap(), 2);
    }

    #[test]
    fn rejects_transactions_when_the_mempool_is_full() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::new(0));
        let (_, bytes) = signed_registration(1);

        assert!(ingress.submit_envelope(&bytes).is_err());
        assert_eq!(ingress.mempool().len().unwrap(), 0);
    }
}
