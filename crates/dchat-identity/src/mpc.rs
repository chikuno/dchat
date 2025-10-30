// Multi-Party Computation (MPC) Threshold Signing for dchat
// Implements 2-of-3 threshold signature scheme for keyless UX fallback

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use sha2::{Sha256, Digest};

/// MPC errors
#[derive(Error, Debug)]
pub enum MpcError {
    #[error("Insufficient signers: need {required}, have {available}")]
    InsufficientSigners { required: usize, available: usize },
    
    #[error("Invalid signature share from signer {0}")]
    InvalidSignatureShare(String),
    
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    
    #[error("Signature aggregation failed: {0}")]
    AggregationFailed(String),
    
    #[error("Signer {0} not found")]
    SignerNotFound(String),
    
    #[error("Communication error: {0}")]
    CommunicationError(String),
    
    #[error("Timeout waiting for signers")]
    Timeout,
}

/// MPC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcConfig {
    /// Threshold (minimum signers required)
    pub threshold: usize,
    /// Total number of signers
    pub total_signers: usize,
    /// Timeout for signature collection (seconds)
    pub timeout_seconds: u64,
    /// Whether to allow fallback to full quorum
    pub allow_full_quorum: bool,
}

impl Default for MpcConfig {
    fn default() -> Self {
        Self {
            threshold: 2,
            total_signers: 3,
            timeout_seconds: 30,
            allow_full_quorum: true,
        }
    }
}

/// Signer identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SignerId(pub String);

impl From<String> for SignerId {
    fn from(s: String) -> Self {
        SignerId(s)
    }
}

impl From<&str> for SignerId {
    fn from(s: &str) -> Self {
        SignerId(s.to_string())
    }
}

/// Signer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signer {
    /// Unique identifier
    pub id: SignerId,
    /// Display name
    pub name: String,
    /// Public key share
    pub public_key_share: Vec<u8>,
    /// Whether this signer is available
    pub available: bool,
    /// Last seen timestamp
    pub last_seen: i64,
}

/// Signature share from a single signer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureShare {
    /// Signer ID
    pub signer_id: SignerId,
    /// Signature share data
    pub share: Vec<u8>,
    /// Timestamp
    pub timestamp: i64,
}

/// Aggregated threshold signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSignature {
    /// Aggregated signature
    pub signature: Vec<u8>,
    /// Signers who participated
    pub signers: Vec<SignerId>,
    /// Timestamp
    pub timestamp: i64,
}

/// Distributed Key Generation (DKG) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgResult {
    /// Public key (combined)
    pub public_key: Vec<u8>,
    /// Private key share (kept secret by each signer)
    pub private_key_share: Vec<u8>,
    /// Verification shares for all signers
    pub verification_shares: HashMap<SignerId, Vec<u8>>,
    /// Commitment to the key
    pub commitment: Vec<u8>,
}

/// MPC signing session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningSession {
    /// Session ID
    pub session_id: String,
    /// Message to sign
    pub message: Vec<u8>,
    /// Required threshold
    pub threshold: usize,
    /// Signature shares collected
    pub shares: Vec<SignatureShare>,
    /// Session start time
    pub started_at: i64,
    /// Session status
    pub status: SessionStatus,
}

/// Session status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Waiting for shares
    Pending,
    /// Sufficient shares collected, aggregating
    Aggregating,
    /// Signature complete
    Complete,
    /// Session failed or timed out
    Failed,
}

/// MPC Signer - handles threshold signature operations
pub struct MpcSigner {
    config: MpcConfig,
    signers: HashMap<SignerId, Signer>,
    active_sessions: HashMap<String, SigningSession>,
}

impl MpcSigner {
    /// Create a new MPC signer
    pub fn new(config: MpcConfig) -> Self {
        Self {
            config,
            signers: HashMap::new(),
            active_sessions: HashMap::new(),
        }
    }

