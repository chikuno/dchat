// Binary to generate ceremony artifacts for dchat ZK proofs
//
// Run this to generate the ceremony binary files:
//   cargo run --release -p dchat-privacy --bin generate_ceremony
//
// Output files will be written to crates/dchat-privacy/ceremony/

use dchat_privacy::ceremony_gen::{generate_ceremony_artifacts, CeremonySeedConfig};
use std::fs;
use std::path::PathBuf;

fn main() {
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!("            dchat ZK Ceremony Artifact Generator");
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!();

    // Use mainnet configuration
    let config = CeremonySeedConfig::mainnet_config();

    dchat_log::out!("Ceremony Configuration:");
    dchat_log::out!("  Bitcoin Block: #{}", config.bitcoin_block_height);
    dchat_log::out!("  Bitcoin Hash:  {}", config.bitcoin_block_hash);
    if let Some(ref eth) = config.ethereum_block_hash {
        dchat_log::out!("  Ethereum Hash: {}", eth);
    }
    dchat_log::out!("  Domain:        {}", config.domain_separator);
    dchat_log::out!("  Protocol Ver:  {}", config.protocol_version);
    dchat_log::out!();

    let seed = config.derive_seed();
    dchat_log::out!("Derived Seed: {}", hex::encode(&seed));
    dchat_log::out!();

    dchat_log::out!("Generating ceremony artifacts (this may take a minute)...");
    dchat_log::out!();

    let start = std::time::Instant::now();
    let artifacts = match generate_ceremony_artifacts(&config) {
        Ok(a) => a,
        Err(e) => {
            edchat_log::out!("ERROR: Failed to generate artifacts: {}", e);
            std::process::exit(1);
        }
    };
    let elapsed = start.elapsed();

    dchat_log::out!("Generation complete in {:.2?}", elapsed);
    dchat_log::out!();

    // Determine output directory
    let ceremony_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ceremony");
    fs::create_dir_all(&ceremony_dir).expect("Failed to create ceremony directory");

    // Write artifact files
    let files = [
        ("pot_final.bin", &artifacts.pot_bytes),
        ("contact_circuit_final.bin", &artifacts.contact_pk_bytes),
        ("contact_vk_final.bin", &artifacts.contact_vk_bytes),
        (
            "reputation_circuit_final.bin",
            &artifacts.reputation_pk_bytes,
        ),
        ("reputation_vk_final.bin", &artifacts.reputation_vk_bytes),
    ];

    dchat_log::out!("Writing artifact files to {:?}", ceremony_dir);
    dchat_log::out!();

    for (filename, data) in &files {
        let path = ceremony_dir.join(filename);
        fs::write(&path, data).expect(&format!("Failed to write {}", filename));
        dchat_log::out!("  {:40} {:>10} bytes", filename, data.len());
    }

    // Write hash file
    let hash_content = format!(
        "# BLAKE3 hash of ceremony artifacts\n\
         # Generated: {}\n\
         # Protocol Version: {}\n\
         # Seed Config Hash: {}\n\
         #\n\
         # Individual artifact hashes:\n\
         #   contact_pk:     {}\n\
         #   contact_vk:     {}\n\
         #   reputation_pk:  {}\n\
         #   reputation_vk:  {}\n\
         #\n\
         # Combined hash (verify with: blake3sum pot_final.bin contact_circuit_final.bin contact_vk_final.bin reputation_circuit_final.bin reputation_vk_final.bin)\n\
         {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.protocol_version,
        artifacts.metadata.seed_config_hash,
        artifacts.metadata.contact_pk_hash,
        artifacts.metadata.contact_vk_hash,
        artifacts.metadata.reputation_pk_hash,
        artifacts.metadata.reputation_vk_hash,
        artifacts.artifacts_hash,
    );

    let hash_path = ceremony_dir.join("final_hash.txt");
    fs::write(&hash_path, &hash_content).expect("Failed to write final_hash.txt");
    dchat_log::out!("  {:40}", "final_hash.txt");

    // Write metadata JSON
    let metadata_json =
        serde_json::to_string_pretty(&artifacts.metadata).expect("Failed to serialize metadata");
    let metadata_path = ceremony_dir.join("ceremony_metadata.json");
    fs::write(&metadata_path, &metadata_json).expect("Failed to write metadata");
    dchat_log::out!("  {:40}", "ceremony_metadata.json");

    dchat_log::out!();
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!("                    CEREMONY ARTIFACTS HASH");
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!();
    dchat_log::out!("  {}", artifacts.artifacts_hash);
    dchat_log::out!();
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!();
    dchat_log::out!("IMPORTANT: Before mainnet launch, verify this hash matches the");
    dchat_log::out!("published value at: https://dchat.network/ceremony/verification");
    dchat_log::out!();
    dchat_log::out!("To verify artifacts independently:");
    dchat_log::out!("  1. Regenerate using the same seed configuration");
    dchat_log::out!("  2. Compare the resulting hash");
    dchat_log::out!("  3. Hash must match exactly");
    dchat_log::out!();
}
