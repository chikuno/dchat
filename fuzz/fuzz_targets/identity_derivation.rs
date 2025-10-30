#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip if data is too small for key derivation
    if data.len() < 64 {
        return;
    }

    // Test BIP-32 style key derivation
    use blake3::Hasher;
    
    let seed = &data[..32];
    let chain_code = &data[32..64];
    
    // Derive child key
    let mut hasher = Hasher::new();
    hasher.update(seed);
    hasher.update(chain_code);
    hasher.update(&[0u8; 4]); // child index
    let derived = hasher.finalize();
    
    // Try to create signing key from derived data
    use ed25519_dalek::SigningKey;
    let _ = SigningKey::from_bytes(derived.as_bytes());
});
