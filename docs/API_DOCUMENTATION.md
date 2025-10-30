# dchat API Documentation

## Overview

dchat is a decentralized end-to-end encrypted chat application with the following core features:

- **End-to-End Encryption**: Noise Protocol (XX pattern) with rotating keys
- **Sovereign Identity**: Hierarchical key derivation (BIP-32/44), multi-device sync
- **Blockchain Message Ordering**: Tamper-proof message sequencing on chat chain
- **Metadata Resistance**: Onion routing, ZK proofs, blind tokens
- **Decentralized Relay Network**: Incentivized nodes with proof-of-delivery
- **Account Recovery**: Multi-signature guardian system with timelocked recovery
- **Privacy-First**: Contact graph hiding, cover traffic, stealth addresses

## Architecture

### Two-Chain Design

**Chat Chain**: Identity, messaging, channels, permissions, governance, reputation
**Currency Chain**: Payments, staking, rewards, economics

### Core Components

- **dchat-core**: Configuration, types, error handling, event bus
- **dchat-crypto**: Noise Protocol, Ed25519 identity, key rotation, post-quantum hybrid
- **dchat-identity**: Hierarchical derivation, multi-device, guardians, burner identities
- **dchat-messaging**: Message ordering, delivery proofs, offline queue, expiration
- **dchat-network**: libp2p, NAT traversal, relay nodes, onion routing, rate limiting
- **dchat-storage**: SQLite/RocksDB, deduplication, encrypted backups, lifecycle
- **dchat-chain**: Blockchain integration, snapshots, dispute resolution
- **dchat-privacy**: ZK proofs, blind tokens, metadata hiding
- **dchat-governance**: DAO voting, moderation, ethics constraints
- **dchat-bridge**: Cross-chain atomic transactions
- **dchat-observability**: Prometheus metrics, distributed tracing
- **dchat-accessibility**: WCAG 2.1 AA+ compliance, keyless UX

## Getting Started

### Installation

```bash
cargo install dchat
```

### Basic Usage

```rust
use dchat::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize configuration
    let config = Config::default();
    
    // Generate identity keypair
    let keypair = KeyPair::generate();
    
    // Start network manager
    let network = NetworkManager::new(NetworkConfig::default()).await?;
    
    // Create identity
    let identity = Identity::new(keypair.public_key());
    
    // Send a message
    let message = MessageBuilder::new()
        .sender(identity.id())
        .content("Hello, dchat!")
        .build()?;
    
    network.send_message(message).await?;
    
    Ok(())
}
```

## API Reference

### Cryptography (dchat-crypto)

#### Key Generation

```rust
use dchat_crypto::keys::KeyPair;

// Generate Ed25519 identity keypair
let keypair = KeyPair::generate();

// Generate X25519 DH keypair
let dh_keypair = KeyPair::generate_dh();

// Import from bytes
let keypair = KeyPair::from_bytes(&secret_bytes)?;
```

#### Noise Protocol Handshake

```rust
use dchat_crypto::noise::{NoiseHandshake, NoisePattern};

// Initiator side
let mut handshake = NoiseHandshake::new(NoisePattern::XX, true)?;
let msg1 = handshake.write_message(&[])?;

// Responder side
let mut responder = NoiseHandshake::new(NoisePattern::XX, false)?;
let _ = responder.read_message(&msg1)?;
let msg2 = responder.write_message(&[])?;

// Complete handshake
let _ = handshake.read_message(&msg2)?;
let session = handshake.into_transport_mode()?;
```

#### Key Rotation

```rust
use dchat_crypto::rotation::{KeyRotationManager, RotationPolicy};

let policy = RotationPolicy {
    max_messages: 1000,
    max_age_seconds: 3600 * 24, // 24 hours
};

let mut rotation_mgr = KeyRotationManager::new(keypair, policy);

// Check if rotation needed
if rotation_mgr.should_rotate() {
    rotation_mgr.rotate()?;
}
```

#### Post-Quantum Hybrid Encryption

