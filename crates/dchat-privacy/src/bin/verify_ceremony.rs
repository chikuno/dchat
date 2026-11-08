// Binary to verify ceremony artifacts
//
// Run this to verify ceremony artifacts match expected hash:
//   cargo run --release -p dchat-privacy --bin verify_ceremony
//

use dchat_privacy::ceremony_gen::{verify_ceremony_reproducibility, CeremonySeedConfig};
use std::fs;
use std::path::PathBuf;

fn main() {
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!("            dchat ZK Ceremony Artifact Verifier");
    dchat_log::out!("═══════════════════════════════════════════════════════════════");
    dchat_log::out!();

    let ceremony_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ceremony");
    let hash_file = ceremony_dir.join("final_hash.txt");

    if !hash_file.exists() {
        edchat_log::out!("ERROR: Ceremony artifacts not found at {:?}", ceremony_dir);
        edchat_log::out!("Run generate_ceremony first to create artifacts.");
        std::process::exit(1);
    }

    // Read expected hash from file
    let hash_content = fs::read_to_string(&hash_file).expect("Failed to read hash file");
    let expected_hash = hash_content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .next()
        .expect("No hash found in file")
        .trim();

    dchat_log::out!("Expected hash from file:");
    dchat_log::out!("  {}", expected_hash);
    dchat_log::out!();

    if expected_hash.starts_with("PLACEHOLDER") {
        edchat_log::out!("ERROR: Ceremony artifacts contain placeholder hash.");
        edchat_log::out!("Run generate_ceremony to create actual artifacts.");
        std::process::exit(1);
    }

    dchat_log::out!("Regenerating artifacts with mainnet seed configuration...");
    dchat_log::out!("(This verifies reproducibility)");
    dchat_log::out!();

    let config = CeremonySeedConfig::mainnet_config();
    let start = std::time::Instant::now();

    match verify_ceremony_reproducibility(&config, expected_hash) {
        Ok(true) => {
            let elapsed = start.elapsed();
            dchat_log::out!("═══════════════════════════════════════════════════════════════");
            dchat_log::out!("                    VERIFICATION PASSED ✓");
            dchat_log::out!("═══════════════════════════════════════════════════════════════");
            dchat_log::out!();
            dchat_log::out!("Ceremony artifacts are valid and reproducible.");
            dchat_log::out!("Verification completed in {:.2?}", elapsed);
            dchat_log::out!();
        }
        Ok(false) => {
            edchat_log::out!("═══════════════════════════════════════════════════════════════");
            edchat_log::out!("                    VERIFICATION FAILED ✗");
            edchat_log::out!("═══════════════════════════════════════════════════════════════");
            edchat_log::out!();
            edchat_log::out!("Hash mismatch! Artifacts may be tampered or corrupted.");
            edchat_log::out!("DO NOT USE THESE ARTIFACTS IN PRODUCTION.");
            edchat_log::out!();
            std::process::exit(1);
        }
        Err(e) => {
            edchat_log::out!("ERROR: Verification failed with error: {}", e);
            std::process::exit(1);
        }
    }

    // Also verify the files on disk match
    dchat_log::out!("Verifying on-disk artifacts...");

    let files = [
        "pot_final.bin",
        "contact_circuit_final.bin",
        "contact_vk_final.bin",
        "reputation_circuit_final.bin",
        "reputation_vk_final.bin",
    ];

    let mut hasher = blake3::Hasher::new();
    for filename in &files {
        let path = ceremony_dir.join(filename);
        if !path.exists() {
            edchat_log::out!("Missing file: {}", filename);
            std::process::exit(1);
        }
        let data = fs::read(&path).expect(&format!("Failed to read {}", filename));
        hasher.update(&data);
        dchat_log::out!("  ✓ {}", filename);
    }

    let computed_hash = hasher.finalize().to_hex().to_string();

    if computed_hash == expected_hash {
        dchat_log::out!();
        dchat_log::out!("On-disk artifacts verified successfully.");
        dchat_log::out!();
        dchat_log::out!("These artifacts are ready for production use.");
    } else {
        edchat_log::out!();
        edchat_log::out!("ERROR: On-disk artifact hash mismatch!");
        edchat_log::out!("Expected: {}", expected_hash);
        edchat_log::out!("Got:      {}", computed_hash);
        edchat_log::out!();
        edchat_log::out!("Regenerate artifacts with: cargo run --release -p dchat-privacy --bin generate_ceremony");
        std::process::exit(1);
    }
}
