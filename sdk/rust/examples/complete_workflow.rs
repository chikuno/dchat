// Example: Complete user workflow with blockchain integration

use dchat_sdk::blockchain::BlockchainClient;
use dchat_sdk::user::UserManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize blockchain client
    let blockchain = BlockchainClient::local();

    // Create user manager
    let user_manager = UserManager::new(blockchain, "http://localhost:8080".to_string());

    // Create two users
    println!("Creating users...");
    let alice = user_manager.create_user("alice").await?;
    let bob = user_manager.create_user("bob").await?;

    println!("Alice: {} (confirmed: {})", alice.user_id, alice.on_chain_confirmed);
    println!("Bob: {} (confirmed: {})", bob.user_id, bob.on_chain_confirmed);

    // Send direct message
    println!("\nSending direct message...");
    let message = user_manager
        .send_direct_message(&alice.user_id, &bob.user_id, "Hello Bob!", None)
        .await?;
    println!(
        "Message sent: {} (TX: {:?})",
        message.message_id, message.tx_id
    );

    // Create channel
    println!("\nCreating channel...");
    let channel = user_manager
        .create_channel(&alice.user_id, "General", Some("General discussion"))
        .await?;
    println!(
        "Channel created: {} (TX: {:?})",
        channel.channel_id, channel.tx_id
    );

    // Post to channel
    println!("\nPosting to channel...");
    let post = user_manager
        .post_to_channel(&alice.user_id, &channel.channel_id, "Welcome everyone!")
        .await?;
    println!("Posted: {} (TX: {:?})", post.message_id, post.tx_id);

    Ok(())
}
