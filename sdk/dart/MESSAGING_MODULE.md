# Dart P2P Messaging Module

The dchat Dart SDK includes a comprehensive P2P messaging system with end-to-end encryption, DHT-based routing, and cryptographically-verified proof of delivery.

## Features

### 1. **End-to-End Encryption**
- **Noise Protocol**: Rotating key derivation with periodic key rotation
- **ChaCha20-Poly1305**: Symmetric encryption with authentication
- **Forward Secrecy**: Keys rotate every N messages or T time units
- **Key Management**: Ed25519 keypair generation and storage

### 2. **Decentralized Peer Discovery**
- **DHT (Kademlia)**: Distributed hash table for peer discovery
- **Distance Metrics**: XOR-based peer proximity calculation
- **Bucket Management**: K-bucket storage for organizing peers
- **Node Lookup**: Efficient closest-node queries

### 3. **Peer Connection Management**
- **Connection States**: Unknown → Connecting → Connected → Disconnected
- **Trust Scoring**: Dynamic trust scores based on delivery success
- **Peer Statistics**: Message counts, bytes transferred, latency metrics
- **Stale Peer Pruning**: Automatic removal of inactive peers

### 4. **Proof of Delivery**
- **Delivery Tracking**: Pending → Delivered → Read progression
- **Cryptographic Signatures**: Ed25519 signature verification
- **On-Chain Anchoring**: Block height and relay node tracking
- **Delivery Statistics**: Success rates, pending messages, failure tracking

### 5. **Message Management**
- **Encryption Toggle**: Per-message encryption control
- **Routing Path Optimization**: Direct vs. DHT-routed delivery
- **Message Caching**: Local message storage with TTL
- **Batch Operations**: Cleanup and pruning utilities

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  MessageManager                      │
│  (High-level P2P messaging orchestration)           │
└──────────────┬──────────────────────────────────────┘
               │
       ┌───────┼───────┐
       │       │       │
   ┌───▼──┐ ┌──▼──┐ ┌──▼─────────┐
   │Crypto│ │ DHT │ │PeerManager │
   └───┬──┘ └──┬──┘ └──┬─────────┘
       │      │       │
       │      │  ┌────▼─────────────┐
       │      │  │ProofOfDelivery   │
       │      │  │Tracker           │
       │      │  └──────────────────┘
       │      │
   ┌───▼──────▼──────────────────────────┐
   │   BlockchainClient (On-Chain)       │
   │   (Transaction anchoring)           │
   └────────────────────────────────────┘
```

## Core Components

### `MessageCrypto`

Handles Noise Protocol encryption with key rotation.

```dart
final crypto = MessageCrypto();

// Encrypt message
final encrypted = crypto.encrypt(
  "Hello, World!",
  additionalData: "recipient-id"
);

// Decrypt message
final plaintext = crypto.decrypt(encrypted);
```

**Key Features:**
- ChaCha20-Poly1305 AEAD cipher
- Automatic key rotation every 100 messages
- Nonce generation (24 bytes)
- Authentication tag verification (16 bytes)

### `DHT`

Kademlia-based distributed hash table for peer discovery.

```dart
final dht = DHT(localNodeId: "peer-id");

// Add nodes to routing table
dht.addNode(DHTNode(
  nodeId: "peer-2-id",
  peerId: "peer-2-id",
  address: "192.168.1.1",
  port: 5000,
));

// Find closest nodes to target
final closest = dht.findClosest("target-id", count: 20);

// Route to peer
final route = dht.routeToClosest("target-id");
```

**Algorithm:**
- K-bucket size: 20 nodes
- XOR distance metric
- Bucket depth: 160 (for 256-bit keyspace)
- Automatic stale node pruning

### `PeerManager`

Manages P2P peer connections and trust.

```dart
final peerManager = PeerManager(localPeerId: "local-id");

// Add peer
final peer = peerManager.addPeer(
  "peer-id",
  "peer-public-key",
  address: "192.168.1.1",
  port: 5000,
);

// Connect to peer
peer.connect();
peer.markConnected();

// Record message activity
peer.recordMessage(512, incoming: false);

// Get statistics
final stats = peerManager.getStats();
```

**Features:**
- Dynamic peer eviction (LRU based on trust score)
- Connection state management
- Peer blocking/allow-listing
- Message and bandwidth tracking
- Trust score updates

### `ProofOfDeliveryTracker`

Tracks message delivery proofs with on-chain anchoring.

```dart
final tracker = ProofOfDeliveryTracker();

// Mark message as pending
tracker.markPending("message-id", "recipient-id");

// Record delivery proof
tracker.recordProof(DeliveryProof(
  messageId: "message-id",
  recipientId: "recipient-id",
  senderPublicKey: "public-key",
  status: DeliveryStatus.delivered,
  blockHeight: 12345,
));

// Check delivery status
final isDelivered = tracker.isDelivered("message-id");
final isRead = tracker.isRead("message-id");

// Get statistics
final stats = tracker.getStats();
```

**Tracking States:**
- `pending`: Message sent, awaiting confirmation
- `delivered`: Message received by recipient
- `read`: Recipient has read the message
- `failed`: Delivery failed after timeout

### `MessageManager`

High-level message orchestration combining all components.

```dart
final messageManager = MessageManager(
  localPeerId: "local-id",
  localPublicKey: "public-key",
  peerManager: peerManager,
  dht: dht,
);

