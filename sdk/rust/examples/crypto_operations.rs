// Example: Cryptographic operations

use dchat_sdk::crypto::{KeyPair, hash_content};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate new key pair
    dchat_log::out!("Generating key pair...");
    let keypair = KeyPair::generate();
    dchat_log::out!("Public Key: {}", keypair.public_key_hex());
    dchat_log::out!("Private Key: {}", keypair.private_key_hex());

    // Sign a message
    let message = b"Hello, blockchain!";
    dchat_log::out!("\nSigning message: {}", String::from_utf8_lossy(message));
    let signature = keypair.sign(message);
    dchat_log::out!("Signature: {}", hex::encode(&signature));

    // Verify signature
    keypair.verify(message, &signature)?;
    dchat_log::out!("Signature valid!");

    // Hash content
    let content = "Message content";
    let content_hash = hash_content(content);
    dchat_log::out!("\nContent: {}", content);
    dchat_log::out!("Hash: {}", content_hash);

    // Export/import key pair
    let keypair_dict = keypair.to_dict();
    dchat_log::out!("\nExported keypair: {:?}", keypair_dict);

    let restored_keypair = KeyPair::from_dict(&keypair_dict)?;
    dchat_log::out!("Restored public key: {}", restored_keypair.public_key_hex());

    Ok(())
}
