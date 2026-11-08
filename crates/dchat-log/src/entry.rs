//! Log entry, severity level, and output domain definitions

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

/// The operational domain or kind of log/output entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogKind {
    /// Smart contract execution log (msg!, log_64!, etc.)
    Program = 0,
    /// System or daemon leveled log (info!, warn!, etc.)
    System = 1,
    /// CLI standard user-facing output (out!)
    CliOut = 2,
    /// CLI error output (err!)
    CliErr = 3,
    /// CLI success banner (success!)
    CliSuccess = 4,
    /// CLI failure banner (failure!)
    CliFailure = 5,
    /// CLI section header (header!)
    CliHeader = 6,
    /// CLI key-value attribute (kv!)
    CliKv = 7,
    /// Developer diagnostic trace (dbg!)
    Debug = 8,
}

/// A structured log entry recorded during execution
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogEntry {
    /// Log message content
    pub message: String,
    /// The domain / origin of this entry
    pub kind: LogKind,
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
            kind: LogKind::Program,
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
            kind: LogKind::System,
            program_id: None,
            depth: 0,
            level,
            timestamp_ms: 0,
        }
    }

    /// Create a new CLI / user-facing output entry
    pub fn new_cli(kind: LogKind, message: String) -> Self {
        let level = match kind {
            LogKind::CliErr | LogKind::CliFailure => LogLevel::Error,
            LogKind::Debug => LogLevel::Debug,
            _ => LogLevel::Info,
        };
        Self {
            message,
            kind,
            program_id: None,
            depth: 0,
            level,
            timestamp_ms: 0,
        }
    }
}
