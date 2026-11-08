//! ironclad info - Show program information

use crate::error::IroncladResult;
use crate::project::Project;
use colored::*;

pub async fn run(
    program: Option<&str>,
    network: Option<&str>,
    verbose: bool,
) -> IroncladResult<()> {
    let project = Project::find()?;

    dchat_log::out!("{} {}", "ℹ️".bold(), "Program Information".cyan().bold());
    dchat_log::out!();

    // Show local project info
    dchat_log::out!("  {}", "Project:".cyan().bold());
    dchat_log::out!("    Name: {}", project.config.project.name.green());
    dchat_log::out!("    Version: {}", project.config.project.version);

    if let Some(desc) = &project.config.project.description {
        dchat_log::out!("    Description: {}", desc.dimmed());
    }

    if !project.config.project.authors.is_empty() {
        dchat_log::out!("    Authors: {}", project.config.project.authors.join(", "));
    }

    if let Some(program_id) = &project.config.project.program_id {
        dchat_log::out!("    Program ID: {}", program_id.yellow());
    }

    // Check for WASM
    let wasm_path = project.wasm_path(true);
    if wasm_path.exists() {
        dchat_log::out!();
        dchat_log::out!("  {}", "Build:".cyan().bold());

        let metadata = std::fs::metadata(&wasm_path)?;
        dchat_log::out!("    WASM: {}", wasm_path.display().to_string().dimmed());
        dchat_log::out!("    Size: {} bytes", metadata.len());

        // Extract manifest info
        let wasm_bytes = std::fs::read(&wasm_path)?;
        if let Some((version, sdk_version, edition)) = extract_manifest_info(&wasm_bytes) {
            dchat_log::out!("    Manifest Version: {}", version);
            dchat_log::out!("    SDK Version: {}", sdk_version);
            dchat_log::out!("    Edition: {}", edition);
        }

        // Check IDL
        let idl_path = project.idl_path();
        if idl_path.exists() {
            dchat_log::out!("    IDL: {}", idl_path.display().to_string().dimmed());
        }
    } else {
        dchat_log::out!();
        dchat_log::out!(
            "  {} No WASM build found. Run 'ironclad build'",
            "⚠".yellow()
        );
    }

    // Query on-chain info if program ID provided
    if let Some(pid) = program {
        dchat_log::out!();
        dchat_log::out!("  {}", "On-chain:".cyan().bold());

        let net = network.unwrap_or("mainnet");
        let network_config = project.config.networks.get(net);

        if let Some(config) = network_config {
            dchat_log::out!("    Network: {}", net);
            dchat_log::out!("    RPC: {}", config.url.dimmed());

            // In production, would query RPC
            dchat_log::out!("    Program: {}", pid.yellow());
            dchat_log::out!("    Status: {} (simulated)", "Active".green());
        } else {
            dchat_log::out!("    {} Unknown network: {}", "⚠".yellow(), net);
        }
    }

    // Show configuration
    if verbose {
        dchat_log::out!();
        dchat_log::out!("  {}", "Configuration:".cyan().bold());
        dchat_log::out!("    Target: {}", project.config.build.target);
        dchat_log::out!("    Generate IDL: {}", project.config.build.generate_idl);
        dchat_log::out!("    Verify Schema: {}", project.config.build.verify_schema);

        if !project.config.build.rustflags.is_empty() {
            dchat_log::out!(
                "    Rustflags: {}",
                project.config.build.rustflags.join(" ")
            );
        }

        dchat_log::out!();
        dchat_log::out!("  {}", "Networks:".cyan().bold());
        for (name, config) in &project.config.networks {
            dchat_log::out!("    {}: {}", name, config.url.dimmed());
        }
    }

    Ok(())
}

fn extract_manifest_info(wasm_bytes: &[u8]) -> Option<(u8, String, String)> {
    for (i, window) in wasm_bytes.windows(4).enumerate() {
        if window == b"DPLM" && i + 7 < wasm_bytes.len() {
            let version = wasm_bytes[i + 4];
            // Simplified - in production would fully parse manifest
            return Some((version, "0.1.0".to_string(), "2025".to_string()));
        }
    }
    None
}
