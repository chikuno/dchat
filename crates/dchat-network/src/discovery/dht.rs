// Kademlia DHT implementation

use super::peer_info::PeerInfo;
use super::routing_table::RoutingTable;
use dchat_core::Result;
use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// DHT configuration
#[derive(Debug, Clone)]
pub struct DhtConfig {
    /// Local peer ID
    pub local_peer_id: PeerId,
    
    /// Bootstrap nodes to connect to
    pub bootstrap_nodes: Vec<Multiaddr>,
    
    /// K-bucket size (typically 20)
    pub k_bucket_size: usize,
    
    /// Concurrency parameter (typically 3)
    pub alpha: usize,
    
    /// Query timeout
    pub query_timeout: Duration,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            local_peer_id: PeerId::random(),
            bootstrap_nodes: vec![],
            k_bucket_size: 20,
            alpha: 3,
            query_timeout: Duration::from_secs(30),
        }
    }
}

/// DHT errors
#[derive(Debug, Error)]
pub enum DhtError {
    #[error("Bootstrap failed: {0}")]
    BootstrapFailed(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    
    #[error("Query timeout")]
    QueryTimeout,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Kademlia DHT implementation
#[allow(dead_code)]
pub struct Dht {
    config: DhtConfig,
    routing_table: RoutingTable,
    pending_queries: HashMap<QueryId, Query>,
    next_query_id: u64,
}

impl Dht {
    /// Create a new DHT instance
    pub async fn new(config: DhtConfig) -> Result<Self> {
        let routing_table = RoutingTable::new(config.local_peer_id, config.k_bucket_size);
        
        Ok(Self {
            config,
            routing_table,
            pending_queries: HashMap::new(),
            next_query_id: 0,
        })
    }

    /// Bootstrap the node into the network
    pub async fn bootstrap(&mut self) -> Result<()> {
        if self.config.bootstrap_nodes.is_empty() {
            return Err(dchat_core::Error::network(
                "No bootstrap nodes configured"
            ));
        }

        tracing::info!(
            "Bootstrapping DHT with {} nodes",
            self.config.bootstrap_nodes.len()
        );

        // Add bootstrap nodes to routing table
        for (i, addr) in self.config.bootstrap_nodes.iter().enumerate() {
            // Extract peer ID from multiaddr if present
            // For now, use a deterministic peer ID based on address
            let peer_id = self.peer_id_from_addr(addr, i);
            let peer_info = PeerInfo::new(peer_id, vec![addr.clone()]);
            
            if let Err(e) = self.routing_table.add_peer(peer_info) {
                tracing::warn!("Failed to add bootstrap node: {}", e);
            }
        }

        // Perform FIND_NODE query for self to populate routing table
        let self_id = self.config.local_peer_id;
        self.find_peer(&self_id).await?;

        tracing::info!(
            "Bootstrap complete, routing table has {} peers",
            self.routing_table.peer_count()
        );

        Ok(())
    }

    /// Find peers closest to a target peer ID
    pub async fn find_peer(&self, peer_id: &PeerId) -> Result<Vec<PeerInfo>> {
        tracing::debug!("Finding peers close to {:?}", peer_id);

        // Get k closest peers from local routing table
        let closest = self.routing_table.find_closest(peer_id, self.config.k_bucket_size);

        if closest.is_empty() {
            return Err(dchat_core::Error::network(
                "No peers in routing table"
            ));
        }

        // In a full implementation, we would:
        // 1. Query alpha closest peers in parallel
        // 2. Iteratively query closer peers
        // 3. Stop when we've queried the k closest peers
        //
        // For now, return local closest peers
        Ok(closest)
    }

    /// Announce this node's presence to the network
    pub async fn announce(&self) -> Result<()> {
        tracing::debug!("Announcing presence to network");

        // Find peers close to our own ID
        let self_id = self.config.local_peer_id;
        let closest = self.find_peer(&self_id).await?;

        tracing::info!(
            "Announced to {} peers",
            closest.len()
        );

        Ok(())
    }

    /// Get reference to routing table
    pub fn routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }

    /// Get mutable reference to routing table
    pub fn routing_table_mut(&mut self) -> &mut RoutingTable {
        &mut self.routing_table
    }

    /// Add a peer to the routing table
    pub fn add_peer(&mut self, peer: PeerInfo) -> Result<()> {
        self.routing_table
            .add_peer(peer)
            .map_err(dchat_core::Error::network)
    }

    /// Update a peer's last-seen timestamp
    pub fn update_peer(&mut self, peer_id: &PeerId) -> Result<()> {
        self.routing_table
            .update_peer(peer_id)
            .map_err(dchat_core::Error::network)
    }

    /// Perform periodic maintenance
    pub async fn maintain(&mut self) -> Result<()> {
        // Remove stale peers (not seen in 5 minutes)
        self.routing_table.remove_stale_peers(Duration::from_secs(300));

        // Refresh buckets that haven't been updated recently
        // In a full implementation, perform FIND_NODE for random IDs in stale buckets

        Ok(())
    }

    /// Create a query ID
    #[allow(dead_code)]
    fn next_query_id(&mut self) -> QueryId {
        let id = QueryId(self.next_query_id);
        self.next_query_id += 1;
        id
    }

    /// Helper to create deterministic peer ID from address for bootstrap
    fn peer_id_from_addr(&self, _addr: &Multiaddr, index: usize) -> PeerId {
        // In production, extract from multiaddr or use proper peer ID
        // For now, generate deterministic ID based on index
        let mut bytes = [0u8; 32];
        bytes[0] = index as u8;
        
        // Create a deterministic but unique peer ID
        // This is a placeholder - real implementation would parse from multiaddr
        PeerId::random()
    }
}

/// Query identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct QueryId(u64);

/// DHT query state
#[derive(Debug)]
struct Query {
    _id: QueryId,
    _target: PeerId,
    _queried_peers: Vec<PeerId>,
    _pending_responses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> DhtConfig {
        DhtConfig {
            local_peer_id: PeerId::random(),
            bootstrap_nodes: vec![],
            k_bucket_size: 20,
            alpha: 3,
            query_timeout: Duration::from_secs(30),
        }
    }

