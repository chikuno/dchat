# Sprint 9: Network Topology & Peer Discovery - PLAN

**Phase 7 Sprint 9** - Decentralized Network Infrastructure
**Status**: Planning Complete, Ready for Implementation
**Target Duration**: 3-4 focused sessions
**Priority**: High (Core network functionality)

## Executive Summary

Sprint 9 establishes the foundational peer-to-peer networking layer for dchat, implementing DHT-based peer discovery, gossip-based message propagation, NAT traversal strategies, and connection lifecycle management. This sprint transforms dchat from a standalone service into a fully distributed network participant.

**Key Objectives**:
1. ✅ DHT peer discovery (Kademlia-based)
2. ✅ Gossip protocol for message propagation
3. ✅ NAT traversal (UPnP, STUN, hole punching, TURN fallback)
4. ✅ Connection management and pruning
5. ✅ Relay protocol foundations
6. ✅ Integration testing with network simulation

---

## Architecture Overview

```
┌─ Network Layer (dchat-network) ─────────────────────────┐
│                                                          │
│  ┌─ Peer Discovery (DHT) ──────────────────┐            │
│  │  - Kademlia routing table               │            │
│  │  - Bootstrap nodes                      │            │
│  │  - Peer lookup and announcement         │            │
│  │  - Distance metric (XOR)                │            │
│  └──────────────────────────────────────────┘            │
│                   │                                      │
│  ┌─ Gossip Protocol ────────────────────────┐            │
│  │  - Message flooding with fanout         │            │
│  │  - Deduplication (bloom filters)        │            │
│  │  - TTL management                       │            │
│  │  - Selective forwarding                 │            │
│  └──────────────────────────────────────────┘            │
│                   │                                      │
│  ┌─ NAT Traversal ──────────────────────────┐            │
│  │  - UPnP port mapping                    │            │
│  │  - STUN server queries                  │            │
│  │  - UDP hole punching                    │            │
│  │  - TURN relay fallback                  │            │
│  └──────────────────────────────────────────┘            │
│                   │                                      │
│  ┌─ Connection Manager ─────────────────────┐            │
│  │  - Active connection pool               │            │
│  │  - Peer scoring and pruning             │            │
│  │  - Health checks and timeouts           │            │
│  │  - Automatic reconnection               │            │
│  └──────────────────────────────────────────┘            │
│                   │                                      │
│  ┌─ Relay Protocol ─────────────────────────┐            │
│  │  - Message forwarding rules             │            │
│  │  - Proof-of-delivery generation         │            │
│  │  - Bandwidth accounting                 │            │
│  │  - Anti-spam filtering                  │            │
│  └──────────────────────────────────────────┘            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## Sprint 9 Components

### Component 1: DHT Peer Discovery

**File Structure**:
```
crates/dchat-network/src/discovery/
├── mod.rs              # Public API
├── dht.rs              # Kademlia DHT implementation
├── routing_table.rs    # K-bucket routing table
├── peer_info.rs        # Peer metadata and scoring
└── bootstrap.rs        # Bootstrap node management
```

**Key Features**:

#### 1.1 Kademlia DHT
```rust
pub struct Dht {
    node_id: PeerId,
    routing_table: RoutingTable,
    bootstrap_nodes: Vec<Multiaddr>,
    k_bucket_size: usize,  // Typically 20
    alpha: usize,          // Concurrency parameter (3)
}

impl Dht {
    pub async fn new(config: DhtConfig) -> Result<Self>;
    pub async fn bootstrap(&mut self) -> Result<()>;
    pub async fn find_peer(&self, peer_id: &PeerId) -> Result<Vec<PeerInfo>>;
    pub async fn announce(&self) -> Result<()>;
    pub fn routing_table(&self) -> &RoutingTable;
}
```

**Operations**:
- `FIND_NODE`: Locate k closest peers to target
- `FIND_VALUE`: Retrieve stored values (future: for content routing)
- `STORE`: Store key-value pairs in network
- `PING`: Liveness check for peers

#### 1.2 Routing Table (K-Buckets)
```rust
pub struct RoutingTable {
    local_id: PeerId,
    buckets: Vec<KBucket>,  // 256 buckets (one per bit distance)
    k: usize,               // Bucket size (20)
}

