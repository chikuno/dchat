# Parallel Chain Architecture Implementation

## Overview

dchat implements a **parallel chain architecture** with two independent but coordinated chains:

- **Chat Chain**: Identity, messaging, channels, permissions, governance, reputation, account recovery
- **Currency Chain**: Payments, staking, rewards, economics

This design enables:
- **Separation of concerns**: Chat operations isolated from economics
- **Independent scaling**: Each chain can optimize for its workload
- **Atomic cross-chain operations**: Coordinated transactions via bridge
- **Economic security**: Token incentives managed separately from messaging

## Architecture Components

### Chat Chain (`crates/dchat-blockchain/src/chat_chain.rs`)

**Responsibilities:**
- User identity registration and verification
- Direct message ordering and sequencing
- Channel creation and management
- Governance voting and moderation
- Reputation tracking
- Permission enforcement

**Transaction Types:**
```rust
enum TransactionType {
    RegisterUser,           // Create on-chain identity
    SendDirectMessage,      // Establish message ordering
    CreateChannel,          // Create channel with ownership
    PostToChannel,          // Channel message sequencing
}
```

**Key Operations:**
```rust
pub fn register_user(&self, user_id: &UserId, public_key: Vec<u8>) -> Result<Uuid>
pub fn send_direct_message(&self, sender: &UserId, recipient: &UserId, message_id: MessageId) -> Result<Uuid>
pub fn create_channel(&self, owner: &UserId, channel_id: &ChannelId, name: String) -> Result<Uuid>
pub fn post_to_channel(&self, sender: &UserId, channel_id: &ChannelId, message_id: MessageId) -> Result<Uuid>
pub fn get_reputation(&self, user_id: &UserId) -> Result<u32>
pub fn update_reputation(&self, user_id: &UserId, delta: i32) -> Result<u32>
```

### Currency Chain (`crates/dchat-blockchain/src/currency_chain.rs`)

**Responsibilities:**
- Wallet management and balances
- Token transfers and payments
- Staking and reward distribution
- Economic incentives for relay nodes
- Slashing for misbehavior
- Atomic swaps and cross-chain coordination

**Transaction Types:**
```rust
pub enum CurrencyTransactionType {
    Payment,    // Transfer between users
    Stake,      // Lock tokens for rewards
    Unstake,    // Unlock staked tokens
    Reward,     // Claim earned rewards
    Slash,      // Penalty for misbehavior
    Swap,       // Cross-chain atomic swap
}
```

**Key Operations:**
```rust
pub fn create_wallet(&self, user_id: &UserId, initial_balance: u64) -> Result<Wallet>
pub fn get_balance(&self, user_id: &UserId) -> Result<u64>
pub fn transfer(&self, from: &UserId, to: &UserId, amount: u64) -> Result<Uuid>
pub fn stake(&self, user_id: &UserId, amount: u64, lock_duration_seconds: i64) -> Result<Uuid>
pub fn claim_rewards(&self, user_id: &UserId) -> Result<Uuid>
pub fn get_wallet(&self, user_id: &UserId) -> Result<Option<Wallet>>
```

### Cross-Chain Bridge (`crates/dchat-blockchain/src/cross_chain.rs`)

**Responsibilities:**
- Coordinate atomic operations between chains
- Ensure consistency across chain boundaries
- Handle failure scenarios with rollback
- Track atomic transaction status

**Atomic Operations:**

#### Register User with Stake
```rust
pub fn register_user_with_stake(
    &self,
    user_id: &UserId,
    public_key: Vec<u8>,
    stake_amount: u64,
) -> Result<Uuid>
```

Steps:
1. Create wallet on currency chain
2. Register identity on chat chain
3. Stake tokens on currency chain
4. Record cross-chain transaction
5. Finalize when both chains confirm

#### Create Channel with Fee
```rust
pub fn create_channel_with_fee(
    &self,
    owner: &UserId,
    channel_name: String,
    creation_fee: u64,
) -> Result<Uuid>
```

Steps:
1. Transfer creation fee to treasury (currency chain)
2. Create channel on chat chain
3. Record cross-chain transaction
4. Finalize when both chains confirm

**Status Tracking:**
```rust
pub enum CrossChainStatus {
    Pending,                    // Awaiting both chains
    ChatChainConfirmed,        // Chat chain confirmed
    CurrencyChainConfirmed,    // Currency chain confirmed
    AtomicSuccess,             // Both chains confirmed
    RolledBack,                // Rolled back on failure
    Failed,                    // Permanent failure
}
```

## SDK Implementation

### Dart SDK

