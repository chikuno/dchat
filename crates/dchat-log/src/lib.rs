//! # Dchat Log (`dchat-log`)
//!
//! Universal, deterministic, high-performance logging and output system for the
//! entire Dchat ecosystem. Replaces unmetered `println!`, `eprintln!`, and `dbg!`
//! across smart contracts, VM runtimes, node daemons, CLI tools, and tests.
//!
//! ## Key Capabilities
//! - **Drop-in Solana Compatibility**: `msg!`, `log_64!`, `log_pubkey!`, `log_compute_units!`, `log_data!`.
//! - **`#![no_std]` Native**: Zero-overhead WASM imports without bringing in WASI stdio or heavy allocators.
//! - **CLI & User Output**: `out!`, `err!`, `success!`, `failure!`, `header!`, `kv!` replacing `println!`.
//! - **Developer Diagnostics**: `dchat_dbg!(expr)` / `dbg!(expr)` safely replacing `std::dbg!`.
//! - **Deterministic Host Capture**: `dchat_log::testing::capture_logs` captures logs in thread-local buffers.
//! - **Metered & DoS Protected**: Compute units consumed per byte of log data at the VM boundary.
//! - **Leveled Logging for Nodes**: Provides `info!`, `warn!`, `error!`, `debug!`, and `trace!`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod cli;
pub mod entry;
pub mod formatter;
pub mod macros;
pub mod syscalls;

#[cfg(feature = "std")]
pub mod collector;

#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "std")]
    pub use std::format;

    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    pub use alloc::format;
}

// Re-export core types
pub use entry::{LogKind, LogLevel};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use entry::LogEntry;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use formatter::{encode_base64, format_pubkey};
pub use syscalls::{log, log_64, log_compute_units, log_data, log_leveled, log_pubkey};

// Re-export CLI functions and state
pub use cli::{cli_dbg, cli_err, cli_failure, cli_header, cli_kv, cli_out, cli_success, OutputMode};
#[cfg(feature = "std")]
pub use cli::{
    get_output_mode, is_color_enabled, is_json_mode, is_quiet, set_color_enabled, set_json_mode,
    set_output_mode, set_quiet,
};

#[cfg(feature = "std")]
pub use collector::{record_host_entry, set_global_sink, LogSink};

/// Testing utilities for intercepting and inspecting logs
#[cfg(feature = "std")]
pub mod testing {
    pub use crate::collector::{capture_logs, capture_logs_only};
}

/// Common prelude for contracts, nodes, CLIs, and modules using `dchat-log`
pub mod prelude {
    pub use crate::entry::{LogKind, LogLevel};
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub use crate::entry::LogEntry;
    #[cfg(feature = "std")]
    pub use crate::cli::{
        get_output_mode, is_color_enabled, is_json_mode, is_quiet, set_color_enabled,
        set_json_mode, set_output_mode, set_quiet, OutputMode,
    };
    pub use crate::{
        dchat_dbg, debug, err, error, failure, header, info, kv, log_64, log_compute_units,
        log_data, log_pubkey, log_slice, msg, out, sol_log, sol_log_64, sol_log_compute_units,
        sol_log_data, success, trace, warn,
    };
}