pub struct KBucket {
    peers: VecDeque<PeerInfo>,
    max_size: usize,
    last_updated: Instant,
}

impl RoutingTable {
    pub fn add_peer(&mut self, peer: PeerInfo) -> Result<()>;
    pub fn remove_peer(&mut self, peer_id: &PeerId);
    pub fn find_closest(&self, target: &PeerId, count: usize) -> Vec<PeerInfo>;
    pub fn update_peer(&mut self, peer_id: &PeerId) -> Result<()>;
}
```

**Distance Metric**: XOR-based distance for efficient routing
```rust
fn xor_distance(a: &PeerId, b: &PeerId) -> u256 {
    // XOR of peer IDs (Ed25519 public keys)
}
```

#### 1.3 Peer Information
```rust
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub last_seen: Instant,
    pub latency: Option<Duration>,
    pub reputation: i32,        // For future use
    pub capabilities: PeerCapabilities,
}

pub struct PeerCapabilities {
    pub is_relay: bool,
    pub supports_nat_traversal: bool,
    pub max_bandwidth: Option<u64>,
}
```

#### 1.4 Bootstrap Process
```rust
pub struct Bootstrap {
    nodes: Vec<Multiaddr>,
}

impl Bootstrap {
    pub async fn connect_to_network(&self, dht: &mut Dht) -> Result<()> {
        // 1. Connect to bootstrap nodes
        // 2. Perform FIND_NODE for self
        // 3. Populate routing table
        // 4. Announce presence
    }
}
```

**Bootstrap Nodes** (Hardcoded + DNS discovery):
```
/dns4/bootstrap-1.dchat.network/tcp/9000/p2p/12D3KooW...
/dns4/bootstrap-2.dchat.network/tcp/9000/p2p/12D3KooW...
/ip4/203.0.113.1/tcp/9000/p2p/12D3KooW...
```

---

### Component 2: Gossip Protocol

**File Structure**:
```
crates/dchat-network/src/gossip/
├── mod.rs              # Public API
├── protocol.rs         # Gossip protocol implementation
├── message_cache.rs    # Deduplication cache
└── flood_control.rs    # Rate limiting and spam prevention
```

**Key Features**:

#### 2.1 Gossip Protocol
```rust
pub struct GossipProtocol {
    local_id: PeerId,
    fanout: usize,              // Number of peers to forward to (typically 6)
    message_cache: MessageCache,
    flood_control: FloodControl,
}

impl GossipProtocol {
    pub async fn broadcast(&self, message: GossipMessage) -> Result<()>;
    pub async fn handle_incoming(&mut self, from: PeerId, message: GossipMessage) -> Result<()>;
    pub fn should_forward(&self, message: &GossipMessage) -> bool;
}
```

**Gossip Message Structure**:
```rust
pub struct GossipMessage {
    pub id: MessageId,          // SHA-256 hash for deduplication
    pub sender: PeerId,
    pub ttl: u8,                // Hop count (max 32)
    pub payload: Vec<u8>,       // Encrypted message
    pub timestamp: u64,
    pub signature: Signature,
}
```

#### 2.2 Message Deduplication
```rust
pub struct MessageCache {
    seen_messages: BloomFilter,
    recent_ids: LruCache<MessageId, Instant>,
    cache_size: usize,          // Typically 10,000
    ttl: Duration,              // 5 minutes
}

impl MessageCache {
    pub fn has_seen(&self, id: &MessageId) -> bool;
    pub fn mark_seen(&mut self, id: MessageId);
    pub fn cleanup_expired(&mut self);
}
```

**Bloom Filter**: Space-efficient probabilistic set membership
- False positive rate: 0.01
- Capacity: 10,000 messages
- Size: ~120 KB

#### 2.3 Flood Control
```rust
pub struct FloodControl {
    peer_limits: HashMap<PeerId, RateLimiter>,
    global_rate: RateLimiter,
}

pub struct RateLimiter {
    max_messages_per_second: u32,
    window: Duration,
    message_count: u32,
    window_start: Instant,
}

