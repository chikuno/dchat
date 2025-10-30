//! Channel-scoped sharding for horizontal scalability
//!
//! Implements Section 17 (Scalability via Sharding) from ARCHITECTURE.md
//! - Channel-based state partitioning
//! - Cross-shard message routing
//! - Light client support
//! - BLS signature aggregation for efficiency

use dchat_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use blake3::Hasher;

/// Shard identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShardId(pub u32);

/// Channel identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub String);

/// Shard assignment for a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardAssignment {
    pub channel_id: ChannelId,
    pub shard_id: ShardId,
    pub assigned_at: i64,
}

/// Shard state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardState {
    pub shard_id: ShardId,
    pub channels: Vec<ChannelId>,
    pub state_root: Vec<u8>,
    pub message_count: u64,
    pub last_updated: i64,
}

/// Cross-shard message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardMessage {
    pub id: String,
    pub from_shard: ShardId,
    pub to_shard: ShardId,
    pub from_channel: ChannelId,
    pub to_channel: ChannelId,
    pub payload: Vec<u8>,
    pub timestamp: i64,
    pub proof: Vec<u8>,
}

/// Sharding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Total number of shards
    pub num_shards: u32,
    /// Activity threshold for channel assignment (messages/hour)
    pub high_activity_threshold: u64,
    /// Enable BLS signature aggregation
    pub enable_bls_aggregation: bool,
    /// Light client mode (subscribe to subset of shards)
    pub light_client_mode: bool,
    /// Shards to track in light client mode
    pub tracked_shards: Vec<ShardId>,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            num_shards: 16,
            high_activity_threshold: 1000,
            enable_bls_aggregation: true,
            light_client_mode: false,
            tracked_shards: Vec::new(),
        }
    }
}

/// Shard manager
pub struct ShardManager {
    config: ShardConfig,
    shard_states: HashMap<ShardId, ShardState>,
    channel_assignments: HashMap<ChannelId, ShardId>,
    pending_cross_shard: Vec<CrossShardMessage>,
}

impl ShardManager {
    pub fn new(config: ShardConfig) -> Self {
        let mut shard_states = HashMap::new();
        
        // Initialize all shards
        for i in 0..config.num_shards {
            let shard_id = ShardId(i);
            shard_states.insert(
                shard_id.clone(),
                ShardState {
                    shard_id,
                    channels: Vec::new(),
                    state_root: vec![0; 32],
                    message_count: 0,
                    last_updated: chrono::Utc::now().timestamp(),
                },
            );
        }

        Self {
            config,
            shard_states,
            channel_assignments: HashMap::new(),
            pending_cross_shard: Vec::new(),
        }
    }

    /// Assign channel to shard using consistent hashing
    pub fn assign_channel(&mut self, channel_id: ChannelId) -> Result<ShardId> {
        // Check if already assigned
        if let Some(shard_id) = self.channel_assignments.get(&channel_id) {
            return Ok(shard_id.clone());
        }

        // Use consistent hashing
        let shard_id = self.hash_to_shard(&channel_id);

        // Update shard state
        if let Some(shard_state) = self.shard_states.get_mut(&shard_id) {
            shard_state.channels.push(channel_id.clone());
            shard_state.last_updated = chrono::Utc::now().timestamp();
        }

        self.channel_assignments.insert(channel_id, shard_id.clone());

        Ok(shard_id)
    }

    /// Hash channel ID to shard using BLAKE3
    fn hash_to_shard(&self, channel_id: &ChannelId) -> ShardId {
        let mut hasher = Hasher::new();
        hasher.update(channel_id.0.as_bytes());
        let hash = hasher.finalize();
        
        let shard_num = u32::from_le_bytes([hash.as_bytes()[0], hash.as_bytes()[1], hash.as_bytes()[2], hash.as_bytes()[3]]);
        ShardId(shard_num % self.config.num_shards)
    }

    /// Get shard for a channel
    pub fn get_shard(&self, channel_id: &ChannelId) -> Option<ShardId> {
        self.channel_assignments.get(channel_id).cloned()
    }

    /// Route message (may be cross-shard)
    pub fn route_message(
        &mut self,
        from_channel: ChannelId,
        to_channel: ChannelId,
        payload: Vec<u8>,
    ) -> Result<()> {
        let from_shard = self.get_shard(&from_channel)
            .ok_or_else(|| Error::network("Source channel not assigned to shard"))?;
        
        let to_shard = self.get_shard(&to_channel)
            .ok_or_else(|| Error::network("Destination channel not assigned to shard"))?;

        if from_shard == to_shard {
            // Same-shard message: direct delivery
            self.deliver_same_shard(&from_channel, &to_channel, payload)?;
        } else {
            // Cross-shard message: requires proof
            let cross_shard_msg = self.create_cross_shard_message(
                from_shard,
                to_shard,
                from_channel,
                to_channel,
                payload,
            )?;
            
            self.pending_cross_shard.push(cross_shard_msg);
        }

        Ok(())
    }

