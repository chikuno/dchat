//! Basic chat example using dchat Rust SDK
//!
//! Run with: cargo run --package dchat-sdk-rust --example basic_chat

use dchat_sdk_rust::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 dchat SDK Basic Chat Example\n");

    // Create a client with the builder pattern
    let alice = Client::builder()
        .name("Alice")
        .data_dir("/tmp/dchat_alice")
        .listen_port(9001)
        .encryption(true)
        .build()
        .await?;

    println!("✅ Client created for: {}", alice.identity().username);
    println!("📍 User ID: {}", alice.identity().user_id);
    println!("🔑 Public key fingerprint: {}\n", alice.identity().fingerprint());

    // Connect to the network
    println!("🌐 Connecting to dchat network...");
    alice.connect().await?;
    println!("✅ Connected!\n");

    // Send a message
    println!("📤 Sending message...");
    alice.send_message("Hello, decentralized world!").await?;
    println!("✅ Message sent!\n");

    // Receive messages
    println!("📥 Fetching messages...");
    let messages = alice.receive_messages().await?;
    println!("✅ Received {} message(s)\n", messages.len());

    for (i, msg) in messages.iter().enumerate() {
        println!("Message #{}: {:?}", i + 1, msg.content);
    }

    // Disconnect
    println!("\n🔌 Disconnecting...");
    alice.disconnect().await?;
    println!("✅ Disconnected cleanly\n");

    println!("🎉 Example completed successfully!");

    Ok(())
}
