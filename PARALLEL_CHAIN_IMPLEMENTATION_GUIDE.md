# Parallel Chain Implementation Guide

## Quick Start

### Using Dart SDK

```dart
import 'package:dchat_sdk/blockchain/chat_chain_client.dart';
import 'package:dchat_sdk/blockchain/currency_chain_client.dart';
import 'package:dchat_sdk/blockchain/cross_chain_bridge.dart';

void main() async {
  // Initialize clients
  final chatChain = ChatChainClient(rpcUrl: 'http://localhost:8545');
  final currencyChain = CurrencyChainClient(rpcUrl: 'http://localhost:8546');
  final bridge = CrossChainBridge(
    chatChain: chatChain,
    currencyChain: currencyChain,
    bridgeRpcUrl: 'http://localhost:8548',
  );

  // Register user with stake atomically
  final bridgeTx = await bridge.registerUserWithStake(
    'alice',
    [1, 2, 3, 4, 5], // public key
    1000, // stake amount
  );

  print('Bridge TX ID: ${bridgeTx.id}');
  print('Status: ${bridgeTx.status}');
}
```

### Using TypeScript SDK

```typescript
import { ChatChainClient, CurrencyChainClient, CrossChainBridge } from 'dchat-sdk';

async function main() {
  const chatChain = new ChatChainClient();
  const currencyChain = new CurrencyChainClient();
  const bridge = new CrossChainBridge(chatChain, currencyChain);

  // Create channel with fee
  const bridgeTx = await bridge.createChannelWithFee(
    'alice',
    'general',
    100 // creation fee
  );

  // Wait for atomic completion
  await bridge.waitForAtomicCompletion(bridgeTx.id);
  console.log('Channel created successfully!');
}

main().catch(console.error);
```

### Using Python SDK

```python
import asyncio
from dchat.blockchain import ChatChainClient, CurrencyChainClient, CrossChainBridge

async def main():
    # Initialize clients
    chat_chain = ChatChainClient()
    currency_chain = CurrencyChainClient()
    bridge = CrossChainBridge(chat_chain, currency_chain)

    # Register user with stake
    bridge_tx = await bridge.register_user_with_stake(
        'alice',
        b'\x01\x02\x03\x04\x05', # public key
        1000 # stake amount
    )

    print(f"Bridge TX ID: {bridge_tx.id}")
    print(f"Status: {bridge_tx.status}")

    # Wait for completion
    await bridge.wait_for_atomic_completion(bridge_tx.id)

asyncio.run(main())
```

### Using Rust SDK

```rust
use dchat::blockchain::{
    ChatChainClient, CurrencyChainClient, CrossChainBridge,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize clients
    let chat_chain = Arc::new(ChatChainClient::new(
        "http://localhost:8545".to_string()
    ));
    let currency_chain = Arc::new(CurrencyChainClient::new(
        "http://localhost:8546".to_string()
    ));
    let bridge = CrossChainBridge::new(
        chat_chain,
        currency_chain,
        "http://localhost:8548".to_string(),
    );

    // Register user with stake
    let bridge_tx_id = bridge.register_user_with_stake(
        "alice",
        vec![1, 2, 3, 4, 5],
        1000,
    ).await?;

    println!("Bridge TX ID: {}", bridge_tx_id);

    // Get status
    let status = bridge.get_status(&bridge_tx_id).await?;
    println!("Status: {:?}", status);

    Ok(())
}
```

## Architecture Decisions

### Why Parallel Chains?

1. **Separation of Concerns**: Chat operations (identity, messaging) isolated from economics (payments, staking)
2. **Independent Scaling**: Each chain optimizes for its specific workload
3. **Security Isolation**: Compromise of one chain doesn't immediately affect the other
4. **Economic Flexibility**: Token economics managed separately from chat functionality
5. **Governance Separation**: Chat governance can evolve independently from economic governance

### Why Cross-Chain Bridge?

1. **Atomic Operations**: Some operations require both chains (e.g., create channel with fee)
2. **Consistent State**: Ensures both chains stay in sync for critical operations
3. **Rollback Safety**: Failed cross-chain operations can be rolled back
4. **User Experience**: Seamless atomic operations from user perspective

### Transaction Confirmation Strategy

- **6 Block Confirmations**: Provides strong finality guarantee (~60-120 seconds)
- **Polling Mechanism**: Regular status checks from SDK without heavy load
- **Timeout Handling**: 2-minute max wait before failure
- **Both Chains Required**: Cross-chain operations wait for both chains independently

