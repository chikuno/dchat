//! Cryptographic utilities for key management and signing

use ed25519_dalek::{SigningKey, VerifyingKey};
use hex::FromHex;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Ed25519 key pair for identity management
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Create from existing private key bytes
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, String> {
        if private_key.len() != 32 {
            return Err("Private key must be 32 bytes".to_string());
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(private_key);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        Ok(Self { signing_key })
    }

    /// Get public key as hex string
    pub fn public_key_hex(&self) -> String {
        let verifying_key = VerifyingKey::from(&self.signing_key);
        hex::encode(verifying_key.as_bytes())
    }

    /// Get private key as hex string
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.signing_key.as_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), String> {
        use ed25519_dalek::Signature;

        if signature.len() != 64 {
            return Err("Invalid signature length".to_string());
        }

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let signature = Signature::from_bytes(&sig_bytes);

        let verifying_key = VerifyingKey::from(&self.signing_key);

        verifying_key
            .verify_strict(message, &signature)
            .map_err(|e| e.to_string())
    }

    /// Export key pair to dictionary format
    pub fn to_dict(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("public_key".to_string(), self.public_key_hex());
        map.insert("private_key".to_string(), self.private_key_hex());
        map
    }

    /// Import key pair from dictionary format
    pub fn from_dict(dict: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        let private_key_hex = dict
            .get("private_key")
            .ok_or("Missing private_key")?;

        let private_key = Vec::from_hex(private_key_hex)
            .map_err(|e| format!("Failed to decode private key: {}", e))?;

        Self::from_private_key(&private_key)
    }
}

/// Hash content using SHA-256
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Hash bytes using SHA-256
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Convert bytes to hex string
pub fn bytes_to_hex(data: &[u8]) -> String {
    hex::encode(data)
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    Vec::from_hex(hex_str).map_err(|e| e.to_string())
}
