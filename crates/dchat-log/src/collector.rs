//! Host-side log collection, testing harness, and dispatch

#[cfg(feature = "std")]
use std::cell::RefCell;
#[cfg(feature = "std")]
use std::sync::RwLock;

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::entry::LogEntry;

#[cfg(feature = "std")]
thread_local! {
    /// Thread-local buffer for capturing logs during test execution
    static TEST_BUFFER: RefCell<Option<Vec<LogEntry>>> = const { RefCell::new(None) };
}

/// Global sink trait for custom host log processing (e.g. Validator / Node / RPC)
#[cfg(feature = "std")]
pub trait LogSink: Send + Sync {
    /// Record a log entry
    fn log(&self, entry: &LogEntry);
}

#[cfg(feature = "std")]
static GLOBAL_SINK: RwLock<Option<Box<dyn LogSink>>> = RwLock::new(None);

/// Register a global log sink for host execution
#[cfg(feature = "std")]
pub fn set_global_sink(sink: Box<dyn LogSink>) {
    if let Ok(mut lock) = GLOBAL_SINK.write() {
        *lock = Some(sink);
    }
}

/// Record a log entry on the host
#[cfg(feature = "std")]
pub fn record_host_entry(entry: LogEntry) {
    // 1. Priority: Thread-local test capture
    let captured = TEST_BUFFER.with(|buf| {
        let mut b = buf.borrow_mut();
        if let Some(vec) = b.as_mut() {
            vec.push(entry.clone());
            true
        } else {
            false
        }
    });

    if captured {
        return;
    }

    // 2. Global custom sink if registered
    if let Ok(lock) = GLOBAL_SINK.read() {
        if let Some(sink) = lock.as_ref() {
            sink.log(&entry);
            return;
        }
    }

    // 3. Tracing bridge if enabled
    #[cfg(feature = "tracing-compat")]
    {
        match entry.level {
            crate::entry::LogLevel::Trace => tracing::trace!(target: "dchat_log", "{}", entry.message),
            crate::entry::LogLevel::Debug => tracing::debug!(target: "dchat_log", "{}", entry.message),
            crate::entry::LogLevel::Info => tracing::info!(target: "dchat_log", "{}", entry.message),
            crate::entry::LogLevel::Warn => tracing::warn!(target: "dchat_log", "{}", entry.message),
            crate::entry::LogLevel::Error => tracing::error!(target: "dchat_log", "{}", entry.message),
        }
        return;
    }

    // 4. Default fallback: print to stderr (preserving Solana program log prefix)
    #[cfg(not(feature = "tracing-compat"))]
    {
        eprintln!("{}", entry.message);
    }
}

/// Test harness helper: capture all logs executed within the closure
#[cfg(feature = "std")]
pub fn capture_logs<F, R>(f: F) -> (R, Vec<LogEntry>)
where
    F: FnOnce() -> R,
{
    TEST_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(Vec::new());
    });

    let result = f();

    let logs = TEST_BUFFER.with(|buf| buf.borrow_mut().take().unwrap_or_default());

    (result, logs)
}

/// Test harness helper: capture logs returning only the captured vector
#[cfg(feature = "std")]
pub fn capture_logs_only<F>(f: F) -> Vec<LogEntry>
where
    F: FnOnce(),
{
    let ((), logs) = capture_logs(f);
    logs
}
