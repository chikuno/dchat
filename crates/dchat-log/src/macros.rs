//! Logging macros for dchat contracts, VM, and host nodes

/// Log a message to program execution logs
///
/// Works identically to Solana's `msg!` macro.
/// Supports both zero-allocation string literals and formatted strings.
///
/// # Examples
/// ```
/// use dchat_log::msg;
///
/// msg!("Starting token transfer");
/// let amount = 500u64;
/// msg!("Transferring {} motes", amount);
/// ```
#[macro_export]
macro_rules! msg {
    ($msg:literal) => {
        $crate::syscalls::log($msg);
    };
    ($($arg:tt)*) => {
        $crate::syscalls::log(&$crate::__private::format!($($arg)*));
    };
}

/// Log up to 5 unsigned 64-bit integers with zero heap allocation
///
/// If fewer than 5 arguments are provided, the remaining slots are zero-padded.
///
/// # Examples
/// ```
/// use dchat_log::log_64;
///
/// log_64!(1, 2, 3, 4, 5);
/// log_64!(42, 100); // Remaining 3 values padded with 0
/// ```
#[macro_export]
macro_rules! log_64 {
    ($v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr) => {
        $crate::syscalls::log_64($v1 as u64, $v2 as u64, $v3 as u64, $v4 as u64, $v5 as u64);
    };
    ($v1:expr, $v2:expr, $v3:expr, $v4:expr) => {
        $crate::log_64!($v1, $v2, $v3, $v4, 0u64);
    };
    ($v1:expr, $v2:expr, $v3:expr) => {
        $crate::log_64!($v1, $v2, $v3, 0u64, 0u64);
    };
    ($v1:expr, $v2:expr) => {
        $crate::log_64!($v1, $v2, 0u64, 0u64, 0u64);
    };
    ($v1:expr) => {
        $crate::log_64!($v1, 0u64, 0u64, 0u64, 0u64);
    };
}

/// Log a 32-byte public key / account address
///
/// # Examples
/// ```
/// use dchat_log::log_pubkey;
///
/// let pubkey = [7u8; 32];
/// log_pubkey!(&pubkey);
/// ```
#[macro_export]
macro_rules! log_pubkey {
    ($pubkey:expr) => {
        $crate::syscalls::log_pubkey($pubkey);
    };
}

/// Log remaining compute units from the runtime meter
///
/// # Examples
/// ```
/// use dchat_log::log_compute_units;
///
/// log_compute_units!();
/// ```
#[macro_export]
macro_rules! log_compute_units {
    () => {
        $crate::syscalls::log_compute_units();
    };
}

/// Log arbitrary binary data as Base64
///
/// # Examples
/// ```
/// use dchat_log::log_data;
///
/// let payload = b"event:transfer";
/// log_data!(payload);
/// ```
#[macro_export]
macro_rules! log_data {
    ($data:expr) => {
        $crate::syscalls::log_data($data);
    };
}

/// Direct byte slice logging
#[macro_export]
macro_rules! log_slice {
    ($slice:expr) => {
        $crate::syscalls::log_data($slice);
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEVELED LOGGING MACROS (REPLACING RUST LOGS / PRINTLN ACROSS NODES & CRATES)
// ═══════════════════════════════════════════════════════════════════════════════

/// Log an informational message
#[macro_export]
macro_rules! info {
    ($msg:literal) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Info, $msg);
    };
    ($($arg:tt)*) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Info, &$crate::__private::format!($($arg)*));
    };
}

/// Log a warning message
#[macro_export]
macro_rules! warn {
    ($msg:literal) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Warn, $msg);
    };
    ($($arg:tt)*) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Warn, &$crate::__private::format!($($arg)*));
    };
}

/// Log an error message
#[macro_export]
macro_rules! error {
    ($msg:literal) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Error, $msg);
    };
    ($($arg:tt)*) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Error, &$crate::__private::format!($($arg)*));
    };
}

/// Log a debug message
#[macro_export]
macro_rules! debug {
    ($msg:literal) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Debug, $msg);
    };
    ($($arg:tt)*) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Debug, &$crate::__private::format!($($arg)*));
    };
}