impl FloodControl {
    pub fn check_rate_limit(&mut self, peer_id: &PeerId) -> bool;
    pub fn record_message(&mut self, peer_id: &PeerId);
}
```

**Rate Limits**:
- Per-peer: 10 messages/second
- Global: 1000 messages/second
- Burst allowance: 2x sustained rate

#### 2.4 Selective Forwarding
```rust
impl GossipProtocol {
    fn select_forward_peers(&self, message: &GossipMessage, count: usize) -> Vec<PeerId> {
        // Criteria:
        // 1. Exclude sender
        // 2. Exclude peers who likely already have message
        // 3. Prioritize by latency and reputation
        // 4. Geographic diversity (optional)
    }
}
```

---

### Component 3: NAT Traversal

**File Structure**:
```
crates/dchat-network/src/nat/
├── mod.rs              # Public API
├── upnp.rs             # UPnP Internet Gateway Device (IGD)
├── stun.rs             # STUN client for public address discovery
├── hole_punching.rs    # UDP/TCP hole punching
└── turn.rs             # TURN relay fallback
```

**Key Features**:

#### 3.1 UPnP Port Mapping
```rust
pub struct UpnpManager {
    gateway: Option<Gateway>,
    mapped_ports: Vec<PortMapping>,
}

pub struct PortMapping {
    pub external_port: u16,
    pub internal_port: u16,
    pub protocol: Protocol,  // TCP or UDP
    pub lease_duration: Duration,
}

impl UpnpManager {
    pub async fn discover_gateway() -> Result<Self>;
    pub async fn add_port_mapping(&mut self, mapping: PortMapping) -> Result<()>;
    pub async fn remove_port_mapping(&mut self, mapping: &PortMapping) -> Result<()>;
    pub async fn get_external_ip(&self) -> Result<IpAddr>;
}
```

**UPnP Discovery**:
1. Broadcast SSDP M-SEARCH on 239.255.255.250:1900
2. Parse IGD device description XML
3. Call AddPortMapping on WANIPConnection service

#### 3.2 STUN Client
```rust
pub struct StunClient {
    servers: Vec<SocketAddr>,
}

pub struct StunResult {
    pub public_addr: SocketAddr,
    pub nat_type: NatType,
}

pub enum NatType {
    OpenInternet,           // No NAT
    FullCone,               // Easy traversal
    RestrictedCone,         // Moderate difficulty
    PortRestrictedCone,     // Moderate difficulty
    Symmetric,              // Difficult (needs TURN)
}

impl StunClient {
    pub async fn discover_nat(&self) -> Result<StunResult>;
}
```

**STUN Servers** (Public):
```
stun.l.google.com:19302
stun1.l.google.com:19302
stun2.l.google.com:19302
```

#### 3.3 UDP Hole Punching
```rust
pub struct HolePuncher {
    local_addr: SocketAddr,
}

impl HolePuncher {
    pub async fn punch_hole(&self, peer_addr: SocketAddr, peer_id: PeerId) -> Result<Connection> {
        // 1. Send SYN packets to peer's public address
        // 2. Coordinate timing via signaling server
        // 3. Establish bidirectional UDP flow
        // 4. Verify connection with echo test
    }
}
```

**Coordination**:
- Use bootstrap nodes as signaling servers
- Exchange public addresses via DHT
- Synchronized packet transmission

#### 3.4 TURN Relay Fallback
```rust
pub struct TurnClient {
    server: SocketAddr,
    username: String,
    password: String,
}

impl TurnClient {
    pub async fn allocate_relay(&self) -> Result<RelayConnection>;
    pub async fn send_via_relay(&self, data: &[u8], peer: PeerId) -> Result<()>;
}
```

**TURN Usage**:
- Last resort for symmetric NAT
- Bandwidth-limited (relay resource constraint)
- Paid service or community-run relays

---

### Component 4: Connection Management

**File Structure**:
```
crates/dchat-network/src/connection/
├── mod.rs              # Public API
├── manager.rs          # Connection pool management
├── health.rs           # Connection health monitoring
└── pruning.rs          # Peer eviction strategies
```

**Key Features**:

#### 4.1 Connection Manager
```rust
pub struct ConnectionManager {
    connections: HashMap<PeerId, Connection>,
    max_connections: usize,     // Typically 50
    target_connections: usize,  // Typically 30
    pruning_strategy: PruningStrategy,
}

