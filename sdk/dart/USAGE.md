# dchat Flutter/Dart SDK Example

This example demonstrates how to use the dchat SDK for blockchain-integrated chat operations.

## Installation

Add the SDK to your `pubspec.yaml`:

```yaml
dependencies:
  dchat_sdk:
    path: ../path/to/sdk/dart
```

## Usage

### Initialize Blockchain Client

```dart
import 'package:dchat_sdk/dchat.dart';

void main() async {
  // Create blockchain client
  final blockchain = BlockchainClient(
    config: BlockchainConfig(
      rpcUrl: 'http://localhost:8545',
      wsUrl: 'ws://localhost:8546',
      confirmationBlocks: 6,
      confirmationTimeout: Duration(seconds: 300),
    ),
  );

  // Create user manager
  final userManager = UserManager(
    blockchain: blockchain,
    baseUrl: 'http://localhost:8080',
  );
}
```

### Create a User

```dart
// Create new user with automatic blockchain registration
final userResponse = await userManager.createUser('alice');

print('User ID: ${userResponse.userId}');
print('Username: ${userResponse.username}');
print('Public Key: ${userResponse.publicKey}');
print('On-chain Confirmed: ${userResponse.onChainConfirmed}');
print('Transaction ID: ${userResponse.txId}');

// Save the private key securely!
print('Private Key: ${userResponse.privateKey}');
```

### Send Direct Message

```dart
final messageResponse = await userManager.sendDirectMessage(
  senderId: aliceUserId,
  recipientId: bobUserId,
  content: 'Hello Bob!',
  relayNodeId: 'relay-node-1', // Optional
);

print('Message ID: ${messageResponse.messageId}');
print('Content Hash: ${messageResponse.contentHash}');
print('On-chain Confirmed: ${messageResponse.onChainConfirmed}');
print('Transaction ID: ${messageResponse.txId}');
```

### Create Channel

```dart
final channelResponse = await userManager.createChannel(
  creatorId: aliceUserId,
  channelName: 'General Chat',
  description: 'A general discussion channel',
);

print('Channel ID: ${channelResponse.channelId}');
print('Channel Name: ${channelResponse.name}');
print('On-chain Confirmed: ${channelResponse.onChainConfirmed}');
print('Transaction ID: ${channelResponse.txId}');
```

### Post to Channel

```dart
final postResponse = await userManager.postToChannel(
  senderId: aliceUserId,
  channelId: channelId,
  content: 'Welcome to the channel!',
);

print('Message ID: ${postResponse.messageId}');
print('Content Hash: ${postResponse.contentHash}');
print('On-chain Confirmed: ${postResponse.onChainConfirmed}');
print('Transaction ID: ${postResponse.txId}');
```

### Wait for Transaction Confirmation

```dart
// Submit transaction
final txId = await blockchain.registerUser(
  userId: userId,
  username: 'charlie',
  publicKey: publicKeyHex,
);

// Wait for confirmation (blocking)
try {
  final receipt = await blockchain.waitForConfirmation(txId);
  
  if (receipt.success) {
    print('Transaction confirmed!');
    print('Block Height: ${receipt.blockHeight}');
    print('Block Hash: ${receipt.blockHash}');
  } else {
    print('Transaction failed: ${receipt.error}');
  }
} on TimeoutException {
  print('Transaction timed out after 5 minutes');
}
```

### Subscribe to Confirmations (WebSocket)

```dart
// Submit transaction
final txId = await blockchain.createChannel(
  channelId: channelId,
  name: 'Real-time Channel',
  description: 'Channel with WebSocket updates',
  creatorId: userId,
);

// Subscribe to real-time updates
blockchain.subscribeToConfirmations(txId).listen(
  (receipt) {
    if (receipt.success) {
      print('Transaction confirmed in real-time!');
      print('Block: ${receipt.blockHeight}');
    } else {
      print('Transaction failed: ${receipt.error}');
    }
  },
  onError: (error) {
    print('WebSocket error: $error');
  },
);
```

### Check Transaction Status

```dart
// Check if transaction is confirmed
final isConfirmed = await blockchain.isTransactionConfirmed(txId);

if (isConfirmed) {
  print('Transaction is confirmed on-chain');
} else {
  print('Transaction pending or failed');
}

// Get detailed receipt
final receipt = await blockchain.getTransactionReceipt(txId);
if (receipt != null) {
  print('Status: ${receipt.success ? "Success" : "Failed"}');
  print('Block Height: ${receipt.blockHeight}');
}
```