```rust
use dchat_crypto::hybrid::{HybridKeyPair, HybridCiphertext};

// Generate hybrid keypair (Curve25519 + Kyber768)
let keypair = HybridKeyPair::generate();

// Encrypt with hybrid scheme
let plaintext = b"secret message";
let ciphertext = keypair.public_key().encrypt(plaintext)?;

// Decrypt
let recovered = keypair.decrypt(&ciphertext)?;
```

### Identity (dchat-identity)

#### Creating an Identity

```rust
use dchat_identity::identity::{Identity, IdentityManager};

let manager = IdentityManager::new();
let identity = manager.create_identity("Alice").await?;

// Get identity ID
let id = identity.id();

// Export for backup
let exported = identity.export_encrypted(password)?;
```

#### Hierarchical Key Derivation

```rust
use dchat_identity::derivation::{IdentityDerivation, KeyPath};

let seed = b"secure random seed 32 bytes long";
let derivation = IdentityDerivation::from_seed(seed);

// Derive keys at path m/44'/0'/0'/0/0
let path = KeyPath::from_str("m/44'/0'/0'/0/0")?;
let derived_key = derivation.derive(&path)?;
```

#### Multi-Device Synchronization

```rust
use dchat_identity::device::{Device, DeviceManager, DeviceType};

let mut device_mgr = DeviceManager::new(identity);

// Register new device
let device = Device::new(DeviceType::Desktop, "My Laptop");
device_mgr.register_device(device).await?;

// Sync state across devices
device_mgr.sync().await?;
```

#### Burner Identities

```rust
use dchat_identity::burner::{BurnerIdentity, BurnerManager};

let mut burner_mgr = BurnerManager::new(main_identity);

// Create temporary burner identity
let burner = burner_mgr.create_burner().await?;

// Use burner for anonymous messaging
let message = MessageBuilder::new()
    .sender(burner.id())
    .content("Anonymous message")
    .build()?;

// Destroy burner after use
burner_mgr.destroy_burner(burner.id()).await?;
```

#### Guardian-Based Account Recovery

```rust
use dchat_identity::guardian::{Guardian, GuardianManager, RecoveryRequest};

let mut guardian_mgr = GuardianManager::new();

// Add guardians (3-of-5 threshold)
let guardians = vec![
    Guardian::new(alice_pubkey, "Alice"),
    Guardian::new(bob_pubkey, "Bob"),
    Guardian::new(charlie_pubkey, "Charlie"),
    Guardian::new(dave_pubkey, "Dave"),
    Guardian::new(eve_pubkey, "Eve"),
];

guardian_mgr.set_guardians(guardians, 3).await?;

// Initiate recovery
let request = RecoveryRequest::new(identity_id, new_keypair.public_key());
let recovery = guardian_mgr.initiate_recovery(request).await?;

// Guardians approve (3 required)
guardian_mgr.approve_recovery(recovery.id(), alice_signature).await?;
guardian_mgr.approve_recovery(recovery.id(), bob_signature).await?;
guardian_mgr.approve_recovery(recovery.id(), charlie_signature).await?;

// Execute recovery after timelock (24 hours)
guardian_mgr.execute_recovery(recovery.id()).await?;
```

### Messaging (dchat-messaging)

#### Creating and Sending Messages

```rust
use dchat_messaging::types::{MessageBuilder, MessageType};

let message = MessageBuilder::new()
    .sender(identity.id())
    .recipient(bob_id)
    .content("Hello, Bob!")
    .message_type(MessageType::Direct)
    .build()?;

// Send via network
network.send_message(message).await?;
```

#### Message Ordering and Sequence Numbers

```rust
use dchat_messaging::ordering::{MessageOrder, SequenceNumber};

let mut order = MessageOrder::new();

// Messages receive sequential numbers
let seq_num = order.next_sequence();
let message = MessageBuilder::new()
    .sequence(seq_num)
    .sender(identity.id())
    .content("Ordered message")
    .build()?;

// Verify ordering on-chain
order.verify_sequence(&message).await?;
```