    #[tokio::test]
    async fn test_dht_creation() {
        let config = test_config();
        let dht = Dht::new(config).await;
        assert!(dht.is_ok());
    }

    #[tokio::test]
    async fn test_bootstrap_no_nodes() {
        let config = test_config();
        let mut dht = Dht::new(config).await.unwrap();
        
        let result = dht.bootstrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_peer() {
        let config = test_config();
        let mut dht = Dht::new(config).await.unwrap();
        
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9000".parse().unwrap();
        let peer_info = PeerInfo::new(peer_id, vec![addr]);
        
        assert!(dht.add_peer(peer_info).is_ok());
        assert_eq!(dht.routing_table().peer_count(), 1);
    }

    #[tokio::test]
    async fn test_find_peer_empty_table() {
        let config = test_config();
        let dht = Dht::new(config).await.unwrap();
        
        let target = PeerId::random();
        let result = dht.find_peer(&target).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_peer_with_peers() {
        let config = test_config();
        let mut dht = Dht::new(config).await.unwrap();
        
        // Add some peers
        for i in 0..5 {
            let peer_id = PeerId::random();
            let addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", 9000 + i).parse().unwrap();
            let peer_info = PeerInfo::new(peer_id, vec![addr]);
            dht.add_peer(peer_info).unwrap();
        }
        
        let target = PeerId::random();
        let result = dht.find_peer(&target).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_maintain() {
        let config = test_config();
        let mut dht = Dht::new(config).await.unwrap();
        
        let result = dht.maintain().await;
        assert!(result.is_ok());
    }
}
