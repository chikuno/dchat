# Parallel Chain SDK Reference

## Quick Reference for All 4 SDKs

### Initialization

#### Dart
```dart
import 'package:dchat_sdk/blockchain/chat_chain_client.dart';
import 'package:dchat_sdk/blockchain/currency_chain_client.dart';
import 'package:dchat_sdk/blockchain/cross_chain_bridge.dart';

final chatChain = ChatChainClient(rpcUrl: 'http://localhost:8545');
final currencyChain = CurrencyChainClient(rpcUrl: 'http://localhost:8546');
final bridge = CrossChainBridge(chatChain: chatChain, currencyChain: currencyChain);
```

#### TypeScript
```typescript
import { ChatChainClient, CurrencyChainClient, CrossChainBridge } from 'dchat-sdk';

const chatChain = new ChatChainClient();
const currencyChain = new CurrencyChainClient();
const bridge = new CrossChainBridge(chatChain, currencyChain);
```

#### Python
```python
from dchat.blockchain import ChatChainClient, CurrencyChainClient, CrossChainBridge

chat_chain = ChatChainClient()
currency_chain = CurrencyChainClient()
bridge = CrossChainBridge(chat_chain, currency_chain)
```

#### Rust
```rust
use dchat::blockchain::{ChatChainClient, CurrencyChainClient, CrossChainBridge};
use std::sync::Arc;

let chat_chain = Arc::new(ChatChainClient::new("http://localhost:8545".to_string()));
let currency_chain = Arc::new(CurrencyChainClient::new("http://localhost:8546".to_string()));
let bridge = CrossChainBridge::new(chat_chain, currency_chain, "http://localhost:8548".to_string());
```

## Chat Chain Operations

### Register User

#### Dart
```dart
final txId = await chatChain.registerUser('alice', publicKeyBytes);
final status = await chatChain.getReputation('alice');
```

#### TypeScript
```typescript
const tx = await chatChain.registerUser('alice', publicKey);
await chatChain.waitForConfirmation(tx.id);
const reputation = await chatChain.getReputation('alice');
```

#### Python
```python
tx = await chat_chain.register_user('alice', public_key)
await chat_chain.wait_for_confirmation(tx.id)
reputation = await chat_chain.get_reputation('alice')
```

#### Rust
```rust
let tx_id = chat_chain.register_user("alice", public_key)?;
let reputation = chat_chain.get_reputation("alice").await?;
```

### Create Channel

#### Dart
```dart
final txId = await chatChain.createChannel('owner', 'chan1', 'General');
final status = await chatChain.getUserTransactions('owner');
```

#### TypeScript
```typescript
const tx = await chatChain.createChannel('owner', 'chan1', 'General');
await chatChain.waitForConfirmation(tx.id);
```

#### Python
```python
tx = await chat_chain.create_channel('owner', 'chan1', 'General')
await chat_chain.wait_for_confirmation(tx.id)
```

#### Rust
```rust
let tx_id = chat_chain.create_channel("owner", "chan1", "General".to_string())?;
let tx = chat_chain.get_transaction(&tx_id).await?;
```

### Post to Channel

#### Dart
```dart
final txId = await chatChain.postToChannel('alice', 'chan1', 'msg123');
```

#### TypeScript
```typescript
const tx = await chatChain.postToChannel('alice', 'chan1', 'msg123');
```

#### Python
```python
tx = await chat_chain.post_to_channel('alice', 'chan1', 'msg123')
```

#### Rust
```rust
let tx_id = chat_chain.post_to_channel("alice", "chan1", "msg123")?;
```

## Currency Chain Operations

### Create Wallet

#### Dart
```dart
final wallet = await currencyChain.createWallet('alice', 10000);
print('Balance: ${wallet.balance}');
```

#### TypeScript
```typescript
const wallet = await currencyChain.createWallet('alice', 10000);
const balance = await currencyChain.getBalance('alice');
```

#### Python
```python
wallet = await currency_chain.create_wallet('alice', 10000)
balance = await currency_chain.get_balance('alice')
```

#### Rust
```rust
let wallet = currency_chain.create_wallet("alice", 10000).await?;
let balance = currency_chain.get_balance("alice").await?;
```

### Transfer Tokens

#### Dart
```dart
try {
  final txId = await currencyChain.transfer('alice', 'bob', 100);
  print('Transfer successful: $txId');
} catch (e) {
  print('Transfer failed: $e');
}
```

#### TypeScript
```typescript
try {
  const tx = await currencyChain.transfer('alice', 'bob', 100);
  await currencyChain.waitForConfirmation(tx.id);
} catch (error) {
  console.error('Transfer failed:', error);
}
```

