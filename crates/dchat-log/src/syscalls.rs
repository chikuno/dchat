//! Syscall interface and target dispatch for dchat-log

use crate::entry::LogLevel;

// ═══════════════════════════════════════════════════════════════════════════════
// WASM GUEST SYSCALLS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "wasm32")]
mod wasm_ffi {
    extern "C" {
        // Primary Dchat syscall ABI
        pub fn dchat_log_(ptr: *const u8, len: u64);
        pub fn dchat_log_64_(v1: u64, v2: u64, v3: u64, v4: u64, v5: u64);
        pub fn dchat_log_compute_units_();
        pub fn dchat_log_pubkey_(pubkey_ptr: *const u8);
        pub fn dchat_log_data_(data_ptr: *const u8, data_len: u64);

        // Fallback / Solana compatibility symbols
        pub fn sol_log_(ptr: *const u8, len: u64);
        pub fn sol_log_64_(v1: u64, v2: u64, v3: u64, v4: u64, v5: u64);
        pub fn sol_log_compute_units_();
        pub fn sol_log_pubkey(pubkey_ptr: *const u8);
        pub fn sol_log_data(data_ptr: *const u8, data_len: u64);
    }
}

/// Log a UTF-8 message
pub fn log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        // In WASM, directly invoke host syscall with pointer & length
        wasm_ffi::sol_log_(msg.as_ptr(), msg.len() as u64);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "std")]
        {
            let formatted = format!("Program log: {}", msg);
            crate::collector::record_host_entry(crate::entry::LogEntry::new_program_log(
                None, formatted, 0,
            ));
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = msg;
        }
    }
}

/// Log five 64-bit unsigned integers without allocation
pub fn log_64(v1: u64, v2: u64, v3: u64, v4: u64, v5: u64) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm_ffi::sol_log_64_(v1, v2, v3, v4, v5);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "std")]
        {
            let formatted = format!(
                "Program log_64: {:#x}, {:#x}, {:#x}, {:#x}, {:#x}",
                v1, v2, v3, v4, v5
            );
            crate::collector::record_host_entry(crate::entry::LogEntry::new_program_log(
                None, formatted, 0,
            ));
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = (v1, v2, v3, v4, v5);
        }
    }
}

/// Query and log the remaining compute units from the runtime meter
pub fn log_compute_units() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm_ffi::sol_log_compute_units_();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "std")]
        {
            crate::collector::record_host_entry(crate::entry::LogEntry::new_program_log(
                None,
                "Program consumed [native check] compute units".to_string(),
                0,
            ));
        }
    }
}

/// Log a 32-byte public key / account address
pub fn log_pubkey(pubkey: &[u8; 32]) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm_ffi::sol_log_pubkey(pubkey.as_ptr());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "std")]
        {
            let addr_str = crate::formatter::format_pubkey(pubkey);
            let formatted = format!("Program log: {}", addr_str);
            crate::collector::record_host_entry(crate::entry::LogEntry::new_program_log(
                None, formatted, 0,
            ));
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = pubkey;
        }
    }
}

/// Log binary data as Base64
pub fn log_data(data: &[u8]) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm_ffi::sol_log_data(data.as_ptr(), data.len() as u64);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "std")]
        {
            let b64 = crate::formatter::encode_base64(data);
            let formatted = format!("Program data: {}", b64);
            crate::collector::record_host_entry(crate::entry::LogEntry::new_program_log(
                None, formatted, 0,
            ));
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = data;
        }
    }
}

/// Log a leveled message on the host
pub fn log_leveled(level: LogLevel, msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        // On WASM, collapse to program log with level prefix
        #[cfg(feature = "alloc")]
        {
            let formatted = alloc::format!("[{}] {}", level, msg);
            log(&formatted);
        }
        #[cfg(not(feature = "alloc"))]
        {
            log(msg);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "std")]
        {
            let mut entry = crate::entry::LogEntry::new_leveled(level, msg.to_string());
            entry.message = format!("[{}] {}", level, msg);
            crate::collector::record_host_entry(entry);
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = (level, msg);
        }
    }
}