### Cryptographic Operations

```dart
import 'package:dchat_sdk/dchat.dart';

// Generate new key pair
final keyPair = KeyPair.generate();
print('Public Key: ${keyPair.publicKeyHex}');
print('Private Key: ${keyPair.privateKeyHex}');

// Sign a message
final message = 'Hello, blockchain!';
final messageBytes = utf8.encode(message);
final signature = keyPair.sign(Uint8List.fromList(messageBytes));

// Verify signature
final isValid = keyPair.verify(
  Uint8List.fromList(messageBytes),
  signature,
);
print('Signature valid: $isValid');

// Hash content
final content = 'Message content';
final hash = hashContent(content);
print('Content hash: $hash');
```

### Error Handling

```dart
try {
  final user = await userManager.createUser('alice');
  print('User created: ${user.userId}');
} on TimeoutException catch (e) {
  print('Blockchain confirmation timed out: $e');
} on Exception catch (e) {
  print('Error creating user: $e');
}
```

### Complete Example

```dart
import 'package:dchat_sdk/dchat.dart';

Future<void> main() async {
  // Initialize SDK
  final blockchain = BlockchainClient(
    config: BlockchainConfig.local(),
  );

  final userManager = UserManager(
    blockchain: blockchain,
    baseUrl: 'http://localhost:8080',
  );

  try {
    // Create two users
    print('Creating users...');
    final alice = await userManager.createUser('alice');
    final bob = await userManager.createUser('bob');

    print('Alice: ${alice.userId} (confirmed: ${alice.onChainConfirmed})');
    print('Bob: ${bob.userId} (confirmed: ${bob.onChainConfirmed})');

    // Send direct message
    print('\nSending direct message...');
    final message = await userManager.sendDirectMessage(
      senderId: alice.userId,
      recipientId: bob.userId,
      content: 'Hello Bob!',
    );
    print('Message sent: ${message.messageId} (TX: ${message.txId})');

    // Create channel
    print('\nCreating channel...');
    final channel = await userManager.createChannel(
      creatorId: alice.userId,
      channelName: 'General',
      description: 'General discussion',
    );
    print('Channel created: ${channel.channelId} (TX: ${channel.txId})');

    // Post to channel
    print('\nPosting to channel...');
    final post = await userManager.postToChannel(
      senderId: alice.userId,
      channelId: channel.channelId,
      content: 'Welcome everyone!',
    );
    print('Posted: ${post.messageId} (TX: ${post.txId})');

    // Check current block
    final blockNumber = await blockchain.getBlockNumber();
    print('\nCurrent block: $blockNumber');

  } catch (e) {
    print('Error: $e');
  } finally {
    blockchain.dispose();
  }
}
```

## Key Concepts

### On-Chain Confirmation

All operations (user registration, messages, channels) create blockchain transactions. The `onChainConfirmed` field indicates whether the transaction has been confirmed on-chain with the required number of confirmations (default: 6 blocks).

### Transaction IDs

Every operation returns a `txId` (transaction ID) that can be used to:
- Query transaction status
- Wait for confirmations
- Verify on blockchain explorers
- Provide audit trails

### Content Hashing

Message content is hashed using SHA-256 before submission to the blockchain. This ensures:
- Content integrity verification
- Tamper detection
- Efficient on-chain storage

### Asynchronous Operations

All blockchain operations are asynchronous and may take time:
- User registration: ~6-60 seconds (6 confirmations)
- Messages: ~6-60 seconds
- Channel operations: ~6-60 seconds

Use `await` for blocking behavior or handle with `.then()` for non-blocking.

## Configuration

### Local Development

```dart
final config = BlockchainConfig(
  rpcUrl: 'http://localhost:8545',
  wsUrl: 'ws://localhost:8546',
  confirmationBlocks: 1, // Fast for testing
  confirmationTimeout: Duration(seconds: 60),
);
```

### Production

```dart
final config = BlockchainConfig(
  rpcUrl: 'https://mainnet.dchat.io/rpc',
  wsUrl: 'wss://mainnet.dchat.io/ws',
  confirmationBlocks: 6, // Secure for production
  confirmationTimeout: Duration(seconds: 300),
  maxRetries: 3,
);
```

## Next Steps

- Implement message retrieval APIs
- Add WebSocket subscriptions for real-time updates
- Integrate with Flutter UI components
- Add offline message queuing
- Implement multi-device synchronization