#### Offline Message Queue

```rust
use dchat_messaging::queue::{MessageQueue, OfflineQueue};

let mut queue = OfflineQueue::new(database);

// Queue messages while offline
queue.enqueue(message).await?;

// Sync when back online
let pending = queue.pending_messages().await?;
for msg in pending {
    network.send_message(msg).await?;
}
```

#### Proof of Delivery

```rust
use dchat_messaging::delivery::{DeliveryProof, DeliveryTracker};

let mut tracker = DeliveryTracker::new();

// Relay submits proof on-chain
let proof = DeliveryProof::new(message_id, relay_signature, timestamp);
tracker.submit_proof(proof).await?;

// Verify delivery and claim reward
if tracker.is_delivered(message_id).await? {
    relay.claim_reward(message_id).await?;
}
```

#### Message Expiration

```rust
use dchat_messaging::expiration::{MessageExpiration, ExpirationPolicy};

let policy = ExpirationPolicy {
    ttl_seconds: 3600 * 24 * 7, // 7 days
    auto_delete: true,
};

let mut expiration = MessageExpiration::new(policy);

// Messages expire automatically
expiration.cleanup_expired().await?;
```

### Networking (dchat-network)

#### Starting a Network Node

```rust
use dchat_network::swarm::{NetworkManager, NetworkConfig};

let config = NetworkConfig {
    listen_address: "/ip4/0.0.0.0/tcp/7070".parse()?,
    bootstrap_peers: vec![
        "/ip4/1.2.3.4/tcp/7070/p2p/12D3KooWRf...".parse()?,
    ],
    ..Default::default()
};

let network = NetworkManager::new(config).await?;
```

#### NAT Traversal

```rust
use dchat_network::nat::{NatTraversal, NatConfig};

let nat_config = NatConfig {
    enable_upnp: true,
    enable_turn: true,
    turn_servers: vec!["turn:turn.dchat.network:3478".to_string()],
};

let nat = NatTraversal::new(nat_config);
nat.setup().await?;
```

#### Relay Node

```rust
use dchat_network::relay::{RelayNode, RelayConfig};

let relay_config = RelayConfig {
    stake_amount: 1000, // Token stake
    max_connections: 1000,
    uptime_tracking: true,
};

let relay = RelayNode::new(relay_config).await?;
relay.start().await?;

// Earn rewards for message delivery
relay.submit_delivery_proof(message_id, proof).await?;
```

#### Onion Routing

```rust
use dchat_network::onion::{OnionRouter, CircuitBuilder};

let router = OnionRouter::new();

// Build 3-hop circuit
let circuit = CircuitBuilder::new()
    .add_relay(relay1)
    .add_relay(relay2)
    .add_relay(relay3)
    .build()?;

// Send message through circuit
router.send_through_circuit(&circuit, message).await?;
```

#### Rate Limiting

```rust
use dchat_network::rate_limit::{RateLimiter, RateLimitConfig};

let config = RateLimitConfig {
    messages_per_second: 10,
    burst_capacity: 20,
    reputation_multiplier: true,
};

let mut limiter = RateLimiter::new(config);

// Check rate limit before sending
if limiter.check_limit(peer_id).await? {
    network.send_message(message).await?;
} else {
    // Rate limited - backpressure
}
```

### Storage (dchat-storage)

#### Database Operations

```rust
use dchat_storage::database::{Database, DatabaseConfig};

let config = DatabaseConfig {
    path: "data/dchat.db".into(),
    max_connections: 10,
};

let db = Database::new(config).await?;

// Store message
db.store_message(message).await?;

// Query messages
let messages = db.get_messages_for_user(user_id).await?;
```

#### Content-Addressable Deduplication

```rust
use dchat_storage::deduplication::{ContentAddressable, DeduplicationStore};

let mut dedup = DeduplicationStore::new(db);

// Store content by hash
let content = b"message content";
let hash = dedup.store(content).await?;

// Retrieve by hash
let retrieved = dedup.retrieve(&hash).await?;
```

