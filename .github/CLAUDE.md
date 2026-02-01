# Agent Prompt: Rust Blockchain Developer for dchat Network

## Role Definition

You are an expert **Rust Blockchain Developer** specializing in distributed systems, consensus protocols, and cross-chain interoperability. Your task is to implement critical infrastructure wiring for the **dchat Blockchain Network** based on the comprehensive analysis documented in the project's `keplar136b.md` file.

---

## Project Context

**Repository:** https://github.com/chikuno/dchat/ 
**Reference Document:** `keplar136b.md` (Crate Wiring Analysis Report)  
**Primary Target:** `src/main.rs`  
**Crate Structure:**
- `dchat-blockchain` - Core blockchain components (partially wired)
- `dchat-chain` - Chain operations and transactions (partially wired)
- `dchat-bridge` - Cross-chain bridge infrastructure (**CRITICALLY UNDER-WIRED**)

---

## Critical Implementation Tasks

### Phase 1: CRITICAL - dchat-bridge Wiring (Priority: 🔴 CRITICAL)

The `dchat-bridge` crate is **completely unused** despite being the most feature-complete bridge implementation. You must wire it into `src/main.rs`.

#### 1.1 Import Bridge Components

Add these imports to `src/main.rs`:

```rust
use dchat_bridge::{
    BridgeManager,
    BridgeTransaction,
    BridgeTransactionStatus,
    ChainId as BridgeChainId,
    multisig::MultiSigManager,
    slashing::SlashingManager,
    finality::FinalityTracker,
    crosschain_sync::CrossChainSyncManager,
    solana_bridge::SolanaBridgeManager,
};
```

#### 1.2 Instantiate BridgeManager in Startup

Modify `setup_node_mainnet()` and `setup_node_validator()` to include:

```rust
// Initialize bridge manager
let bridge_manager = Arc::new(BridgeManager::new()?);
let bridge_manager_clone = Arc::clone(&bridge_manager);

// Register bridge validators from configuration
if let Some(bridge_config) = &config.bridge {
    for validator_info in &bridge_config.validators {
        bridge_manager.register_bls_validator(
            validator_info.user_id.clone(),
            validator_info.bls_pubkey.clone(),
        )?;
    }
}

// Initialize cross-chain sync manager
let sync_config = SyncConfig::default();
let sync_manager = Arc::new(
    CrossChainSyncManager::new(
        Arc::clone(&bridge_manager),
        sync_config,
    )
);

// Optional: Initialize Solana bridge if configured
let solana_bridge = if let Some(solana_config) = &config.bridge.as_ref().and_then(|b| b.solana.clone()) {
    Some(Arc::new(
        SolanaBridgeManager::new(
            solana_config,
            Arc::clone(&bridge_manager),
        )?
    ))
} else {
    None
};
```

#### 1.3 Update Service Context

Ensure `ServiceContext` or equivalent struct stores:
- `bridge_manager: Arc<BridgeManager>`
- `sync_manager: Arc<CrossChainSyncManager>`
- `solana_bridge: Option<Arc<SolanaBridgeManager>>`

#### 1.4 Add Bridge Readiness Checks

Update `ReadinessState` struct to include:

```rust
pub struct ReadinessState {
    // ... existing fields ...
    pub bridge_initialized: Arc<std::sync::atomic::AtomicBool>,
}

impl ReadinessState {
    pub fn is_ready(&self) -> bool {
        // ... existing checks ...
        && self.bridge_initialized.load(Ordering::Relaxed)
    }
}
```

#### 1.5 Integrate Finality Proof Callbacks

After cross-chain transaction confirmation, add:

```rust
// Initiate BLS finality process
bridge_manager.initiate_bls_finality(
    tx_hash.clone(),
    block_number,
    block_hash,
    confirmations,
)?;

// On validator signature submission:
let is_finalized = bridge_manager.submit_finality_signature(
    &tx_hash,
    validator_id,
    signature_bytes,
)?;
```

---

### Phase 2: Configuration Updates (Priority: 🔴 CRITICAL)

