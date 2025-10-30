#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip if data is too small
    if data.len() < 32 {
        return;
    }

    // Try to generate Ed25519 keypair from seed
    use ed25519_dalek::SigningKey;
    if data.len() >= 32 {
        let seed = &data[..32];
        let _ = SigningKey::from_bytes(seed.try_into().unwrap());
    }
    
    // Try X25519 keypair (x25519-dalek 2.0 API)
    use x25519_dalek::PublicKey;
    if data.len() >= 32 {
        let bytes: [u8; 32] = data[..32].try_into().unwrap();
        let _ = PublicKey::from(bytes);
    }
});