    /// Perform distributed key generation (DKG)
    /// In production, this would use a library like TSS, GG20, or FROST
    pub async fn distributed_key_generation(
        &mut self,
        signer_ids: Vec<SignerId>,
    ) -> Result<DkgResult, MpcError> {
        if signer_ids.len() != self.config.total_signers {
            return Err(MpcError::KeyGenerationFailed(format!(
                "Expected {} signers, got {}",
                self.config.total_signers,
                signer_ids.len()
            )));
        }

        // Simulate DKG - in production, use a proper MPC library
        // This would involve:
        // 1. Each party generates a secret polynomial
        // 2. Shares are distributed using Shamir's Secret Sharing
        // 3. Public key is derived from commitments
        // 4. Verification shares are computed
        
        let mut hasher = Sha256::new();
        hasher.update(b"dchat-mpc-dkg");
        for id in &signer_ids {
            hasher.update(id.0.as_bytes());
        }
        let seed = hasher.finalize();

        // Generate dummy keys (replace with real MPC in production)
        let public_key = seed.to_vec();
        let private_key_share = seed[0..16].to_vec();
        
        let mut verification_shares = HashMap::new();
        for id in &signer_ids {
            let mut share_hasher = Sha256::new();
            share_hasher.update(&public_key);
            share_hasher.update(id.0.as_bytes());
            verification_shares.insert(id.clone(), share_hasher.finalize().to_vec());
        }

        let commitment = public_key.clone();

        Ok(DkgResult {
            public_key,
            private_key_share,
            verification_shares,
            commitment,
        })
    }

    /// Register a signer
    pub fn register_signer(&mut self, signer: Signer) {
        self.signers.insert(signer.id.clone(), signer);
    }

    /// Remove a signer
    pub fn remove_signer(&mut self, signer_id: &SignerId) -> Result<(), MpcError> {
        self.signers.remove(signer_id)
            .ok_or_else(|| MpcError::SignerNotFound(signer_id.0.clone()))?;
        Ok(())
    }

    /// Get available signers
    pub fn get_available_signers(&self) -> Vec<&Signer> {
        self.signers.values()
            .filter(|s| s.available)
            .collect()
    }

