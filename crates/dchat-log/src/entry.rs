//! Log entry and severity level definitions

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;

/// Severity levels for dchat logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogLevel {
    /// Highly detailed tracing for debugging
    Trace = 0,
    /// Detailed diagnostic information
    Debug = 1,
    /// General operational messages
    Info = 2,
    /// Non-fatal warnings or recoverable errors
    Warn = 3,
    /// Fatal or execution-blocking errors
    Error = 4,
}

impl LogLevel {
    /// Return the string representation of the log level
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl core::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A structured log entry recorded during execution
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogEntry {
    /// Log message content
    pub message: String,
    /// Program address (32 bytes) that emitted the log, if applicable
    pub program_id: Option<[u8; 32]>,
    /// Call depth for Cross-Program Invocations (0 for root invocation)
    pub depth: u8,
    /// Severity level
    pub level: LogLevel,
    /// Optional timestamp in milliseconds (host execution time)
    pub timestamp_ms: u64,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl LogEntry {
    /// Create a new program log entry
    pub fn new_program_log(program_id: Option<[u8; 32]>, message: String, depth: u8) -> Self {
        Self {
            message,
            program_id,
            depth,
            level: LogLevel::Info,
            timestamp_ms: 0,
        }
    }

    /// Create a new leveled log entry
    pub fn new_leveled(level: LogLevel, message: String) -> Self {
        Self {
            message,
            program_id: None,
            depth: 0,
            level,
            timestamp_ms: 0,
        }
    }
}
