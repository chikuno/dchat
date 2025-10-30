#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize as bincode
    let _ = bincode::deserialize::<serde_json::Value>(data);
    
    // Try to parse as JSON
    let _ = serde_json::from_slice::<serde_json::Value>(data);
    
    // Try parsing as UTF-8 text
    if let Ok(text) = std::str::from_utf8(data) {
        // Try parsing as TOML
        let _ = toml::from_str::<toml::Value>(text);
    }
});
