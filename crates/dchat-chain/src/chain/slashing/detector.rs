// Slashing Detector - Byzantine Behavior Detection
//
// Detects equivocation (double-signing), invalid proofs, censorship,
// and other slashable offenses by validators and relays.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime};
use thiserror::Error;

use dchat_core::config::constants::{
    SLASH_RATE_CENSORSHIP, SLASH_RATE_DOUBLE_SIGN, SLASH_RATE_INVALID_PROOF, SLASH_RATE_LOW_UPTIME,
};

/// Errors that can occur during slashing detection
#[derive(Debug, Error)]
pub enum SlashingError {
    #[error("Invalid signature format")]
    InvalidSignature,

    #[error("Evidence verification failed: {0}")]
    VerificationFailed(String),

    #[error("Insufficient evidence for slashing")]
    InsufficientEvidence,

    #[error("Validator not found: {0:?}")]
    ValidatorNotFound(VerifyingKey),

    #[error("Already slashed for this offense")]
    AlreadySlashed,
}

/// Types of slashable offenses
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SlashableOffense {
    /// Double-signing: signing two conflicting blocks at same height
    DoubleSigning {
        block_height: u64,
        signature1: Vec<u8>,
        signature2: Vec<u8>,
        message1: Vec<u8>,
        message2: Vec<u8>,
        message1_hash: Vec<u8>,
        message2_hash: Vec<u8>,
    },

    /// Invalid proof submission (delivery proof, fraud proof, etc.)
    InvalidProof {
        proof_type: String,
        proof_hash: Vec<u8>,
        reason: String,
    },

    /// Censorship: withholding valid transactions
    Censorship {
        transaction_hashes: Vec<Vec<u8>>,
        withheld_duration_secs: u64,
    },

    /// Low uptime below threshold
    LowUptime {
        uptime_percentage: f64,
        measurement_period_secs: u64,
    },
}

impl SlashableOffense {
    /// Get the slash rate for this offense
    pub fn slash_rate(&self) -> f64 {
        match self {
            SlashableOffense::DoubleSigning { .. } => SLASH_RATE_DOUBLE_SIGN,
            SlashableOffense::InvalidProof { .. } => SLASH_RATE_INVALID_PROOF,
            SlashableOffense::Censorship { .. } => SLASH_RATE_CENSORSHIP,
            SlashableOffense::LowUptime { .. } => SLASH_RATE_LOW_UPTIME,
        }
    }

    /// Is this offense severe enough to ban the validator?
    pub fn requires_ban(&self) -> bool {
        matches!(self, SlashableOffense::DoubleSigning { .. })
    }
}

/// Signature record for double-sign detection
#[derive(Debug, Clone)]
pub struct SignatureRecord {
    /// Validator public key
    pub validator: VerifyingKey,
    /// Block height of the signature
    pub block_height: u64,
    /// The signature itself
    pub signature: Signature,
    /// Exact bytes that were signed
    pub message: Vec<u8>,
    /// Hash of the message that was signed
    pub message_hash: [u8; 32],
    /// When the signature was recorded
    pub timestamp: SystemTime,
}

/// Slashing detector tracks validator behavior and detects offenses
pub struct SlashingDetector {
    /// Map of (validator, block_height) -> signature records
    seen_signatures: Arc<RwLock<HashMap<(VerifyingKey, u64), Vec<SignatureRecord>>>>,

    /// Detected offenses awaiting confirmation
    pending_slashes: Arc<RwLock<HashMap<VerifyingKey, Vec<SlashableOffense>>>>,

    /// Already slashed validators (to prevent double-slashing)
    slashed_validators: Arc<RwLock<HashSet<VerifyingKey>>>,

    /// Last check timestamp
    last_check: Arc<RwLock<Instant>>,
}

impl SlashingDetector {
    /// Get the timestamp of the last check
    pub fn last_check(&self) -> Instant {
        *self.last_check.read().unwrap()
    }