#### Encrypted Backups

```rust
use dchat_storage::backup::{BackupManager, EncryptedBackup};

let backup_mgr = BackupManager::new(encryption_key);

// Create encrypted backup
let backup = backup_mgr.create_backup(db).await?;

// Upload to cloud (user's choice)
cloud_provider.upload(backup.encrypted_data()).await?;

// Restore from backup
let restored_db = backup_mgr.restore_backup(backup, password).await?;
```

#### Message Lifecycle and TTL

```rust
use dchat_storage::lifecycle::{LifecycleManager, TtlConfig};

let ttl_config = TtlConfig {
    default_ttl_days: 30,
    max_ttl_days: 365,
    cleanup_interval_hours: 24,
};

let mut lifecycle = LifecycleManager::new(db, ttl_config);

// Auto-cleanup expired messages
lifecycle.start_cleanup_task().await?;
```

### Privacy (dchat-privacy)

#### Zero-Knowledge Proofs

```rust
use dchat_privacy::zk::{ZkProof, ZkStatement};

// Prove statement without revealing details
let statement = ZkStatement::ContactGraphIndependent(user1, user2);
let proof = ZkProof::generate(&statement, witness)?;

// Verify proof
if proof.verify(&statement)? {
    // Statement is true, details remain private
}
```

#### Blind Tokens

```rust
use dchat_privacy::tokens::{BlindToken, TokenServer};

let server = TokenServer::new();

// User blinds token request
let (blinded_token, unblinding_factor) = BlindToken::blind(message_id);

// Server signs without seeing message_id
let signed = server.sign(&blinded_token)?;

// User unblinds to get valid signature
let token = BlindToken::unblind(signed, unblinding_factor);

// Use token for anonymous action
anonymous_action.execute(token).await?;
```

#### Metadata Hiding

```rust
use dchat_privacy::metadata::{CoverTraffic, TimingObfuscation};

let mut cover = CoverTraffic::new();

// Send cover traffic to hide real messages
cover.start(rate_per_minute = 10).await?;

// Obfuscate timing
let obfuscated_timestamp = TimingObfuscation::randomize(real_timestamp);
```

### Governance (dchat-governance)

#### DAO Voting

```rust
use dchat_governance::voting::{Proposal, Vote, VotingManager};

let voting = VotingManager::new();

// Create proposal
let proposal = Proposal::new(
    "Increase relay rewards by 10%",
    "Detailed description...",
    ProposalType::ParameterChange,
);

voting.create_proposal(proposal).await?;

// Vote on proposal
let vote = Vote::new(proposal_id, identity.id(), VoteChoice::Yes);
voting.cast_vote(vote).await?;

// Execute if passed
if voting.is_passed(proposal_id).await? {
    voting.execute_proposal(proposal_id).await?;
}
```

#### Decentralized Moderation

```rust
use dchat_governance::moderation::{ModerationCase, JuryVote};

// Report abuse (ZK-encrypted)
let report = AbureReport::new(message_id, violation_type);
let encrypted_report = report.encrypt_with_zk()?;
governance.submit_report(encrypted_report).await?;

// Random jury selected
let jury = governance.select_jury(case_id).await?;

// Jury votes
let verdict = JuryVote::new(case_id, juror_id, guilty = true);
governance.record_vote(verdict).await?;

// Execute if consensus
if governance.has_consensus(case_id).await? {
    governance.execute_moderation(case_id).await?;
}
```

#### Ethical Constraints

```rust
use dchat_governance::ethics::{VotingPowerCaps, TermLimits};

// No single entity can have >5% voting power
let caps = VotingPowerCaps::new(max_percentage = 5.0);
caps.enforce(voter_id, token_amount).await?;

// Term limits for governance positions
let term_limits = TermLimits::new(max_consecutive_terms = 2);
term_limits.check_eligibility(candidate_id).await?;
```