#### Python
```python
try:
    tx = await currency_chain.transfer('alice', 'bob', 100)
    await currency_chain.wait_for_confirmation(tx.id)
except RuntimeError as e:
    print(f'Transfer failed: {e}')
```

#### Rust
```rust
match currency_chain.transfer("alice", "bob", 100).await {
    Ok(tx_id) => println!("Transfer: {}", tx_id),
    Err(e) => eprintln!("Failed: {}", e),
}
```

### Stake Tokens

#### Dart
```dart
final txId = await currencyChain.stake('alice', 1000, 2592000); // 30 days
final wallet = await currencyChain.getWallet('alice');
print('Staked: ${wallet.staked}');
```

#### TypeScript
```typescript
const tx = await currencyChain.stake('alice', 1000, 2592000);
const wallet = await currencyChain.getWallet('alice');
console.log('Staked:', wallet.staked);
```

#### Python
```python
tx = await currency_chain.stake('alice', 1000, 2592000)
wallet = await currency_chain.get_wallet('alice')
print(f"Staked: {wallet.staked}")
```

#### Rust
```rust
let tx_id = currency_chain.stake("alice", 1000, 2592000).await?;
let wallet = currency_chain.get_wallet("alice").await?;
println!("Staked: {}", wallet.staked);
```

### Claim Rewards

#### Dart
```dart
final txId = await currencyChain.claimRewards('alice');
final wallet = await currencyChain.getWallet('alice');
print('Rewards: ${wallet.rewardsPending}');
```

#### TypeScript
```typescript
const tx = await currencyChain.claimRewards('alice');
const wallet = await currencyChain.getWallet('alice');
```

#### Python
```python
tx = await currency_chain.claim_rewards('alice')
wallet = await currency_chain.get_wallet('alice')
```

#### Rust
```rust
let tx_id = currency_chain.claim_rewards("alice").await?;
```

## Cross-Chain Operations

### Register User with Stake (Atomic)

#### Dart
```dart
final bridgeTx = await bridge.registerUserWithStake(
  'alice',
  publicKeyBytes,
  1000, // stake amount
);
print('Bridge TX: ${bridgeTx.id}');
print('Status: ${bridgeTx.status}');
// Status progresses: Pending → ChatChainConfirmed → CurrencyChainConfirmed → AtomicSuccess
```

#### TypeScript
```typescript
const bridgeTx = await bridge.registerUserWithStake('alice', publicKey, 1000);
console.log('Status:', bridgeTx.status);
// Poll status
const status = await bridge.getStatus(bridgeTx.id);
console.log('Current:', status?.status);
```

#### Python
```python
bridge_tx = await bridge.register_user_with_stake('alice', public_key, 1000)
print(f'Bridge TX: {bridge_tx.id}')
status = await bridge.get_status(bridge_tx.id)
print(f'Status: {status.status}')
```

#### Rust
```rust
let bridge_tx_id = bridge.register_user_with_stake("alice", public_key, 1000).await?;
println!("Bridge TX: {}", bridge_tx_id);
// Monitor status
loop {
    if let Ok(Some(tx)) = bridge.get_status(&bridge_tx_id).await {
        println!("Status: {:?}", tx.status);
        if tx.status == CrossChainStatus::AtomicSuccess {
            break;
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

### Create Channel with Fee (Atomic)

#### Dart
```dart
final bridgeTx = await bridge.createChannelWithFee(
  'alice',
  'general',
  100, // creation fee
);
final finalStatus = await bridge.getStatus(bridgeTx.id);
```

#### TypeScript
```typescript
const tx = await bridge.createChannelWithFee('alice', 'general', 100);
await bridge.waitForAtomicCompletion(tx.id);
```

#### Python
```python
tx = await bridge.create_channel_with_fee('alice', 'general', 100)
await bridge.wait_for_atomic_completion(tx.id)
```

#### Rust
```rust
let tx_id = bridge.create_channel_with_fee("alice", "general", 100).await?;
bridge.wait_for_confirmations(&tx_id, 6).await?;
```

## Error Handling

### Chat Chain Errors

#### Dart
```dart
try {
  await chatChain.registerUser('alice', publicKey);
} catch (e) {
  print('Chat chain error: $e');
}
```

#### TypeScript
```typescript
try {
  await chatChain.registerUser('alice', publicKey);
} catch (error) {
  if (error.code === 'INVALID_IDENTITY') {
    // Handle identity error
  }
}
```

#### Python
```python
try:
    await chat_chain.register_user('alice', public_key)
except RuntimeError as e:
    if 'invalid' in str(e):
        # Handle error
        pass
