// Example: Using blockchain client directly

use dchat_sdk::blockchain::BlockchainClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create blockchain client
    let blockchain = BlockchainClient::local();

    // Register user
    dchat_log::out!("Registering user...");
    let tx_id = blockchain
        .register_user(
            "user-123".to_string(),
            "alice".to_string(),
            "ed25519-public-key-hex".to_string(),
        )
        .await?;
    dchat_log::out!("Transaction submitted: {}", tx_id);

    // Wait for confirmation
    dchat_log::out!("Waiting for confirmation...");
    let receipt = blockchain.wait_for_confirmation(&tx_id).await?;

    if receipt.success {
        dchat_log::out!("Transaction confirmed!");
        dchat_log::out!("  Block Height: {:?}", receipt.block_height);
        dchat_log::out!("  Block Hash: {:?}", receipt.block_hash);
    } else {
        dchat_log::out!("Transaction failed: {:?}", receipt.error);
    }

    // Check transaction status
    let is_confirmed = blockchain.is_transaction_confirmed(&tx_id).await;
    dchat_log::out!("\nTransaction confirmed: {}", is_confirmed);

    // Get current block number
    let block_number = blockchain.get_block_number().await?;
    dchat_log::out!("Current block: {}", block_number);

    Ok(())
}