### Cross-Chain Bridge (dchat-bridge)

#### Atomic Swap

```rust
use dchat_bridge::atomic::{AtomicSwap, SwapProposal};

// Propose swap between chat chain and currency chain
let proposal = SwapProposal {
    chat_chain_asset: reputation_points(100),
    currency_chain_asset: tokens(10),
    timeout_blocks: 144, // 24 hours
};

let swap = AtomicSwap::new(proposal);

// Initiate on chat chain
swap.initiate_chat_side(identity).await?;

// Complete on currency chain
swap.complete_currency_side(counterparty).await?;

// Or refund if timeout
if swap.is_expired().await? {
    swap.refund().await?;
}
```

---

## TypeScript/JavaScript SDK

### Installation

```bash
npm install @dchat/sdk
```

### Basic Usage

#### Creating a Client

```typescript
import { Client } from '@dchat/sdk';

// Create a client with builder pattern
const client = await Client.builder()
  .name('Alice')
  .dataDir('/path/to/data')
  .listenPort(9001)
  .encryption(true)
  .build();

// Get identity information
const identity = client.getIdentity();
console.log(`User ID: ${identity.userId}`);
console.log(`Username: ${identity.username}`);
console.log(`Public Key: ${identity.publicKey}`);
console.log(`Reputation: ${identity.reputation}`);
```

#### Connecting to the Network

```typescript
// Connect to dchat network
await client.connect();

// Check connection status
const config = client.getConfig();
console.log(`Connected on port: ${config.listenPort}`);
```

#### Sending Messages

```typescript
// Send text message
await client.sendMessage('Hello, decentralized world!');

// Send with options
await client.sendMessage('Private message', {
  recipientId: 'user-id-here',
  expiresIn: 3600, // 1 hour TTL
});
```

#### Receiving Messages

```typescript
// Fetch all messages
const messages = await client.receiveMessages();

for (const msg of messages) {
  switch (msg.content.type) {
    case 'Text':
      console.log(`Text: ${msg.content.text}`);
      break;
    case 'Image':
      console.log(`Image: ${msg.content.mimeType}`);
      break;
    case 'File':
      console.log(`File: ${msg.content.filename}`);
      break;
  }
}

// Filter by status
const unreadMessages = messages.filter(
  m => m.status !== 'Read'
);
```

#### Message Types

```typescript
import { MessageContent, MessageStatus } from '@dchat/sdk';

// Text message
const textContent: MessageContent = {
  type: 'Text',
  text: 'Hello world'
};

// Image message
const imageContent: MessageContent = {
  type: 'Image',
  data: imageBuffer,
  mimeType: 'image/png'
};

// File message
const fileContent: MessageContent = {
  type: 'File',
  data: fileBuffer,
  filename: 'document.pdf',
  mimeType: 'application/pdf'
};

// Audio message
const audioContent: MessageContent = {
  type: 'Audio',
  data: audioBuffer,
  durationMs: 30000 // 30 seconds
};

// Video message
const videoContent: MessageContent = {
  type: 'Video',
  data: videoBuffer,
  durationMs: 60000,
  width: 1920,
  height: 1080
};

// Sticker message
const stickerContent: MessageContent = {
  type: 'Sticker',
  packId: 'pack-123',
  stickerId: 'sticker-456'
};

// Message status enum
enum MessageStatus {
  Created = 'Created',
  Sent = 'Sent',
  Delivered = 'Delivered',
  Read = 'Read',
  Failed = 'Failed',
  Expired = 'Expired',
}
```

#### Identity and Profile

```typescript
import { Identity } from '@dchat/sdk';

const identity: Identity = client.getIdentity();

// Basic info
console.log(identity.userId);
console.log(identity.username);
console.log(identity.publicKey);

// Profile info
console.log(identity.displayName);
console.log(identity.bio);

// Reputation and verification
console.log(identity.reputation); // 0-100
console.log(identity.verified); // true/false
console.log(identity.badges); // ['early-adopter', 'verified']

// Timestamps
console.log(identity.createdAt);
```