**Chat Chain Client:**
```dart
class ChatChainClient extends BlockchainClient {
  Future<Map<String, dynamic>> registerUser(String userId, List<int> publicKey)
  Future<Map<String, dynamic>> sendDirectMessage(String sender, String recipient, String messageId)
  Future<Map<String, dynamic>> createChannel(String owner, String channelId, String name)
  Future<Map<String, dynamic>> postToChannel(String sender, String channelId, String messageId)
  Future<int> getReputation(String userId)
  Future<List<Map<String, dynamic>>> getUserTransactions(String userId)
}
```

**Currency Chain Client:**
```dart
class CurrencyChainClient extends BlockchainClient {
  Future<Wallet> createWallet(String userId, int initialBalance)
  Future<Wallet> getWallet(String userId)
  Future<Map<String, dynamic>> transfer(String from, String to, int amount)
  Future<Map<String, dynamic>> stake(String userId, int amount, int lockDurationSeconds)
  Future<Map<String, dynamic>> claimRewards(String userId)
  Future<List<Map<String, dynamic>>> getUserTransactions(String userId)
}
```

**Cross-Chain Bridge:**
```dart
class CrossChainBridge {
  Future<CrossChainTransaction> registerUserWithStake(String userId, List<int> publicKey, int stakeAmount)
  Future<CrossChainTransaction> createChannelWithFee(String owner, String channelName, int creationFee)
  Future<CrossChainTransaction?> getStatus(String bridgeTxId)
  Future<List<CrossChainTransaction>> getUserTransactions(String userId)
}
```

### TypeScript SDK

Similar structure with async/await pattern:
```typescript
export class ChatChainClient {
  async registerUser(userId: string, publicKey: Buffer): Promise<ChatChainTransaction>
  async sendDirectMessage(sender: string, recipient: string, messageId: string): Promise<ChatChainTransaction>
  async createChannel(owner: string, channelId: string, name: string): Promise<ChatChainTransaction>
  async postToChannel(sender: string, channelId: string, messageId: string): Promise<ChatChainTransaction>
  async getReputation(userId: string): Promise<number>
  async getUserTransactions(userId: string): Promise<ChatChainTransaction[]>
}

export class CurrencyChainClient {
  async createWallet(userId: string, initialBalance: number): Promise<Wallet>
  async getWallet(userId: string): Promise<Wallet>
  async transfer(from: string, to: string, amount: number): Promise<CurrencyChainTransaction>
  async stake(userId: string, amount: number, lockDurationSeconds: number): Promise<CurrencyChainTransaction>
  async claimRewards(userId: string): Promise<CurrencyChainTransaction>
  async getUserTransactions(userId: string): Promise<CurrencyChainTransaction[]>
}

export class CrossChainBridge {
  async registerUserWithStake(userId: string, publicKey: Buffer, stakeAmount: number): Promise<CrossChainTransaction>
  async createChannelWithFee(owner: string, channelName: string, creationFee: number): Promise<CrossChainTransaction>
  async getStatus(bridgeTxId: string): Promise<CrossChainTransaction | null>
  async getUserTransactions(userId: string): Promise<CrossChainTransaction[]>
  async waitForAtomicCompletion(bridgeTxId: string, maxWaitMs?: number): Promise<CrossChainTransaction>
}
```

### Python SDK

Async-first design:
```python
class ChatChainClient:
    async def register_user(self, user_id: str, public_key: bytes) -> ChatChainTransaction
    async def send_direct_message(self, sender: str, recipient: str, message_id: str) -> ChatChainTransaction
    async def create_channel(self, owner: str, channel_id: str, name: str) -> ChatChainTransaction
    async def post_to_channel(self, sender: str, channel_id: str, message_id: str) -> ChatChainTransaction
    async def get_reputation(self, user_id: str) -> int
    async def get_user_transactions(self, user_id: str) -> List[ChatChainTransaction]

class CurrencyChainClient:
    async def create_wallet(self, user_id: str, initial_balance: int) -> Wallet
    async def get_wallet(self, user_id: str) -> Wallet
    async def transfer(self, from_user: str, to_user: str, amount: int) -> CurrencyChainTransaction
    async def stake(self, user_id: str, amount: int, lock_duration_seconds: int) -> CurrencyChainTransaction
    async def claim_rewards(self, user_id: str) -> CurrencyChainTransaction
    async def get_user_transactions(self, user_id: str) -> List[CurrencyChainTransaction]

class CrossChainBridge:
    async def register_user_with_stake(self, user_id: str, public_key: bytes, stake_amount: int) -> CrossChainTransaction
    async def create_channel_with_fee(self, owner: str, channel_name: str, creation_fee: int) -> CrossChainTransaction
    async def get_status(self, bridge_tx_id: str) -> Optional[CrossChainTransaction]
    async def get_user_transactions(self, user_id: str) -> List[CrossChainTransaction]
    async def wait_for_atomic_completion(self, bridge_tx_id: str, max_wait_ms: int = 60000) -> CrossChainTransaction
```

### Rust SDK

