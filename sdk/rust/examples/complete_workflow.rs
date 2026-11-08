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
    dchat_log::out!("Creating users...");
    let alice = user_manager.create_user("alice").await?;
    let bob = user_manager.create_user("bob").await?;

    dchat_log::out!("Alice: {} (confirmed: {})", alice.user_id, alice.on_chain_confirmed);
    dchat_log::out!("Bob: {} (confirmed: {})", bob.user_id, bob.on_chain_confirmed);

    // Send direct message
    dchat_log::out!("\nSending direct message...");
    let message = user_manager
        .send_direct_message(&alice.user_id, &bob.user_id, "Hello Bob!", None)
        .await?;
    dchat_log::out!(
        "Message sent: {} (TX: {:?})",
        message.message_id, message.tx_id
    );

    // Create channel
    dchat_log::out!("\nCreating channel...");
    let channel = user_manager
        .create_channel(&alice.user_id, "General", Some("General discussion"))
        .await?;
    dchat_log::out!(
        "Channel created: {} (TX: {:?})",
        channel.channel_id, channel.tx_id
    );

    // Post to channel
    dchat_log::out!("\nPosting to channel...");
    let post = user_manager
        .post_to_channel(&alice.user_id, &channel.channel_id, "Welcome everyone!")
        .await?;
    dchat_log::out!("Posted: {} (TX: {:?})", post.message_id, post.tx_id);

    Ok(())
}
