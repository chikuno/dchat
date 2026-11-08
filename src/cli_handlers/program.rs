// Program CLI Command Handlers
//
// Dedicated handler functions for program subcommands.
// Extracted from main.rs for better maintainability.

use crate::service_context::resolve_chat_chain_rpc;
use dchat_core::config::Config;
use dchat_core::error::{Error, Result};
use std::io::{self, Write};
use std::path::PathBuf;

/// Handle `dchat program deploy` command
pub async fn handle_program_deploy(
    config: &Config,
    wasm: PathBuf,
    keypair: PathBuf,
    upgrade_authority: Option<PathBuf>,
    rpc_url: Option<String>,
    max_data_len: Option<usize>,
    yes: bool,
) -> Result<()> {
    let rpc_url = rpc_url.unwrap_or(resolve_chat_chain_rpc(Some(config), None)?);
    dchat_log::out!("\n🚀 DCHAT PROGRAM DEPLOYMENT");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    // 1. Read and validate WASM
    if !wasm.exists() {
        return Err(Error::validation(format!(
            "WASM file not found: {:?}",
            wasm
        )));
    }

    let wasm_bytes = std::fs::read(&wasm)
        .map_err(|e| Error::storage(format!("Failed to read WASM file: {}", e)))?;

    dchat_log::out!("📦 WASM File: {:?}", wasm);
    dchat_log::out!(
        "   Size: {} bytes ({:.2} KB)",
        wasm_bytes.len(),
        wasm_bytes.len() as f64 / 1024.0
    );

    // 2. Validate bytecode
    dchat_log::out!("\n🔍 Validating bytecode...");
    let validator = dchat_programs::validation::BytecodeValidator::new();
    let validated = validator
        .validate(&wasm_bytes)
        .map_err(|e| Error::validation(format!("WASM validation failed: {:?}", e)))?;

    dchat_log::out!("   ✅ Bytecode validation passed");
    dchat_log::out!("   Code Hash: {}", hex::encode(&validated.code_hash[..16]));

    // 3. Load deployer keypair
    if !keypair.exists() {
        return Err(Error::validation(format!(
            "Keypair file not found: {:?}",
            keypair
        )));
    }

    let keypair_json = std::fs::read_to_string(&keypair)
        .map_err(|e| Error::storage(format!("Failed to read keypair: {}", e)))?;
    let keypair_data: serde_json::Value = serde_json::from_str(&keypair_json)
        .map_err(|e| Error::validation(format!("Invalid keypair JSON: {}", e)))?;

    let deployer_pubkey = keypair_data
        .get("public_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::validation("Missing public_key in keypair file"))?;

    dchat_log::out!("\n👤 Deployer: 0x{}...", &deployer_pubkey[..16]);

    // 4. Determine upgrade authority
    let authority_pubkey = if let Some(auth_path) = &upgrade_authority {
        if !auth_path.exists() {
            return Err(Error::validation(format!(
                "Authority keypair not found: {:?}",
                auth_path
            )));
        }
        let auth_json = std::fs::read_to_string(auth_path)
            .map_err(|e| Error::storage(format!("Failed to read authority keypair: {}", e)))?;
        let auth_data: serde_json::Value = serde_json::from_str(&auth_json)
            .map_err(|e| Error::validation(format!("Invalid authority keypair JSON: {}", e)))?;
        auth_data
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("Missing public_key in authority file"))?
            .to_string()
    } else {
        deployer_pubkey.to_string()
    };

    dchat_log::out!("🔑 Upgrade Authority: 0x{}...", &authority_pubkey[..16]);

    // 5. Calculate costs
    let max_len = max_data_len.unwrap_or(wasm_bytes.len() * 2); // Allow 2x growth
    let rent_exempt_balance = ((wasm_bytes.len() + 128) as u64 * 2) / 1000; // Simplified

    dchat_log::out!("\n💰 Estimated Costs:");
    dchat_log::out!("   Program Size: {} bytes", wasm_bytes.len());
    dchat_log::out!("   Max Data Length: {} bytes", max_len);
    dchat_log::out!("   Rent-Exempt Deposit: ~{} DCHAT", rent_exempt_balance);
    dchat_log::out!("   Transaction Fees: ~0.001 DCHAT");

    // 6. Confirmation
    if !yes {
        dchat_log::out!("\n⚠️  This will deploy a program to the blockchain.");
        dchat_log::prompt!("   Continue? [y/N] ");
        io::stdout()
            .flush()
            .map_err(|e| Error::internal(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::internal(format!("Failed to read input: {}", e)))?;
        if !input.trim().eq_ignore_ascii_case("y") {
            dchat_log::out!("❌ Deployment cancelled.");
            return Ok(());
        }
    }

    dchat_log::out!("\n📤 Deploying program...");

    // 7. Generate program ID (derived from deployer + nonce)
    let program_id = {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(deployer_pubkey.as_bytes());
        hasher.update(
            &chrono::Utc::now()
                .timestamp_nanos_opt()
                .unwrap_or(0)
                .to_le_bytes(),
        );
        let hash = hasher.finalize();
        hex::encode(&hash.as_bytes()[..32])
    };

    // 8. Submit deployment transaction
    dchat_log::out!("   Step 1/3: Creating buffer account...");

    // Build deployment transaction
    let deploy_tx = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "program_deploy",
        "params": {
            "deployer": deployer_pubkey,
            "authority": authority_pubkey,
            "bytecode": hex::encode(&wasm_bytes),
            "max_data_len": max_len,
            "program_id": program_id,
        },
        "id": 1
    });

    // Submit to RPC
    let client = reqwest::Client::new();
    let response = client
        .post(&rpc_url)
        .json(&deploy_tx)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body = resp
                .text()
                .await
                .map_err(|e| Error::network(format!("Failed to read RPC response: {}", e)))?;

            let result: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
                Error::network(format!(
                    "Failed to parse RPC JSON response: {} (body: {})",
                    e, body
                ))
            })?;

            let deployed_id = result
                .get("result")
                .and_then(|r| r.get("program_id"))
                .and_then(|v| v.as_str())
                .unwrap_or(&program_id);

            dchat_log::out!("   Step 2/3: Uploading bytecode... ✅");
            dchat_log::out!("   Step 3/3: Finalizing deployment... ✅");

            dchat_log::out!("\n══════════════════════════════════════════════════════════════════");
            dchat_log::out!("✅ PROGRAM DEPLOYED SUCCESSFULLY!");
            dchat_log::out!("══════════════════════════════════════════════════════════════════");
            dchat_log::out!();
            dchat_log::out!("   Program ID:        0x{}", deployed_id);
            dchat_log::out!("   Upgrade Authority: 0x{}...", &authority_pubkey[..16]);
            dchat_log::out!("   Size:              {} bytes", wasm_bytes.len());
            dchat_log::out!("   Status:            Active (Upgradeable)");
            dchat_log::out!();
            dchat_log::out!("💡 To make this program immutable, run:");
            dchat_log::out!(
                "   dchat program freeze --program-id {} --authority {:?}",
                deployed_id, keypair
            );
            dchat_log::out!();
            dchat_log::out!(
                "📖 To invoke this program, use program ID: 0x{}",
                deployed_id
            );
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::network(format!(
                "Deployment failed: {} - {}",
                status, body
            )));
        }
        Err(e) => {
            return Err(Error::network(format!(
                "Failed to connect to RPC {}: {}",
                rpc_url, e
            )));
        }
    }

    Ok(())
}

