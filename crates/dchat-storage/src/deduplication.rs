//! Content-addressed storage and deduplication

use blake3::Hash;
use std::collections::HashMap;

/// Content-addressable storage interface
pub trait ContentAddressable {
    /// Calculate content hash
    fn content_hash(&self) -> Hash;
}

/// Deduplication store
pub struct DeduplicationStore {
    /// Map content hashes to reference counts
    hash_refs: HashMap<String, usize>,
    
    /// Map content hashes to stored data
    content_store: HashMap<String, Vec<u8>>,
}

impl DeduplicationStore {
    pub fn new() -> Self {
        Self {
            hash_refs: HashMap::new(),
            content_store: HashMap::new(),
        }
    }
    
    /// Store content and return its hash
    pub fn store(&mut self, content: Vec<u8>) -> String {
        let hash = blake3::hash(&content);
        let hash_str = hash.to_hex().to_string();
        
        // Increment reference count
        *self.hash_refs.entry(hash_str.clone()).or_insert(0) += 1;
        
        // Store content if not already present
        self.content_store.entry(hash_str.clone()).or_insert(content);
        
        hash_str
    }
    
    /// Retrieve content by hash
    pub fn retrieve(&self, hash: &str) -> Option<&[u8]> {
        self.content_store.get(hash).map(|v| v.as_slice())
    }
    
    /// Decrement reference count and potentially remove content
    pub fn release(&mut self, hash: &str) -> bool {
        if let Some(refs) = self.hash_refs.get_mut(hash) {
            *refs -= 1;
            
            if *refs == 0 {
                self.hash_refs.remove(hash);
                self.content_store.remove(hash);
                return true;
            }
        }
        
        false
    }
    
    /// Get reference count for a hash
    pub fn ref_count(&self, hash: &str) -> usize {
        self.hash_refs.get(hash).copied().unwrap_or(0)
    }
    
    /// Get total stored items
    pub fn item_count(&self) -> usize {
        self.content_store.len()
    }
    
    /// Get total storage size
    pub fn total_size(&self) -> usize {
        self.content_store.values().map(|v| v.len()).sum()
    }
    
    /// Calculate storage savings from deduplication
    pub fn savings(&self) -> usize {
        let total_refs: usize = self.hash_refs.values().sum();
        let unique_items = self.item_count();
        
        if unique_items == 0 {
            return 0;
        }
        
        let avg_size = self.total_size() / unique_items;
        let would_be_size = total_refs * avg_size;
        
        would_be_size.saturating_sub(self.total_size())
    }
}

impl Default for DeduplicationStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta encoding for message history
pub struct DeltaEncoder {
    /// Store base versions
    base_versions: HashMap<String, Vec<u8>>,
}

impl DeltaEncoder {
    pub fn new() -> Self {
        Self {
            base_versions: HashMap::new(),
        }
    }
    
    /// Store base version
    pub fn store_base(&mut self, key: String, content: Vec<u8>) {
        self.base_versions.insert(key, content);
    }
    
    /// Calculate delta from base version
    pub fn calculate_delta(&self, key: &str, new_content: &[u8]) -> Option<Vec<u8>> {
        self.base_versions.get(key).map(|base| {
            // Simple delta: store only changed bytes
            // In a real implementation, use a proper diff algorithm
            self.simple_delta(base, new_content)
        })
    }
    
    /// Apply delta to base version
    pub fn apply_delta(&self, key: &str, delta: &[u8]) -> Option<Vec<u8>> {
        self.base_versions.get(key).map(|base| {
            // Simple delta application
            self.apply_simple_delta(base, delta)
        })
    }
    
    fn simple_delta(&self, _base: &[u8], new: &[u8]) -> Vec<u8> {
        // Placeholder: just return new content
        // Real implementation would use Myers diff or similar
        new.to_vec()
    }
    
    fn apply_simple_delta(&self, _base: &[u8], delta: &[u8]) -> Vec<u8> {
        // Placeholder: just return delta
        delta.to_vec()
    }
}

impl Default for DeltaEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_store() {
        let mut store = DeduplicationStore::new();
        
        let content = b"Hello, World!".to_vec();
        let hash1 = store.store(content.clone());
        let hash2 = store.store(content.clone());
        
        // Same content should have same hash
        assert_eq!(hash1, hash2);
        
        // Should only store once
        assert_eq!(store.item_count(), 1);
        
        // Reference count should be 2
        assert_eq!(store.ref_count(&hash1), 2);
        
        // Retrieve content
        let retrieved = store.retrieve(&hash1);
        assert_eq!(retrieved, Some(content.as_slice()));
        
        // Release one reference
        assert!(!store.release(&hash1));
        assert_eq!(store.ref_count(&hash1), 1);
        
        // Release second reference
        assert!(store.release(&hash1));
        assert_eq!(store.item_count(), 0);
    }
    
    #[test]
    fn test_storage_savings() {
        let mut store = DeduplicationStore::new();
        
        let content = vec![0u8; 1000]; // 1KB
        
        // Store same content 10 times
        for _ in 0..10 {
            store.store(content.clone());
        }
        
        // Should save 9KB (10KB - 1KB)
        assert_eq!(store.savings(), 9000);
    }
}
