// Example: Using blockchain client directly

use dchat_sdk::blockchain::BlockchainClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create blockchain client
    let blockchain = BlockchainClient::local();

    // Register user
    println!("Registering user...");
    let tx_id = blockchain
        .register_user(
            "user-123".to_string(),
            "alice".to_string(),
            "ed25519-public-key-hex".to_string(),
        )
        .await?;
    println!("Transaction submitted: {}", tx_id);

    // Wait for confirmation
    println!("Waiting for confirmation...");
    let receipt = blockchain.wait_for_confirmation(&tx_id).await?;

    if receipt.success {
        println!("Transaction confirmed!");
        println!("  Block Height: {:?}", receipt.block_height);
        println!("  Block Hash: {:?}", receipt.block_hash);
    } else {
        println!("Transaction failed: {:?}", receipt.error);
    }

    // Check transaction status
    let is_confirmed = blockchain.is_transaction_confirmed(&tx_id).await;
    println!("\nTransaction confirmed: {}", is_confirmed);

    // Get current block number
    let block_number = blockchain.get_block_number().await?;
    println!("Current block: {}", block_number);

    Ok(())
}
