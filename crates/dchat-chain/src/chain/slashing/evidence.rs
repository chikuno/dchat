// Evidence Collection & Verification
//
// Collects cryptographic evidence of slashable offenses,
// verifies validity, and prepares for on-chain submission.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;

use super::detector::SlashableOffense;

/// Errors during evidence processing
#[derive(Debug, Error)]
pub enum EvidenceError {
    #[error("Evidence verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid signature in evidence")]
    InvalidSignature,

    #[error("Evidence expired")]
    Expired,

    #[error("Incomplete evidence")]
    Incomplete,

    #[error("Cryptographic verification failed")]
    CryptoVerificationFailed,
}

/// Types of evidence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    /// Cryptographic proof of double-signing
    DoubleSigning,

    /// Invalid proof with verification failure details
    InvalidProof,

    /// Transaction withholding evidence
    Censorship,

    /// Uptime metrics below threshold
    LowUptime,
}

/// Slashing evidence bundle ready for on-chain submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    /// Accused validator's public key
    pub accused: Vec<u8>,

    /// Type of evidence
    pub evidence_type: EvidenceType,

    /// Serialized offense data
    pub offense_data: Vec<u8>,

    /// Witnesses who observed/verified this evidence
    pub witnesses: Vec<Vec<u8>>,

    /// Timestamp of evidence collection
    pub timestamp: u64,

    /// Evidence hash for deduplication
    pub evidence_hash: Vec<u8>,

    /// Reporter's signature (cryptographic proof of submission)
    pub reporter_signature: Option<Vec<u8>>,
}

impl SlashingEvidence {
    /// Create evidence from a slashable offense
    pub fn from_offense(
        accused: VerifyingKey,
        offense: &SlashableOffense,
        witnesses: Vec<VerifyingKey>,
    ) -> Self {
        let evidence_type = match offense {
            SlashableOffense::DoubleSigning { .. } => EvidenceType::DoubleSigning,
            SlashableOffense::InvalidProof { .. } => EvidenceType::InvalidProof,
            SlashableOffense::Censorship { .. } => EvidenceType::Censorship,
            SlashableOffense::LowUptime { .. } => EvidenceType::LowUptime,
        };

        let offense_data = serde_json::to_vec(&offense).unwrap_or_default();
        let accused_bytes = accused.as_bytes().to_vec();
        let witness_bytes: Vec<Vec<u8>> = witnesses.iter().map(|w| w.as_bytes().to_vec()).collect();

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compute evidence hash (accused || evidence_type || offense_data || timestamp)
        let mut hash_input = accused_bytes.clone();
        hash_input.push(evidence_type.clone() as u8);
        hash_input.extend_from_slice(&offense_data);
        hash_input.extend_from_slice(&timestamp.to_le_bytes());

        let evidence_hash = blake3::hash(&hash_input).as_bytes().to_vec();

        Self {
            accused: accused_bytes,
            evidence_type,
            offense_data,
            witnesses: witness_bytes,
            timestamp,
            evidence_hash,
            reporter_signature: None,
        }
    }

    /// Verify the cryptographic integrity of double-signing evidence
    pub fn verify_double_sign_evidence(&self) -> Result<(), EvidenceError> {
        if self.evidence_type != EvidenceType::DoubleSigning {
            return Err(EvidenceError::VerificationFailed(
                "Not a double-signing evidence".to_string(),
            ));
        }

        // Deserialize offense data
        let offense: SlashableOffense = serde_json::from_slice(&self.offense_data)
            .map_err(|e| EvidenceError::VerificationFailed(e.to_string()))?;

        if let SlashableOffense::DoubleSigning {
            signature1,
            signature2,
            message1,
            message2,
            message1_hash,
            message2_hash,
            ..
        } = offense
        {
            // Verify signatures are different
            if signature1 == signature2 {
                return Err(EvidenceError::VerificationFailed(
                    "Identical signatures".to_string(),
                ));
            }

            // Verify message hashes are different
            if message1_hash == message2_hash {
                return Err(EvidenceError::VerificationFailed(
                    "Identical message hashes".to_string(),
                ));
            }
            if message1 == message2 {
                return Err(EvidenceError::VerificationFailed(
                    "Identical signed messages".to_string(),
                ));
            }

            let accused_bytes: [u8; 32] = self
                .accused
                .as_slice()
                .try_into()
                .map_err(|_| EvidenceError::InvalidSignature)?;
            let accused = VerifyingKey::from_bytes(&accused_bytes)
                .map_err(|_| EvidenceError::InvalidSignature)?;
            let signature1_bytes: [u8; 64] = signature1
                .as_slice()
                .try_into()
                .map_err(|_| EvidenceError::InvalidSignature)?;
            let signature2_bytes: [u8; 64] = signature2
                .as_slice()
                .try_into()
                .map_err(|_| EvidenceError::InvalidSignature)?;

            if blake3::hash(&message1).as_bytes().as_slice() != message1_hash.as_slice()
                || blake3::hash(&message2).as_bytes().as_slice() != message2_hash.as_slice()
            {
                return Err(EvidenceError::CryptoVerificationFailed);
            }
            accused
                .verify(&message1, &Signature::from_bytes(&signature1_bytes))
                .map_err(|_| EvidenceError::CryptoVerificationFailed)?;
            accused
                .verify(&message2, &Signature::from_bytes(&signature2_bytes))
                .map_err(|_| EvidenceError::CryptoVerificationFailed)?;

            Ok(())
        } else {
            Err(EvidenceError::VerificationFailed(
                "Malformed double-signing evidence".to_string(),
            ))
        }
    }

