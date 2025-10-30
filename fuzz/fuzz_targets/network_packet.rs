#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try parsing as various network packet formats
    
    // Try as raw bytes packet
    if data.len() >= 4 {
        let _length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        // Would parse rest of packet here
    }
    
    // Try as length-prefixed message
    if data.len() >= 2 {
        let length = u16::from_be_bytes([data[0], data[1]]) as usize;
        if length < data.len() - 2 {
            let _payload = &data[2..2 + length];
        }
    }
    
    // Try parsing IPv4/IPv6 addresses
    use std::net::{Ipv4Addr, Ipv6Addr};
    if data.len() >= 4 {
        let _ = Ipv4Addr::new(data[0], data[1], data[2], data[3]);
    }
    if data.len() >= 16 {
        let octets: [u8; 16] = data[..16].try_into().unwrap();
        let _ = Ipv6Addr::from(octets);
    }
});
