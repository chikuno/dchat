//! CLI and user-facing output engine
//!
//! Provides clean, structured terminal output, quiet mode, JSON mode, and
//! first-class replacements for `println!`, `eprintln!`, and `dbg!`.

#[cfg(feature = "std")]
use crate::entry::LogKind;

/// Output mode for CLI and runtime interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OutputMode {
    /// Standard human-readable terminal output
    Standard = 0,
    /// Machine-readable JSON output
    Json = 1,
    /// Quiet mode: suppress stdout, only emit errors
    Quiet = 2,
    /// Daemon mode: prefix logs with timestamps and targets
    Daemon = 3,
}

#[cfg(feature = "std")]
mod state {
    use super::OutputMode;
    use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

    static OUTPUT_MODE: AtomicU8 = AtomicU8::new(OutputMode::Standard as u8);
    static COLOR_ENABLED: AtomicBool = AtomicBool::new(true);

    pub fn set_output_mode(mode: OutputMode) {
        OUTPUT_MODE.store(mode as u8, Ordering::Relaxed);
    }

    pub fn get_output_mode() -> OutputMode {
        match OUTPUT_MODE.load(Ordering::Relaxed) {
            1 => OutputMode::Json,
            2 => OutputMode::Quiet,
            3 => OutputMode::Daemon,
            _ => OutputMode::Standard,
        }
    }

    pub fn set_quiet(quiet: bool) {
        if quiet {
            set_output_mode(OutputMode::Quiet);
        } else if get_output_mode() == OutputMode::Quiet {
            set_output_mode(OutputMode::Standard);
        }
    }

    pub fn is_quiet() -> bool {
        get_output_mode() == OutputMode::Quiet
    }

    pub fn set_json_mode(json: bool) {
        if json {
            set_output_mode(OutputMode::Json);
        } else if get_output_mode() == OutputMode::Json {
            set_output_mode(OutputMode::Standard);
        }
    }

    pub fn is_json_mode() -> bool {
        get_output_mode() == OutputMode::Json
    }

    pub fn set_color_enabled(enabled: bool) {
        COLOR_ENABLED.store(enabled, Ordering::Relaxed);
    }

    pub fn is_color_enabled() -> bool {
        COLOR_ENABLED.load(Ordering::Relaxed)
    }
}

#[cfg(feature = "std")]
pub use state::*;

// ═══════════════════════════════════════════════════════════════════════════════
// CLI OUTPUT DISPATCH
// ═══════════════════════════════════════════════════════════════════════════════

/// Emit standard user-facing output (replaces `println!`)
pub fn cli_out(msg: &str) {
    #[cfg(feature = "std")]
    {
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliOut, msg.to_string());
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_quiet() {
            return;
        }

        if is_json_mode() {
            println!("{{\"type\":\"out\",\"message\":{}}}", escape_json(msg));
        } else {
            println!("{}", msg);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        crate::syscalls::log(msg);
    }
}

/// Emit user-facing error output (replaces `eprintln!`)
pub fn cli_err(msg: &str) {
    #[cfg(feature = "std")]
    {
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliErr, msg.to_string());
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_json_mode() {
            eprintln!("{{\"type\":\"err\",\"message\":{}}}", escape_json(msg));
        } else {
            eprintln!("{}", msg);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        crate::syscalls::log(msg);
    }
}

/// Emit a success banner (green checkmark or JSON success status)
pub fn cli_success(msg: &str) {
    #[cfg(feature = "std")]
    {
        let formatted = format!("✓ {}", msg);
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliSuccess, formatted);
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_quiet() {
            return;
        }

        if is_json_mode() {
            println!(
                "{{\"type\":\"success\",\"status\":\"ok\",\"message\":{}}}",
                escape_json(msg)
            );
        } else if is_color_enabled() {
            println!("\x1b[32m✓\x1b[0m {}", msg);
        } else {
            println!("✓ {}", msg);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        crate::syscalls::log(msg);
    }
}

/// Emit a failure banner (red cross or JSON error status)
pub fn cli_failure(msg: &str) {
    #[cfg(feature = "std")]
    {
        let formatted = format!("✗ {}", msg);
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliFailure, formatted);
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_json_mode() {
            eprintln!(
                "{{\"type\":\"failure\",\"status\":\"error\",\"message\":{}}}",
                escape_json(msg)
            );
        } else if is_color_enabled() {
            eprintln!("\x1b[31m✗\x1b[0m {}", msg);
        } else {
            eprintln!("✗ {}", msg);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        crate::syscalls::log(msg);
    }
}

/// Emit a bold section header
pub fn cli_header(title: &str) {
    #[cfg(feature = "std")]
    {
        let formatted = format!("=== {} ===", title);
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliHeader, formatted);
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_quiet() {
            return;
        }

        if is_json_mode() {
            println!("{{\"type\":\"header\",\"title\":{}}}", escape_json(title));
        } else if is_color_enabled() {
            println!("\n\x1b[1;36m=== {} ===\x1b[0m", title);
        } else {
            println!("\n=== {} ===", title);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        crate::syscalls::log(title);
    }
}

/// Emit an aligned key-value pair
pub fn cli_kv(key: &str, val: &str) {
    #[cfg(feature = "std")]
    {
        let formatted = format!("{}: {}", key, val);
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliKv, formatted);
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_quiet() {
            return;
        }

        if is_json_mode() {
            println!(
                "{{\"type\":\"kv\",\"key\":{},\"value\":{}}}",
                escape_json(key),
                escape_json(val)
            );
        } else if is_color_enabled() {
            println!("  \x1b[1m{}:\x1b[0m {}", key, val);
        } else {
            println!("  {}: {}", key, val);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        let _ = (key, val);
    }
}

/// Emit a safe diagnostic trace (replaces `dbg!`)
pub fn cli_dbg(expr_str: &str, file: &str, line: u32, val_str: &str) {
    #[cfg(feature = "std")]
    {
        let msg = if expr_str.is_empty() {
            format!("[DEBUG {}:{}]", file, line)
        } else {
            format!("[DEBUG {}:{}] {} = {}", file, line, expr_str, val_str)
        };

        let entry = crate::entry::LogEntry::new_cli(LogKind::Debug, msg.clone());
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_json_mode() {
            eprintln!(
                "{{\"type\":\"debug\",\"file\":{},\"line\":{},\"expr\":{},\"value\":{}}}",
                escape_json(file),
                line,
                escape_json(expr_str),
                escape_json(val_str)
            );
        } else if is_color_enabled() {
            eprintln!("\x1b[33m[DEBUG {}:{}]\x1b[0m {} = {}", file, line, expr_str, val_str);
        } else {
            eprintln!("{}", msg);
        }
    }

    #[cfg(not(feature = "std"))]
    {
        let _ = (expr_str, file, line, val_str);
    }
}

#[cfg(feature = "std")]
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}


/// Emit an interactive prompt without trailing newline and flush stdout (replaces `print!`)
pub fn cli_prompt(msg: &str) {
    #[cfg(feature = "std")]
    {
        use std::io::Write;
        let entry = crate::entry::LogEntry::new_cli(LogKind::CliOut, msg.to_string());
        if crate::collector::record_host_test_capture(&entry) {
            return;
        }

        if is_quiet() {
            return;
        }

        print!("{}", msg);
        let _ = std::io::stdout().flush();
    }

    #[cfg(not(feature = "std"))]
    {
        crate::syscalls::log(msg);
    }
}