#### Disconnecting

```typescript
// Disconnect cleanly
await client.disconnect();
```

---

### Running a Relay Node

#### Basic Relay Setup

```typescript
import { RelayNode } from '@dchat/sdk';

// Create relay with default config
const relay = RelayNode.create();

// Start the relay
await relay.start();

// Check if running
console.log(`Running: ${relay.isRunning()}`);

// Stop the relay
await relay.stop();
```

#### Custom Relay Configuration

```typescript
// Create relay with custom configuration
const relay = RelayNode.withConfig({
  name: 'MyRelayNode',
  listenAddr: '0.0.0.0',
  listenPort: 9000,
  stakingEnabled: true,
  minUptimePercent: 99.0,
  maxConnections: 1000,
  bootstrapPeers: [
    '/ip4/1.2.3.4/tcp/9000/p2p/12D3KooW...'
  ],
});

await relay.start();
```

#### Relay Statistics

```typescript
import { RelayStats } from '@dchat/sdk';

// Get relay statistics
const stats: RelayStats = await relay.getStats();

console.log(`Connected peers: ${stats.connectedPeers}`);
console.log(`Messages relayed: ${stats.messagesRelayed}`);
console.log(`Uptime: ${stats.uptimePercent.toFixed(2)}%`);
console.log(`Reputation score: ${stats.reputationScore}`);
```

#### Relay Configuration

```typescript
// Get current relay configuration
const config = relay.getConfig();

console.log(`Name: ${config.name}`);
console.log(`Listen address: ${config.listenAddr}:${config.listenPort}`);
console.log(`Staking enabled: ${config.stakingEnabled}`);
console.log(`Min uptime: ${config.minUptimePercent}%`);
console.log(`Max connections: ${config.maxConnections}`);
```

---

### Configuration Options

#### Client Configuration

```typescript
import { ClientConfig, defaultClientConfig } from '@dchat/sdk';

const customConfig: ClientConfig = {
  ...defaultClientConfig,
  dataDir: '/custom/data/path',
  listenPort: 9001,
  encryption: true,
  autoConnect: false,
  reconnectOnFailure: true,
  reconnectDelayMs: 5000,
};

const client = await Client.builder()
  .withConfig(customConfig)
  .build();
```

#### Storage Configuration

```typescript
import { StorageConfig, defaultStorageConfig } from '@dchat/sdk';

const storageConfig: StorageConfig = {
  ...defaultStorageConfig,
  databasePath: '/path/to/dchat.db',
  maxMessageCount: 100000,
  enableBackup: true,
  backupIntervalHours: 24,
};
```

#### Network Configuration

```typescript
import { NetworkConfig, defaultNetworkConfig } from '@dchat/sdk';

const networkConfig: NetworkConfig = {
  ...defaultNetworkConfig,
  bootstrapPeers: [
    '/ip4/1.2.3.4/tcp/9000/p2p/12D3KooW...',
    '/ip4/5.6.7.8/tcp/9000/p2p/12D3KooX...',
  ],
  enableNatTraversal: true,
  enableUpnp: true,
  connectionTimeout: 30000,
  maxRetries: 3,
};
```

#### Relay Configuration

```typescript
import { RelayConfig, defaultRelayConfig } from '@dchat/sdk';

const relayConfig: RelayConfig = {
  ...defaultRelayConfig,
  name: 'MyRelay',
  listenAddr: '0.0.0.0',
  listenPort: 9000,
  stakingEnabled: true,
  stakeAmount: 1000,
  minUptimePercent: 99.0,
  maxConnections: 1000,
  rewardAddress: '0x...',
};

const relay = RelayNode.withConfig(relayConfig);
```

---

### Error Handling

