use dchat_log::prelude::*;
use dchat_log::testing::capture_logs_only;

#[test]
fn test_msg_formatting_and_capture() {
    let logs = capture_logs_only(|| {
        msg!("Simple static message");
        let amount = 1000u64;
        let recipient = "alice";
        msg!("Transferred {} motes to {}", amount, recipient);
    });

    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].message, "Program log: Simple static message");
    assert_eq!(logs[1].message, "Program log: Transferred 1000 motes to alice");
    assert_eq!(logs[0].kind, LogKind::Program);
}

#[test]
fn test_log_64_variations() {
    let logs = capture_logs_only(|| {
        // Full 5 values
        log_64!(10, 20, 30, 40, 50);
        // Partial values (zero padded)
        log_64!(0xdead, 0xbeef);
        log_64!(99);
    });

    assert_eq!(logs.len(), 3);
    assert_eq!(logs[0].message, "Program log_64: 0xa, 0x14, 0x1e, 0x28, 0x32");
    assert_eq!(logs[1].message, "Program log_64: 0xdead, 0xbeef, 0x0, 0x0, 0x0");
    assert_eq!(logs[2].message, "Program log_64: 0x63, 0x0, 0x0, 0x0, 0x0");
}

#[test]
fn test_log_pubkey() {
    let pubkey = [0x42u8; 32];
    let logs = capture_logs_only(|| {
        log_pubkey!(&pubkey);
    });

    assert_eq!(logs.len(), 1);
    assert!(logs[0].message.starts_with("Program log: "));
}

#[test]
fn test_log_data_base64() {
    let data = b"Hello Dchat Log!";
    let logs = capture_logs_only(|| {
        log_data!(data);
    });

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "Program data: SGVsbG8gRGNoYXQgTG9nIQ==");
}

#[test]
fn test_compute_units_log() {
    let logs = capture_logs_only(|| {
        log_compute_units!();
    });

    assert_eq!(logs.len(), 1);
    assert!(logs[0].message.contains("compute units"));
}

#[test]
fn test_leveled_logging() {
    let logs = capture_logs_only(|| {
        info!("Server started on port {}", 8080);
        warn!("High memory consumption detected");
        error!("Database connection dropped");
        debug!("Processing packet header: len={}", 64);
        trace!("Entering function compute_root");
    });

    assert_eq!(logs.len(), 5);
    assert_eq!(logs[0].level, LogLevel::Info);
    assert_eq!(logs[0].kind, LogKind::System);
    assert_eq!(logs[0].message, "[INFO] Server started on port 8080");
    assert_eq!(logs[1].level, LogLevel::Warn);
    assert_eq!(logs[1].message, "[WARN] High memory consumption detected");
    assert_eq!(logs[2].level, LogLevel::Error);
    assert_eq!(logs[2].message, "[ERROR] Database connection dropped");
    assert_eq!(logs[3].level, LogLevel::Debug);
    assert_eq!(logs[4].level, LogLevel::Trace);
}

#[test]
fn test_cli_output_macros() {
    let logs = capture_logs_only(|| {
        out!("Welcome to Dchat CLI v{}", "1.0.0");
        err!("Warning: Config file not found, using defaults");
        success!("Connected to bootstrap node {}", "127.0.0.1:8000");
        failure!("Failed to sync block {}", 404);
        header!("Account Status");
        kv!("Balance", "1,500.00 DCHAT");
    });

    assert_eq!(logs.len(), 6);
    assert_eq!(logs[0].kind, LogKind::CliOut);
    assert_eq!(logs[0].message, "Welcome to Dchat CLI v1.0.0");

    assert_eq!(logs[1].kind, LogKind::CliErr);
    assert_eq!(logs[1].message, "Warning: Config file not found, using defaults");

    assert_eq!(logs[2].kind, LogKind::CliSuccess);
    assert!(logs[2].message.contains("Connected to bootstrap node"));

    assert_eq!(logs[3].kind, LogKind::CliFailure);
    assert!(logs[3].message.contains("Failed to sync block 404"));

    assert_eq!(logs[4].kind, LogKind::CliHeader);
    assert_eq!(logs[4].message, "=== Account Status ===");

    assert_eq!(logs[5].kind, LogKind::CliKv);
    assert_eq!(logs[5].message, "Balance: 1,500.00 DCHAT");
}

#[test]
fn test_dbg_macro_passthrough() {
    let logs = capture_logs_only(|| {
        let x = 42;
        let y = dchat_dbg!(x * 2);
        assert_eq!(y, 84);

        // Also test direct alias
        let z = dchat_log::dbg!(y + 16);
        assert_eq!(z, 100);
    });

    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].kind, LogKind::Debug);
    assert!(logs[0].message.contains("x * 2 = 84"));
    assert_eq!(logs[1].kind, LogKind::Debug);
    assert!(logs[1].message.contains("y + 16 = 100"));
}

#[test]
fn test_cli_mode_flags() {
    set_quiet(true);
    assert!(is_quiet());
    assert_eq!(get_output_mode(), OutputMode::Quiet);

    set_quiet(false);
    assert!(!is_quiet());
    assert_eq!(get_output_mode(), OutputMode::Standard);

    set_json_mode(true);
    assert!(is_json_mode());
    assert_eq!(get_output_mode(), OutputMode::Json);

    set_json_mode(false);
    assert!(!is_json_mode());
    assert_eq!(get_output_mode(), OutputMode::Standard);
}

#[test]
fn test_solana_compatibility_aliases() {
    let logs = capture_logs_only(|| {
        sol_log!("Solana program log compatibility test");
        sol_log_64!(1, 2, 3, 4, 5);
        sol_log_compute_units!();
        sol_log_data!(b"solana");
    });

    assert_eq!(logs.len(), 4);
    assert_eq!(logs[0].message, "Program log: Solana program log compatibility test");
    assert_eq!(logs[1].message, "Program log_64: 0x1, 0x2, 0x3, 0x4, 0x5");
    assert!(logs[2].message.contains("compute units"));
    assert_eq!(logs[3].message, "Program data: c29sYW5h");
}
