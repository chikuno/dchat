# dchat Flutter/Dart SDK

Official Flutter/Dart SDK for dchat - a decentralized chat application with blockchain-enforced message ordering and end-to-end encryption.

## Features

- ✅ **Blockchain Integration**: All operations create on-chain transactions
- ✅ **User Management**: Create users with Ed25519 keypairs
- ✅ **Direct Messaging**: Send encrypted direct messages with SHA-256 content hashing
- ✅ **Channel Operations**: Create and post to channels
- ✅ **Transaction Confirmation**: Wait for blockchain confirmations with configurable blocks
- ✅ **Real-time Updates**: WebSocket subscriptions for transaction confirmations
- ✅ **Cryptographic Utilities**: Ed25519 signing, SHA-256 hashing, key management
- ✅ **P2P Messaging**: End-to-end encrypted messaging with Noise Protocol
- ✅ **DHT Routing**: Decentralized peer discovery and routing (Kademlia)
- ✅ **Proof of Delivery**: Cryptographic delivery proofs with on-chain anchoring
- ✅ **Peer Management**: Trust scoring, connection management, peer statistics
- ✅ **Message Encryption**: ChaCha20-Poly1305 AEAD with key rotation

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  dchat_sdk:
    path: ../path/to/sdk/dart
```

Or from pub.dev (once published):

```yaml
dependencies:
  dchat_sdk: ^0.1.0
```

## Quick Start

```dart
import 'package:dchat_sdk/dchat.dart';

void main() async {
  // Initialize blockchain client
  final blockchain = BlockchainClient(
    config: BlockchainConfig.local(),
  );

  // Create user manager
  final userManager = UserManager(
    blockchain: blockchain,
    baseUrl: 'http://localhost:8080',
  );

  // Create user
  final user = await userManager.createUser('alice');
  print('User created: ${user.userId}');
  print('On-chain confirmed: ${user.onChainConfirmed}');
  print('Transaction ID: ${user.txId}');

  // Cleanup
  blockchain.dispose();
}
```

## Documentation

See [USAGE.md](USAGE.md) for comprehensive examples and API documentation.

For P2P messaging, see [MESSAGING_MODULE.md](MESSAGING_MODULE.md) for detailed documentation on:
- End-to-end encryption
- DHT-based peer discovery
- Proof of delivery tracking
- Peer trust management
- Complete usage examples

## Core Concepts

### Blockchain-First Architecture

Every operation creates a blockchain transaction:
- **User Registration**: RegisterUserTx with Ed25519 public key
- **Direct Messages**: SendDirectMessageTx with SHA-256 content hash
- **Channel Creation**: CreateChannelTx with visibility settings
- **Channel Posts**: PostToChannelTx with content hash

### Transaction Lifecycle

1. **Submit**: Transaction is submitted to blockchain RPC
2. **Pending**: Transaction is in mempool, awaiting inclusion
3. **Confirmed**: Transaction included in block with required confirmations
4. **Failed/TimedOut**: Transaction rejected or timed out

### On-Chain Confirmation

The `onChainConfirmed` field reflects actual blockchain state:
- `true`: Transaction confirmed with required confirmations (default: 6 blocks)
- `false`: Transaction pending or failed

## API Overview

### BlockchainClient

```dart
final client = BlockchainClient(config: BlockchainConfig.local());

// Register user
final txId = await client.registerUser(
  userId: 'user-id',
  username: 'alice',
  publicKey: 'ed25519-public-key-hex',
);

// Wait for confirmation
final receipt = await client.waitForConfirmation(txId);
print('Confirmed: ${receipt.success}');
```

### UserManager

```dart
final manager = UserManager(
  blockchain: client,
  baseUrl: 'http://localhost:8080',
);

// Create user
final user = await manager.createUser('alice');

// Send message
final message = await manager.sendDirectMessage(
  senderId: user.userId,
  recipientId: 'recipient-id',
  content: 'Hello!',
);

// Create channel
final channel = await manager.createChannel(
  creatorId: user.userId,
  channelName: 'General',
);