pub struct Connection {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub state: ConnectionState,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub metrics: ConnectionMetrics,
}

pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnecting,
    Failed(String),
}

impl ConnectionManager {
    pub async fn connect(&mut self, peer: PeerInfo) -> Result<()>;
    pub async fn disconnect(&mut self, peer_id: &PeerId) -> Result<()>;
    pub fn get_connection(&self, peer_id: &PeerId) -> Option<&Connection>;
    pub async fn maintain_connections(&mut self) -> Result<()>;
}
```

#### 4.2 Connection Metrics
```rust
pub struct ConnectionMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub latency: Option<Duration>,
    pub packet_loss: f32,           // Percentage
    pub uptime: Duration,
}
```

#### 4.3 Health Monitoring
```rust
pub struct HealthMonitor {
    check_interval: Duration,       // 30 seconds
    timeout: Duration,              // 10 seconds
}

impl HealthMonitor {
    pub async fn check_connection(&self, conn: &Connection) -> HealthStatus;
    pub async fn ping_peer(&self, peer_id: &PeerId) -> Result<Duration>;
}

pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
```

**Health Checks**:
- Periodic PING messages
- Response time measurement
- Packet loss detection
- Idle timeout (5 minutes)

#### 4.4 Pruning Strategy
```rust
pub enum PruningStrategy {
    LeastRecentlyUsed,
    LowestReputation,
    HighestLatency,
    Combined(Vec<PruningStrategy>),
}

impl ConnectionManager {
    fn should_prune(&self) -> bool {
        self.connections.len() > self.max_connections
    }
    
    fn select_peer_to_prune(&self) -> Option<PeerId> {
        // Score peers based on pruning strategy
        // Never prune bootstrap or relay nodes
        // Prefer pruning idle or low-quality connections
    }
}
```

**Pruning Criteria**:
- Idle time > 10 minutes
- High latency (> 500ms)
- Low message volume
- Poor connection quality
- Protect: relays, bootstrap nodes, recent peers

---

### Component 5: Relay Protocol Foundations

**File Structure**:
```
crates/dchat-network/src/relay/
├── mod.rs              # Public API
├── protocol.rs         # Message forwarding logic
├── proof.rs            # Proof-of-delivery generation
└── accounting.rs       # Bandwidth tracking
```

**Key Features**:

#### 5.1 Message Forwarding
```rust
pub struct RelayProtocol {
    local_id: PeerId,
    max_relay_hops: u8,         // Typically 3
    bandwidth_limit: u64,       // Bytes per second
}

pub struct RelayMessage {
    pub message_id: MessageId,
    pub sender: PeerId,
    pub recipient: PeerId,
    pub hops: Vec<PeerId>,      // Relay path
    pub encrypted_payload: Vec<u8>,
    pub proof_of_delivery: Option<ProofOfDelivery>,
}

impl RelayProtocol {
    pub async fn forward_message(&self, message: RelayMessage) -> Result<()>;
    pub fn should_relay(&self, message: &RelayMessage) -> bool;
}
```

#### 5.2 Proof of Delivery
```rust
pub struct ProofOfDelivery {
    pub message_id: MessageId,
    pub relay_path: Vec<PeerId>,
    pub timestamp: u64,
    pub signatures: Vec<Signature>,  // Each relay signs
}

impl ProofOfDelivery {
    pub fn new(message_id: MessageId) -> Self;
    pub fn add_hop(&mut self, relay_id: PeerId, signature: Signature);
    pub fn verify(&self) -> Result<bool>;
    pub fn serialize_for_chain(&self) -> Vec<u8>;
}
```

**On-Chain Submission**:
- Batch proofs every 100 messages or 1 hour
- Submit Merkle root to chat chain
- Relays claim rewards based on verified proofs

#### 5.3 Bandwidth Accounting
```rust
pub struct BandwidthAccountant {
    peer_usage: HashMap<PeerId, BandwidthUsage>,
    global_limit: u64,
}

