//! Validates signed chat and currency transactions before they enter their mempools.

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

/// Currency-chain signed-envelope ingress with an independent nonce domain and mempool.
pub struct CurrencyTransactionIngress {
    mempool: Mempool,
    state: Mutex<IngressState>,
}

impl CurrencyTransactionIngress {
    /// Create currency ingress for one chain ID.
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

    /// Verify a signed currency transfer envelope and enqueue it once.
    pub fn submit_envelope(&self, envelope_bytes: &[u8]) -> Result<String> {
        let envelope: SignedTransactionEnvelope = deserialize_bounded(envelope_bytes)
            .map_err(|error| Error::validation(format!("Invalid signed envelope: {}", error)))?;
        if envelope.domain != EnvelopeDomain::Currency
            || envelope.tx_type
                != UnifiedTransactionType::Currency(crate::currency_transactions::CurrencyTransactionType::Transfer)
        {
            return Err(Error::validation(
                "Only currency transfer envelopes are accepted",
            ));
        }
        let transfer: crate::currency_transactions::TransferTx = decode_payload(&envelope)?;
        let sender = UserId(Uuid::from_bytes(envelope.sender));
        if transfer.from != sender || transfer.nonce != envelope.nonce || transfer.amount == 0 {
            return Err(Error::validation(
                "Currency transfer identity, nonce, or amount is invalid",
            ));
        }

        let mut hash_input = envelope.signing_bytes();
        hash_input.extend_from_slice(&envelope.signature);
        let hash = *blake3::hash(&hash_input).as_bytes();
        let tx_hash = format!("0x{}", hex::encode(hash));

        let mut state = self
            .state
            .lock()
            .map_err(|_| Error::internal("Currency transaction ingress lock poisoned"))?;
        if let Some(accepted_hash) = state.accepted.get(&hash) {
            return Ok(accepted_hash.clone());
        }
        let sender = envelope.sender_uuid();
        let expected_nonce = state.verifier.next_nonce(&sender);
        if envelope.nonce != expected_nonce {
            return Err(Error::validation(format!(
                "Unexpected currency transaction nonce: expected {}, got {}",
                expected_nonce, envelope.nonce
            )));
        }
        state.verifier.verify(&envelope).map_err(|error| {
            Error::validation(format!(
                "Currency envelope verification failed: {}",
                error
            ))
        })?;

        let mut tx_id_bytes = [0; 16];
        tx_id_bytes.copy_from_slice(&hash[..16]);
        let tx = Transaction {
            tx_id: Uuid::from_bytes(tx_id_bytes),
            tx_type: TransactionType::CurrencyTransfer,
            payload: envelope_bytes.to_vec(),
            tx_hash: tx_hash.clone(),
            status: TransactionStatus::Pending,
            submitted_at: Utc::now(),
            confirmed_at: None,
            fee_paid: transfer.fee,
        };
        self.mempool.add_transaction(tx)?;

        state
            .verifier
            .verify_and_accept(&envelope)
            .map_err(|error| {
                Error::internal(format!(
                    "Currency envelope verification changed during acceptance: {}",
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

    /// Return the mempool populated by this ingress.
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
        TransactionType::UpdateProfile => {
            let tx: crate::transactions::UpdateProfileTx = decode_payload(envelope)?;
            if tx.user_id != sender
                || tx.username.trim().is_empty()
                || tx.username.len() > 64
                || tx
                    .display_name
                    .as_ref()
                    .is_some_and(|name| name.len() > 128)
                || tx.avatar_hash.as_ref().is_some_and(|hash| hash.len() > 128)
                || tx.bio.as_ref().is_some_and(|bio| bio.len() > 512)
            {
                return Err(Error::validation(
                    "Profile update identity or fields are invalid",
                ));
            }
        }
        TransactionType::SubmitDeliveryProof => {
            let tx: crate::transactions::OnChainDeliveryProofTx = decode_payload(envelope)?;
            tx.verify(&envelope.public_key).map_err(|error| {
                Error::validation(format!("Invalid delivery proof: {}", error))
            })?;
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
    use crate::signed_envelope::{
        address_from_public_key, chain_ids, EnvelopeBuilder,
    };
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

    fn signed_delivery_proof() -> Vec<u8> {
        use ed25519_dalek::Signer;

        let relay_key = SigningKey::from_bytes(&[8; 32]);
        let recipient_key = SigningKey::from_bytes(&[7; 32]);
        let message_id = [3; 32];
        let timestamp = 1_000;
        let mut recipient_message = Vec::new();
        recipient_message.extend_from_slice(&message_id);
        recipient_message.extend_from_slice(relay_key.verifying_key().as_bytes());
        recipient_message.extend_from_slice(&timestamp.to_le_bytes());
        let recipient_signature = recipient_key.sign(&recipient_message).to_bytes();
        let mut relay_message = Vec::new();
        relay_message.extend_from_slice(&message_id);
        relay_message.extend_from_slice(recipient_key.verifying_key().as_bytes());
        relay_message.extend_from_slice(&recipient_signature);
        relay_message.extend_from_slice(&timestamp.to_le_bytes());
        let proof = crate::transactions::OnChainDeliveryProofTx {
            message_id,
            recipient_id: UserId(address_from_public_key(
                &recipient_key.verifying_key().to_bytes(),
            )),
            recipient_public_key: recipient_key.verifying_key().to_bytes(),
            recipient_signature: recipient_signature.to_vec(),
            timestamp,
            relay_signature: relay_key.sign(&relay_message).to_bytes().to_vec(),
        };
        let envelope = EnvelopeBuilder::new(
            chain_ids::TESTNET_CHAT,
            relay_key.verifying_key().to_bytes(),
        )
        .nonce(1)
        .chat_tx(TransactionType::SubmitDeliveryProof)
        .payload(bincode::serialize(&proof).unwrap())
        .build_and_sign(&relay_key)
        .unwrap();
        envelope.to_bytes().unwrap()
    }

    fn signed_currency_transfer(signing_key: &SigningKey, nonce: u64) -> Vec<u8> {
        let public_key = signing_key.verifying_key().to_bytes();
        let transfer = crate::currency_transactions::TransferTx {
            tx_id: Uuid::new_v4(),
            from: UserId(address_from_public_key(&public_key)),
            to: UserId::new(),
            amount: 25,
            memo: None,
            fee: 2,
            timestamp: Utc::now(),
            nonce,
            signature: Vec::new(),
        };
        EnvelopeBuilder::new(chain_ids::TESTNET_CURRENCY, public_key)
            .nonce(nonce)
            .currency_tx(crate::currency_transactions::CurrencyTransactionType::Transfer)
            .payload(bincode::serialize(&transfer).unwrap())
            .build_and_sign(signing_key)
            .unwrap()
            .to_bytes()
            .unwrap()
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
    fn accepts_and_deduplicates_a_currency_transfer_envelope() {
        let mempool = Mempool::default();
        let ingress =
            CurrencyTransactionIngress::new(chain_ids::TESTNET_CURRENCY, mempool.clone());
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let envelope = signed_currency_transfer(&signing_key, 1);

        let first = ingress.submit_envelope(&envelope).unwrap();
        let second = ingress.submit_envelope(&envelope).unwrap();

        assert_eq!(first, second);
        assert_eq!(mempool.len().unwrap(), 1);
        assert_eq!(
            mempool.pending_transactions().unwrap()[0].tx_type,
            TransactionType::CurrencyTransfer
        );
    }

    #[test]
    fn rejects_currency_nonce_gaps_without_queueing() {
        let mempool = Mempool::default();
        let ingress =
            CurrencyTransactionIngress::new(chain_ids::TESTNET_CURRENCY, mempool.clone());
        let signing_key = SigningKey::from_bytes(&[10; 32]);

        assert!(ingress
            .submit_envelope(&signed_currency_transfer(&signing_key, 2))
            .is_err());
        assert!(mempool.is_empty().unwrap());
    }

    #[test]
    fn accepts_a_profile_update_for_the_signed_user() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let user_id = UserId(address_from_public_key(&public_key));
        let payload = bincode::serialize(&crate::transactions::UpdateProfileTx {
            user_id,
            username: "alice-new".to_string(),
            display_name: Some("Alice".to_string()),
            avatar_hash: Some("ab".repeat(32)),
            bio: Some("Hello".to_string()),
        })
        .unwrap();
        let envelope = EnvelopeBuilder::new(chain_ids::TESTNET_CHAT, public_key)
            .nonce(1)
            .chat_tx(TransactionType::UpdateProfile)
            .payload(payload)
            .build_and_sign(&signing_key)
            .unwrap();

        assert!(ingress
            .submit_envelope(&envelope.to_bytes().unwrap())
            .is_ok());
        assert_eq!(ingress.mempool().len().unwrap(), 1);
    }

    #[test]
    fn rejects_profile_updates_for_another_user() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let payload = bincode::serialize(&crate::transactions::UpdateProfileTx {
            user_id: UserId::new(),
            username: "alice-new".to_string(),
            display_name: None,
            avatar_hash: None,
            bio: None,
        })
        .unwrap();
        let envelope = EnvelopeBuilder::new(chain_ids::TESTNET_CHAT, public_key)
            .nonce(1)
            .chat_tx(TransactionType::UpdateProfile)
            .payload(payload)
            .build_and_sign(&signing_key)
            .unwrap();

        assert!(ingress
            .submit_envelope(&envelope.to_bytes().unwrap())
            .is_err());
        assert_eq!(ingress.mempool().len().unwrap(), 0);
    }

    #[test]
    fn accepts_delivery_proofs_with_both_valid_signatures() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let bytes = signed_delivery_proof();

        assert!(ingress.submit_envelope(&bytes).is_ok());
        assert_eq!(ingress.mempool().len().unwrap(), 1);
    }

    #[test]
    fn rejects_delivery_proofs_with_an_invalid_recipient_signature() {
        let ingress = TransactionIngress::new(chain_ids::TESTNET_CHAT, Mempool::default());
        let relay_key = SigningKey::from_bytes(&[8; 32]);
        let mut envelope =
            SignedTransactionEnvelope::from_bytes(&signed_delivery_proof()).unwrap();
        let mut proof: crate::transactions::OnChainDeliveryProofTx =
            bincode::deserialize(&envelope.payload).unwrap();
        proof.recipient_signature[0] ^= 0x80;
        envelope.payload = bincode::serialize(&proof).unwrap();
        envelope.payload_hash = *blake3::hash(&envelope.payload).as_bytes();
        envelope.sign(&relay_key);

        assert!(ingress
            .submit_envelope(&envelope.to_bytes().unwrap())
            .is_err());
        assert_eq!(ingress.mempool().len().unwrap(), 0);
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