```typescript
import { SdkError, ErrorCode } from '@dchat/sdk';

try {
  await client.connect();
} catch (error) {
  if (error instanceof SdkError) {
    switch (error.code) {
      case ErrorCode.NetworkError:
        console.error('Network connection failed:', error.message);
        break;
      case ErrorCode.AuthenticationError:
        console.error('Authentication failed:', error.message);
        break;
      case ErrorCode.ConfigurationError:
        console.error('Invalid configuration:', error.message);
        break;
      case ErrorCode.StorageError:
        console.error('Storage operation failed:', error.message);
        break;
      default:
        console.error('Unknown error:', error.message);
    }
  } else {
    console.error('Unexpected error:', error);
  }
}
```

---

### Complete Examples

#### Example 1: Basic Chat Client

```typescript
import { Client } from '@dchat/sdk';

async function basicChat() {
  // Create and connect client
  const alice = await Client.builder()
    .name('Alice')
    .dataDir('/tmp/dchat_alice')
    .listenPort(9001)
    .encryption(true)
    .build();

  const identity = alice.getIdentity();
  console.log(`User: ${identity.username} (${identity.userId})`);

  // Connect to network
  await alice.connect();
  console.log('Connected to dchat network');

  // Send a message
  await alice.sendMessage('Hello, decentralized world!');
  console.log('Message sent');

  // Receive messages
  const messages = await alice.receiveMessages();
  console.log(`Received ${messages.length} messages`);

  for (const msg of messages) {
    if (msg.content.type === 'Text') {
      console.log(`[${msg.senderId}]: ${msg.content.text}`);
    }
  }

  // Disconnect
  await alice.disconnect();
  console.log('Disconnected');
}

basicChat().catch(console.error);
```

#### Example 2: Relay Node Operation

```typescript
import { RelayNode } from '@dchat/sdk';

async function runRelay() {
  // Create relay with custom config
  const relay = RelayNode.withConfig({
    name: 'MyRelayNode',
    listenAddr: '0.0.0.0',
    listenPort: 9000,
    stakingEnabled: true,
    minUptimePercent: 99.0,
  });

  console.log('Starting relay node...');
  await relay.start();
  console.log('Relay is running');

  // Monitor statistics every 10 seconds
  const interval = setInterval(async () => {
    const stats = await relay.getStats();
    console.log(`
      Peers: ${stats.connectedPeers}
      Relayed: ${stats.messagesRelayed}
      Uptime: ${stats.uptimePercent.toFixed(2)}%
      Reputation: ${stats.reputationScore}
    `);
  }, 10000);

  // Graceful shutdown on SIGINT
  process.on('SIGINT', async () => {
    clearInterval(interval);
    console.log('\nShutting down relay...');
    await relay.stop();
    console.log('Relay stopped');
    process.exit(0);
  });
}

runRelay().catch(console.error);
```

#### Example 3: Multi-User Chat Room

```typescript
import { Client } from '@dchat/sdk';

async function chatRoom() {
  // Create multiple clients
  const alice = await Client.builder().name('Alice').listenPort(9001).build();
  const bob = await Client.builder().name('Bob').listenPort(9002).build();
  const charlie = await Client.builder().name('Charlie').listenPort(9003).build();

  // Connect all clients
  await Promise.all([
    alice.connect(),
    bob.connect(),
    charlie.connect(),
  ]);

  console.log('All users connected');

  // Alice sends a message
  await alice.sendMessage('Hi everyone!');

  // Wait for message propagation
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Bob and Charlie receive messages
  const bobMessages = await bob.receiveMessages();
  const charlieMessages = await charlie.receiveMessages();

  console.log(`Bob received ${bobMessages.length} messages`);
  console.log(`Charlie received ${charlieMessages.length} messages`);

  // Cleanup
  await Promise.all([
    alice.disconnect(),
    bob.disconnect(),
    charlie.disconnect(),
  ]);
}

chatRoom().catch(console.error);
```

---

### TypeScript Type Definitions

The SDK is fully typed with TypeScript. Import types as needed:

