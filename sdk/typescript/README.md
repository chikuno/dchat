# dchat TypeScript/JavaScript SDK

High-level API for building decentralized chat applications with dchat.

## Installation

```bash
npm install @dchat/sdk
```

## Quick Start

### Client Usage

```typescript
import { Client } from '@dchat/sdk';

// Create and connect a client
const client = await Client.builder()
  .name('Alice')
  .build();

await client.connect();

// Send a message
await client.sendMessage('Hello, dchat!');

// Receive messages
const messages = await client.receiveMessages();
console.log('Received', messages.length, 'messages');

// Disconnect
await client.disconnect();
```

### Relay Node

```typescript
import { RelayNode } from '@dchat/sdk';

// Create and start a relay
const relay = RelayNode.withConfig({
  name: 'MyRelay',
  listenPort: 9000,
  stakingEnabled: true,
});

await relay.start();

// Get statistics
const stats = await relay.getStats();
console.log('Connected peers:', stats.connectedPeers);

await relay.stop();
```

## API Reference

### Client

- `Client.builder()` - Create a new client builder
- `connect()` - Connect to the dchat network
- `disconnect()` - Disconnect from the network
- `sendMessage(text)` - Send a text message
- `receiveMessages()` - Fetch messages
- `getIdentity()` - Get client identity
- `getConfig()` - Get client configuration

### RelayNode

- `RelayNode.create()` - Create relay with default config
- `RelayNode.withConfig(config)` - Create relay with custom config
- `start()` - Start the relay node
- `stop()` - Stop the relay node
- `isRunning()` - Check if relay is running
- `getStats()` - Get relay statistics
- `getConfig()` - Get relay configuration

## Examples

See the `examples/` directory for complete usage examples:

- `basic-chat.ts` - Simple chat client
- `relay-node.ts` - Relay node operation

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Lint
npm run lint
```

## License

MIT OR Apache-2.0