## Common Patterns

### Pattern 1: User Onboarding

```
1. Create wallet on currency chain
2. Register identity on chat chain
3. Stake initial tokens
4. Wait for both confirmations
5. User can now use chat and participate in economics
```

**Implementation:**
```typescript
// All SDKs provide registerUserWithStake() for this pattern
const bridgeTx = await bridge.registerUserWithStake(userId, publicKey, stakeAmount);
await bridge.waitForAtomicCompletion(bridgeTx.id);
```

### Pattern 2: Channel Creation with Economic Incentive

```
1. Transfer channel creation fee
2. Create channel on chat chain
3. Track both transactions
4. Await confirmations
5. Channel ready with treasury payment
```

**Implementation:**
```typescript
const bridgeTx = await bridge.createChannelWithFee(owner, channelName, fee);
await bridge.waitForAtomicCompletion(bridgeTx.id);
```

### Pattern 3: Message with Proof-of-Delivery Reward

```
1. Send direct message on chat chain
2. Relay captures message hash
3. Relay submits proof-of-delivery on currency chain
4. Sender verified on chat chain
5. Relay receives reward on currency chain
```

**Implementation:**
```dart
// Send message
final chatTx = await chatChain.sendDirectMessage(sender, recipient, messageId);
await chatChain.waitForConfirmation(chatTx.id);

// Later, relay submits proof
final currencyTx = await currencyChain.claimRewards(relayNodeId);
```

### Pattern 4: Reputation-Based Access Control

```
1. User performs positive actions → reputation increases
2. Access to channels gated by reputation threshold
3. Reputation tracked on chat chain
4. Access decisions made locally with on-chain verification
```

**Implementation:**
```python
reputation = await chat_chain.get_reputation(user_id)
if reputation >= CHANNEL_REPUTATION_REQUIREMENT:
    await chat_chain.post_to_channel(user_id, channel_id, message_id)
```

## Error Handling

### Handling Chat Chain Errors

```typescript
try {
  await chatChain.registerUser(userId, publicKey);
} catch (error) {
  if (error.code === 'INVALID_IDENTITY') {
    console.error('User not registered');
  } else if (error.code === 'INSUFFICIENT_PERMISSIONS') {
    console.error('No permission for this operation');
  } else {
    console.error('Chat chain error:', error);
  }
}
```

### Handling Currency Chain Errors

```python
try:
    await currency_chain.transfer(from_user, to_user, amount)
except RuntimeError as e:
    if 'insufficient_balance' in str(e):
        print("Not enough tokens for transfer")
    elif 'invalid_wallet' in str(e):
        print("Wallet not found")
    else:
        print(f"Currency chain error: {e}")
```

### Handling Cross-Chain Errors

```rust
match bridge.register_user_with_stake(user_id, public_key, stake).await {
    Ok(bridge_tx_id) => {
        match bridge.wait_for_confirmations(&bridge_tx_id, 6).await {
            Ok(()) => println!("User registered with stake"),
            Err(e) => eprintln!("Cross-chain confirmation failed: {}", e),
        }
    }
    Err(e) => eprintln!("Cross-chain operation failed: {}", e),
}
```

## Testing Strategies

### Unit Testing Individual Chains

```rust
#[tokio::test]
async fn test_chat_chain_reputation() {
    let client = ChatChainClient::new("http://localhost:8545".to_string());
    
    // Register user
    client.register_user("alice", vec![1, 2, 3]).await.unwrap();
    
    // Update reputation
    client.update_reputation("alice", 10).await.unwrap();
    
    // Verify
    let rep = client.get_reputation("alice").await.unwrap();
    assert_eq!(rep, 10);
}
```

### Integration Testing Cross-Chain Operations

```typescript
describe('CrossChainBridge', () => {
  it('should register user with stake atomically', async () => {
    const bridgeTx = await bridge.registerUserWithStake(
      'alice',
      Buffer.from([1, 2, 3]),
      1000
    );

    // Verify both chains updated
    const wallet = await currencyChain.getWallet('alice');
    expect(wallet.staked).toBe(1000);

    const reputation = await chatChain.getReputation('alice');
    expect(reputation).toBeGreaterThan(0);
  });
});
```

### Stress Testing