// Send message
final messageId = await messageManager.sendMessage(
  "recipient-id",
  "Hello!",
  encrypt: true,
);

// Handle incoming message
messageManager.onMessageReceived.add((message) {
  print("Received: ${message.content}");
});

messageManager.handleIncomingMessage(incomingJson);

// Handle delivery proof
messageManager.onDeliveryProof.add((proof) {
  print("Proof: ${proof.messageId} - ${proof.status}");
});

messageManager.handleDeliveryProof(proofJson);

// Get statistics
final stats = messageManager.getStats();
```

## Usage Examples

### Basic P2P Messaging

```dart
import 'package:dchat_sdk/dchat.dart';

void main() async {
  final client = BlockchainClient(rpcUrl: 'http://localhost:8545');
  
  // Register user
  final user1 = await client.registerUser('alice');
  final user2 = await client.registerUser('bob');
  
  // Create message manager
  final peerManager = PeerManager(localPeerId: user1.userId);
  final dht = DHT(localNodeId: user1.userId);
  final messageManager = MessageManager(
    localPeerId: user1.userId,
    localPublicKey: user1.publicKey,
    peerManager: peerManager,
    dht: dht,
  );
  
  // Add bob as peer
  peerManager.addPeer(
    user2.userId,
    user2.publicKey,
    address: '192.168.1.100',
    port: 5000,
  ).markConnected();
  
  // Send message
  final msgId = await messageManager.sendMessage(
    user2.userId,
    'Hello Bob!',
  );
  
  print('Message sent: $msgId');
}
```

### DHT-Based Message Routing

```dart
// Initialize DHT with multiple peers
final dht = DHT(localNodeId: "alice");

for (final peer in peers) {
  dht.addNode(DHTNode(
    nodeId: peer.peerId,
    peerId: peer.peerId,
    address: peer.address,
    port: peer.port,
  ));
}

// Find route to Bob
final route = dht.routeToClosest("bob-peer-id");
if (route != null) {
  print('Route found with ${route.hopCount} hops');
  for (final hop in route.hops) {
    print('  → ${hop.address}:${hop.port}');
  }
}
```

### Delivery Proof Verification

```dart
final tracker = ProofOfDeliveryTracker();

// Send message and track
messageManager.onDeliveryProof.add((proof) {
  if (proof.verifySignature(proof.senderPublicKey)) {
    print('✓ Proof verified for ${proof.messageId}');
    print('  Status: ${proof.status}');
    print('  Block: ${proof.blockHeight}');
    print('  Relay: ${proof.relayNodeId}');
  }
});

// Get delivery statistics
final stats = tracker.getStats();
print('Delivery success rate: ${stats['successRate']}%');
print('Pending deliveries: ${stats['pending']}');
print('Failed deliveries: ${stats['failed']}');
```

### Peer Trust Management

```dart
// Monitor peer trust evolution
for (final peer in peerManager.getConnectedPeers()) {
  peer.recordMessage(256); // Successful message
  print('${peer.peerId}: trust=${peer.trustScore}');
}

// Block misbehaving peers
if (someMetric > threshold) {
  peerManager.blockPeer(suspiciousPeerId);
}

// Auto-cleanup
peerManager.pruneStalepeers(timeout: Duration(minutes: 10));
```

## Integration with Blockchain

Messages can be anchored on-chain for:
- **Cryptographic Proof**: Message hash stored on ledger
- **Timestamp Verification**: Immutable message timestamps
- **Delivery Verification**: On-chain delivery proof storage
- **Regulatory Compliance**: Audit trail for message history

```dart
// After sending message
messageManager.onDeliveryProof.add((proof) async {
  if (proof.blockHeight != null) {
    print('✓ Proof anchored on-chain at block ${proof.blockHeight}');
  }
});
```

## Performance Characteristics

| Metric | Value |
|--------|-------|
| **Encryption** | ChaCha20-Poly1305 (AEAD) |
| **Key Rotation** | Every 100 messages |
| **DHT Lookup** | O(log n) |
| **Max Peers** | 100 (configurable) |
| **Delivery Timeout** | 30 minutes (configurable) |
| **Proof Retention** | 7 days (configurable) |
| **Message Cache** | In-memory with TTL pruning |

## Security Considerations

1. **Key Rotation**: Keys automatically rotate to prevent known-plaintext attacks
2. **Nonce Uniqueness**: Random 24-byte nonces for every encryption
3. **Authentication**: Poly1305 MAC prevents tampering
4. **Forward Secrecy**: Old keys cannot decrypt new messages
5. **Trust-on-First-Use**: Initial peer connections establish trust
6. **Sybil Resistance**: Trust scores prevent peer proliferation
7. **DHT Resistance**: Closest-node queries resist Sybil attacks

## Cleanup & Maintenance

```dart
// Remove old messages
messageManager.cleanup(retention: Duration(days: 7));

// Remove stale peers
peerManager.pruneStalepeers();

// Remove old delivery proofs
deliveryTracker.prune(retention: Duration(days: 7));
```

## Error Handling

```dart
try {
  final msgId = await messageManager.sendMessage(recipientId, content);
} on Exception catch (e) {
  print('Failed to send message: $e');
  // Fallback to relay node routing
}
```

## Testing

See `example/messaging_example.dart` for comprehensive examples including:
- Peer discovery
- Message encryption/decryption
- DHT routing
- Delivery proof tracking
- Trust score management
- Statistics collection
