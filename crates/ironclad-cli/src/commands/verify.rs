//! ironclad verify - Verify manifest and schema

use crate::error::{IroncladError, IroncladResult};
use crate::project::Project;
use colored::*;
use std::process::Command;

pub async fn run(
    program: Option<&str>,
    schema_hash: Option<&str>,
    verbose: bool,
) -> IroncladResult<()> {
    let project = Project::find()?;

    dchat_log::out!(
        "{} {} {}",
        "🔍".bold(),
        "Verifying".cyan().bold(),
        project.config.project.name.green()
    );

    let wasm_path = if let Some(p) = program {
        std::path::PathBuf::from(p)
    } else {
        project.wasm_path(true)
    };

    if !wasm_path.exists() {
        return Err(IroncladError::WasmError(format!(
            "WASM file not found: {}. Run 'ironclad build --release' first.",
            wasm_path.display()
        )));
    }

    // Step 1: Validate WASM bytecode
    dchat_log::out!("  {} Validating WASM bytecode...", "→".dimmed());

    let validate_output = Command::new("dchat")
        .arg("program")
        .arg("validate")
        .arg("--wasm")
        .arg(&wasm_path)
        .arg("--verbose")
        .output()
        .map_err(|e| IroncladError::CommandFailed(format!("Failed to run dchat: {}", e)))?;

    if !validate_output.status.success() {
        let stderr = String::from_utf8_lossy(&validate_output.stderr);
        return Err(IroncladError::VerificationFailed(format!(
            "WASM validation failed: {}",
            stderr
        )));
    }

    dchat_log::out!("  {} WASM bytecode valid", "✓".green());

    if verbose {
        let stdout = String::from_utf8_lossy(&validate_output.stdout);
        for line in stdout.lines() {
            dchat_log::out!("    {}", line.dimmed());
        }
    }

    // Step 2: Extract and display manifest
    dchat_log::out!("  {} Checking manifest...", "→".dimmed());

    let manifest_output = Command::new("dchat")
        .arg("program")
        .arg("manifest")
        .arg("--program")
        .arg(&wasm_path)
        .output()
        .map_err(|e| IroncladError::CommandFailed(format!("Failed to run dchat: {}", e)))?;

    if !manifest_output.status.success() {
        return Err(IroncladError::MissingManifest(
            "No manifest found in WASM".to_string(),
        ));
    }

    let manifest_info = String::from_utf8_lossy(&manifest_output.stdout);
    dchat_log::out!("  {} Manifest found", "✓".green());

    if verbose {
        for line in manifest_info.lines() {
            dchat_log::out!("    {}", line.dimmed());
        }
    }

    // Step 3: Verify schema hash if provided
    if let Some(expected_hash) = schema_hash {
        dchat_log::out!("  {} Verifying schema hash...", "→".dimmed());

        let verify_output = Command::new("dchat")
            .arg("program")
            .arg("verify-manifest")
            .arg("--program")
            .arg(&wasm_path)
            .arg("--expected-hash")
            .arg(expected_hash)
            .output()
            .map_err(|e| IroncladError::CommandFailed(format!("Failed to run dchat: {}", e)))?;

        if !verify_output.status.success() {
            let stderr = String::from_utf8_lossy(&verify_output.stderr);
            return Err(IroncladError::SchemaMismatch(format!(
                "Schema hash mismatch: {}",
                stderr
            )));
        }

        dchat_log::out!(
            "  {} Schema hash verified: {}",
            "✓".green(),
            &expected_hash[..16.min(expected_hash.len())]
                .to_string()
                .dimmed()
        );
    }

    // Step 4: Check build reproducibility
    if project.config.build.verify_schema {
        dchat_log::out!("  {} Checking reproducibility...", "→".dimmed());
        // In production, would rebuild and compare hashes
        dchat_log::out!("  {} Build reproducibility check skipped", "⚠".yellow());
    }

    dchat_log::out!();
    dchat_log::out!("{}", "✅ Verification passed".green().bold());

    Ok(())
}