    /// Deliver message within same shard
    fn deliver_same_shard(
        &mut self,
        _from_channel: &ChannelId,
        _to_channel: &ChannelId,
        _payload: Vec<u8>,
    ) -> Result<()> {
        // In production: update shard state, emit events
        Ok(())
    }

    /// Create cross-shard message with proof
    fn create_cross_shard_message(
        &self,
        from_shard: ShardId,
        to_shard: ShardId,
        from_channel: ChannelId,
        to_channel: ChannelId,
        payload: Vec<u8>,
    ) -> Result<CrossShardMessage> {
        // Generate Merkle proof that message exists in source shard
        let proof = self.generate_merkle_proof(&from_shard, &from_channel)?;

        Ok(CrossShardMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from_shard,
            to_shard,
            from_channel,
            to_channel,
            payload,
            timestamp: chrono::Utc::now().timestamp(),
            proof,
        })
    }

    /// Generate Merkle proof for cross-shard message
    fn generate_merkle_proof(&self, shard_id: &ShardId, _channel_id: &ChannelId) -> Result<Vec<u8>> {
        let shard_state = self.shard_states.get(shard_id)
            .ok_or_else(|| Error::network("Shard not found"))?;

        // Placeholder: In production, generate actual Merkle proof
        Ok(shard_state.state_root.clone())
    }

    /// Verify cross-shard message proof
    pub fn verify_cross_shard_proof(&self, msg: &CrossShardMessage) -> Result<bool> {
        let source_shard = self.shard_states.get(&msg.from_shard)
            .ok_or_else(|| Error::network("Source shard not found"))?;

        // Placeholder: In production, verify Merkle proof against state root
        Ok(msg.proof == source_shard.state_root)
    }

    /// Process pending cross-shard messages
    pub fn process_cross_shard_messages(&mut self) -> Result<usize> {
        let mut processed = 0;

        // In light client mode, only process messages for tracked shards
        let messages_to_process: Vec<_> = if self.config.light_client_mode {
            self.pending_cross_shard.iter()
                .filter(|msg| self.config.tracked_shards.contains(&msg.to_shard))
                .cloned()
                .collect()
        } else {
            self.pending_cross_shard.clone()
        };

        // Collect verified message IDs to remove later
        let mut verified_msg_ids = Vec::new();

        for msg in messages_to_process {
            // Verify proof
            if self.verify_cross_shard_proof(&msg)? {
                // Deliver to destination shard
                if let Some(dest_shard) = self.shard_states.get_mut(&msg.to_shard) {
                    dest_shard.message_count += 1;
                    dest_shard.last_updated = chrono::Utc::now().timestamp();
                    processed += 1;
                    verified_msg_ids.push(msg.id.clone());
                }
            }
        }

        // Remove processed messages
        self.pending_cross_shard.retain(|msg| {
            !verified_msg_ids.contains(&msg.id)
        });

        Ok(processed)
    }

    /// Aggregate BLS signatures for shard finality
    pub fn aggregate_signatures(&self, signatures: &[Vec<u8>]) -> Result<Vec<u8>> {
        if !self.config.enable_bls_aggregation {
            return Err(Error::network("BLS aggregation disabled"));
        }

        // Placeholder: In production, use BLS12-381 signature aggregation
        let mut aggregated = Vec::new();
        for sig in signatures {
            aggregated.extend_from_slice(sig);
        }

        Ok(aggregated)
    }

    /// Get shard statistics
    pub fn get_shard_stats(&self, shard_id: &ShardId) -> Option<ShardStats> {
        self.shard_states.get(shard_id).map(|state| ShardStats {
            shard_id: shard_id.clone(),
            num_channels: state.channels.len(),
            message_count: state.message_count,
            last_updated: state.last_updated,
        })
    }

    /// Rebalance shards (move channels between shards)
    pub fn rebalance_shards(&mut self) -> Result<usize> {
        // Placeholder: In production, implement load-based rebalancing
        // For now, just return 0 (no channels moved)
        Ok(0)
    }

    /// Get global statistics
    pub fn get_global_stats(&self) -> GlobalShardStats {
        let total_channels: usize = self.channel_assignments.len();
        let total_messages: u64 = self.shard_states.values()
            .map(|s| s.message_count)
            .sum();
        let active_shards = self.shard_states.values()
            .filter(|s| !s.channels.is_empty())
            .count();

        GlobalShardStats {
            total_shards: self.config.num_shards as usize,
            active_shards,
            total_channels,
            total_messages,
            pending_cross_shard: self.pending_cross_shard.len(),
        }
    }
}

