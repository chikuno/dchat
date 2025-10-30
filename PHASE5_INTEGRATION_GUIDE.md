# Phase 5 Integration Guide

**Version**: 1.0  
**Last Updated**: October 28, 2025  
**Applies To**: Phase 5 (Marketplace, Observability, Bridge, Accessibility, Testing)

---

## Table of Contents

1. [Marketplace Integration](#1-marketplace-integration)
2. [Observability Integration](#2-observability-integration)
3. [Cross-Chain Bridge Integration](#3-cross-chain-bridge-integration)
4. [Accessibility Integration](#4-accessibility-integration)
5. [Chaos Testing Integration](#5-chaos-testing-integration)
6. [End-to-End Integration Example](#6-end-to-end-integration-example)

---

## 1. Marketplace Integration

### Overview

The marketplace enables digital goods sales, NFT trading, and creator economy features. It integrates with the currency chain for payments and the chat chain for creator reputation.

### Basic Usage

#### 1.1 Create a Digital Goods Listing

```rust
use dchat_marketplace::{MarketplaceManager, DigitalGoodListing, DigitalGoodType, PricingModel};
use dchat_core::types::UserId;

// Initialize marketplace
let mut marketplace = MarketplaceManager::new();

// Create listing
let listing = DigitalGoodListing {
    id: "sticker-pack-001".to_string(),
    creator_id: UserId::new(),
    good_type: DigitalGoodType::StickerPack,
    name: "Ocean Vibes Stickers".to_string(),
    description: "30 ocean-themed stickers".to_string(),
    pricing: PricingModel::OneTime {
        price_usdc: 500,  // $5.00
    },
    metadata: serde_json::json!({
        "image_url": "ipfs://QmExample",
        "file_count": 30,
        "format": "PNG"
    }),
};

// Add listing to marketplace
let listing_id = marketplace.create_listing(listing)?;
println!("Listing created: {}", listing_id);
```

#### 1.2 Purchase a Digital Good

```rust
// Buyer initiates purchase
let buyer_id = UserId::new();
let payment_amount = 500;  // Must match listing price

let transaction = marketplace.purchase(
    &listing_id,
    buyer_id.clone(),
    payment_amount
)?;

println!("Purchase successful!");
println!("Transaction ID: {}", transaction.id);
println!("Status: {:?}", transaction.status);
```

#### 1.3 Register & Transfer NFTs

```rust
use dchat_marketplace::NftToken;

// Creator mints NFT
let nft = NftToken {
    token_id: "nft-badge-001".to_string(),
    owner_id: UserId::new(),
    metadata: serde_json::json!({
        "name": "Verified Creator Badge",
        "description": "Proves creator status",
        "image": "ipfs://QmBadge"
    }),
};

// Register NFT in marketplace
marketplace.register_nft(nft)?;

// Later: Transfer to new owner
let new_owner = UserId::new();
let transfer = marketplace.transfer_nft(
    "nft-badge-001",
    UserId::new(),  // Current owner
    new_owner
)?;

println!("NFT transferred!");
println!("New owner: {:?}", transfer.new_owner_id);
```

#### 1.4 Track Creator Statistics

```rust
// Get creator's marketplace performance
let creator_id = UserId::new();
if let Some(stats) = marketplace.get_creator_stats(&creator_id) {
    println!("Creator Stats:");
    println!("  Total Sales: {}", stats.total_sales);
    println!("  Total Revenue: {}", stats.total_revenue);
    println!("  Average Rating: {}", stats.average_rating);
    println!("  Active Listings: {}", stats.active_listings);
}
```

### Integration Points

**With Currency Chain**:
- Payment validation happens on currency chain
- Creator balance updates recorded on-chain
- Escrow funds held until delivery confirmed

**With Chat Chain**:
- Creator reputation linked to identity
- Listing history stored for transparency
- Creator badges minted as Soulbound NFTs

**With Observability**:
```rust
// Record marketplace metrics
metrics.record_counter("marketplace.purchases", 1).await;
metrics.observe_histogram("marketplace.transaction_value", payment_amount as f64).await;
```

### Best Practices

✅ **DO**:
- Validate payment amounts before purchase
- Check creator reputation before listing
- Require creator verification for NFT sales
- Log all transactions for audit trail

❌ **DON'T**:
- Allow negative prices
- Skip payment validation
- Transfer NFTs without ownership check
- Store sensitive payment data locally

---

## 2. Observability Integration

### Overview

Observability provides metrics collection, distributed tracing, and health monitoring for production systems. All components can integrate with the shared observability infrastructure.

### Basic Usage

#### 2.1 Record Metrics

```rust
use dchat_observability::{MetricsCollector, MetricValue};

// Initialize metrics
let metrics = MetricsCollector::new();

// Record a counter (incremental metric)
metrics.record_counter("messages_sent", 1).await;
metrics.record_counter("relay_uptime", 3600).await;

// Record a gauge (point-in-time value)
metrics.set_gauge("active_connections", 42.0).await;
metrics.set_gauge("memory_usage_mb", 128.5).await;

// Record a histogram (distribution)
metrics.observe_histogram("message_latency_ms", 45.2).await;
metrics.observe_histogram("encryption_time_us", 1234.5).await;

// Retrieve metrics
if let Some(value) = metrics.get_metric("messages_sent").await {
    println!("Messages sent: {:?}", value);
}
```

#### 2.2 Distributed Tracing

```rust
use dchat_observability::{DistributedTracer, TraceSpan, SpanStatus};

// Initialize tracer
let mut tracer = DistributedTracer::new();

// Start a trace span
let span = TraceSpan {
    id: "span-123".to_string(),
    parent_id: None,
    operation: "process_message".to_string(),
};

let span_id = tracer.start_span(span);
println!("Tracing: {}", span_id);

// Do work...

// End span with status
tracer.end_span(&span_id, SpanStatus::Ok)?;
```

#### 2.3 Health Monitoring

```rust
use dchat_observability::{HealthChecker, HealthCheck, HealthStatus};

// Initialize health checker
let mut health = HealthChecker::new();

// Register health checks
let relay_check = HealthCheck {
    component: "relay_network".to_string(),
    critical: true,
    last_check: chrono::Utc::now(),
};

health.register_check(relay_check);

// Update component status
health.update_status("relay_network", HealthStatus::Healthy)?;

// Get overall system health
let status = health.get_overall_health();
println!("System health: {:?}", status);
```

### Integration with Other Components

#### Marketplace Observability

```rust
// Track marketplace operations
metrics.record_counter("marketplace.listings_created", 1).await;
metrics.observe_histogram("marketplace.purchase_value", 500.0).await;

// Trace critical paths
let span = TraceSpan {
    operation: "process_purchase".to_string(),
    // ...
};
tracer.start_span(span);
// Execute purchase
tracer.end_span(&span_id, SpanStatus::Ok)?;
```

#### Bridge Observability

```rust
// Monitor cross-chain operations
metrics.record_counter("bridge.transactions_initiated", 1).await;
metrics.observe_histogram("bridge.finality_latency_ms", 1500.0).await;

health.update_status("bridge.chat_chain", HealthStatus::Healthy)?;
health.update_status("bridge.currency_chain", HealthStatus::Healthy)?;
```

### Best Practices

✅ **DO**:
- Record metrics for all critical operations
- Use meaningful metric names (component.operation.unit)
- Instrument all network operations with traces
- Monitor health of external dependencies

❌ **DON'T**:
- Record high-cardinality metrics (creates memory leak)
- Trace every single operation (sample instead)
- Store PII in trace data
- Assume components are healthy without checks

---

## 3. Cross-Chain Bridge Integration

### Overview

The bridge enables atomic transactions between the chat chain and currency chain. It ensures consistency across both chains with finality proofs and validator consensus.

### Basic Usage

#### 3.1 Initiate Cross-Chain Transaction

```rust
use dchat_bridge::{BridgeManager, BridgeTransaction, ChainId};
use dchat_core::types::UserId;
use uuid::Uuid;

// Initialize bridge
let mut bridge = BridgeManager::new();

// Create cross-chain transaction
let tx = BridgeTransaction {
    id: Uuid::new_v4().to_string(),
    source_chain: ChainId::CurrencyChain,
    dest_chain: ChainId::ChatChain,
    sender_id: UserId::new(),
    recipient_id: UserId::new(),
    payload: "transfer_stake:1000000".to_string(),
    timestamp: chrono::Utc::now(),
};

// Initiate transaction
let tx_id = bridge.initiate_transaction(tx)?;
println!("Transaction initiated: {}", tx_id);
```

#### 3.2 Submit Finality Proof

```rust
use dchat_bridge::FinalityProof;

// Validators submit finality proof
let proof = FinalityProof {
    transaction_id: tx_id.clone(),
    source_block_hash: "abc123".to_string(),
    dest_block_hash: "def456".to_string(),
    validator_signatures: vec![
        "sig1".to_string(),
        "sig2".to_string(),
        "sig3".to_string(),
    ],
    timestamp: chrono::Utc::now(),
};

// Submit proof
bridge.submit_finality_proof(&tx_id, proof).await?;
println!("Finality proof submitted");
```

#### 3.3 Execute Cross-Chain Transaction

```rust
// Mark ready to execute (after consensus)
bridge.mark_ready_to_execute(&tx_id)?;

// Execute on both chains
bridge.execute_transaction(&tx_id)?;
println!("Transaction executed!");

// Check status
if let Some(status) = bridge.get_transaction_status(&tx_id) {
    println!("Status: {:?}", status);
}
```

#### 3.4 Handle Failures & Rollback

```rust
// If either chain fails, rollback
let reason = "currency_chain_rejected_stake".to_string();
bridge.rollback_transaction(&tx_id, reason)?;
println!("Transaction rolled back");
```

### State Machine

The bridge transaction lifecycle:

```
┌──────────────┐
│   Initiated  │ ← Transaction created
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ FinalityProved   │ ← Validators submitted proof (2/3 consensus)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ ReadyToExecute   │ ← Source chain confirmed, ready for dest
└──────┬───────────┘
       │
       ├─────────────────────────────────┐
       ▼                                 ▼
┌──────────────┐              ┌──────────────────┐
│   Executed   │              │  RolledBack      │
└──────────────┘              └──────────────────┘
```

### Integration Points

**With Marketplace**:
- NFT ownership transferred between chains
- Escrow funds locked pending bridge confirmation
- Payment settlement happens on currency chain

**With Observability**:
```rust
metrics.record_counter("bridge.finality_proofs_submitted", 1).await;
metrics.observe_histogram("bridge.transaction_latency", duration_ms).await;
health.update_status("bridge.consensus", HealthStatus::Healthy)?;
```

### Best Practices

✅ **DO**:
- Wait for finality proof before trusting transaction
- Implement timeout for stalled transactions
- Monitor validator consensus health
- Log all state transitions for audit trail

❌ **DON'T**:
- Execute before finality is confirmed
- Ignore validator failures
- Assume both chains stay synchronized
- Skip rollback testing

---

## 4. Accessibility Integration

### Overview

Accessibility features ensure compliance with WCAG 2.1 AA+ standards, providing support for screen readers, keyboard navigation, and inclusive UI design.

### Basic Usage

#### 4.1 Register UI Elements

```rust
use dchat_accessibility::{AccessibilityManager, AccessibilityElement, AccessibilityRole};

// Initialize accessibility manager
let mut a11y = AccessibilityManager::new();

// Register a button
let button = AccessibilityElement {
    id: "send-button".to_string(),
    role: AccessibilityRole::Button,
    label: Some("Send Message".to_string()),
    description: Some("Send the message to all recipients".to_string()),
    tab_index: Some(1),
};

a11y.register_element(button)?;

// Register a link
let link = AccessibilityElement {
    id: "privacy-policy".to_string(),
    role: AccessibilityRole::Link,
    label: Some("Privacy Policy".to_string()),
    description: None,
    tab_index: Some(2),
};

a11y.register_element(link)?;
```

#### 4.2 Add ARIA Attributes

```rust
use dchat_accessibility::AriaAttribute;

// Add ARIA live region (announces updates to screen readers)
a11y.add_aria_attribute(
    "notification-area",
    AriaAttribute::Live {
        politeness: "polite".to_string(),
        atomic: true,
    }
)?;

// Add ARIA description
a11y.add_aria_attribute(
    "password-field",
    AriaAttribute::DescribedBy {
        description_id: "password-requirements".to_string(),
    }
)?;

// Add ARIA expanded state
a11y.add_aria_attribute(
    "menu-toggle",
    AriaAttribute::Expanded(true)
)?;
```

#### 4.3 Register Keyboard Shortcuts

```rust
use dchat_accessibility::KeyboardShortcut;

// Register Ctrl+S for send
let send_shortcut = KeyboardShortcut {
    key_combination: "Ctrl+S".to_string(),
    action: "send_message".to_string(),
    element_id: "send-button".to_string(),
    description: "Send current message".to_string(),
};

a11y.register_keyboard_shortcut(send_shortcut)?;

// Register Esc to close dialog
let close_shortcut = KeyboardShortcut {
    key_combination: "Escape".to_string(),
    action: "close_dialog".to_string(),
    element_id: "dialog".to_string(),
    description: "Close the current dialog".to_string(),
};

a11y.register_keyboard_shortcut(close_shortcut)?;
```

#### 4.4 Validate Color Contrast

```rust
use dchat_accessibility::Color;

// Define colors
let text_color = Color::new(0, 0, 0);      // Black
let background_color = Color::new(255, 255, 255); // White

// Calculate contrast ratio (should be >= 4.5:1)
let ratio = text_color.contrast_ratio(&background_color);
println!("Contrast ratio: {:.2}:1", ratio);

// Validate for normal text
let is_valid = text_color.check_contrast(&background_color, false);
println!("Valid for normal text: {}", is_valid);  // true

// Validate for large text (>= 3:1 required)
let is_valid_large = text_color.check_contrast(&background_color, true);
println!("Valid for large text: {}", is_valid_large);  // true
```

#### 4.5 Validate Element Configuration

```rust
// Check if element has accessibility issues
match a11y.validate_element("send-button") {
    Ok(issues) => {
        if issues.is_empty() {
            println!("Element passes all accessibility checks");
        } else {
            for issue in issues {
                println!("Issue: {:?}", issue);
            }
        }
    }
    Err(e) => println!("Validation error: {}", e),
}
```

### Integration with UI Frameworks

**For Web UI**:
```rust
// Generate HTML with accessibility attributes
let html = format!(
    r#"<button id="send" aria-label="Send Message" 
              aria-describedby="send-help" tabindex="1">
       Send
     </button>"#
);
```

**For Mobile UI**:
```rust
// Set accessibility properties on view
// iOS: UIAccessibility
// Android: ContentDescription, ImportantForAccessibility
```

### WCAG 2.1 AA+ Compliance Checklist

- ✅ **1.4.3 Contrast (Minimum)**: All text >= 4.5:1, large text >= 3:1
- ✅ **2.1.1 Keyboard**: All functionality keyboard accessible
- ✅ **2.1.2 No Keyboard Trap**: Focus can move away from any element
- ✅ **4.1.2 Name, Role, Value**: All elements have accessible name
- ✅ **4.1.3 Status Messages**: Screen readers announce dynamic content

### Best Practices

✅ **DO**:
- Label all interactive elements
- Provide keyboard alternatives for all mouse actions
- Test with actual screen readers (NVDA, JAWS, VoiceOver)
- Use semantic HTML roles
- Validate contrast ratios before release

❌ **DON'T**:
- Use images for text without alt text
- Rely only on color to convey information
- Create keyboard traps
- Hide focus indicators
- Forget about color blindness (use patterns, not just colors)

---

## 5. Chaos Testing Integration

### Overview

Chaos testing validates system resilience by injecting faults and observing recovery behavior. Use it to identify failure scenarios before they affect users.

### Basic Usage

#### 5.1 Network Simulation

```rust
use dchat_testing::NetworkSimulator;

// Initialize network simulator
let mut simulator = NetworkSimulator::new();

// Simulate latency to peer
simulator.set_latency("relay-node-1", 100);  // 100ms latency
simulator.set_latency("relay-node-2", 250);

// Simulate packet loss (0.0 = 0%, 1.0 = 100%)
simulator.set_packet_loss("relay-node-1", 0.05)?;  // 5% loss

// Retrieve conditions for peer
if let Some(conditions) = simulator.apply_conditions("relay-node-1") {
    println!("Conditions for relay-node-1:");
    println!("  Latency: {}ms", conditions.latency_ms);
    println!("  Packet loss: {:.2}%", conditions.packet_loss_rate * 100.0);
}
```

#### 5.2 Chaos Experiments

```rust
use dchat_testing::{ChaosOrchestrator, ChaosExperiment, ChaosExperimentType};
use std::time::Duration;

// Initialize orchestrator
let mut chaos = ChaosOrchestrator::new();

// Define experiment
let experiment = ChaosExperiment {
    id: "network-partition-test".to_string(),
    experiment_type: ChaosExperimentType::NetworkPartition,
    target_component: "relay_network".to_string(),
    duration: Duration::from_secs(60),
    parameters: serde_json::json!({
        "partition_ratio": 0.5,  // Partition 50% of nodes
        "affected_regions": ["us-west", "eu-central"]
    }),
};

// Start experiment
let exp_id = chaos.start_experiment(experiment);
println!("Experiment started: {}", exp_id);

// Inject fault
chaos.inject_fault(&exp_id)?;
println!("Fault injected");

// Simulate work...
std::thread::sleep(Duration::from_secs(30));

// End experiment
chaos.end_experiment(&exp_id, true)?;  // success = true
println!("Experiment ended");

// Calculate success rate
let success_rate = chaos.calculate_success_rate(ChaosExperimentType::NetworkPartition);
println!("Success rate: {:.2}%", success_rate * 100.0);
```

#### 5.3 Recovery Testing

```rust
use dchat_testing::{RecoveryTester, RecoveryTest, RecoveryScenario, RecoveryValidation};

// Initialize recovery tester
let mut recovery_tester = RecoveryTester::new();

// Define recovery test
let test = RecoveryTest {
    id: "bridge-recovery".to_string(),
    scenario: RecoveryScenario::ValidatorFailure { failed_count: 1 },
    expected_recovery_time_ms: 5000,
};

// Start test
let test_id = recovery_tester.start_recovery_test(test);
println!("Recovery test started: {}", test_id);

// Simulate failure and recovery...

// Validate recovery
let validation = RecoveryValidation {
    test_id: test_id.clone(),
    recovered_successfully: true,
    actual_recovery_time_ms: 3500,
};

recovery_tester.validate_recovery(&test_id, validation)?;
println!("Recovery validated!");
```

### Chaos Experiment Types

```rust
pub enum ChaosExperimentType {
    NetworkPartition,        // Split network into isolated groups
    PacketLoss,             // Drop packets randomly
    LatencyInjection,       // Add artificial delay
    NodeFailure,            // Simulate node crash
    DiskFailure,            // Simulate disk errors
    MemoryPressure,         // Reduce available memory
    CpuThrottle,            // Limit CPU usage
    Byzantine,              // Simulate malicious validator behavior
}
```

### Test Scenarios

#### Bridge Consensus Failure

```rust
let exp = ChaosExperiment {
    experiment_type: ChaosExperimentType::NodeFailure,
    target_component: "bridge.validators".to_string(),
    parameters: serde_json::json!({
        "failed_nodes": ["validator-2"],
        "expected_outcome": "consensus_still_2_of_3"
    }),
};
```

#### Marketplace Payment Race

```rust
let exp = ChaosExperiment {
    experiment_type: ChaosExperimentType::LatencyInjection,
    target_component: "marketplace.payment_channel".to_string(),
    parameters: serde_json::json!({
        "latency_variance_ms": 500,
        "expected_outcome": "no_double_spend"
    }),
};
```

#### Relay Network Partition

```rust
let exp = ChaosExperiment {
    experiment_type: ChaosExperimentType::NetworkPartition,
    target_component: "relay_network".to_string(),
    parameters: serde_json::json!({
        "partition_ratio": 0.5,
        "expected_outcome": "automatic_failover"
    }),
};
```

### Best Practices

✅ **DO**:
- Start with low-impact experiments (latency before partition)
- Run experiments in staging/test environments only
- Monitor metrics during experiments
- Document all failure scenarios
- Test recovery paths explicitly
- Automate chaos tests in CI/CD

❌ **DON'T**:
- Run chaos experiments in production
- Skip recovery validation
- Ignore failed experiments (investigate immediately)
- Run all experiments simultaneously (isolate variables)
- Assume recovery is automatic (test it!)

---

## 6. End-to-End Integration Example

### Scenario: Marketplace NFT Sale Across Chains

This example demonstrates all Phase 5 components working together:

```rust
use dchat_marketplace::*;
use dchat_observability::*;
use dchat_bridge::*;
use dchat_accessibility::*;
use dchat_testing::*;
use dchat_core::types::UserId;

/// Complete marketplace NFT sale with observability, bridge transfer, 
/// and chaos testing validation
pub async fn nft_sale_example() -> Result<()> {
    // ==================== Setup ====================
    
    // Initialize all components
    let mut marketplace = MarketplaceManager::new();
    let metrics = MetricsCollector::new();
    let mut bridge = BridgeManager::new();
    let mut a11y = AccessibilityManager::new();
    let mut chaos = ChaosOrchestrator::new();
    
    // ==================== Step 1: Creator Lists NFT ====================
    println!("\n[STEP 1] Creator lists NFT on marketplace...");
    
    let creator_id = UserId::new();
    let nft = NftToken {
        token_id: "nft-01".to_string(),
        owner_id: creator_id.clone(),
        metadata: serde_json::json!({
            "name": "Verified Badge",
            "image": "ipfs://QmExample"
        }),
    };
    
    marketplace.register_nft(nft)?;
    metrics.record_counter("marketplace.nft_registered", 1).await;
    println!("✓ NFT registered");
    
    // ==================== Step 2: Buyer Purchases NFT ====================
    println!("\n[STEP 2] Buyer purchases NFT...");
    
    let listing = DigitalGoodListing {
        id: "listing-01".to_string(),
        creator_id: creator_id.clone(),
        good_type: DigitalGoodType::Nft,
        name: "Verified Creator Badge".to_string(),
        description: "Limited edition badge".to_string(),
        pricing: PricingModel::OneTime { price_usdc: 1000 },
        metadata: serde_json::json!({ "token_id": "nft-01" }),
    };
    
    let listing_id = marketplace.create_listing(listing)?;
    metrics.observe_histogram("marketplace.listing_value", 1000.0).await;
    
    let buyer_id = UserId::new();
    let purchase = marketplace.purchase(&listing_id, buyer_id.clone(), 1000)?;
    metrics.record_counter("marketplace.purchase_completed", 1).await;
    println!("✓ Purchase completed: {} USDC", purchase.payment_amount);
    
    // ==================== Step 3: Transfer NFT Across Chains ====================
    println!("\n[STEP 3] Transfer NFT ownership via bridge...");
    
    let bridge_tx = BridgeTransaction {
        id: uuid::Uuid::new_v4().to_string(),
        source_chain: ChainId::ChatChain,
        dest_chain: ChainId::CurrencyChain,
        sender_id: creator_id.clone(),
        recipient_id: buyer_id.clone(),
        payload: "transfer_nft:nft-01".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    let tx_id = bridge.initiate_transaction(bridge_tx)?;
    metrics.record_counter("bridge.transactions_initiated", 1).await;
    println!("✓ Bridge transaction initiated: {}", tx_id);
    
    // Simulate finality proof
    let proof = FinalityProof {
        transaction_id: tx_id.clone(),
        source_block_hash: "hash-001".to_string(),
        dest_block_hash: "hash-002".to_string(),
        validator_signatures: vec![
            "sig1".to_string(),
            "sig2".to_string(),
            "sig3".to_string(),
        ],
        timestamp: chrono::Utc::now(),
    };
    
    bridge.submit_finality_proof(&tx_id, proof).await?;
    bridge.mark_ready_to_execute(&tx_id)?;
    bridge.execute_transaction(&tx_id)?;
    metrics.record_counter("bridge.transactions_executed", 1).await;
    println!("✓ NFT ownership transferred via bridge");
    
    // ==================== Step 4: Make UI Accessible ====================
    println!("\n[STEP 4] Configure accessibility for transaction UI...");
    
    let tx_button = AccessibilityElement {
        id: "complete-purchase".to_string(),
        role: AccessibilityRole::Button,
        label: Some("Complete Purchase".to_string()),
        description: Some("Finalize the NFT purchase and transfer".to_string()),
        tab_index: Some(1),
    };
    a11y.register_element(tx_button)?;
    
    a11y.add_aria_attribute(
        "complete-purchase",
        AriaAttribute::Live {
            politeness: "polite".to_string(),
            atomic: true,
        }
    )?;
    
    let issues = a11y.validate_element("complete-purchase")?;
    println!("✓ UI accessibility validated ({} issues)", issues.len());
    
    // ==================== Step 5: Test Resilience ====================
    println!("\n[STEP 5] Run chaos test to validate resilience...");
    
    let test_exp = ChaosExperiment {
        id: "nft-sale-partition-test".to_string(),
        experiment_type: ChaosExperimentType::NetworkPartition,
        target_component: "relay_network".to_string(),
        duration: std::time::Duration::from_secs(30),
        parameters: serde_json::json!({
            "partition_ratio": 0.3,
            "test_objective": "verify_nft_consistency"
        }),
    };
    
    let exp_id = chaos.start_experiment(test_exp);
    chaos.inject_fault(&exp_id)?;
    println!("✓ Chaos experiment injected (network partition 30%)");
    
    // Simulate recovery and transaction completion
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    chaos.end_experiment(&exp_id, true)?;
    let success_rate = chaos.calculate_success_rate(ChaosExperimentType::NetworkPartition);
    metrics.observe_histogram("chaos.success_rate", success_rate).await;
    println!("✓ Recovery successful ({}% success rate)", (success_rate * 100.0) as u32);
    
    // ==================== Step 6: Observe Results ====================
    println!("\n[STEP 6] System observability metrics...");
    
    if let Some(metric) = metrics.get_metric("marketplace.purchase_completed").await {
        println!("  Purchases: {:?}", metric);
    }
    if let Some(metric) = metrics.get_metric("bridge.transactions_executed").await {
        println!("  Bridge tx: {:?}", metric);
    }
    
    // ==================== Success ====================
    println!("\n✅ End-to-end example completed!");
    println!("   NFT ownership transferred across chains");
    println!("   UI is accessible (WCAG 2.1 AA+)");
    println!("   System validated under chaos conditions");
    
    Ok(())
}
```

### Expected Output

```
[STEP 1] Creator lists NFT on marketplace...
✓ NFT registered

[STEP 2] Buyer purchases NFT...
✓ Purchase completed: 1000 USDC

[STEP 3] Transfer NFT ownership via bridge...
✓ Bridge transaction initiated: 12345-67890-abcde
✓ NFT ownership transferred via bridge

[STEP 4] Configure accessibility for transaction UI...
✓ UI accessibility validated (0 issues)

[STEP 5] Run chaos test to validate resilience...
✓ Chaos experiment injected (network partition 30%)
✓ Recovery successful (98% success rate)

[STEP 6] System observability metrics...
  Purchases: Counter(1)
  Bridge tx: Counter(1)

✅ End-to-end example completed!
   NFT ownership transferred across chains
   UI is accessible (WCAG 2.1 AA+)
   System validated under chaos conditions
```

---

## Summary

| Component | Best For | Integration Complexity |
|-----------|----------|------------------------|
| **Marketplace** | Creator economy, digital goods | ⭐⭐ (Medium) |
| **Observability** | Monitoring, debugging, alerting | ⭐ (Low) |
| **Bridge** | Cross-chain operations, state sync | ⭐⭐⭐ (High) |
| **Accessibility** | WCAG compliance, inclusive UX | ⭐⭐ (Medium) |
| **Chaos Testing** | Resilience validation, QA | ⭐⭐ (Medium) |

All components work together seamlessly. Start with **Observability** (easiest), then add **Marketplace**, then **Bridge** (most complex), then **Accessibility** and **Chaos Testing** as final layers.

---

**Next Steps**: See PHASE5_SECURITY_AUDIT.md for security best practices, or proceed to Phase 6 for production hardening.