    /// Start a new signing session
    pub async fn start_signing_session(
        &mut self,
        message: Vec<u8>,
    ) -> Result<String, MpcError> {
        let available = self.get_available_signers();
        
        if available.len() < self.config.threshold {
            return Err(MpcError::InsufficientSigners {
                required: self.config.threshold,
                available: available.len(),
            });
        }

        let session_id = self.generate_session_id(&message);
        
        let session = SigningSession {
            session_id: session_id.clone(),
            message,
            threshold: self.config.threshold,
            shares: Vec::new(),
            started_at: chrono::Utc::now().timestamp(),
            status: SessionStatus::Pending,
        };

        self.active_sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Add a signature share to a session
    pub async fn add_signature_share(
        &mut self,
        session_id: &str,
        share: SignatureShare,
    ) -> Result<(), MpcError> {
        // Verify the signer is registered first
        if !self.signers.contains_key(&share.signer_id) {
            return Err(MpcError::SignerNotFound(share.signer_id.0.clone()));
        }

        // Get session and extract message before verification
        let message = {
            let session = self.active_sessions.get_mut(session_id)
                .ok_or_else(|| MpcError::SignerNotFound(session_id.to_string()))?;

            // Check if we already have a share from this signer
            if session.shares.iter().any(|s| s.signer_id == share.signer_id) {
                return Ok(()); // Already have share from this signer
            }

            session.message.clone()
        }; // Mutable borrow dropped here

        // Verify the share
        if !self.verify_signature_share(&message, &share)? {
            return Err(MpcError::InvalidSignatureShare(share.signer_id.0.clone()));
        }

        // Get session again after verification
        let session = self.active_sessions.get_mut(session_id)
            .ok_or_else(|| MpcError::SignerNotFound(session_id.to_string()))?;

        session.shares.push(share);

        // Check if we have enough shares
        if session.shares.len() >= session.threshold {
            session.status = SessionStatus::Aggregating;
        }

        Ok(())
    }

    /// Aggregate signature shares into final signature
    pub async fn aggregate_signature(
        &mut self,
        session_id: &str,
    ) -> Result<ThresholdSignature, MpcError> {
        // Extract data we need before getting mutable reference
        let (shares, signers) = {
            let session = self.active_sessions.get(session_id)
                .ok_or_else(|| MpcError::SignerNotFound(session_id.to_string()))?;

            if session.shares.len() < session.threshold {
                return Err(MpcError::InsufficientSigners {
                    required: session.threshold,
                    available: session.shares.len(),
                });
            }

            let shares = session.shares.clone();
            let signers: Vec<SignerId> = session.shares.iter()
                .map(|s| s.signer_id.clone())
                .collect();
            
            (shares, signers)
        };

        // Aggregate shares using Lagrange interpolation
        // In production, use proper threshold signature aggregation (e.g., BLS, Schnorr)
        let aggregated_sig = self.aggregate_shares(&shares)?;

        // Update session status
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.status = SessionStatus::Complete;
        }

        Ok(ThresholdSignature {
            signature: aggregated_sig,
            signers,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// Get signing session status
    pub fn get_session_status(&self, session_id: &str) -> Option<SessionStatus> {
        self.active_sessions.get(session_id).map(|s| s.status)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = chrono::Utc::now().timestamp();
        let timeout = self.config.timeout_seconds as i64;

        self.active_sessions.retain(|_, session| {
            now - session.started_at < timeout || session.status == SessionStatus::Complete
        });
    }

    // Helper methods

    fn generate_session_id(&self, message: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"dchat-signing-session");
        hasher.update(message);
        hasher.update(chrono::Utc::now().timestamp().to_le_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn verify_signature_share(&self, _message: &[u8], share: &SignatureShare) -> Result<bool, MpcError> {
        // Simplified verification - in production, use proper cryptographic verification
        // This would verify the share against the signer's public key share
        
        let signer = self.signers.get(&share.signer_id)
            .ok_or_else(|| MpcError::SignerNotFound(share.signer_id.0.clone()))?;

        // Dummy verification
        Ok(!share.share.is_empty() && !signer.public_key_share.is_empty())
    }

    fn aggregate_shares(&self, shares: &[SignatureShare]) -> Result<Vec<u8>, MpcError> {
        // Simplified aggregation - in production, use proper threshold signature aggregation
        // This would use Lagrange interpolation to combine shares
        
        if shares.is_empty() {
            return Err(MpcError::AggregationFailed("No shares to aggregate".to_string()));
        }

        // Dummy aggregation: XOR all shares
        let mut result = vec![0u8; 64];
        for share in shares {
            for (i, &byte) in share.share.iter().enumerate() {
                if i < result.len() {
                    result[i] ^= byte;
                }
            }
        }

        Ok(result)
    }
}

/// High-level MPC signing coordinator
pub struct MpcCoordinator {
    signer: MpcSigner,
}

impl MpcCoordinator {
    /// Create a new MPC coordinator
    pub fn new(config: MpcConfig) -> Self {
        Self {
            signer: MpcSigner::new(config),
        }
    }

    /// Setup MPC with multiple signers (e.g., user device, cloud backup, trusted contact)
    pub async fn setup(
        &mut self,
        signer_configs: Vec<(String, String)>, // (id, name) pairs
    ) -> Result<DkgResult, MpcError> {
        let signer_ids: Vec<SignerId> = signer_configs.iter()
            .map(|(id, _)| SignerId(id.clone()))
            .collect();

        // Perform distributed key generation
        let dkg_result = self.signer.distributed_key_generation(signer_ids.clone()).await?;

        // Register signers
        for ((id, name), verification_share) in signer_configs.iter().zip(dkg_result.verification_shares.values()) {
            let signer = Signer {
                id: SignerId(id.clone()),
                name: name.clone(),
                public_key_share: verification_share.clone(),
                available: true,
                last_seen: chrono::Utc::now().timestamp(),
            };
            self.signer.register_signer(signer);
        }

        Ok(dkg_result)
    }

    /// Sign a message using threshold signatures
    pub async fn sign(&mut self, message: Vec<u8>) -> Result<ThresholdSignature, MpcError> {
        // Start signing session
        let session_id = self.signer.start_signing_session(message.clone()).await?;

        // In a real implementation, this would:
        // 1. Broadcast signing request to all available signers
        // 2. Wait for threshold number of signature shares
        // 3. Aggregate shares into final signature

        // For now, simulate receiving shares from available signers
        // Clone the data we need to avoid borrowing issues
        let available_signers = self.signer.get_available_signers();
        let required = self.signer.config.threshold.min(available_signers.len());
        let signer_ids: Vec<SignerId> = available_signers.iter()
            .take(required)
            .map(|s| s.id.clone())
            .collect();

        for signer_id in signer_ids {
            let share = self.generate_signature_share(&signer_id, &message).await?;
            self.signer.add_signature_share(&session_id, share).await?;
        }

        // Aggregate signature
        self.signer.aggregate_signature(&session_id).await
    }

    /// Generate a signature share (called by each signer)
    async fn generate_signature_share(
        &self,
        signer_id: &SignerId,
        message: &[u8],
    ) -> Result<SignatureShare, MpcError> {
        // In production, this would:
        // 1. Use the signer's private key share
        // 2. Compute signature share using MPC protocol
        // 3. Return the share

        // Dummy implementation
        let mut hasher = Sha256::new();
        hasher.update(signer_id.0.as_bytes());
        hasher.update(message);
        let share = hasher.finalize().to_vec();

        Ok(SignatureShare {
            signer_id: signer_id.clone(),
            share,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// Update signer availability
    pub fn set_signer_available(&mut self, signer_id: &SignerId, available: bool) -> Result<(), MpcError> {
        let signer = self.signer.signers.get_mut(signer_id)
            .ok_or_else(|| MpcError::SignerNotFound(signer_id.0.clone()))?;
        
        signer.available = available;
        if available {
            signer.last_seen = chrono::Utc::now().timestamp();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mpc_config_default() {
        let config = MpcConfig::default();
        assert_eq!(config.threshold, 2);
        assert_eq!(config.total_signers, 3);
    }

    #[tokio::test]
    async fn test_distributed_key_generation() {
        let config = MpcConfig::default();
        let mut signer = MpcSigner::new(config);

        let signer_ids = vec![
            SignerId("device".to_string()),
            SignerId("cloud".to_string()),
            SignerId("recovery".to_string()),
        ];

        let result = signer.distributed_key_generation(signer_ids).await;
        assert!(result.is_ok());

        let dkg = result.unwrap();
        assert!(!dkg.public_key.is_empty());
        assert_eq!(dkg.verification_shares.len(), 3);
    }

    #[tokio::test]
    async fn test_threshold_signing() {
        let config = MpcConfig::default();
        let mut coordinator = MpcCoordinator::new(config);

        // Setup signers
        let signers = vec![
            ("device".to_string(), "User Device".to_string()),
            ("cloud".to_string(), "Cloud Backup".to_string()),
            ("recovery".to_string(), "Recovery Contact".to_string()),
        ];

        let dkg = coordinator.setup(signers).await.unwrap();
        assert!(!dkg.public_key.is_empty());

        // Sign a message
        let message = b"Hello, dchat MPC!".to_vec();
        let signature = coordinator.sign(message).await.unwrap();

        assert!(!signature.signature.is_empty());
        assert_eq!(signature.signers.len(), 2); // threshold = 2
    }

    #[tokio::test]
    async fn test_insufficient_signers() {
        let config = MpcConfig {
            threshold: 2,
            total_signers: 3,
            timeout_seconds: 30,
            allow_full_quorum: true,
        };
        let mut signer = MpcSigner::new(config);

        // Register only one signer
        signer.register_signer(Signer {
            id: SignerId("device".to_string()),
            name: "Device".to_string(),
            public_key_share: vec![1, 2, 3],
            available: true,
            last_seen: chrono::Utc::now().timestamp(),
        });

        let result = signer.start_signing_session(vec![0u8; 32]).await;
        assert!(matches!(result, Err(MpcError::InsufficientSigners { .. })));
    }
}