// Post to channel
final post = await manager.postToChannel(
  senderId: user.userId,
  channelId: channel.channelId,
  content: 'Welcome!',
);
```

### Cryptographic Utilities

```dart
// Generate key pair
final keyPair = KeyPair.generate();
print('Public: ${keyPair.publicKeyHex}');
print('Private: ${keyPair.privateKeyHex}');

// Sign message
final signature = keyPair.sign(messageBytes);

// Verify signature
final isValid = keyPair.verify(messageBytes, signature);

// Hash content
final hash = hashContent('message content');
```

## Configuration

### Local Development

```dart
BlockchainConfig.local() // Default: localhost:8545
```

### Custom Configuration

```dart
BlockchainConfig(
  rpcUrl: 'https://rpc.dchat.io',
  wsUrl: 'wss://ws.dchat.io',
  confirmationBlocks: 6,
  confirmationTimeout: Duration(seconds: 300),
  maxRetries: 3,
)
```

## Examples

See [USAGE.md](USAGE.md) for complete examples including:
- User creation and registration
- Direct messaging with encryption
- Channel creation and posting
- Transaction confirmation tracking
- WebSocket subscriptions
- Error handling patterns

## Requirements

- Dart SDK: >=3.0.0 <4.0.0
- Flutter: >=3.0.0 (for Flutter apps)

## Dependencies

- `http`: ^1.1.0 - HTTP client for RPC calls
- `web_socket_channel`: ^2.4.0 - WebSocket for real-time updates
- `crypto`: ^3.0.3 - SHA-256 hashing
- `uuid`: ^4.0.0 - UUID generation
- `ed25519_edwards`: ^0.3.1 - Ed25519 cryptography
- `pointycastle`: ^3.7.3 - Additional crypto primitives

## Project Structure

```
sdk/dart/
├── lib/
│   ├── dchat.dart                    # Main export file
│   └── src/
│       ├── blockchain/
│       │   ├── client.dart           # Blockchain client
│       │   └── transaction.dart      # Transaction types
│       ├── user/
│       │   ├── manager.dart          # User management
│       │   └── models.dart           # User models
│       ├── crypto/
│       │   └── keypair.dart          # Cryptographic utilities
│       └── messaging/
│           ├── crypto.dart           # Noise Protocol encryption
│           ├── dht.dart              # Kademlia DHT routing
│           ├── peer_manager.dart     # Peer connection management
│           ├── proof_of_delivery.dart # Delivery proofs
│           ├── message_manager.dart  # High-level message API
│           └── mod.dart              # Module exports
├── example/
│   └── messaging_example.dart        # P2P messaging examples
├── pubspec.yaml                      # Package configuration
├── README.md                         # This file
├── USAGE.md                          # Usage examples
└── MESSAGING_MODULE.md               # P2P messaging documentation
```

## Development Status

**Current Version**: 0.1.0 (Alpha)

### Implemented ✅

- Blockchain client with transaction submission
- User registration with Ed25519 keypairs
- Direct messaging with content hashing
- Channel creation and posting
- Transaction confirmation polling
- WebSocket subscriptions
- Cryptographic utilities
- **P2P messaging with Noise Protocol encryption**
- **DHT-based peer discovery and routing**
- **Proof of delivery tracking with on-chain anchoring**
- **Peer connection management with trust scoring**
- **Message encryption with ChaCha20-Poly1305**

### Planned 🚧

- Message retrieval APIs (getDirectMessages, getChannelMessages)
- User profile queries
- Offline message queuing
- Multi-device synchronization
- Integration with backend APIs
- Unit and integration tests

## Testing

```bash
cd sdk/dart
dart test
```

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) in the main repository.

## License

See [LICENSE](../../LICENSE) in the main repository.

## Support

- Documentation: See [ARCHITECTURE.md](../../ARCHITECTURE.md)
- Issues: [GitHub Issues](https://github.com/dchat/dchat/issues)
- Discussions: [GitHub Discussions](https://github.com/dchat/dchat/discussions)

## Related

- [TypeScript SDK](../typescript/) - SDK for web applications
- [Rust SDK](../rust/) - SDK for Rust applications
- [Backend API](../../src/) - Core Rust implementation
- [Architecture](../../ARCHITECTURE.md) - System design documentation
