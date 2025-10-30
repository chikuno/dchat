// Example: Cryptographic operations

use dchat_sdk::crypto::{KeyPair, hash_content};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate new key pair
    println!("Generating key pair...");
    let keypair = KeyPair::generate();
    println!("Public Key: {}", keypair.public_key_hex());
    println!("Private Key: {}", keypair.private_key_hex());

    // Sign a message
    let message = b"Hello, blockchain!";
    println!("\nSigning message: {}", String::from_utf8_lossy(message));
    let signature = keypair.sign(message);
    println!("Signature: {}", hex::encode(&signature));

    // Verify signature
    keypair.verify(message, &signature)?;
    println!("Signature valid!");

    // Hash content
    let content = "Message content";
    let content_hash = hash_content(content);
    println!("\nContent: {}", content);
    println!("Hash: {}", content_hash);

    // Export/import key pair
    let keypair_dict = keypair.to_dict();
    println!("\nExported keypair: {:?}", keypair_dict);

    let restored_keypair = KeyPair::from_dict(&keypair_dict)?;
    println!("Restored public key: {}", restored_keypair.public_key_hex());

    Ok(())
}