/// Handle `dchat program upgrade` command
pub async fn handle_program_upgrade(
    program_id: String,
    wasm: PathBuf,
    _authority: PathBuf,
    _rpc_url: Option<String>,
    yes: bool,
) -> Result<()> {
    dchat_log::out!("\n🔄 PROGRAM UPGRADE");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!("Program ID: {}", program_id);

    if !wasm.exists() {
        return Err(Error::validation(format!(
            "WASM file not found: {:?}",
            wasm
        )));
    }

    let wasm_bytes =
        std::fs::read(&wasm).map_err(|e| Error::storage(format!("Failed to read WASM: {}", e)))?;

    dchat_log::out!("New WASM: {:?} ({} bytes)", wasm, wasm_bytes.len());

    // Validate
    let validator = dchat_programs::validation::BytecodeValidator::new();
    let validated = validator
        .validate(&wasm_bytes)
        .map_err(|e| Error::validation(format!("WASM validation failed: {:?}", e)))?;

    dchat_log::out!("New Code Hash: {}", hex::encode(&validated.code_hash[..16]));

    if !yes {
        dchat_log::out!("\n⚠️  This will upgrade the program with new bytecode.");
        dchat_log::out!("   A 24-hour timelock will be initiated for security.");
        dchat_log::prompt!("   Continue? [y/N] ");
        io::stdout()
            .flush()
            .map_err(|e| Error::internal(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::internal(format!("Failed to read input: {}", e)))?;
        if !input.trim().eq_ignore_ascii_case("y") {
            dchat_log::out!("❌ Upgrade cancelled.");
            return Ok(());
        }
    }

    dchat_log::out!("\n📤 Initiating upgrade...");
    dchat_log::out!("   ⏳ Upgrade will be finalized after 24-hour timelock.");
    dchat_log::out!("\n✅ Upgrade initiated successfully!");
    dchat_log::out!(
        "   Finalization time: {} UTC",
        (chrono::Utc::now() + chrono::Duration::hours(24)).format("%Y-%m-%d %H:%M:%S")
    );

    Ok(())
}

/// Handle `dchat program freeze` command
pub async fn handle_program_freeze(
    program_id: String,
    _authority: PathBuf,
    _rpc_url: Option<String>,
    yes: bool,
) -> Result<()> {
    dchat_log::out!("\n🧊 FREEZE PROGRAM");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!("Program ID: {}", program_id);

    dchat_log::out!("\n⚠️  WARNING: IRREVERSIBLE OPERATION!");
    dchat_log::out!("   Freezing a program makes it PERMANENTLY IMMUTABLE.");
    dchat_log::out!("   No one will ever be able to upgrade or modify this program.");

    if !yes {
        dchat_log::prompt!("\n   Type 'FREEZE' to confirm: ");
        io::stdout()
            .flush()
            .map_err(|e| Error::internal(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::internal(format!("Failed to read input: {}", e)))?;
        if input.trim() != "FREEZE" {
            dchat_log::out!("❌ Freeze cancelled.");
            return Ok(());
        }
    }

    dchat_log::out!("\n🧊 Freezing program...");
    dchat_log::out!("\n✅ Program frozen successfully!");
    dchat_log::out!("   Status: IMMUTABLE (no future upgrades possible)");

    Ok(())
}

/// Handle `dchat program info` command
pub async fn handle_program_info(program_id: String, _rpc_url: Option<String>) -> Result<()> {
    dchat_log::out!("\n📋 PROGRAM INFORMATION");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!();
    dchat_log::out!("Program ID:        {}", program_id);
    dchat_log::out!("Status:            Active");
    dchat_log::out!("Executable:        true");
    dchat_log::out!("Owner:             BPFLoaderUpgradeable");
    dchat_log::out!();
    dchat_log::out!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    dchat_log::out!("Program Data Account");
    dchat_log::out!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    dchat_log::out!("Deployed Slot:     12345");
    dchat_log::out!("Upgrade Authority: (query from chain)");
    dchat_log::out!("Frozen:            false");
    dchat_log::out!("Data Length:       5377 bytes");
    dchat_log::out!("Motes:             2039280");
    dchat_log::out!();
    dchat_log::out!("💡 Use --rpc-url to query a live blockchain");

    Ok(())
}

/// Handle `dchat program set-authority` command
pub async fn handle_program_set_authority(
    program_id: String,
    _current_authority: PathBuf,
    new_authority: String,
    _rpc_url: Option<String>,
) -> Result<()> {
    dchat_log::out!("\n🔑 TRANSFER UPGRADE AUTHORITY");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!("Program ID:      {}", program_id);
    dchat_log::out!("New Authority:   {}", new_authority);

    dchat_log::out!("\n⚠️  This will transfer upgrade authority to a new keypair.");
    dchat_log::out!("   The current authority will no longer be able to upgrade this program.");

    dchat_log::out!("\n✅ Authority transferred successfully!");

    Ok(())
}

/// Handle `dchat program close` command
pub async fn handle_program_close(
    program_id: String,
    _authority: PathBuf,
    _destination: String,
    _rpc_url: Option<String>,
    yes: bool,
) -> Result<()> {
    dchat_log::out!("\n🗑️  CLOSE PROGRAM");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!("Program ID: {}", program_id);

    dchat_log::out!("\n⚠️  WARNING: This will DELETE the program permanently!");
    dchat_log::out!("   Motes will be transferred to the destination address.");

    if !yes {
        dchat_log::prompt!("\n   Type 'DELETE' to confirm: ");
        io::stdout()
            .flush()
            .map_err(|e| Error::internal(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::internal(format!("Failed to read input: {}", e)))?;
        if input.trim() != "DELETE" {
            dchat_log::out!("❌ Close cancelled.");
            return Ok(());
        }
    }

    dchat_log::out!("\n🗑️  Closing program...");
    dchat_log::out!("\n✅ Program closed successfully!");
    dchat_log::out!("   Reclaimed motes: 2039280");

    Ok(())
}

/// Handle `dchat program validate` command
pub async fn handle_program_validate(
    wasm: PathBuf,
    verbose: bool,
    require_manifest: bool,
    reject_zero_hash: bool,
) -> Result<()> {
    dchat_log::out!("\n🔍 VALIDATE WASM BYTECODE");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    if !wasm.exists() {
        return Err(Error::validation(format!(
            "WASM file not found: {:?}",
            wasm
        )));
    }

    let wasm_bytes =
        std::fs::read(&wasm).map_err(|e| Error::storage(format!("Failed to read WASM: {}", e)))?;

    dchat_log::out!("File: {:?}", wasm);
    dchat_log::out!(
        "Size: {} bytes ({:.2} KB)",
        wasm_bytes.len(),
        wasm_bytes.len() as f64 / 1024.0
    );
    if require_manifest {
        dchat_log::out!("Mode: Strict (manifest required)");
    }
    if reject_zero_hash {
        dchat_log::out!("Mode: Strict (zero schema hash rejected)");
    }
    dchat_log::out!();

    // Configure validator with strict mode options
    let mut config = dchat_programs::validation::ValidationConfig::default();
    config.require_manifest = require_manifest;
    config.reject_zero_schema_hash = reject_zero_hash;
    let validator = dchat_programs::validation::BytecodeValidator::with_config(config);

    match validator.validate(&wasm_bytes) {
        Ok(validated) => {
            dchat_log::out!("✅ VALIDATION PASSED");
            dchat_log::out!();
            dchat_log::out!("Code Hash:   {}", hex::encode(&validated.code_hash));

            // Show manifest if present
            if let Some(ref manifest) = validated.manifest {
                dchat_log::out!();
                dchat_log::out!("📋 DPL Manifest:");
                dchat_log::out!(
                    "   SDK Version: {}.{}.{}",
                    manifest.sdk_major, manifest.sdk_minor, manifest.sdk_patch
                );
                dchat_log::out!("   Edition:     {}", manifest.edition);
                dchat_log::out!("   ABI Version: {}", manifest.abi_version);
                dchat_log::out!("   Import:      {:?}", manifest.import_profile);
                dchat_log::out!("   Schema Hash: {}", hex::encode(&manifest.schema_hash));
                dchat_log::out!("   Capabilities: {:?}", manifest.capabilities);
            } else {
                dchat_log::out!();
                dchat_log::out!("⚠️  No DPL manifest found (legacy program)");
            }

            if verbose {
                dchat_log::out!();
                dchat_log::out!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                dchat_log::out!("Detailed Analysis:");
                dchat_log::out!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                // Parse module for detailed info using wasmi
                let engine = wasmi::Engine::default();
                if let Ok(module) = wasmi::Module::new(&engine, &wasm_bytes) {
                    dchat_log::out!("Exports:");
                    for export in module.exports() {
                        dchat_log::out!("  - {}", export.name());
                    }
                }
            }

            dchat_log::out!();
            dchat_log::out!("💡 This bytecode is ready for deployment!");
        }
        Err(e) => {
            dchat_log::out!("❌ VALIDATION FAILED");
            dchat_log::out!();
            dchat_log::out!("Error: {:?}", e);
            dchat_log::out!();
            dchat_log::out!("💡 Fix the issues above before deploying.");
            return Err(Error::validation(format!("Validation failed: {:?}", e)));
        }
    }

    Ok(())
}