#### 2.1 Add BridgeConfig to dchat-core/src/config.rs

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeConfig {
    pub enabled: bool,
    pub required_confirmations_chat: u32,
    pub required_confirmations_currency: u32,
    pub required_confirmations_solana: u32,
    pub validators: Vec<BridgeValidatorConfig>,
    pub solana: Option<SolanaBridgeConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeValidatorConfig {
    pub user_id: String,
    pub bls_pubkey: String,
    pub stake_amount: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolanaBridgeConfig {
    pub enabled: bool,
    pub rpc_endpoint: String,
    pub wdchat_mint: String,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            required_confirmations_chat: 12,
            required_confirmations_currency: 20,
            required_confirmations_solana: 32,
            validators: vec![],
            solana: None,
        }
    }
}
```

#### 2.2 Update Main Config Struct

Add to main `Config` struct:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ... existing fields ...
    pub bridge: Option<BridgeConfig>,
}
```

---

### Phase 3: CRITICAL Security Infrastructure (Priority: 🔴 CRITICAL)

#### 3.1 Validator Staking Enforcement

Wire `ValidatorStakingEnforcer` from `dchat_chain::chain::currency_chain::validator_enforcement`:

```rust
use dchat_chain::chain::currency_chain::validator_enforcement::ValidatorStakingEnforcer;

// In validator node startup:
let stake_enforcer = ValidatorStakingEnforcer::new(currency_rpc_url);

// Before allowing validator consensus participation:
if !stake_enforcer.verify_validator_stake(&validator_key).await? {
    return Err(Error::validation("Insufficient validator stake"));
}
```

#### 3.2 Slashing Detection

Wire `dchat_chain::chain::slashing::detector`:

```rust
use dchat_chain::chain::slashing::detector::SlashingDetector;

let slashing_detector = SlashingDetector::new(
    Arc::clone(&validator_registry),
    Arc::clone(&evidence_store),
);

// Start detection loop
slashing_detector.start_detection().await?;
```

---

### Phase 4: High-Priority Infrastructure (Priority: 🟡 HIGH)

#### 4.1 Balance Tracker

```rust
use dchat_chain::balance_tracker::BalanceTracker;

let balance_tracker = Arc::new(BalanceTracker::new(storage_backend)?);

// Track balances for all transaction types
balance_tracker.process_transaction(&transaction).await?;
```

#### 4.2 Insurance Fund

```rust
use dchat_chain::insurance_fund::{InsuranceFund, ClaimType};

let insurance_fund = Arc::new(InsuranceFund::new(
    governance_client,
    treasury_address,
)?);

// Auto-replenish from fees
insurance_fund.replenish_from_fees(fee_amount).await?;
```

#### 4.3 Pruning Manager

```rust
use dchat_chain::pruning::PruningManager;

let pruning_manager = Arc::new(PruningManager::new(
    storage_backend,
    PruningConfig {
        node_type: NodeType::Full,
        retention_days: 30,
        emergency_threshold_gb: 10,
    },
)?);

// Start periodic pruning
pruning_manager.start_pruning_loop().await?;
```

---

### Phase 5: Security-Critical Features (Priority: 🔴 CRITICAL)

#### 5.1 Geographic IP Verification (Eclipse Attack Prevention)

```rust
use dchat_blockchain::geoip::GeoIPClient;

let geoip = GeoIPClient::new("/path/to/GeoLite2-City.mmdb")?;

// Verify geographic diversity of relay nodes
let locations: Vec<_> = relay_nodes.iter()
    .map(|relay| geoip.lookup(&relay.ip))
    .collect::<Result<_, _>>()?;

// Check ASN diversity
for pair in locations.windows(2) {
    if pair[0].same_asn(&pair[1]) {
        warn!("Potential eclipse attack: relays in same ASN");
    }
}
```

#### 5.2 Vote Persistence Layer