/// Shard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    pub shard_id: ShardId,
    pub num_channels: usize,
    pub message_count: u64,
    pub last_updated: i64,
}

/// Global shard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalShardStats {
    pub total_shards: usize,
    pub active_shards: usize,
    pub total_channels: usize,
    pub total_messages: u64,
    pub pending_cross_shard: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_config_default() {
        let config = ShardConfig::default();
        assert_eq!(config.num_shards, 16);
        assert!(config.enable_bls_aggregation);
        assert!(!config.light_client_mode);
    }

    #[test]
    fn test_shard_initialization() {
        let config = ShardConfig::default();
        let manager = ShardManager::new(config);

        assert_eq!(manager.shard_states.len(), 16);
        assert_eq!(manager.channel_assignments.len(), 0);
    }

    #[test]
    fn test_channel_assignment() {
        let config = ShardConfig::default();
        let mut manager = ShardManager::new(config);

        let channel = ChannelId("test-channel".to_string());
        let shard_id = manager.assign_channel(channel.clone()).unwrap();

        assert!(shard_id.0 < 16);
        assert_eq!(manager.get_shard(&channel), Some(shard_id.clone()));

        // Reassigning should return same shard
        let shard_id2 = manager.assign_channel(channel.clone()).unwrap();
        assert_eq!(shard_id, shard_id2);
    }

    #[test]
    fn test_consistent_hashing() {
        let config = ShardConfig::default();
        let manager = ShardManager::new(config);

        let channel = ChannelId("test-channel".to_string());
        let shard1 = manager.hash_to_shard(&channel);
        let shard2 = manager.hash_to_shard(&channel);

        // Same channel should always hash to same shard
        assert_eq!(shard1, shard2);
    }

    #[test]
    fn test_same_shard_routing() {
        let config = ShardConfig::default();
        let mut manager = ShardManager::new(config);

        let channel1 = ChannelId("channel1".to_string());
        let channel2 = ChannelId("channel2".to_string());

        // Force same shard by assigning explicitly
        let shard = ShardId(0);
        manager.channel_assignments.insert(channel1.clone(), shard.clone());
        manager.channel_assignments.insert(channel2.clone(), shard.clone());

        let result = manager.route_message(
            channel1,
            channel2,
            b"test message".to_vec(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.pending_cross_shard.len(), 0);
    }

    #[test]
    fn test_cross_shard_routing() {
        let config = ShardConfig::default();
        let mut manager = ShardManager::new(config);

        let channel1 = ChannelId("channel1".to_string());
        let channel2 = ChannelId("channel2".to_string());

        // Force different shards
        manager.channel_assignments.insert(channel1.clone(), ShardId(0));
        manager.channel_assignments.insert(channel2.clone(), ShardId(1));

        let result = manager.route_message(
            channel1,
            channel2,
            b"cross-shard message".to_vec(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.pending_cross_shard.len(), 1);
    }

    #[test]
    fn test_bls_signature_aggregation() {
        let config = ShardConfig::default();
        let manager = ShardManager::new(config);

        let sig1 = vec![1, 2, 3];
        let sig2 = vec![4, 5, 6];
        let signatures = vec![sig1.clone(), sig2.clone()];

        let aggregated = manager.aggregate_signatures(&signatures).unwrap();
        
        // Placeholder implementation just concatenates
        assert_eq!(aggregated.len(), 6);
    }

    #[test]
    fn test_light_client_mode() {
        let config = ShardConfig {
            light_client_mode: true,
            tracked_shards: vec![ShardId(0), ShardId(1)],
            ..Default::default()
        };
        let mut manager = ShardManager::new(config);

        // Create cross-shard message to tracked shard
        let channel1 = ChannelId("channel1".to_string());
        let channel2 = ChannelId("channel2".to_string());
        
        manager.channel_assignments.insert(channel1.clone(), ShardId(0));
        manager.channel_assignments.insert(channel2.clone(), ShardId(1));

        manager.route_message(channel1, channel2, b"message".to_vec()).unwrap();

        // Should process message for tracked shard
        let processed = manager.process_cross_shard_messages().unwrap();
        assert_eq!(processed, 1);
    }

    #[test]
    fn test_global_stats() {
        let config = ShardConfig::default();
        let mut manager = ShardManager::new(config);

        // Add some channels
        manager.assign_channel(ChannelId("channel1".to_string())).unwrap();
        manager.assign_channel(ChannelId("channel2".to_string())).unwrap();

        let stats = manager.get_global_stats();
        
        assert_eq!(stats.total_shards, 16);
        assert_eq!(stats.total_channels, 2);
        assert!(stats.active_shards >= 1);
    }
}
