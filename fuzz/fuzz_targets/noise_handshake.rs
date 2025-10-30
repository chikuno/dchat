#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip if data is too small
    if data.len() < 32 {
        return;
    }

    // Try to parse noise handshake messages
    use snow::Builder;
    use snow::params::NoiseParams;
    
    let params: Option<NoiseParams> = "Noise_XX_25519_ChaChaPoly_BLAKE2s".parse().ok();
    if let Some(params) = params {
        // Try initiator
        if let Ok(mut initiator) = Builder::new(params.clone()).build_initiator() {
            let mut buf = vec![0u8; 65535];
            let _ = initiator.write_message(data, &mut buf);
        }
        
        // Try responder
        if let Ok(mut responder) = Builder::new(params).build_responder() {
            let mut buf = vec![0u8; 65535];
            let _ = responder.read_message(data, &mut buf);
        }
    }
});