/// Log a trace message
#[macro_export]
macro_rules! trace {
    ($msg:literal) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Trace, $msg);
    };
    ($($arg:tt)*) => {
        $crate::syscalls::log_leveled($crate::entry::LogLevel::Trace, &$crate::__private::format!($($arg)*));
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLI & USER OUTPUT MACROS (REPLACING PRINTLN! & EPRINTLN! ACROSS CLI/COMMANDS)
// ═══════════════════════════════════════════════════════════════════════════════

/// Emit standard user-facing output (replaces `println!`)
///
/// Respects quiet mode (`--quiet`) and JSON mode (`--json`).
#[macro_export]
macro_rules! out {
    () => {
        $crate::cli::cli_out("");
    };
    ($msg:literal) => {
        $crate::cli::cli_out($msg);
    };
    ($($arg:tt)*) => {
        $crate::cli::cli_out(&$crate::__private::format!($($arg)*));
    };
}

/// Emit user-facing error output (replaces `eprintln!`)
#[macro_export]
macro_rules! err {
    () => {
        $crate::cli::cli_err("");
    };
    ($msg:literal) => {
        $crate::cli::cli_err($msg);
    };
    ($($arg:tt)*) => {
        $crate::cli::cli_err(&$crate::__private::format!($($arg)*));
    };
}

/// Emit a success banner (green checkmark or JSON success status)
#[macro_export]
macro_rules! success {
    ($msg:literal) => {
        $crate::cli::cli_success($msg);
    };
    ($($arg:tt)*) => {
        $crate::cli::cli_success(&$crate::__private::format!($($arg)*));
    };
}

/// Emit a failure banner (red cross or JSON error status)
#[macro_export]
macro_rules! failure {
    ($msg:literal) => {
        $crate::cli::cli_failure($msg);
    };
    ($($arg:tt)*) => {
        $crate::cli::cli_failure(&$crate::__private::format!($($arg)*));
    };
}

/// Emit a bold section header banner
#[macro_export]
macro_rules! header {
    ($msg:literal) => {
        $crate::cli::cli_header($msg);
    };
    ($($arg:tt)*) => {
        $crate::cli::cli_header(&$crate::__private::format!($($arg)*));
    };
}

/// Emit an aligned key-value pair
#[macro_export]
macro_rules! kv {
    ($key:expr, $val:expr) => {
        $crate::cli::cli_kv(&$crate::__private::format!("{}", $key), &$crate::__private::format!("{}", $val));
    };
}

/// Safe diagnostic tracing macro (replaces `std::dbg!`)
///
/// Prints expression, line, and evaluated value without leaking into production.
/// Returns the evaluated expression matching `std::dbg!`.
#[macro_export]
macro_rules! dchat_dbg {
    () => {
        $crate::cli::cli_dbg("", file!(), line!(), "");
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::cli::cli_dbg(
                    stringify!($val),
                    file!(),
                    line!(),
                    &$crate::__private::format!("{:?}", &tmp),
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dchat_dbg!($val)),+,)
    };
}

/// Direct alias for `dchat_dbg!` when explicitly imported
#[macro_export]
macro_rules! dbg {
    ($($arg:tt)*) => {
        $crate::dchat_dbg!($($arg)*);
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// SOLANA COMPATIBILITY ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

/// Solana-compatible alias for `msg!`
#[macro_export]
macro_rules! sol_log {
    ($($arg:tt)*) => {
        $crate::msg!($($arg)*);
    };
}

/// Solana-compatible alias for `log_64!`
#[macro_export]
macro_rules! sol_log_64 {
    ($($arg:tt)*) => {
        $crate::log_64!($($arg)*);
    };
}

/// Solana-compatible alias for `log_compute_units!`
#[macro_export]
macro_rules! sol_log_compute_units {
    ($($arg:tt)*) => {
        $crate::log_compute_units!($($arg)*);
    };
}

/// Solana-compatible alias for `log_data!`
#[macro_export]
macro_rules! sol_log_data {
    ($($arg:tt)*) => {
        $crate::log_data!($($arg)*);
    };
}

/// Solana-compatible alias for `log_slice!`
#[macro_export]
macro_rules! sol_log_slice {
    ($($arg:tt)*) => {
        $crate::log_slice!($($arg)*);
    };
}


/// Emit an interactive prompt without trailing newline (replaces `print!`)
#[macro_export]
macro_rules! prompt {
    ($msg:literal) => {
        $crate::cli::cli_prompt($msg);
    };
    ($($arg:tt)*) => {
        $crate::cli::cli_prompt(&$crate::__private::format!($($arg)*));
    };
}