pub struct BandwidthUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub window_start: Instant,
    pub window_duration: Duration,
}

impl BandwidthAccountant {
    pub fn record_sent(&mut self, peer: &PeerId, bytes: u64);
    pub fn record_received(&mut self, peer: &PeerId, bytes: u64);
    pub fn check_limit(&self, peer: &PeerId) -> bool;
}
```

---

## Configuration Schema

**File**: `crates/dchat-core/src/config.rs`

```rust
pub struct NetworkConfig {
    // DHT settings
    pub dht_enabled: bool,                  // Default: true
    pub k_bucket_size: usize,               // Default: 20
    pub dht_alpha: usize,                   // Default: 3
    pub bootstrap_nodes: Vec<String>,       // Multiaddr strings
    
    // Gossip settings
    pub gossip_fanout: usize,               // Default: 6
    pub gossip_message_cache_size: usize,   // Default: 10000
    pub gossip_max_ttl: u8,                 // Default: 32
    
    // Connection settings
    pub max_connections: usize,             // Default: 50
    pub target_connections: usize,          // Default: 30
    pub connection_timeout_secs: u64,       // Default: 30
    pub idle_timeout_secs: u64,             // Default: 300
    
    // NAT traversal
    pub enable_upnp: bool,                  // Default: true
    pub enable_hole_punching: bool,         // Default: true
    pub stun_servers: Vec<String>,          // Default: Google STUN
    pub turn_servers: Vec<TurnServerConfig>,
    
    // Relay settings
    pub enable_relay: bool,                 // Default: false (opt-in)
    pub max_relay_hops: u8,                 // Default: 3
    pub relay_bandwidth_limit_mbps: u64,    // Default: 10
}

pub struct TurnServerConfig {
    pub address: String,
    pub username: String,
    pub password: String,
}
```

**config.example.toml**:
```toml
[network]
# DHT peer discovery
dht_enabled = true
k_bucket_size = 20
bootstrap_nodes = [
    "/dns4/bootstrap-1.dchat.network/tcp/9000/p2p/12D3KooW...",
    "/dns4/bootstrap-2.dchat.network/tcp/9000/p2p/12D3KooW..."
]

# Gossip protocol
gossip_fanout = 6
gossip_message_cache_size = 10000
gossip_max_ttl = 32

# Connection management
max_connections = 50
target_connections = 30
connection_timeout_secs = 30
idle_timeout_secs = 300

# NAT traversal
enable_upnp = true
enable_hole_punching = true
stun_servers = [
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302"
]

# Relay node (opt-in)
enable_relay = false
max_relay_hops = 3
relay_bandwidth_limit_mbps = 10
```

---

## Testing Strategy

### Test Categories

#### 1. Unit Tests
- DHT routing table operations
- Message deduplication
- Rate limiting logic
- Pruning algorithms

#### 2. Integration Tests
```
tests/network_integration_tests.rs
├── test_dht_peer_discovery
├── test_gossip_message_propagation
├── test_nat_traversal_scenarios
├── test_connection_management
├── test_relay_forwarding
└── test_network_partitions
```

#### 3. Network Simulation
```rust
pub struct NetworkSimulator {
    nodes: Vec<SimulatedNode>,
    latency_matrix: HashMap<(PeerId, PeerId), Duration>,
    partition_groups: Vec<Vec<PeerId>>,
}