    /// Create a new slashing detector
    pub fn new() -> Self {
        Self {
            seen_signatures: Arc::new(RwLock::new(HashMap::new())),
            pending_slashes: Arc::new(RwLock::new(HashMap::new())),
            slashed_validators: Arc::new(RwLock::new(HashSet::new())),
            last_check: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Record a validator signature and check for double-signing
    pub fn record_signature(
        &self,
        validator: VerifyingKey,
        block_height: u64,
        signature: Signature,
        message: &[u8],
    ) -> Result<(), SlashingError> {
        validator
            .verify(message, &signature)
            .map_err(|_| SlashingError::InvalidSignature)?;

        let message_hash = *blake3::hash(message).as_bytes();
        let mut signatures = self.seen_signatures.write().unwrap();
        let key = (validator, block_height);

        let record = SignatureRecord {
            validator,
            block_height,
            signature,
            message_hash,
            message: message.to_vec(),
            timestamp: SystemTime::now(),
        };

        // Check if we've seen a different signature for this validator/height
        if let Some(existing_records) = signatures.get(&key) {
            for existing in existing_records {
                if existing.message.as_slice() == message {
                    return Ok(());
                }

                // Different message hash = equivocation!
                tracing::warn!(
                    "🚨 DOUBLE-SIGN DETECTED: Validator {:?} at height {}",
                    validator,
                    block_height
                );

                let offense = SlashableOffense::DoubleSigning {
                    block_height,
                    signature1: existing.signature.to_bytes().to_vec(),
                    signature2: signature.to_bytes().to_vec(),
                    message1: existing.message.clone(),
                    message2: message.to_vec(),
                    message1_hash: existing.message_hash.to_vec(),
                    message2_hash: message_hash.to_vec(),
                };

                self.flag_for_slashing(validator, offense)?;
                return Err(SlashingError::VerificationFailed(
                    "Double-signing detected".to_string(),
                ));
            }
        }

        // Store this signature
        signatures.entry(key).or_insert_with(Vec::new).push(record);

        Ok(())
    }

    /// Flag a validator for slashing
    pub fn flag_for_slashing(
        &self,
        validator: VerifyingKey,
        offense: SlashableOffense,
    ) -> Result<(), SlashingError> {
        // Check if already slashed
        let slashed = self.slashed_validators.read().unwrap();
        if slashed.contains(&validator) {
            return Err(SlashingError::AlreadySlashed);
        }
        drop(slashed);

        let mut pending = self.pending_slashes.write().unwrap();
        let offenses = pending.entry(validator).or_default();
        if offenses.contains(&offense) {
            return Ok(());
        }
        offenses.push(offense.clone());

        tracing::warn!(
            "⚠️  Validator {:?} flagged for slashing: {:?}",
            validator,
            offense
        );

        Ok(())
    }

    /// Report invalid proof submission
    pub fn report_invalid_proof(
        &self,
        validator: VerifyingKey,
        proof_type: String,
        proof_hash: Vec<u8>,
        reason: String,
    ) -> Result<(), SlashingError> {
        let offense = SlashableOffense::InvalidProof {
            proof_type,
            proof_hash,
            reason,
        };

        self.flag_for_slashing(validator, offense)
    }

    /// Report censorship behavior
    pub fn report_censorship(
        &self,
        validator: VerifyingKey,
        transaction_hashes: Vec<Vec<u8>>,
        withheld_duration_secs: u64,
    ) -> Result<(), SlashingError> {
        let offense = SlashableOffense::Censorship {
            transaction_hashes,
            withheld_duration_secs,
        };

        self.flag_for_slashing(validator, offense)
    }

    /// Report double signing evidence directly
    pub fn report_double_sign(
        &self,
        validator: VerifyingKey,
        block_height: u64,
        signature1: Vec<u8>,
        signature2: Vec<u8>,
        message1: Vec<u8>,
        message2: Vec<u8>,
        message1_hash: Vec<u8>,
        message2_hash: Vec<u8>,
    ) -> Result<(), SlashingError> {
        let expected_message1_hash = blake3::hash(&message1);
        let expected_message2_hash = blake3::hash(&message2);
        if message1 == message2
            || message1_hash.as_slice() != expected_message1_hash.as_bytes().as_slice()
            || message2_hash.as_slice() != expected_message2_hash.as_bytes().as_slice()
        {
            return Err(SlashingError::VerificationFailed(
                "Invalid double-sign message hashes".to_string(),
            ));
        }
        let signature1_bytes: [u8; 64] = signature1
            .as_slice()
            .try_into()
            .map_err(|_| SlashingError::InvalidSignature)?;
        let signature2_bytes: [u8; 64] = signature2
            .as_slice()
            .try_into()
            .map_err(|_| SlashingError::InvalidSignature)?;
        validator
            .verify(&message1, &Signature::from_bytes(&signature1_bytes))
            .map_err(|_| SlashingError::InvalidSignature)?;
        validator
            .verify(&message2, &Signature::from_bytes(&signature2_bytes))
            .map_err(|_| SlashingError::InvalidSignature)?;

        let offense = SlashableOffense::DoubleSigning {
            block_height,
            signature1,
            signature2,
            message1,
            message2,
            message1_hash,
            message2_hash,
        };
        self.flag_for_slashing(validator, offense)
    }

    /// Report low uptime
    pub fn report_low_uptime(
        &self,
        validator: VerifyingKey,
        uptime_percentage: f64,
        measurement_period_secs: u64,
    ) -> Result<(), SlashingError> {
        if uptime_percentage >= 0.90 {
            return Ok(()); // Above threshold, no slashing
        }

        let offense = SlashableOffense::LowUptime {
            uptime_percentage,
            measurement_period_secs,
        };

        self.flag_for_slashing(validator, offense)
    }

    /// Get all pending slashes for submission to chain
    pub fn get_pending_slashes(&self) -> HashMap<VerifyingKey, Vec<SlashableOffense>> {
        self.pending_slashes.read().unwrap().clone()
    }

    /// Mark a validator as slashed (after on-chain confirmation)
    pub fn confirm_slashing(&self, validator: VerifyingKey) {
        let mut slashed = self.slashed_validators.write().unwrap();
        slashed.insert(validator);

        // Remove from pending
        let mut pending = self.pending_slashes.write().unwrap();
        pending.remove(&validator);

        tracing::info!("✅ Slashing confirmed for validator {:?}", validator);
    }

    /// Clear old signature records (older than 1 hour)
    pub fn prune_old_records(&self) {
        let cutoff = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(3600))
            .unwrap();

        let mut signatures = self.seen_signatures.write().unwrap();
        signatures.retain(|_, records| {
            records.retain(|r| r.timestamp > cutoff);
            !records.is_empty()
        });
    }

    /// Get statistics
    pub fn get_stats(&self) -> SlashingStats {
        let signatures = self.seen_signatures.read().unwrap();
        let pending = self.pending_slashes.read().unwrap();
        let slashed = self.slashed_validators.read().unwrap();

        SlashingStats {
            tracked_signatures: signatures.len(),
            pending_slashes: pending.len(),
            total_slashed: slashed.len(),
        }
    }
}

impl Default for SlashingDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SlashingStats {
    pub tracked_signatures: usize,
    pub pending_slashes: usize,
    pub total_slashed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    #[test]
    fn test_double_sign_detection() {
        let detector = SlashingDetector::new();
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let block_height = 100u64;
        let message1 = b"block_data_1";
        let message2 = b"block_data_2";

        let sig1 = signing_key.sign(message1);
        let sig2 = signing_key.sign(message2);

        let hash1 = blake3::hash(message1);
        let hash2 = blake3::hash(message2);
        let sig1_bytes = sig1.to_bytes();
        let sig2_bytes = sig2.to_bytes();

        // First signature should succeed
        assert!(detector
            .record_signature(verifying_key, block_height, sig1, message1)
            .is_ok());

        // Second signature with different message hash should fail
        let result = detector.record_signature(verifying_key, block_height, sig2, message2);
        assert!(result.is_err());

        // Should have pending slash
        let pending = detector.get_pending_slashes();
        assert_eq!(pending.len(), 1);
        let offenses = pending.get(&verifying_key).unwrap();
        assert_eq!(offenses.len(), 1);
        assert!(matches!(
            &offenses[0],
            SlashableOffense::DoubleSigning {
                block_height: 100,
                signature1,
                signature2,
                message1: recorded_message1,
                message2: recorded_message2,
                message1_hash,
                message2_hash,
            } if signature1.as_slice() == sig1_bytes.as_slice()
                && signature2.as_slice() == sig2_bytes.as_slice()
                && recorded_message1 == message1
                && recorded_message2 == message2
                && message1_hash.as_slice() == hash1.as_bytes()
                && message2_hash.as_slice() == hash2.as_bytes()
        ));

        assert!(detector
            .record_signature(
                verifying_key,
                block_height,
                signing_key.sign(message2),
                message2
            )
            .is_err());
        assert_eq!(
            detector
                .get_pending_slashes()
                .get(&verifying_key)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_record_signature_rejects_invalid_signature() {
        let detector = SlashingDetector::new();
        let signing_key = SigningKey::generate(&mut OsRng);
        let other_key = SigningKey::generate(&mut OsRng);
        let message = b"block_data";
        let invalid_signature = other_key.sign(message);

        assert!(matches!(
            detector.record_signature(signing_key.verifying_key(), 100, invalid_signature, message),
            Err(SlashingError::InvalidSignature)
        ));
        assert!(detector.get_pending_slashes().is_empty());
    }

    #[test]
    fn test_invalid_proof_reporting() {
        let detector = SlashingDetector::new();
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let result = detector.report_invalid_proof(
            verifying_key,
            "delivery_proof".to_string(),
            vec![1, 2, 3, 4],
            "Invalid signature".to_string(),
        );

        assert!(result.is_ok());

        let pending = detector.get_pending_slashes();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_censorship_reporting() {
        let detector = SlashingDetector::new();
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let tx_hashes = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let result = detector.report_censorship(verifying_key, tx_hashes, 300);

        assert!(result.is_ok());
    }

    #[test]
    fn test_low_uptime_threshold() {
        let detector = SlashingDetector::new();
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        // Above threshold - no slashing
        assert!(detector
            .report_low_uptime(verifying_key, 0.95, 86400)
            .is_ok());
        assert_eq!(detector.get_pending_slashes().len(), 0);

        // Below threshold - should slash
        assert!(detector
            .report_low_uptime(verifying_key, 0.85, 86400)
            .is_ok());
        assert_eq!(detector.get_pending_slashes().len(), 1);
    }

    #[test]
    fn test_prevent_double_slashing() {
        let detector = SlashingDetector::new();
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        // Flag for slashing
        let offense = SlashableOffense::InvalidProof {
            proof_type: "test".to_string(),
            proof_hash: vec![1, 2, 3],
            reason: "test".to_string(),
        };
        detector
            .flag_for_slashing(verifying_key, offense.clone())
            .unwrap();

        // Confirm slashing
        detector.confirm_slashing(verifying_key);

        // Try to flag again - should fail
        let result = detector.flag_for_slashing(verifying_key, offense);
        assert!(matches!(result, Err(SlashingError::AlreadySlashed)));
    }
}
