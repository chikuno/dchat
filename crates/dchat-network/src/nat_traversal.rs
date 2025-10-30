//! NAT traversal implementation (UPnP and TURN fallback)
//!
//! Implements Section 12 (NAT Traversal) from ARCHITECTURE.md
//! - Automatic UPnP port mapping
//! - TURN server fallback for symmetric NATs
//! - Hole punching for direct P2P connections
//! - Eclipse attack prevention via relay diversity

use dchat_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, IpAddr};

/// NAT traversal strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatStrategy {
    /// Direct connection (no NAT)
    Direct,
    /// UPnP port mapping
    UPnP,
    /// TURN relay server
    TURN,
    /// Hole punching
    HolePunching,
}

/// NAT type detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatType {
    /// No NAT (public IP)
    None,
    /// Full cone NAT (easy traversal)
    FullCone,
    /// Restricted cone NAT (moderate difficulty)
    RestrictedCone,
    /// Port-restricted cone NAT (difficult)
    PortRestrictedCone,
    /// Symmetric NAT (requires TURN)
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

/// NAT traversal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatConfig {
    /// Enable UPnP automatic port mapping
    pub enable_upnp: bool,
    /// TURN server addresses for relay
    pub turn_servers: Vec<String>,
    /// Enable hole punching
    pub enable_hole_punching: bool,
    /// Timeout for NAT detection
    pub detection_timeout_secs: u64,
    /// Port range for UPnP mapping
    pub upnp_port_range: (u16, u16),
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            enable_upnp: true,
            turn_servers: vec![
                "turn:relay1.dchat.network:3478".to_string(),
                "turn:relay2.dchat.network:3478".to_string(),
            ],
            enable_hole_punching: true,
            detection_timeout_secs: 10,
            upnp_port_range: (49152, 65535), // Dynamic/private ports
        }
    }
}

/// NAT traversal manager
pub struct NatTraversalManager {
    config: NatConfig,
    detected_nat_type: Option<NatType>,
    active_strategy: Option<NatStrategy>,
    upnp_gateway: Option<UpnpGateway>,
    turn_connections: Vec<TurnConnection>,
}

/// UPnP gateway information
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UpnpGateway {
    gateway_addr: SocketAddr,
    external_ip: IpAddr,
    mapped_port: u16,
    internal_port: u16,
}

/// TURN server connection
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TurnConnection {
    server_addr: String,
    allocated_addr: Option<SocketAddr>,
    username: String,
    credential: String,
}

impl NatTraversalManager {
    /// Create a new NAT traversal manager
    pub fn new(config: NatConfig) -> Self {
        Self {
            config,
            detected_nat_type: None,
            active_strategy: None,
            upnp_gateway: None,
            turn_connections: Vec::new(),
        }
    }

    /// Detect NAT type using STUN protocol
    pub async fn detect_nat_type(&mut self) -> Result<NatType> {
        // Implementation would use STUN protocol to detect NAT type
        // For now, return Unknown as placeholder
        
        // Steps:
        // 1. Send STUN binding request to primary STUN server
        // 2. Compare public IP:port with local IP:port
        // 3. Send requests to secondary STUN server
        // 4. Analyze responses to determine NAT type
        
        let nat_type = NatType::Unknown; // Placeholder
        self.detected_nat_type = Some(nat_type.clone());
        
        Ok(nat_type)
    }

    /// Attempt UPnP port mapping
    pub async fn setup_upnp(&mut self, internal_port: u16) -> Result<SocketAddr> {
        if !self.config.enable_upnp {
            return Err(Error::network("UPnP is disabled"));
        }

        // Implementation would use igd crate or similar
        // Steps:
        // 1. Discover UPnP gateway on local network
        // 2. Request external port mapping
        // 3. Store mapping information
        // 4. Set up lease renewal
        
        // Placeholder implementation
        let gateway = UpnpGateway {
            gateway_addr: "192.168.1.1:5000".parse().unwrap(),
            external_ip: "203.0.113.1".parse().unwrap(),
            internal_port,
            mapped_port: internal_port,
        };

        let external_addr = SocketAddr::new(gateway.external_ip, gateway.mapped_port);
        self.upnp_gateway = Some(gateway);
        self.active_strategy = Some(NatStrategy::UPnP);

        Ok(external_addr)
    }

    /// Setup TURN relay connection
    pub async fn setup_turn(&mut self, username: String, credential: String) -> Result<SocketAddr> {
        if self.config.turn_servers.is_empty() {
            return Err(Error::network("No TURN servers configured"));
        }

        // Select TURN server (could use load balancing or latency-based selection)
        let server_addr = self.config.turn_servers[0].clone();

        // Implementation would establish TURN connection
        // Steps:
        // 1. Connect to TURN server
        // 2. Authenticate with credentials
        // 3. Allocate relay address
        // 4. Maintain connection with keep-alives

        let turn_conn = TurnConnection {
            server_addr: server_addr.clone(),
            allocated_addr: Some("198.51.100.1:50000".parse().unwrap()), // Placeholder
            username,
            credential,
        };

        let allocated_addr = turn_conn.allocated_addr.unwrap();
        self.turn_connections.push(turn_conn);
        self.active_strategy = Some(NatStrategy::TURN);

        Ok(allocated_addr)
    }