```typescript
import {
  Client,
  ClientBuilder,
  RelayNode,
  ClientConfig,
  RelayConfig,
  Identity,
  Message,
  MessageContent,
  MessageStatus,
  RelayStats,
  SdkError,
  ErrorCode,
} from '@dchat/sdk';
```

---

### SDK Development

#### Building the SDK

```bash
# Install dependencies
npm install

# Build TypeScript to JavaScript
npm run build

# Output: dist/index.js and dist/index.d.ts
```

#### Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run specific test
npm test -- client.test.ts
```

#### Linting and Formatting

```bash
# Lint TypeScript files
npm run lint

# Format with Prettier
npm run format
```

#### Publishing

```bash
# Prepare for publishing
npm run prepublishOnly

# Publish to npm
npm publish
```

## Error Handling

dchat uses a unified error type:

```rust
use dchat::prelude::{Error, Result};

fn example() -> Result<()> {
    match risky_operation() {
        Ok(value) => Ok(value),
        Err(Error::Network(e)) => {
            // Handle network error
            Err(e.into())
        }
        Err(Error::Crypto(e)) => {
            // Handle crypto error
            Err(e.into())
        }
        Err(e) => Err(e),
    }
}
```

## Configuration

### Environment Variables

- `DCHAT_DATA_DIR`: Data directory (default: `~/.dchat`)
- `DCHAT_LOG_LEVEL`: Log level (debug, info, warn, error)
- `DCHAT_BOOTSTRAP_PEERS`: Comma-separated bootstrap peer addresses
- `DCHAT_RELAY_MODE`: Enable relay mode (`true`/`false`)
- `DCHAT_CHAIN_RPC`: Chat chain RPC endpoint
- `DCHAT_CURRENCY_RPC`: Currency chain RPC endpoint

### Config File

```toml
# config.toml
[network]
listen_address = "/ip4/0.0.0.0/tcp/7070"
bootstrap_peers = [
    "/ip4/1.2.3.4/tcp/7070/p2p/12D3KooW..."
]

[identity]
derivation_path = "m/44'/0'/0'/0/0"
multi_device_sync = true

[storage]
database_path = "data/dchat.db"
backup_enabled = true
backup_interval_hours = 24

[crypto]
rotation_max_messages = 1000
rotation_max_age_seconds = 86400
post_quantum_enabled = true

[governance]
voting_enabled = true
moderation_enabled = true
jury_size = 7
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p dchat-crypto

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration_tests

# End-to-end tests
cargo test --test e2e_tests
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench crypto_performance
```

### Load Testing

```bash
# k6 load testing
k6 run tests/load/relay_stress_test.js

# Locust load testing
pip install locust
locust -f tests/load/locustfile.py --host=http://localhost:7071
```

### Fuzz Testing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzz target
cargo fuzz run noise_handshake
cargo fuzz run message_parsing
cargo fuzz run keypair_generation
```

## Deployment

### Docker

```bash
# Build image
docker build -t dchat:latest .

# Run single node
docker run -p 7070:7070 dchat:latest

# Run with docker-compose (3 relays + monitoring)
docker-compose up -d
```

### Monitoring

- Prometheus: http://localhost:9093
- Grafana: http://localhost:3000 (admin/admin)
- Jaeger: http://localhost:16686

## Security

### Threat Model

See `SECURITY.md` and `ARCHITECTURE.md` for detailed threat model and mitigations.

### Responsible Disclosure

Report security vulnerabilities to security@dchat.network with PGP key available at https://dchat.network/security.asc

### Audits

- Phase 7 Sprint 4 Security Audit: `PHASE7_SPRINT4_SECURITY_AUDIT.md`
- Phase 5 Security Audit: `PHASE5_SECURITY_AUDIT.md`

## Contributing

See `CONTRIBUTING.md` for development guidelines.

## License

MIT OR Apache-2.0

## Resources

- **Website**: https://dchat.network
- **Documentation**: https://docs.dchat.network
- **GitHub**: https://github.com/dchat/dchat
- **Discord**: https://discord.gg/dchat
- **Forum**: https://forum.dchat.network
