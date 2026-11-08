//! Relay node example using dchat Rust SDK
//!
//! Run with: cargo run --package dchat-sdk-rust --example relay_node

use dchat_sdk_rust::{RelayConfig, RelayNode, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    dchat_log::out!("🚀 dchat SDK Relay Node Example\n");

    // Create a relay with custom configuration
    let config = RelayConfig {
        name: "MyRelayNode".to_string(),
        listen_addr: "0.0.0.0".to_string(),
        listen_port: 9000,
        staking_enabled: true,
        min_uptime_percent: 99.0,
    };

    let relay = RelayNode::with_config(config);

    dchat_log::out!("✅ Relay node created: {}", relay.config().name);
    dchat_log::out!(
        "📍 Listening on: {}:{}\n",
        relay.config().listen_addr,
        relay.config().listen_port
    );

    // Start the relay
    dchat_log::out!("🌐 Starting relay node...");
    relay.start().await?;
    dchat_log::out!("✅ Relay is running!\n");

    // Simulate running for a bit
    dchat_log::out!("⏳ Running for 5 seconds...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Get statistics
    dchat_log::out!("\n📊 Relay Statistics:");
    let stats = relay.get_stats().await;
    dchat_log::out!("  Connected peers: {}", stats.connected_peers);
    dchat_log::out!("  Messages relayed: {}", stats.messages_relayed);
    dchat_log::out!("  Uptime: {:.2}%", stats.uptime_percent);
    dchat_log::out!("  Reputation score: {}\n", stats.reputation_score);

    // Stop the relay
    dchat_log::out!("🛑 Stopping relay node...");
    relay.stop().await?;
    dchat_log::out!("✅ Relay stopped cleanly\n");

    dchat_log::out!("🎉 Example completed successfully!");

    Ok(())
}
