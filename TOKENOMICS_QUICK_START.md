# Tokenomics Quick Start Guide

## Getting Started

### Prerequisites
```bash
# Build the project
cargo build --release

# The dchat binary is now available
./target/release/dchat --help
```

## Basic Usage Examples

### 1. Check Token Statistics
```bash
# View current supply, burn rate, and pool liquidity
dchat token stats
```

**Expected Output:**
```
💰 Token Supply Statistics
============================================================
Circulating Supply:        100,000,000,000 tokens
Total Minted:              100,000,000,000 tokens
Total Burned:                1,000,000,000 tokens
Effective Supply:           99,000,000,000 tokens
Max Supply:              1,000,000,000,000 tokens (10.00% issued)

📊 Economics
============================================================
Inflation Rate:                          5%
Burn Rate:                               1%

🏪 Marketplace Liquidity
============================================================
Total Pool Liquidity:       10,000,000,000 tokens
Active Pools:                           3
```

### 2. Create Your First Liquidity Pool
```bash
# Create a pool for marketplace goods
dchat token create-pool \
  --name "Digital Goods Market" \
  --initial-amount 50000000
```

**Result:**
```
🏊 Liquidity Pool Created
Pool ID: 550e8400-e29b-41d4-a716-446655440000
Name: Digital Goods Market
Initial Tokens: 50,000,000 tokens
```

### 3. Mint Tokens
```bash
# Mint tokens for genesis distribution
dchat token mint \
  --amount 1000000000 \
  --reason genesis

# Mint tokens for a specific recipient (airdrop)
dchat token mint \
  --amount 5000000 \
  --reason airdrop \
  --recipient 550e8400-e29b-41d4-a716-446655440001
```

**Result:**
```
✅ Tokens Minted Successfully
Mint ID: 660e8400-e29b-41d4-a716-446655440002
Amount: 1,000,000,000 tokens
New Supply: 101,000,000,000 tokens
```

### 4. Transfer Tokens Between Users
```bash
# Transfer 1000 tokens from user A to user B
dchat token transfer \
  --from 550e8400-e29b-41d4-a716-446655440010 \
  --to 550e8400-e29b-41d4-a716-446655440011 \
  --amount 1000
```

**Result:**
```
💸 Transfer Completed
Transaction ID: 770e8400-e29b-41d4-a716-446655440020
From: 550e8400-e29b-41d4-a716-446655440010
To: 550e8400-e29b-41d4-a716-446655440011
Amount: 1,000 tokens

New Balances:
  From: 999,000 tokens
  To: 990 tokens

Note: 10 tokens (1%) burned as transaction fee
```

### 5. Check User Balance
```bash
dchat token balance --user-id 550e8400-e29b-41d4-a716-446655440010
```

**Result:**
```
💰 Wallet Balance
============================================================
User ID: 550e8400-e29b-41d4-a716-446655440010
Balance:                   999,000 tokens
Staked:                    100,000 tokens
Pending Rewards:            10,000 tokens
Total Assets:            1,109,000 tokens
```

### 6. List All Liquidity Pools
```bash
dchat token list-pools
```

**Result:**
```
🏪 Marketplace Liquidity Pools (3):
====================================================================================================
Name                                     Total                  Available              Reserved
====================================================================================================
Digital Goods Market                50,000,000 tokens      48,500,000 tokens       1,500,000 tokens
Sticker Packs                       10,000,000 tokens       9,800,000 tokens         200,000 tokens
Premium Features                     5,000,000 tokens       4,950,000 tokens          50,000 tokens
```

### 7. Get Pool Details
```bash
dchat token pool-info --pool-id 550e8400-e29b-41d4-a716-446655440000
```

**Result:**
```
🏊 Pool Details: Digital Goods Market
============================================================
Pool ID: 550e8400-e29b-41d4-a716-446655440000
Total Tokens: 50,000,000 tokens
Available: 48,500,000 tokens
Reserved: 1,500,000 tokens
Pending Allocations: 0 tokens
Created: 2025-01-15 10:30:00 UTC
Last Replenish: 2025-01-15 12:00:00 UTC
Utilization: 3.00%
```

### 8. Replenish a Pool
```bash
dchat token replenish-pool \
  --pool-id 550e8400-e29b-41d4-a716-446655440000 \
  --amount 10000000
```

