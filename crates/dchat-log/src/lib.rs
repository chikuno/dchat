//! # Dchat Log (`dchat-log`)
//!
//! High-performance, deterministic, metered logging system for Dchat smart contracts,
//! VM runtime, validators, and SDKs. Inspired by Solana's logging architecture.
//!
//! ## Key Capabilities
//! - **Drop-in Solana Compatibility**: `msg!`, `log_64!`, `log_pubkey!`, `log_compute_units!`, `log_data!`.
//! - **`#![no_std]` Native**: Zero-overhead WASM imports without bringing in WASI stdio or heavy allocators.
//! - **Deterministic Host Capture**: `dchat_log::testing::capture_logs` captures logs in thread-local buffers for test assertions.
//! - **Metered & DoS Protected**: Compute units consumed per byte of log data at the VM boundary.
//! - **Leveled Logging for Nodes**: Provides `info!`, `warn!`, `error!`, `debug!`, and `trace!` replacing `println!`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

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
pub use entry::LogLevel;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use entry::LogEntry;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use formatter::{encode_base64, format_pubkey};
pub use syscalls::{log, log_64, log_compute_units, log_data, log_leveled, log_pubkey};

#[cfg(feature = "std")]
pub use collector::{record_host_entry, set_global_sink, LogSink};

/// Testing utilities for intercepting and inspecting logs
#[cfg(feature = "std")]
pub mod testing {
    pub use crate::collector::{capture_logs, capture_logs_only};
}

/// Common prelude for contracts and modules using `dchat-log`
pub mod prelude {
    pub use crate::entry::LogLevel;
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub use crate::entry::LogEntry;
    pub use crate::{
        debug, error, info, log_64, log_compute_units, log_data, log_pubkey, log_slice, msg,
        sol_log, sol_log_64, sol_log_compute_units, sol_log_data, trace, warn,
    };
}