```

#### Rust
```rust
match chat_chain.register_user("alice", public_key) {
    Ok(tx_id) => println!("Registered: {}", tx_id),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Currency Chain Errors

#### Dart
```dart
try {
  await currencyChain.transfer('alice', 'bob', 10000000); // More than balance
} catch (e) {
  if (e.toString().contains('insufficient')) {
    print('Not enough tokens');
  }
}
```

#### TypeScript
```typescript
try {
  await currencyChain.transfer('alice', 'bob', 10000000);
} catch (error) {
  if (error.message.includes('insufficient_balance')) {
    console.log('Insufficient balance');
  }
}
```

#### Python
```python
try:
    await currency_chain.transfer('alice', 'bob', 10000000)
except RuntimeError as e:
    if 'insufficient_balance' in str(e):
        print('Insufficient balance')
```

#### Rust
```rust
if let Err(e) = currency_chain.transfer("alice", "bob", 10000000).await {
    if e.contains("insufficient") {
        println!("Not enough balance");
    }
}
```

### Cross-Chain Errors

#### Dart
```dart
try {
  final tx = await bridge.registerUserWithStake('alice', publicKey, 1000);
  await bridge.waitForAtomicCompletion(tx.id);
} catch (e) {
  print('Atomic operation failed: $e');
}
```

#### TypeScript
```typescript
try {
  const tx = await bridge.registerUserWithStake('alice', publicKey, 1000);
  await bridge.waitForAtomicCompletion(tx.id);
} catch (error) {
  if (error.message.includes('timeout')) {
    console.log('Confirmation timeout');
  } else if (error.message.includes('rolled_back')) {
    console.log('Atomic operation failed - rolled back');
  }
}
```

#### Python
```python
try:
    tx = await bridge.register_user_with_stake('alice', public_key, 1000)
    await bridge.wait_for_atomic_completion(tx.id)
except RuntimeError as e:
    if 'timeout' in str(e):
        print('Confirmation timeout')
    elif 'rolled_back' in str(e):
        print('Operation rolled back')
```

#### Rust
```rust
match bridge.register_user_with_stake("alice", public_key, 1000).await {
    Ok(tx_id) => {
        match bridge.wait_for_confirmations(&tx_id, 6).await {
            Ok(()) => println!("Success"),
            Err(e) if e.contains("timeout") => println!("Timeout"),
            Err(e) => println!("Error: {}", e),
        }
    }
    Err(e) => println!("Failed: {}", e),
}
```

## Status Enum Reference

```
Pending
  ↓
(Wait for Chat Chain confirmations)
  ↓
ChatChainConfirmed
  ↓
(Wait for Currency Chain confirmations)
  ↓
CurrencyChainConfirmed
  ↓
AtomicSuccess ← Operation succeeded
  OR
RolledBack ← One chain failed, rolled back
  OR
Failed ← Permanent failure
```

## Configuration

### RPC Endpoints (Default)
- Chat Chain: `http://localhost:8545`
- Currency Chain: `http://localhost:8546`
- Bridge: `http://localhost:8548`

### Confirmation Settings
- Blocks to confirm: 6 (≈60-120 seconds)
- Confirmation timeout: 2 minutes
- Polling interval: 1 second

### Network Parameters
- Gas per transaction: 21,000 units
- Block time: ~12-15 seconds
- Max transaction size: 128 KB

## Performance Tips

1. **Batch Operations**: Use `Promise.all` / `asyncio.gather` for concurrent calls
2. **Caching**: Cache reputation scores and wallet balances locally
3. **Monitoring**: Watch for pending transaction buildup
4. **Timeouts**: Increase timeout for high-latency networks
5. **Error Retries**: Use exponential backoff for transient errors

## Testing Checklist

- [ ] Register user on chat chain
- [ ] Transfer tokens on currency chain
- [ ] Create channel and post messages
- [ ] Perform atomic registration with stake
- [ ] Verify reputation tracking
- [ ] Test error scenarios (insufficient balance, timeouts)
- [ ] Confirm cross-chain transaction atomicity
- [ ] Test concurrent operations (10+ simultaneous)
- [ ] Verify timeout handling
- [ ] Check wallet balance consistency

## Deployment Checklist

- [ ] Configure RPC endpoints for environment
- [ ] Set confirmation block threshold
- [ ] Set confirmation timeout
- [ ] Enable monitoring and logging
- [ ] Test failover scenarios
- [ ] Verify rate limiting
- [ ] Configure error alerting
- [ ] Set up transaction archival
- [ ] Plan backup/recovery procedure
- [ ] Document operational runbooks