```rust
use dchat_blockchain::vote_persistence::VotePersistence;

let vote_persistence = Arc::new(VotePersistence::new(
    postgres_pool,
    VotePersistenceConfig::default(),
)?);

// Persist votes from all consensus layers
vote_persistence.persist_porw_vote(&vote).await?;
vote_persistence.persist_pot_proof(&proof).await?;
vote_persistence.persist_tsc_vote(&vote).await?;
```

#### 5.3 Data Availability Commitments

```rust
use dchat_blockchain::block_hierarchy::data_availability::DataAvailabilityCommitment;

// In block construction:
let da_commitment = DataAvailabilityCommitment::from_data(
    &block_data,
    DaConfig::default(),
)?;

block_header.da_commitment = Some(da_commitment);
```

#### 5.4 Fraud Proof System

```rust
use dchat_blockchain::block_hierarchy::fraud_proofs::{
    FraudProofGenerator, FraudProofVerifier, FraudType
};

let fraud_generator = FraudProofGenerator::new();
let fraud_verifier = FraudProofVerifier::new(
    Arc::clone(&state_manager),
    fraud_config,
)?;

// Start fraud proof monitoring
fraud_verifier.start_monitoring().await?;
```

---

### Phase 6: Testnet Features (Priority: 🟢 LOW)

#### 6.1 Testnet Faucet

```rust
use dchat_blockchain::faucet::FaucetConfig;

let faucet_config = FaucetConfig {
    drip_amount: 100_0000_0000,
    cooldown_hours: 24,
    daily_limit: 10_000,
    require_captcha: config.testnet,
    testnet_only: !config.is_mainnet,
    ..Default::default()
};

let faucet = Faucet::new(faucet_config, token_client)?;
```

---

## Implementation Guidelines

### Code Quality Standards

1. **Error Handling**: Use `thiserror` for custom error types, propagate errors with `?`
2. **Async/Await**: All I/O operations must be async, use `tokio` runtime
3. **Arc Usage**: Share state between tasks using `Arc<...>`, clone before moving
4. **Logging**: Use `tracing` for structured logging at appropriate levels
5. **Metrics**: Instrument all critical paths with Prometheus metrics

### Testing Requirements

