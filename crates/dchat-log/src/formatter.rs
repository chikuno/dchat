//! Formatting utilities for Solana-style program logs and addresses

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;

/// Format a 32-byte address / public key as Base58 (Solana-style address)
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn format_pubkey(bytes: &[u8; 32]) -> String {
    const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut digits = [0u8; 64];
    let mut digits_len = 1;

    for &byte in bytes.iter() {
        let mut carry = byte as u32;
        for i in 0..digits_len {
            carry += (digits[i] as u32) << 8;
            digits[i] = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits[digits_len] = (carry % 58) as u8;
            digits_len += 1;
            carry /= 58;
        }
    }

    // Leading zeros -> '1'
    let mut leading_zeros = 0;
    while leading_zeros < 32 && bytes[leading_zeros] == 0 {
        leading_zeros += 1;
    }

    let mut out = String::with_capacity(leading_zeros + digits_len);
    for _ in 0..leading_zeros {
        out.push('1');
    }
    for &d in digits[..digits_len].iter().rev() {
        out.push(ALPHABET[d as usize] as char);
    }
    out
}

/// Encode a byte slice as standard Base64 string without external dependencies
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);

    let chunks = input.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
    }

    match remainder.len() {
        1 => {
            let n = (remainder[0] as u32) << 16;
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = ((remainder[0] as u32) << 16) | ((remainder[1] as u32) << 8);
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
            out.push('=');
        }
        _ => {}
    }

    out
}

/// Format program invocation start banner
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn format_program_invoke(program_id: &[u8; 32], depth: u8) -> String {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::format;
    format!("Program {} invoke [{}]", format_pubkey(program_id), depth)
}

/// Format program success banner
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn format_program_success(program_id: &[u8; 32]) -> String {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::format;
    format!("Program {} success", format_pubkey(program_id))
}

/// Format program failure banner
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn format_program_failure(program_id: &[u8; 32], error_desc: &str) -> String {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::format;
    format!("Program {} failed: {}", format_pubkey(program_id), error_desc)
}
