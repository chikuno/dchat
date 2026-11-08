//! Chaos Engineering Test Suite Entry Point
//!
//! Run with: cargo test --test chaos_suite

pub mod chaos_tests;

use chaos_tests::{scenarios, ChaosTestExecutor};

#[tokio::test]
async fn run_all_chaos_scenarios() {
    let mut executor = ChaosTestExecutor::new();

    // Add all predefined scenarios
    executor.add_scenario(scenarios::network_partition());
    executor.add_scenario(scenarios::byzantine_node());
    executor.add_scenario(scenarios::cascading_failures());
    executor.add_scenario(scenarios::high_latency());
    executor.add_scenario(scenarios::resource_exhaustion());

    dchat_log::out!("\n🔥 Running Comprehensive Chaos Engineering Test Suite 🔥\n");
    dchat_log::out!("═══════════════════════════════════════════════════════════\n");

    let results = executor.run_all().await;

    dchat_log::out!("\n═══════════════════════════════════════════════════════════");
    dchat_log::out!("📊 Chaos Test Results Summary:");
    dchat_log::out!("═══════════════════════════════════════════════════════════\n");

    let mut total_passed = 0;
    let mut total_failed = 0;

    for result in &results {
        let status = if result.success { "✅ PASSED" } else { "❌ FAILED" };
        dchat_log::out!("{} - Scenario {:?}", status, result.scenario_id);
        dchat_log::out!("   Duration: {:?}", result.duration);
        dchat_log::out!("   Faults Injected: {}", result.faults_injected);
        dchat_log::out!("   Checks Passed: {}", result.checks_passed);
        dchat_log::out!("   Checks Failed: {}", result.checks_failed);
        dchat_log::out!("   Observations: {}", result.observations.len());
        dchat_log::out!();

        if result.success {
            total_passed += 1;
        } else {
            total_failed += 1;
        }
    }

    dchat_log::out!("═══════════════════════════════════════════════════════════");
    dchat_log::out!("🎯 Final Results:");
    dchat_log::out!("   Total Scenarios: {}", results.len());
    dchat_log::out!("   Passed: {}", total_passed);
    dchat_log::out!("   Failed: {}", total_failed);
    dchat_log::out!("   Success Rate: {:.1}%", (total_passed as f64 / results.len() as f64) * 100.0);
    dchat_log::out!("═══════════════════════════════════════════════════════════\n");

    // All scenarios should pass
    assert_eq!(total_failed, 0, "Some chaos scenarios failed");
}

#[tokio::test]
async fn chaos_network_partition_only() {
    let executor = ChaosTestExecutor::new();
    let scenario = scenarios::network_partition();
    
    let result = executor.run_scenario(&scenario).await;
    
    assert!(result.success, "Network partition scenario should pass");
    assert_eq!(result.checks_failed, 0);
}

#[tokio::test]
async fn chaos_byzantine_detection() {
    let executor = ChaosTestExecutor::new();
    let scenario = scenarios::byzantine_node();
    
    let result = executor.run_scenario(&scenario).await;
    
    assert!(result.success, "Byzantine detection should work");
}

#[tokio::test]
async fn chaos_cascading_failures_recovery() {
    let executor = ChaosTestExecutor::new();
    let scenario = scenarios::cascading_failures();
    
    let result = executor.run_scenario(&scenario).await;
    
    assert!(result.success, "System should recover from cascading failures");
}