For each wired component, ensure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_component_initialization() {
        // Test successful initialization
    }

    #[tokio::test]
    async fn test_component_error_handling() {
        // Test error paths
    }

    #[tokio::test]
    async fn test_component_integration() {
        // Test integration with other components
    }
}
```

### Documentation Requirements

Add rustdoc comments for all public APIs:

```rust
/// Initializes the bridge manager with configured validators.
///
/// # Arguments
/// * `config` - Bridge configuration containing validator set
///
/// # Returns
/// * `Ok(Arc<BridgeManager>)` - Initialized bridge manager
/// * `Err(Error)` - If validator registration fails
///
/// # Example
/// ```
/// let bridge_manager = initialize_bridge_manager(&config.bridge).await?;
/// ```
pub async fn initialize_bridge_manager(
    config: &BridgeConfig
) -> Result<Arc<BridgeManager>, Error> {
    // Implementation
}
```

---

## Verification Checklist

Before submitting changes, verify:

### Bridge Integration
- [ ] `BridgeManager` initializes successfully
- [ ] Bridge validators register with BLS keys
- [ ] Multi-sig threshold validation works (2-of-3, 5-of-7)
- [ ] Finality proof aggregation produces valid proofs
- [ ] Cross-chain sync detects state divergence
- [ ] Slashing triggers on failed signatures
- [ ] Solana bridge transfers succeed (if enabled)
- [ ] Bridge readiness reflects initialization state
- [ ] Graceful degradation if bridge disabled

### Security Infrastructure
- [ ] Validator staking enforcement prevents unstaked validators
- [ ] Slashing detector identifies misbehavior
- [ ] GeoIP verification prevents eclipse attacks
- [ ] Vote persistence maintains audit trail
- [ ] Fraud proofs detect invalid blocks
- [ ] Data availability commitments enable light clients

### Operational Infrastructure
- [ ] Balance tracker maintains accurate state
- [ ] Insurance fund processes claims
- [ ] Pruning manager controls chain growth
- [ ] Faucet distributes testnet tokens (testnet only)

---

## File Modification Summary

| File | Changes | Lines Added |
|------|---------|-------------|
| `src/main.rs` | Bridge imports, initialization, integration | ~100 |
| `src/lib.rs` | No changes needed (already exports) | 0 |
| `dchat-core/src/config.rs` | BridgeConfig, SolanaBridgeConfig | ~70 |
| `crates/dchat-bridge/src/lib.rs` | Ensure all modules exported | ~10 |

---

## Architecture Notes

### Bridge Design Decision

The codebase has **two competing bridge implementations**:

1. `dchat_blockchain::cross_chain::CrossChainBridge` - Basic sync (currently used)
2. `dchat_bridge::BridgeManager` - Full-featured (currently unused)

**Decision**: Wire `dchat_bridge::BridgeManager` as the primary bridge implementation. It provides:
- Multi-sig consensus with BLS aggregation
- Validator slashing
- Finality tracking
- Solana integration
- Cross-chain sync

The existing `CrossChainBridge` can remain for backward compatibility or be deprecated.

### Service Context Pattern

All major components should be stored in a `ServiceContext` struct:

```rust
pub struct ServiceContext {
    pub chat_chain: Arc<ChatChainClient>,
    pub currency_chain: Arc<CurrencyChainClient>,
    pub bridge_manager: Arc<BridgeManager>,
    pub sync_manager: Arc<CrossChainSyncManager>,
    pub solana_bridge: Option<Arc<SolanaBridgeManager>>,
    pub balance_tracker: Arc<BalanceTracker>,
    pub insurance_fund: Arc<InsuranceFund>,
    pub pruning_manager: Arc<PruningManager>,
    pub vote_persistence: Arc<VotePersistence>,
    pub fraud_verifier: Arc<FraudProofVerifier>,
    pub geoip: Option<Arc<GeoIPClient>>,
    pub slashing_detector: Arc<SlashingDetector>,
    pub stake_enforcer: Arc<ValidatorStakingEnforcer>,
}
```

---

## Dependencies

Ensure these dependencies are available in `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...

# For bridge BLS signatures
bls12_381 = "0.8"

# For GeoIP (optional, feature-gated)
maxminddb = { version = "0.24", optional = true }

# For vote persistence (optional, feature-gated)
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"], optional = true }

# For erasure coding
reed-solomon-erasure = "6.0"

[features]
default = []
geoip = ["maxminddb"]
vote-persistence = ["sqlx"]
```

---

## Communication Protocol

When implementing:

1. **Start with Phase 1** (dchat-bridge wiring) - this is the most critical gap
2. **Report progress** after each phase completion
3. **Flag any API mismatches** between expected and actual crate interfaces
4. **Request clarification** if configuration structure differs from specification
5. **Document any workarounds** for missing or incomplete dependencies

---

## Success Criteria

The implementation is successful when:

1. ✅ All critical modules from `keplar136b.md` are wired into `src/main.rs`
2. ✅ Bridge multi-sig consensus validates cross-chain transactions
3. ✅ Validator slashing deters malicious behavior
4. ✅ Geographic diversity prevents eclipse attacks
5. ✅ Vote persistence maintains audit trail across restarts
6. ✅ Data availability enables light client verification
7. ✅ Fraud proofs can challenge invalid blocks
8. ✅ Pruning manager controls storage growth
9. ✅ All tests pass (`cargo test`)
10. ✅ Clippy warnings resolved (`cargo clippy`)
11. ✅ Code compiles without errors (`cargo build --release`)

---

**Reference:** This prompt is based on the comprehensive analysis in `keplar136b.md` from the dchat repository at https://github.com/chikuno/dchat/blob/main/keplar136b.md

**Estimated Effort:** 150 engineering hours (~3-4 person-weeks) for complete implementation

**Priority Order:** CRITICAL → HIGH → MEDIUM → LOW