```python
async def test_concurrent_registrations(num_users: int = 100):
    """Test multiple concurrent user registrations"""
    tasks = [
        bridge.register_user_with_stake(
            f"user{i}",
            b"public_key",
            1000
        )
        for i in range(num_users)
    ]
    
    results = await asyncio.gather(*tasks)
    assert len(results) == num_users
```

## Performance Optimization

### Batch Operations

```typescript
// Instead of sequential calls
for (const userId of userIds) {
  await chatChain.registerUser(userId, publicKey);
}

// Use batch operations when available
const results = await Promise.all(
  userIds.map(userId => chatChain.registerUser(userId, publicKey))
);
```

### Caching Reputation Scores

```python
class CachedChatChainClient:
    def __init__(self, base_client: ChatChainClient):
        self.base_client = base_client
        self.reputation_cache = {}
        self.cache_timeout = 3600  # 1 hour
    
    async def get_reputation(self, user_id: str) -> int:
        if user_id in self.reputation_cache:
            return self.reputation_cache[user_id]
        
        rep = await self.base_client.get_reputation(user_id)
        self.reputation_cache[user_id] = rep
        return rep
```

### Monitoring Pending Transactions

```rust
pub async fn monitor_pending_transactions(bridge: Arc<CrossChainBridge>) {
    loop {
        if let Err(e) = bridge.finalize_pending_transactions().await {
            eprintln!("Failed to finalize pending transactions: {}", e);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
```

## Deployment

### Local Development Setup

```bash
# Terminal 1: Chat Chain RPC
cargo run --release -- --role relay --chain chat --port 8545

# Terminal 2: Currency Chain RPC
cargo run --release -- --role relay --chain currency --port 8546

# Terminal 3: Bridge Service
cargo run --release -- --role bridge --port 8548

# Terminal 4: Client Application
cargo run --example parallel_chains
```

### Docker Deployment

```yaml
version: '3'
services:
  chat-chain:
    image: dchat:latest
    environment:
      - ROLE=relay
      - CHAIN=chat
      - PORT=8545
    ports:
      - "8545:8545"

  currency-chain:
    image: dchat:latest
    environment:
      - ROLE=relay
      - CHAIN=currency
      - PORT=8546
    ports:
      - "8546:8546"

  bridge:
    image: dchat:latest
    environment:
      - ROLE=bridge
      - PORT=8548
    ports:
      - "8548:8548"
    depends_on:
      - chat-chain
      - currency-chain
```

## Troubleshooting

### Transaction Stuck in Pending

**Symptom**: Transaction remains in `Pending` status after 2 minutes

**Solutions**:
1. Check that both chain services are running
2. Verify RPC endpoints are accessible
3. Check network connectivity
4. Review bridge logs for errors

```bash
# Check Chat Chain
curl http://localhost:8545/health

# Check Currency Chain
curl http://localhost:8546/health

# Check Bridge
curl http://localhost:8548/health
```

### Confirmation Timeout

**Symptom**: `CrossChainConfirmationTimeout` error

**Solutions**:
1. Increase polling timeout (default 60s per confirmation)
2. Reduce confirmation requirement (if security allows)
3. Check block production on chains

```typescript
// Increase timeout
await bridge.waitForAtomicCompletion(bridgeTx.id, 120000); // 2 minutes
```

### Inconsistent State

**Symptom**: Chat chain shows transaction, currency chain doesn't

**Solutions**:
1. Enable bridge finalization monitoring
2. Use cross-chain transaction ID for validation
3. Check transaction logs on both chains

```rust
// Monitor and finalize pending
bridge.finalize_pending_transactions().await?;
```

## Best Practices

1. **Always use atomic operations for cross-chain**: Never split chat and currency operations
2. **Handle timeouts gracefully**: Implement exponential backoff for retries
3. **Cache reputation data**: Reduce load on chat chain
4. **Monitor bridge health**: Watch for pending transaction buildup
5. **Validate on both chains**: Don't assume one chain's state
6. **Use typed operations**: Leverage enums to prevent invalid state transitions
7. **Log transaction IDs**: Essential for debugging and auditing
8. **Test rollback scenarios**: Ensure error handling is robust

## Migration Path

For applications upgrading from monolithic chain:

1. **Phase 1**: Run both architectures in parallel
2. **Phase 2**: Gradually migrate users to new SDKs
3. **Phase 3**: Sync state between old and new chains
4. **Phase 4**: Cut over when all users migrated
5. **Phase 5**: Deprecate legacy chain
