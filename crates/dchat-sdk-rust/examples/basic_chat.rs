//! Basic chat example using dchat Rust SDK
//!
//! Run with: cargo run --package dchat-sdk-rust --example basic_chat

use dchat_sdk_rust::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    dchat_log::out!("🚀 dchat SDK Basic Chat Example\n");

    // Create a client with the builder pattern
    let alice = Client::builder()
        .name("Alice")
        .data_dir("/tmp/dchat_alice")
        .listen_port(9001)
        .encryption(true)
        .build()
        .await?;

    dchat_log::out!("✅ Client created for: {}", alice.identity().username);
    dchat_log::out!("📍 User ID: {}", alice.identity().user_id);
    dchat_log::out!(
        "🔑 Public key fingerprint: {}\n",
        alice.identity().fingerprint()
    );

    // Connect to the network
    dchat_log::out!("🌐 Connecting to dchat network...");
    alice.connect().await?;
    dchat_log::out!("✅ Connected!\n");

    // Send a message
    dchat_log::out!("📤 Sending message...");
    // Create a recipient (in a real app, this would be another user's ID)
    let recipient = dchat_core::types::UserId::new();
    alice.send_message(recipient, "Hello, decentralized world!").await?;
    dchat_log::out!("✅ Message sent!\n");

    // Receive messages
    dchat_log::out!("📥 Fetching messages...");
    let messages = alice.receive_messages().await?;
    dchat_log::out!("✅ Received {} message(s)\n", messages.len());

    for (i, msg) in messages.iter().enumerate() {
        dchat_log::out!("Message #{}: {:?}", i + 1, msg.content);
    }

    // Disconnect
    dchat_log::out!("\n🔌 Disconnecting...");
    alice.disconnect().await?;
    dchat_log::out!("✅ Disconnected cleanly\n");

    dchat_log::out!("🎉 Example completed successfully!");

    Ok(())
}