impl NetworkSimulator {
    pub fn add_node(&mut self, node: SimulatedNode);
    pub fn set_latency(&mut self, a: PeerId, b: PeerId, latency: Duration);
    pub fn create_partition(&mut self, groups: Vec<Vec<PeerId>>);
    pub async fn run_simulation(&self, duration: Duration) -> SimulationResults;
}
```

#### 4. Chaos Testing
- Random node failures
- Network partitions
- Message drops
- Byzantine peers
- NAT type variations

---

## Implementation Phases

### Phase 1: DHT Peer Discovery (Session 1)
- [ ] Create discovery module structure
- [ ] Implement Kademlia routing table
- [ ] Add peer info structures
- [ ] Implement bootstrap logic
- [ ] Write DHT unit tests

### Phase 2: Gossip Protocol (Session 1-2)
- [ ] Create gossip module structure
- [ ] Implement message broadcasting
- [ ] Add deduplication with bloom filters
- [ ] Implement flood control
- [ ] Write gossip tests

### Phase 3: NAT Traversal (Session 2)
- [ ] Implement UPnP IGD client
- [ ] Add STUN client
- [ ] Implement hole punching
- [ ] Add TURN fallback
- [ ] Write NAT traversal tests

### Phase 4: Connection Management (Session 3)
- [ ] Create connection manager
- [ ] Implement health monitoring
- [ ] Add pruning strategies
- [ ] Implement automatic reconnection
- [ ] Write connection management tests

### Phase 5: Relay Protocol (Session 3)
- [ ] Create relay module
- [ ] Implement message forwarding
- [ ] Add proof-of-delivery
- [ ] Implement bandwidth accounting
- [ ] Write relay tests

### Phase 6: Integration & Testing (Session 4)
- [ ] Create network simulator
- [ ] Write integration tests
- [ ] Perform chaos testing
- [ ] Benchmark performance
- [ ] Create documentation

---

## Performance Targets

### DHT Operations
- Peer lookup: < 500ms (median)
- Routing table update: < 10ms
- Bootstrap: < 5 seconds

### Gossip Propagation
- Message fanout: 6 peers
- Full network propagation: < 2 seconds (100 nodes)
- Deduplication overhead: < 1ms per message

### Connection Management
- New connection establishment: < 2 seconds
- Health check interval: 30 seconds
- Pruning decision: < 50ms

### NAT Traversal
- UPnP mapping: < 3 seconds
- STUN discovery: < 1 second
- Hole punching success rate: > 80%

---

## Security Considerations

### DHT Security
✅ Sybil attack resistance (reputation + proof-of-work)
✅ Eclipse attack prevention (diverse peer selection)
✅ Routing table poisoning protection (signature verification)

### Gossip Security
✅ Message authentication (Ed25519 signatures)
✅ Replay attack prevention (timestamps + sequence numbers)
✅ Flood attack mitigation (rate limiting)

### Connection Security
✅ Peer ID verification (cryptographic)
✅ Connection encryption (Noise Protocol)
✅ DDoS protection (connection limits)

---

## Dependencies

### New Crates
```toml
[dependencies]
# DHT and routing
libp2p = { version = "0.53", features = ["kad", "identify", "ping"] }
multihash = "0.19"
multiaddr = "0.18"

# Bloom filters
probabilistic-collections = "0.7"

# NAT traversal
igd = "0.12"              # UPnP
stun = "0.5"              # STUN client

# Rate limiting
governor = "0.6"

# Simulation
tokio-test = "0.4"
```

---

## Success Criteria

✅ **Peer Discovery**: Node can discover and connect to network peers
✅ **Message Propagation**: Gossip reaches 90%+ of network within 2 seconds
✅ **NAT Traversal**: 80%+ success rate for NAT hole punching
✅ **Connection Stability**: < 1% connection failure rate
✅ **Performance**: Sub-second latency for DHT lookups
✅ **Tests**: 100% pass rate for all integration tests

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Sybil attacks on DHT | High | Proof-of-work, reputation system |
| Network partitions | High | Multi-path routing, gossip redundancy |
| NAT traversal failures | Medium | TURN fallback, relay nodes |
| Gossip message storms | Medium | Rate limiting, TTL enforcement |
| Connection pool exhaustion | Low | Pruning, connection limits |

---

## Next Sprint Preview (Sprint 10)

After completing Sprint 9:
- **Message Persistence**: Integrate database with message flow
- **End-to-End Encryption**: Noise Protocol integration
- **Channel Management**: On-chain channel creation and permissions
- **Reputation System**: Scoring and Sybil resistance

---

**Date**: October 28, 2025
**Sprint Duration**: 3-4 focused implementation sessions
**Priority**: High (Core networking functionality)
**Status**: ✅ PLAN COMPLETE - Ready for implementation
