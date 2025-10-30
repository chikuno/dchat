//! Hierarchical key derivation for identity management

use dchat_core::error::{Error, Result};
use dchat_crypto::keys::{PrivateKey, KeyPair, KeyDerivation};
use serde::{Deserialize, Serialize};

/// Represents a BIP-44 style key derivation path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyPath {
    pub purpose: u32,      // e.g., 44 for BIP-44
    pub coin_type: u32,    // dchat coin type
    pub account: u32,      // Account index
    pub change: u32,       // External/internal chain
    pub index: u32,        // Address index
}

impl KeyPath {
    /// Create a new key path
    pub fn new(purpose: u32, coin_type: u32, account: u32, change: u32, index: u32) -> Self {
        Self {
            purpose,
            coin_type,
            account,
            change,
            index,
        }
    }
    
    /// Create a standard dchat path
    pub fn dchat_path(account: u32, change: u32, index: u32) -> Self {
        Self::new(44, 1337, account, change, index) // 1337 as dchat coin type
    }
    
    /// Create a device key path
    pub fn device_path(device_index: u32) -> Self {
        Self::new(44, 1337, 0, 1, device_index)
    }
    
    /// Create a burner identity path
    pub fn burner_path(burner_index: u32) -> Self {
        Self::new(44, 1337, 1, 0, burner_index)
    }
    
    /// Convert to array for derivation
    pub fn to_array(&self) -> [u32; 5] {
        [self.purpose, self.coin_type, self.account, self.change, self.index]
    }
    
    /// Parse from string representation (m/44'/1337'/0'/0/0)
    pub fn from_string(path: &str) -> Result<Self> {
        let parts: Vec<&str> = path.trim_start_matches("m/").split('/').collect();
        
        if parts.len() != 5 {
            return Err(Error::identity("Invalid key path format"));
        }
        
        let parse_part = |s: &str| -> Result<u32> {
            s.trim_end_matches('\'')
                .parse()
                .map_err(|_| Error::identity("Invalid key path component"))
        };
        
        Ok(Self {
            purpose: parse_part(parts[0])?,
            coin_type: parse_part(parts[1])?,
            account: parse_part(parts[2])?,
            change: parse_part(parts[3])?,
            index: parse_part(parts[4])?,
        })
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!(
            "m/{}'/{}'/{}'/{}/{}",
            self.purpose, self.coin_type, self.account, self.change, self.index
        )
    }
}

/// Helper for deriving identity keys
pub struct IdentityDerivation;

impl IdentityDerivation {
    /// Derive a key from a master key using a key path
    pub fn derive_key(master_key: &PrivateKey, path: &KeyPath) -> Result<KeyPair> {
        let derived_key = KeyDerivation::derive_key_path(master_key, &path.to_array())?;
        Ok(KeyPair::from_private_key(derived_key))
    }
    
    /// Derive a device key
    pub fn derive_device_key(master_key: &PrivateKey, device_index: u32) -> Result<KeyPair> {
        let path = KeyPath::device_path(device_index);
        Self::derive_key(master_key, &path)
    }
    
    /// Derive a burner identity key
    pub fn derive_burner_key(master_key: &PrivateKey, burner_index: u32) -> Result<KeyPair> {
        let path = KeyPath::burner_path(burner_index);
        Self::derive_key(master_key, &path)
    }
    
    /// Derive a conversation key
    pub fn derive_conversation_key(
        master_key: &PrivateKey,
        conversation_index: u32,
    ) -> Result<KeyPair> {
        let path = KeyPath::new(44, 1337, 0, 2, conversation_index);
        Self::derive_key(master_key, &path)
    }
    
    /// Derive multiple keys for different purposes
    pub fn derive_all_keys(master_key: &PrivateKey) -> Result<DerivedKeys> {
        Ok(DerivedKeys {
            main_identity: Self::derive_key(master_key, &KeyPath::dchat_path(0, 0, 0))?,
            device_0: Self::derive_device_key(master_key, 0)?,
            burner_0: Self::derive_burner_key(master_key, 0)?,
        })
    }
}

/// Collection of derived keys
pub struct DerivedKeys {
    pub main_identity: KeyPair,
    pub device_0: KeyPair,
    pub burner_0: KeyPair,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dchat_crypto::keys::PrivateKey;
    
    #[test]
    fn test_key_path() {
        let path = KeyPath::dchat_path(0, 0, 0);
        assert_eq!(path.purpose, 44);
        assert_eq!(path.coin_type, 1337);
        
        let path_str = path.to_string();
        assert_eq!(path_str, "m/44'/1337'/0'/0/0");
        
        let parsed = KeyPath::from_string(&path_str).unwrap();
        assert_eq!(path, parsed);
    }
    
    #[test]
    fn test_key_derivation() {
        let master_key = PrivateKey::generate();
        
        // Derive the same key twice
        let key1 = IdentityDerivation::derive_device_key(&master_key, 0).unwrap();
        let key2 = IdentityDerivation::derive_device_key(&master_key, 0).unwrap();
        
        // Should be identical
        assert_eq!(key1.public_key().as_bytes(), key2.public_key().as_bytes());
        
        // Different device index should give different key
        let key3 = IdentityDerivation::derive_device_key(&master_key, 1).unwrap();
        assert_ne!(key1.public_key().as_bytes(), key3.public_key().as_bytes());
    }
    
    #[test]
    fn test_derive_all_keys() {
        let master_key = PrivateKey::generate();
        let keys = IdentityDerivation::derive_all_keys(&master_key).unwrap();
        
        // All keys should be different
        assert_ne!(
            keys.main_identity.public_key().as_bytes(),
            keys.device_0.public_key().as_bytes()
        );
        assert_ne!(
            keys.main_identity.public_key().as_bytes(),
            keys.burner_0.public_key().as_bytes()
        );
        assert_ne!(
            keys.device_0.public_key().as_bytes(),
            keys.burner_0.public_key().as_bytes()
        );
    }
}