Async/await with tokio:
```rust
pub struct ChatChainClient {
    pub async fn register_user(&self, user_id: &str, public_key: Vec<u8>) -> Result<String>
    pub async fn send_direct_message(&self, sender: &str, recipient: &str, message_id: &str) -> Result<String>
    pub async fn create_channel(&self, owner: &str, channel_id: &str, name: String) -> Result<String>
    pub async fn get_user_transactions(&self, user_id: &str) -> Result<Vec<ChatChainTransaction>>
}

pub struct CurrencyChainClient {
    pub async fn create_wallet(&self, user_id: &str, initial_balance: u64) -> Result<Wallet>
    pub async fn get_wallet(&self, user_id: &str) -> Result<Option<Wallet>>
    pub async fn transfer(&self, from: &str, to: &str, amount: u64) -> Result<String>
    pub async fn stake(&self, user_id: &str, amount: u64, lock_duration_seconds: i64) -> Result<String>
    pub async fn get_user_transactions(&self, user_id: &str) -> Result<Vec<CurrencyChainTransaction>>
}
```

## Usage Examples

### Dart Example

```dart
final chatChain = ChatChainClient(rpcUrl: 'http://localhost:8545');
final currencyChain = CurrencyChainClient(rpcUrl: 'http://localhost:8546');
final bridge = CrossChainBridge(chatChain: chatChain, currencyChain: currencyChain);

// Register user with stake atomically
final bridgeTx = await bridge.registerUserWithStake(
  'alice',
  publicKey,
  1000, // stake amount
);

// Check status
final status = await bridge.getStatus(bridgeTx.id);
print('Status: ${status?.status}');
```

### TypeScript Example

```typescript
const chatChain = new ChatChainClient();
const currencyChain = new CurrencyChainClient();
const bridge = new CrossChainBridge(chatChain, currencyChain);

// Create channel with fee
const bridgeTx = await bridge.createChannelWithFee(
  'alice',
  'general',
  100, // creation fee
);

// Wait for atomic completion
await bridge.waitForAtomicCompletion(bridgeTx.id);
```

### Python Example

```python
import asyncio

async def main():
    chat_chain = ChatChainClient()
    currency_chain = CurrencyChainClient()
    bridge = CrossChainBridge(chat_chain, currency_chain)

    # Register user with stake
    bridge_tx = await bridge.register_user_with_stake(
        'alice',
        public_key,
        1000
    )

    # Wait for atomic completion
    await bridge.wait_for_atomic_completion(bridge_tx.id)

asyncio.run(main())
```

## Transaction Flow

### Chat Chain Flow
```
User Action → SDK → ChatChainClient → RPC Call → Chat Chain Node
                                    ↓
                         Transaction Validation
                                    ↓
                         Block Confirmation (6 blocks)
                                    ↓
                         SDK Notification
```

### Currency Chain Flow
```
User Action → SDK → CurrencyChainClient → RPC Call → Currency Chain Node
                                       ↓
                            Transaction Validation
                                       ↓
                            Balance Update
                                       ↓
                            Block Confirmation (6 blocks)
                                       ↓
                            SDK Notification
```

### Cross-Chain Atomic Flow
```
User Action → CrossChainBridge
                    ↓
        ┌───────────┴───────────┐
        ↓                       ↓
   Chat Chain            Currency Chain
   (Register)            (Create Wallet)
        ↓                       ↓
   Confirm                  Confirm
        ↓                       ↓
   Chat Chain            Currency Chain
   (Message)             (Stake)
        ↓                       ↓
   Confirm                  Confirm
        ↓                       ↓
   ┌────────────────────────────┐
   ↓
Record AtomicSuccess on Bridge
   ↓
SDK Notification
```

## Error Handling

### Chat Chain Errors
- `TransactionFailed`: Transaction rejected by validators
- `InsufficientPermissions`: Sender lacks channel permissions
- `InvalidIdentity`: User not registered on chain

### Currency Chain Errors
- `InsufficientBalance`: Not enough tokens for operation
- `StakeNotMatured`: Trying to unstake locked tokens
- `InvalidWallet`: Wallet not found

### Cross-Chain Errors
- `ChatChainConfirmationTimeout`: Chat chain didn't confirm
- `CurrencyChainConfirmationTimeout`: Currency chain didn't confirm
- `AtomicAborted`: One chain confirmed, other failed (rolled back)

## Performance Considerations

- **Chat Chain**: Optimized for message ordering, high throughput
- **Currency Chain**: Optimized for atomic writes, consistent state
- **Bridge**: Minimal latency between chain operations
- **Confirmation Time**: ~30-60 seconds for 6 confirmations per chain

## Security

- **Separation**: Chat and currency operations isolated
- **Atomic Guarantees**: All-or-nothing cross-chain operations
- **Replay Protection**: Sequence numbers per chain
- **State Consistency**: Merkle proofs for verification
