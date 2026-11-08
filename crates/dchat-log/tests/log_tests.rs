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
    assert_eq!(logs[0].message, "[INFO] Server started on port 8080");
    assert_eq!(logs[1].level, LogLevel::Warn);
    assert_eq!(logs[1].message, "[WARN] High memory consumption detected");
    assert_eq!(logs[2].level, LogLevel::Error);
    assert_eq!(logs[2].message, "[ERROR] Database connection dropped");
    assert_eq!(logs[3].level, LogLevel::Debug);
    assert_eq!(logs[4].level, LogLevel::Trace);
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
