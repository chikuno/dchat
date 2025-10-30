//! Peer discovery mechanisms

use dchat_core::error::{Error, Result};
use libp2p::{kad, Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Bootstrap nodes for initial DHT seeding
    pub bootstrap_nodes: Vec<(PeerId, Multiaddr)>,
    
    /// Enable mDNS for local discovery
    pub enable_mdns: bool,
    
    /// Minimum number of peers to maintain
    pub min_peers: usize,
    
    /// Maximum number of peers to maintain
    pub max_peers: usize,
    
    /// DHT query timeout
    pub query_timeout: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: vec![],
            enable_mdns: true,
            min_peers: 10,
            max_peers: 100,
            query_timeout: Duration::from_secs(60),
        }
    }
}

/// Peer discovery manager
pub struct Discovery {
    config: DiscoveryConfig,
    known_peers: HashSet<PeerId>,
    connected_peers: HashSet<PeerId>,
}

impl Discovery {
    /// Create a new discovery manager
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            known_peers: HashSet::new(),
            connected_peers: HashSet::new(),
        }
    }
    
    /// Add a bootstrap node
    pub fn add_bootstrap_node(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.config.bootstrap_nodes.push((peer_id, addr));
        self.known_peers.insert(peer_id);
    }
    
    /// Get bootstrap nodes
    pub fn bootstrap_nodes(&self) -> &[(PeerId, Multiaddr)] {
        &self.config.bootstrap_nodes
    }
    
    /// Register a discovered peer
    pub fn register_peer(&mut self, peer_id: PeerId) {
        self.known_peers.insert(peer_id);
    }
    
    /// Mark a peer as connected
    pub fn peer_connected(&mut self, peer_id: PeerId) {
        self.connected_peers.insert(peer_id);
    }
    
    /// Mark a peer as disconnected
    pub fn peer_disconnected(&mut self, peer_id: &PeerId) {
        self.connected_peers.remove(peer_id);
    }
    
    /// Check if we need more peers
    pub fn needs_more_peers(&self) -> bool {
        self.connected_peers.len() < self.config.min_peers
    }
    
    /// Check if we have too many peers
    pub fn has_too_many_peers(&self) -> bool {
        self.connected_peers.len() > self.config.max_peers
    }
    
    /// Get connected peer count
    pub fn connected_count(&self) -> usize {
        self.connected_peers.len()
    }
    
    /// Get known peer count
    pub fn known_count(&self) -> usize {
        self.known_peers.len()
    }
    
    /// Get a peer to potentially disconnect (least recently used)
    pub fn get_excess_peer(&self) -> Option<PeerId> {
        if self.has_too_many_peers() {
            self.connected_peers.iter().next().copied()
        } else {
            None
        }
    }
}

/// Eclipse attack prevention
pub struct EclipseGuard {
    /// Track ASN diversity of connected peers
    peer_asns: std::collections::HashMap<PeerId, u32>,
    
    /// Maximum fraction of peers from same ASN
    max_asn_fraction: f64,
}

impl EclipseGuard {
    pub fn new(max_asn_fraction: f64) -> Self {
        Self {
            peer_asns: std::collections::HashMap::new(),
            max_asn_fraction,
        }
    }
    
    /// Check if connecting to a peer would violate diversity constraints
    pub fn should_allow_peer(&self, _peer_id: &PeerId, asn: u32, total_peers: usize) -> bool {
        if total_peers == 0 {
            return true;
        }
        
        let asn_count = self.peer_asns.values().filter(|&&a| a == asn).count();
        let fraction = (asn_count + 1) as f64 / (total_peers + 1) as f64;
        
        fraction <= self.max_asn_fraction
    }
    
    /// Register a peer's ASN
    pub fn register_peer(&mut self, peer_id: PeerId, asn: u32) {
        self.peer_asns.insert(peer_id, asn);
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peer_asns.remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_basic() {
        let config = DiscoveryConfig::default();
        let mut discovery = Discovery::new(config);
        
        assert_eq!(discovery.connected_count(), 0);
        assert!(discovery.needs_more_peers());
        
        let peer_id = PeerId::random();
        discovery.peer_connected(peer_id);
        assert_eq!(discovery.connected_count(), 1);
    }
    
    #[test]
    fn test_eclipse_guard() {
        let mut guard = EclipseGuard::new(0.5); // Max 50% from same ASN
        
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();
        let peer4 = PeerId::random();
        
        // First peer always allowed
        assert!(guard.should_allow_peer(&peer1, 100, guard.peer_asns.len()));
        guard.register_peer(peer1, 100);
        
        // Second peer from different ASN: (0+1)/(1+1) = 50%, should be allowed
        assert!(guard.should_allow_peer(&peer2, 200, guard.peer_asns.len()));
        guard.register_peer(peer2, 200);
        
        // Third peer from ASN 100: (1+1)/(2+1) = 66% > 50%, should be rejected
        assert!(!guard.should_allow_peer(&peer3, 100, guard.peer_asns.len()));
        
        // Third peer from new ASN 300: (0+1)/(2+1) = 33% < 50%, should be allowed
        assert!(guard.should_allow_peer(&peer3, 300, guard.peer_asns.len()));
        guard.register_peer(peer3, 300);
        
        // Fourth peer from ASN 100: (1+1)/(3+1) = 50%, exactly at threshold, allowed
        assert!(guard.should_allow_peer(&peer4, 100, guard.peer_asns.len()));
    }
}
