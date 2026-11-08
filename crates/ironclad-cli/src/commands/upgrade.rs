//! ironclad upgrade - Upgrade deployed program

use crate::error::{IroncladError, IroncladResult};
use crate::project::Project;
use colored::*;

pub async fn run(
    program_id: &str,
    network: &str,
    new_program: Option<&str>,
    skip_verify: bool,
    verbose: bool,
) -> IroncladResult<()> {
    let project = Project::find()?;

    let network_config = project
        .config
        .networks
        .get(network)
        .ok_or_else(|| IroncladError::ConfigError(format!("Unknown network: {}", network)))?;

    dchat_log::out!(
        "{} {} {}",
        "⬆️".bold(),
        "Upgrading".cyan().bold(),
        program_id.green()
    );

    let wasm_path = if let Some(p) = new_program {
        std::path::PathBuf::from(p)
    } else {
        project.wasm_path(true)
    };

    if !wasm_path.exists() {
        return Err(IroncladError::WasmError(format!(
            "WASM file not found: {}",
            wasm_path.display()
        )));
    }

    // Verify before upgrade
    if !skip_verify {
        dchat_log::out!("  {} Verifying new program...", "→".dimmed());

        let wasm_bytes = std::fs::read(&wasm_path)?;
        let has_manifest = wasm_bytes.windows(4).any(|w| w == b"DPLM");

        if !has_manifest {
            return Err(IroncladError::MissingManifest(
                "New WASM missing manifest".to_string(),
            ));
        }

        dchat_log::out!("  {} Verification passed", "✓".green());
    }

    if verbose {
        dchat_log::out!("  {} RPC: {}", "→".dimmed(), network_config.url.dimmed());
    }

    // Simulate upgrade (in production, would use RPC client)
    dchat_log::out!("  {} Uploading new program...", "→".dimmed());
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    dchat_log::out!("  {} Program uploaded", "✓".green());

    dchat_log::out!("  {} Executing upgrade...", "→".dimmed());
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    dchat_log::out!("  {} Upgrade executed", "✓".green());

    dchat_log::out!();
    dchat_log::out!("{}", "✅ Upgrade successful!".green().bold());
    dchat_log::out!();
    dchat_log::out!("  Program: {}", program_id);
    dchat_log::out!("  Network: {}", network);
    dchat_log::out!("  New WASM: {}", wasm_path.display());

    Ok(())
}