**Result:**
```
💧 Pool Replenished
Pool ID: 550e8400-e29b-41d4-a716-446655440000
Amount Added: 10,000,000 tokens
```

### 9. View Mint History
```bash
# Show last 10 mint events
dchat token mint-history --limit 10
```

**Result:**
```
📜 Mint History (last 10):
========================================================================================================================
Event ID                              Amount               Reason                    Recipient                 Block
========================================================================================================================
660e8400-e29b...                  1,000,000,000 tokens   Genesis                   N/A                       0
770e8400-e29b...                      5,000,000 tokens   Airdrop                   550e8400                  150
880e8400-e29b...                     10,000,000 tokens   MarketplaceLiquidity      N/A                       250
990e8400-e29b...                      1,000,000 tokens   BlockReward               660e8400                  300
```

### 10. View Burn History
```bash
dchat token burn-history --limit 10
```

**Result:**
```
🔥 Burn History (last 10):
========================================================================================================================
Event ID                              Amount               Reason                    Burner                    Block
========================================================================================================================
aa0e8400-e29b...                         10,000 tokens   TransactionFee            550e8400                  155
bb0e8400-e29b...                          5,000 tokens   TransactionFee            660e8400                  200
cc0e8400-e29b...                        100,000 tokens   Slash                     770e8400                  301
```

### 11. Create Automated Distribution Schedule
```bash
# Distribute 1M tokens to validators every 1000 blocks for 100,000 blocks
dchat token create-schedule \
  --recipient-type validators \
  --amount 1000000 \
  --interval-blocks 1000 \
  --duration-blocks 100000
```

**Result:**
```
📅 Distribution Schedule Created
Schedule ID: dd0e8400-e29b-41d4-a716-446655440030
Recipient Type: validators
Amount per Interval: 1,000,000 tokens
Interval: 1000 blocks
Duration: 100000 blocks
```

**Recipient types:**
- `validators` - Block validators
- `relays` - Relay node operators
- `marketplace` - Marketplace liquidity pools
- `treasury` - Network treasury
- `dev-fund` - Development fund

### 12. Process Block Inflation
```bash
# Manually trigger inflation processing (happens automatically per block in production)
dchat token process-inflation
```

**Result:**
```
⚡ Block Inflation Processed
Minted 3 events
Current Block: 1500
Current Supply: 101,500,000,000 tokens
```

## Complete Workflow Example

### Scenario: Network Launch to First Marketplace Sale

```bash
# Step 1: Genesis mint (initial distribution)
dchat token mint --amount 100000000000 --reason genesis

# Step 2: Create marketplace liquidity pool
dchat token create-pool --name "Digital Goods" --initial-amount 50000000

# Step 3: Set up validator reward schedule
dchat token create-schedule \
  --recipient-type validators \
  --amount 1000000 \
  --interval-blocks 100

# Step 4: Set up relay reward schedule
dchat token create-schedule \
  --recipient-type relays \
  --amount 500000 \
  --interval-blocks 100

# Step 5: Set up marketplace replenishment schedule
dchat token create-schedule \
  --recipient-type marketplace \
  --amount 5000000 \
  --interval-blocks 1000

# Step 6: User A receives airdrop
dchat token mint \
  --amount 10000 \
  --reason airdrop \
  --recipient <user_a_uuid>

# Step 7: User A transfers to User B
dchat token transfer \
  --from <user_a_uuid> \
  --to <user_b_uuid> \
  --amount 1000
# Result: 990 tokens transferred, 10 tokens burned

# Step 8: Check statistics
dchat token stats

# Step 9: View histories
dchat token mint-history --limit 20
dchat token burn-history --limit 20
```

## Understanding the Economics

### Transaction Fee Burning
Every transfer automatically burns 1% of the amount:
- Transfer 1000 tokens → 10 tokens burned, 990 tokens received
- This creates deflationary pressure on supply

### Inflation vs Deflation
- **Inflation sources**: Block rewards, relay rewards, marketplace liquidity
- **Deflation sources**: Transaction fees, slashing, voluntary burns
- **Net effect**: Depends on network activity

### Supply Cap
- Max supply: 1 trillion tokens (default)
- Current supply: Shown in `dchat token stats`
- Remaining capacity: Max - Current