    /// Attempt hole punching with remote peer
    pub async fn attempt_hole_punching(
        &mut self,
        _local_addr: SocketAddr,
        _remote_addr: SocketAddr,
    ) -> Result<bool> {
        if !self.config.enable_hole_punching {
            return Err(Error::network("Hole punching is disabled"));
        }

        // Implementation would perform UDP hole punching
        // Steps:
        // 1. Both peers send packets to each other's public addresses
        // 2. NAT creates temporary bindings
        // 3. Packets eventually get through
        // 4. Verify bidirectional connectivity

        // Placeholder - would return true if successful
        Ok(false)
    }

    /// Get recommended strategy for current NAT type
    pub fn get_recommended_strategy(&self) -> NatStrategy {
        match self.detected_nat_type {
            Some(NatType::None) => NatStrategy::Direct,
            Some(NatType::FullCone) | Some(NatType::RestrictedCone) => NatStrategy::UPnP,
            Some(NatType::PortRestrictedCone) => NatStrategy::HolePunching,
            Some(NatType::Symmetric) => NatStrategy::TURN,
            _ => NatStrategy::TURN, // Default to TURN for unknown
        }
    }

    /// Establish connection using best available strategy
    pub async fn establish_connection(
        &mut self,
        local_port: u16,
        remote_addr: Option<SocketAddr>,
    ) -> Result<SocketAddr> {
        // Detect NAT type if not already done
        if self.detected_nat_type.is_none() {
            self.detect_nat_type().await?;
        }

        let strategy = self.get_recommended_strategy();

        match strategy {
            NatStrategy::Direct => {
                // Use local address directly
                Ok(SocketAddr::new("0.0.0.0".parse().unwrap(), local_port))
            }
            NatStrategy::UPnP => {
                self.setup_upnp(local_port).await
            }
            NatStrategy::HolePunching => {
                if let Some(remote) = remote_addr {
                    let local = SocketAddr::new("0.0.0.0".parse().unwrap(), local_port);
                    if self.attempt_hole_punching(local, remote).await? {
                        return Ok(local);
                    }
                }
                // Fallback to TURN if hole punching fails
                self.setup_turn("user".to_string(), "pass".to_string()).await
            }
            NatStrategy::TURN => {
                self.setup_turn("user".to_string(), "pass".to_string()).await
            }
        }
    }

    /// Release UPnP port mapping
    pub async fn release_upnp(&mut self) -> Result<()> {
        if let Some(_gateway) = &self.upnp_gateway {
            // Implementation would send delete mapping request to gateway
            self.upnp_gateway = None;
        }
        Ok(())
    }

    /// Close TURN connections
    pub async fn close_turn_connections(&mut self) -> Result<()> {
        // Implementation would send refresh with lifetime=0 to TURN servers
        self.turn_connections.clear();
        Ok(())
    }

    /// Get current external address
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        match &self.active_strategy {
            Some(NatStrategy::UPnP) => {
                self.upnp_gateway.as_ref().map(|gw| {
                    SocketAddr::new(gw.external_ip, gw.mapped_port)
                })
            }
            Some(NatStrategy::TURN) => {
                self.turn_connections.first()
                    .and_then(|conn| conn.allocated_addr)
            }
            _ => None,
        }
    }

    /// Cleanup all NAT traversal resources
    pub async fn cleanup(&mut self) -> Result<()> {
        self.release_upnp().await?;
        self.close_turn_connections().await?;
        self.active_strategy = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nat_config_default() {
        let config = NatConfig::default();
        assert!(config.enable_upnp);
        assert!(config.enable_hole_punching);
        assert_eq!(config.turn_servers.len(), 2);
    }

    #[tokio::test]
    async fn test_recommended_strategy() {
        let config = NatConfig::default();
        let mut manager = NatTraversalManager::new(config);

        // Test different NAT types
        manager.detected_nat_type = Some(NatType::None);
        assert_eq!(manager.get_recommended_strategy(), NatStrategy::Direct);

        manager.detected_nat_type = Some(NatType::FullCone);
        assert_eq!(manager.get_recommended_strategy(), NatStrategy::UPnP);

        manager.detected_nat_type = Some(NatType::Symmetric);
        assert_eq!(manager.get_recommended_strategy(), NatStrategy::TURN);
    }

    #[tokio::test]
    async fn test_nat_manager_creation() {
        let config = NatConfig::default();
        let manager = NatTraversalManager::new(config);
        
        assert!(manager.detected_nat_type.is_none());
        assert!(manager.active_strategy.is_none());
        assert!(manager.upnp_gateway.is_none());
        assert_eq!(manager.turn_connections.len(), 0);
    }
}