    /// Check if evidence has expired (older than 24 hours)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let age_secs = now.saturating_sub(self.timestamp);
        age_secs > 86400 // 24 hours
    }

    /// Check if evidence has minimum required witnesses (3 for consensus)
    pub fn has_sufficient_witnesses(&self) -> bool {
        self.witnesses.len() >= 3
    }

    /// Add a witness signature to the evidence
    pub fn add_witness(&mut self, witness: VerifyingKey) {
        let witness_bytes = witness.as_bytes().to_vec();
        if !self.witnesses.contains(&witness_bytes) {
            self.witnesses.push(witness_bytes);
        }
    }

    /// Sign evidence with reporter's key
    pub fn sign_evidence(&mut self, reporter_signature: Vec<u8>) {
        self.reporter_signature = Some(reporter_signature);
    }

    /// Verify all evidence integrity checks
    pub fn verify(&self) -> Result<(), EvidenceError> {
        // Check expiration
        if self.is_expired() {
            return Err(EvidenceError::Expired);
        }

        let mut hash_input = self.accused.clone();
        hash_input.push(self.evidence_type.clone() as u8);
        hash_input.extend_from_slice(&self.offense_data);
        hash_input.extend_from_slice(&self.timestamp.to_le_bytes());
        if self.evidence_hash.as_slice() != blake3::hash(&hash_input).as_bytes().as_slice() {
            return Err(EvidenceError::CryptoVerificationFailed);
        }

        // Verify type-specific evidence
        match self.evidence_type {
            EvidenceType::DoubleSigning => self.verify_double_sign_evidence()?,
            EvidenceType::InvalidProof => {
                // Invalid proof evidence should have non-empty offense data
                if self.offense_data.is_empty() {
                    return Err(EvidenceError::Incomplete);
                }
            }
            EvidenceType::Censorship => {
                // Censorship evidence should have transaction hashes
                let offense: SlashableOffense = serde_json::from_slice(&self.offense_data)
                    .map_err(|e| EvidenceError::VerificationFailed(e.to_string()))?;
                if let SlashableOffense::Censorship {
                    transaction_hashes, ..
                } = offense
                {
                    if transaction_hashes.is_empty() {
                        return Err(EvidenceError::Incomplete);
                    }
                } else {
                    return Err(EvidenceError::VerificationFailed(
                        "Malformed censorship evidence".to_string(),
                    ));
                }
            }
            EvidenceType::LowUptime => {
                // Low uptime evidence should have valid percentage
                let offense: SlashableOffense = serde_json::from_slice(&self.offense_data)
                    .map_err(|e| EvidenceError::VerificationFailed(e.to_string()))?;
                if let SlashableOffense::LowUptime {
                    uptime_percentage, ..
                } = offense
                {
                    if uptime_percentage < 0.0 || uptime_percentage > 1.0 {
                        return Err(EvidenceError::VerificationFailed(
                            "Invalid uptime percentage".to_string(),
                        ));
                    }
                } else {
                    return Err(EvidenceError::VerificationFailed(
                        "Malformed uptime evidence".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Evidence collection result
#[derive(Debug)]
pub struct EvidenceBundle {
    pub evidence: Vec<SlashingEvidence>,
    pub total_witnesses: usize,
    pub verified_count: usize,
}

impl EvidenceBundle {
    /// Create a new evidence bundle
    pub fn new() -> Self {
        Self {
            evidence: Vec::new(),
            total_witnesses: 0,
            verified_count: 0,
        }
    }

    /// Add evidence to bundle after verification
    pub fn add_evidence(&mut self, evidence: SlashingEvidence) -> Result<(), EvidenceError> {
        evidence.verify()?;

        self.total_witnesses += evidence.witnesses.len();
        self.verified_count += 1;
        self.evidence.push(evidence);

        Ok(())
    }

    /// Get evidence ready for on-chain submission
    pub fn get_submittable_evidence(&self) -> Vec<SlashingEvidence> {
        self.evidence
            .iter()
            .filter(|e| e.has_sufficient_witnesses() && !e.is_expired())
            .cloned()
            .collect()
    }
}

impl Default for EvidenceBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn valid_double_signing_offense(signing_key: &SigningKey) -> SlashableOffense {
        let message1 = b"block-100-a".to_vec();
        let message2 = b"block-100-b".to_vec();
        SlashableOffense::DoubleSigning {
            block_height: 100,
            signature1: signing_key.sign(&message1).to_bytes().to_vec(),
            signature2: signing_key.sign(&message2).to_bytes().to_vec(),
            message1_hash: blake3::hash(&message1).as_bytes().to_vec(),
            message2_hash: blake3::hash(&message2).as_bytes().to_vec(),
            message1,
            message2,
        }
    }

    #[test]
    fn test_evidence_creation() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let witness1 = SigningKey::generate(&mut OsRng).verifying_key();
        let witness2 = SigningKey::generate(&mut OsRng).verifying_key();

        let offense = valid_double_signing_offense(&signing_key);

        let evidence =
            SlashingEvidence::from_offense(verifying_key, &offense, vec![witness1, witness2]);

        assert_eq!(evidence.evidence_type, EvidenceType::DoubleSigning);
        assert_eq!(evidence.witnesses.len(), 2);
        assert!(!evidence.is_expired());
    }

    #[test]
    fn test_double_sign_evidence_verification() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let offense = valid_double_signing_offense(&signing_key);

        let evidence = SlashingEvidence::from_offense(verifying_key, &offense, vec![]);

        assert!(evidence.verify_double_sign_evidence().is_ok());
    }

    #[test]
    fn test_invalid_double_sign_evidence() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        // Same signatures - should fail
        let mut offense = valid_double_signing_offense(&signing_key);
        if let SlashableOffense::DoubleSigning {
            signature1,
            signature2,
            ..
        } = &mut offense
        {
            *signature2 = signature1.clone();
        }

        let evidence = SlashingEvidence::from_offense(verifying_key, &offense, vec![]);

        assert!(evidence.verify_double_sign_evidence().is_err());
    }

    #[test]
    fn test_double_sign_evidence_rejects_unrelated_signature() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let unrelated_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let mut offense = valid_double_signing_offense(&signing_key);
        if let SlashableOffense::DoubleSigning {
            signature2,
            message2,
            ..
        } = &mut offense
        {
            *signature2 = unrelated_key.sign(message2).to_bytes().to_vec();
        }

        let evidence = SlashingEvidence::from_offense(verifying_key, &offense, vec![]);
        assert!(evidence.verify_double_sign_evidence().is_err());
    }

    #[test]
    fn test_evidence_verification_rejects_tampered_payload() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let offense = valid_double_signing_offense(&signing_key);
        let mut evidence =
            SlashingEvidence::from_offense(signing_key.verifying_key(), &offense, vec![]);
        evidence.offense_data[0] ^= 1;

        assert!(evidence.verify().is_err());
    }

    #[test]
    fn test_witness_requirements() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let offense = SlashableOffense::InvalidProof {
            proof_type: "test".to_string(),
            proof_hash: vec![1, 2, 3],
            reason: "test".to_string(),
        };

        let mut evidence = SlashingEvidence::from_offense(verifying_key, &offense, vec![]);

        // Insufficient witnesses
        assert!(!evidence.has_sufficient_witnesses());

        // Add 3 witnesses
        for _ in 0..3 {
            let witness = SigningKey::generate(&mut OsRng).verifying_key();
            evidence.add_witness(witness);
        }

        assert!(evidence.has_sufficient_witnesses());
    }

    #[test]
    fn test_evidence_bundle() {
        let mut bundle = EvidenceBundle::new();

        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let offense = valid_double_signing_offense(&signing_key);

        let witnesses: Vec<VerifyingKey> = (0..3)
            .map(|_| SigningKey::generate(&mut OsRng).verifying_key())
            .collect();

        let evidence = SlashingEvidence::from_offense(verifying_key, &offense, witnesses);

        assert!(bundle.add_evidence(evidence).is_ok());
        assert_eq!(bundle.verified_count, 1);
        assert_eq!(bundle.get_submittable_evidence().len(), 1);
    }
}