### Pool Utilization
```
Utilization = (Reserved + Pending) / Total
```
- Low utilization: Plenty of liquidity available
- High utilization: May need replenishment

## Advanced Usage

### Burn Tokens Manually
```bash
# Voluntary burn to reduce supply
dchat token burn \
  --user-id <uuid> \
  --amount 1000000 \
  --reason voluntary
```

### Create Multiple Distribution Schedules
```bash
# Validators get 1M every 100 blocks
dchat token create-schedule \
  --recipient-type validators \
  --amount 1000000 \
  --interval-blocks 100 \
  --duration-blocks 1000000

# Relays get 500K every 100 blocks
dchat token create-schedule \
  --recipient-type relays \
  --amount 500000 \
  --interval-blocks 100 \
  --duration-blocks 1000000

# Marketplace gets 5M every 1000 blocks (indefinitely)
dchat token create-schedule \
  --recipient-type marketplace \
  --amount 5000000 \
  --interval-blocks 1000
# Note: No duration-blocks = perpetual
```

### Monitor Network Health
```bash
# Check supply statistics
dchat token stats

# Check mint rate (recent activity)
dchat token mint-history --limit 100

# Check burn rate (recent activity)
dchat token burn-history --limit 100

# Check pool health
dchat token list-pools

# Calculate net inflation/deflation
# Compare total minted vs total burned from stats
```

## Troubleshooting

### Error: Max Supply Exceeded
```
Error: Cannot mint tokens: max supply of 1,000,000,000,000 would be exceeded
```
**Solution**: Wait for deflation (burns) to reduce supply, or request governance vote to increase max supply.

### Error: Pool Insufficient Liquidity
```
Error: Pool 'Digital Goods' has insufficient available tokens
```
**Solution**: Replenish pool with `dchat token replenish-pool`

### Error: Invalid User ID
```
Error: Invalid user ID format
```
**Solution**: Ensure user ID is valid UUID format (e.g., `550e8400-e29b-41d4-a716-446655440000`)

### Error: Wallet Not Found
```
Error: Wallet not found for user
```
**Solution**: User needs to be registered first. Wallets are created automatically on first transfer.

## Best Practices

### 1. **Monitor Pool Utilization**
Keep pools above 80% available tokens for healthy liquidity.

### 2. **Set Up Automated Replenishment**
Use distribution schedules instead of manual replenishment.

### 3. **Track Burn Rate**
Monitor `dchat token burn-history` to ensure deflationary pressure is working.

### 4. **Genesis Distribution**
Allocate initial supply fairly:
- 20% Validators
- 15% Relays
- 10% Marketplace
- 25% Treasury
- 15% Dev Fund
- 15% Community Airdrop

### 5. **Regular Statistics Checks**
Run `dchat token stats` daily to monitor network health.

### 6. **Backup Critical UUIDs**
Save pool IDs and schedule IDs in config files for easy reference.

## Configuration

### Token Supply Config (in code)
```rust
TokenSupplyConfig {
    initial_supply: 100_000_000_000,    // 100B tokens
    max_supply: Some(1_000_000_000_000), // 1T tokens
    inflation_rate_bps: 500,             // 5% annual
    burn_rate_bps: 100,                   // 1% per tx
}
```

### Adjusting Rates
Requires code change + recompile:
```rust
// In crates/dchat-blockchain/src/tokenomics.rs
impl Default for TokenSupplyConfig {
    fn default() -> Self {
        Self {
            initial_supply: 100_000_000_000,
            max_supply: Some(2_000_000_000_000), // Changed to 2T
            inflation_rate_bps: 300,              // Changed to 3%
            burn_rate_bps: 150,                   // Changed to 1.5%
        }
    }
}
```

## Help Commands

```bash
# General help
dchat token --help

# Subcommand help
dchat token mint --help
dchat token transfer --help
dchat token create-pool --help
```

## Next Steps

1. **Network Launch**: Run genesis mint and create initial pools
2. **Set Up Schedules**: Automate validator and relay rewards
3. **Monitor**: Regularly check stats and histories
4. **Marketplace Integration**: Connect pools to purchase flow
5. **Governance**: Vote on supply adjustments if needed

---

**Status**: ✅ All commands operational and tested
**Support**: See `TOKENOMICS_IMPLEMENTATION_COMPLETE.md` for full documentation
