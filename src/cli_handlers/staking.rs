// Staking CLI Command Handlers
//
// Dedicated handler functions for staking subcommands.
// Extracted from main.rs for better maintainability.

use dchat_blockchain::currency_chain::{CurrencyChainClient, CurrencyChainConfig};
use dchat_blockchain::staking::StakingManager;
use dchat_core::config::Config;
use dchat_core::error::{Error, Result};
use dchat_core::UserId;
use ed25519_dalek::VerifyingKey;
use std::sync::Arc;
use uuid::Uuid;

use crate::service_context::resolve_currency_chain_rpc;

/// Format token amounts for display (assumes 6 decimal places)
fn format_tokens(amount: u64) -> String {
    let whole = amount / 1_000_000;
    let frac = amount % 1_000_000;
    if frac == 0 {
        format!("{}", whole)
    } else {
        format!("{}.{:06}", whole, frac)
            .trim_end_matches('0')
            .to_string()
    }
}

/// Truncate string for display
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Initialize currency chain client and staking manager
async fn init_staking_context(
    config: &Config,
) -> Result<(Arc<CurrencyChainClient>, StakingManager, String)> {
    let currency_rpc_url = resolve_currency_chain_rpc(Some(config), None)?;
    let mut chain_config = CurrencyChainConfig::default();
    chain_config.rpc_url = currency_rpc_url.clone();

    let currency_chain = Arc::new(
        CurrencyChainClient::new(chain_config)
            .map_err(|e| Error::chain(format!("Failed to initialize currency chain: {}", e)))?,
    );

    let staking_manager = StakingManager::with_currency_chain(Arc::clone(&currency_chain));

    Ok((currency_chain, staking_manager, currency_rpc_url))
}

/// Handle `dchat staking stake` command
pub async fn handle_stake(
    config: &Config,
    user_id: String,
    amount: u64,
    duration_days: u32,
) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    let (_, staking_manager, currency_rpc_url) = init_staking_context(config).await?;

    dchat_log::out!("\n💎 Staking Tokens:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);
    dchat_log::out!("Amount: {} DCHAT", format_tokens(amount));
    dchat_log::out!("Lock Period: {} days", duration_days);

    // Validate minimum requirements
    if amount < 1000 {
        return Err(Error::validation("Minimum stake is 1,000 DCHAT"));
    }

    if duration_days < 7 {
        return Err(Error::validation("Minimum lock period is 7 days"));
    }

    // Load validator's signing key to generate public key
    let key_path = config
        .storage
        .data_dir
        .join("keys")
        .join(format!("{}.json", user_id));

    let validator_pubkey = if key_path.exists() {
        let key_data = std::fs::read_to_string(&key_path)?;
        let key_json: serde_json::Value = serde_json::from_str(&key_data)?;
        let pubkey_hex = key_json
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("No public_key in key file"))?;
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| Error::validation(format!("Invalid pubkey hex: {}", e)))?;
        let pubkey_array: [u8; 32] = pubkey_bytes
            .try_into()
            .map_err(|_| Error::validation("Public key must be 32 bytes"))?;
        VerifyingKey::from_bytes(&pubkey_array)
            .map_err(|e| Error::crypto(format!("Invalid public key: {}", e)))?
    } else {
        return Err(Error::validation(format!(
            "Validator key file not found: {:?}. Create one with: dchat account create --username <name> --save-to {:?}",
            key_path, key_path
        )));
    };

    dchat_log::out!("\n⏳ Submitting stake transaction to currency chain...");
    dchat_log::out!("   RPC: {}", currency_rpc_url);

    // Submit stake via StakingManager (on-chain)
    let tx_id = staking_manager
        .submit_validator_stake(uid.clone(), amount, validator_pubkey)
        .await?;

    dchat_log::out!();
    dchat_log::out!("✅ Stake submitted successfully!");
    dchat_log::out!("   Transaction ID: {}", tx_id);
    dchat_log::out!("   Expected APY: ~12%");
    dchat_log::out!(
        "   Unlock Date: {}",
        chrono::Utc::now() + chrono::Duration::days(duration_days as i64)
    );
    dchat_log::out!();
    dchat_log::out!(
        "Check status with: dchat staking status --user-id {}",
        user_id
    );

    Ok(())
}

/// Handle `dchat staking unstake` command
pub async fn handle_unstake(config: &Config, user_id: String, amount: u64) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    let (currency_chain, staking_manager, currency_rpc_url) = init_staking_context(config).await?;

    let unstake_amount = if amount == 0 {
        // Get current stake to unstake all
        match currency_chain.get_wallet(&uid) {
            Ok(Some(wallet)) => wallet.staked,
            Ok(None) => return Err(Error::validation("No wallet found for user")),
            Err(e) => return Err(Error::chain(format!("Failed to query wallet: {}", e))),
        }
    } else {
        amount
    };

    dchat_log::out!("\n🔓 Unstaking Tokens:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);
    dchat_log::out!("Amount: {} DCHAT", format_tokens(unstake_amount));
    dchat_log::out!();
    dchat_log::out!("⏳ Submitting unstake transaction...");
    dchat_log::out!("   RPC: {}", currency_rpc_url);

    // Submit unstake via StakingManager (on-chain)
    let tx_id = staking_manager
        .submit_validator_unstake(&uid, unstake_amount)
        .await?;

    dchat_log::out!();
    dchat_log::out!("⏳ Unbonding period: 21 days");
    dchat_log::out!("✅ Unstake request submitted!");
    dchat_log::out!("   Transaction ID: {}", tx_id);
    dchat_log::out!(
        "   Funds will be available: {}",
        (chrono::Utc::now() + chrono::Duration::days(21)).format("%Y-%m-%d")
    );

    Ok(())
}

/// Handle `dchat staking status` command
pub async fn handle_status(config: &Config, user_id: String) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    let (currency_chain, _, currency_rpc_url) = init_staking_context(config).await?;

    dchat_log::out!("\n📊 Staking Status:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);
    dchat_log::out!("RPC: {}", currency_rpc_url);
    dchat_log::out!();

    // Query wallet for staking info
    match currency_chain.get_wallet(&uid) {
        Ok(Some(wallet)) => {
            dchat_log::out!("Active Stakes:");
            if wallet.staked > 0 {
                dchat_log::out!("  💎 {} DCHAT staked", format_tokens(wallet.staked));
            } else {
                dchat_log::out!("  (none)");
            }
            dchat_log::out!();

            // Query unbonding queue
            let unbonding_records = currency_chain.get_unbonding_records(&uid);
            dchat_log::out!("Unbonding:");
            if unbonding_records.is_empty() {
                dchat_log::out!("  (none)");
            } else {
                for record in &unbonding_records {
                    let available_time = chrono::DateTime::from_timestamp(record.available_at, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    dchat_log::out!(
                        "  ⏳ {} DCHAT (available {})",
                        format_tokens(record.amount),
                        available_time
                    );
                }
            }
            dchat_log::out!();
            dchat_log::out!("Total Staked: {} DCHAT", format_tokens(wallet.staked));
            dchat_log::out!(
                "Pending Rewards: {} DCHAT",
                format_tokens(wallet.rewards_pending)
            );
        }
        Ok(None) => {
            dchat_log::out!("Active Stakes:");
            dchat_log::out!("  (none)");
            dchat_log::out!();
            dchat_log::out!("Unbonding:");
            dchat_log::out!("  (none)");
            dchat_log::out!();
            dchat_log::out!("Total Staked: 0 DCHAT");
            dchat_log::out!("Pending Rewards: 0 DCHAT");
        }
        Err(e) => {
            tracing::warn!("Failed to query staking from chain: {}", e);
            dchat_log::out!("⚠️  Unable to query blockchain: {}", e);
        }
    }

    dchat_log::out!();
    dchat_log::out!(
        "💡 Stake tokens with: dchat staking stake --user-id {} --amount <AMOUNT>",
        user_id
    );

    Ok(())
}

/// Handle `dchat staking validators` command
pub async fn handle_validators(config: &Config) -> Result<()> {
    let (currency_chain, _, currency_rpc_url) = init_staking_context(config).await?;

    dchat_log::out!("\n✅ Active Validators:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!(
        "{:<20} {:>15} {:>10} {:>15}",
        "Validator", "Stake", "APY", "Commission"
    );
    dchat_log::out!("{}", "-".repeat(65));
    dchat_log::out!("RPC: {}", currency_rpc_url);

    // Query validator set from blockchain
    match currency_chain.get_validators().await {
        Ok(validators) => {
            let mut total_stake = 0u64;
            for validator in &validators {
                let apy_display = format!("{:.1}%", validator.apy_estimate);
                let commission_display = format!("{:.1}%", validator.commission_rate);
                dchat_log::out!(
                    "{:<20} {:>12} DCHAT {:>10} {:>15}",
                    truncate_str(&validator.name, 18),
                    format_tokens(validator.total_stake),
                    apy_display,
                    commission_display
                );
                total_stake += validator.total_stake;
            }
            dchat_log::out!();
            dchat_log::out!("Total validators: {}", validators.len());
            dchat_log::out!("Total staked: {} DCHAT", format_tokens(total_stake));
        }
        Err(e) => {
            tracing::warn!("Failed to query validators: {}", e);
            dchat_log::out!("⚠️  Unable to query validators: {}", e);
        }
    }

    Ok(())
}

/// Handle `dchat staking delegate` command
pub async fn handle_delegate(
    config: &Config,
    user_id: String,
    validator_id: String,
    amount: u64,
) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    let (currency_chain, _, currency_rpc_url) = init_staking_context(config).await?;

    dchat_log::out!("\n🤝 Delegating Stake:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("Delegator: {}", user_id);
    dchat_log::out!("Validator: {}", validator_id);
    dchat_log::out!("Amount: {} DCHAT", format_tokens(amount));
    dchat_log::out!("RPC: {}", currency_rpc_url);

    // Execute delegation via blockchain
    match currency_chain
        .delegate_stake(&uid, &validator_id, amount)
        .await
    {
        Ok(tx_hash) => {
            dchat_log::out!();
            dchat_log::out!("✅ Delegation successful!");
            dchat_log::out!("   Transaction: {}", tx_hash);
            dchat_log::out!("   Your rewards will be distributed based on validator performance.");
        }
        Err(e) => {
            dchat_log::out!();
            dchat_log::out!("❌ Delegation failed: {}", e);
        }
    }

    Ok(())
}

/// Handle `dchat staking undelegate` command
pub async fn handle_undelegate(
    config: &Config,
    user_id: String,
    validator_id: String,
    amount: u64,
) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    let (currency_chain, _, currency_rpc_url) = init_staking_context(config).await?;

    let amount_str = if amount == 0 {
        "ALL".to_string()
    } else {
        format!("{} DCHAT", format_tokens(amount))
    };

    dchat_log::out!("\n🔄 Undelegating Stake:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("Delegator: {}", user_id);
    dchat_log::out!("Validator: {}", validator_id);
    dchat_log::out!("Amount: {}", amount_str);
    dchat_log::out!("RPC: {}", currency_rpc_url);

    // Execute undelegation via blockchain
    let actual_amount = if amount == 0 {
        // Query current delegation to undelegate all
        currency_chain
            .get_delegation(&uid, &validator_id)
            .await
            .unwrap_or(0)
    } else {
        amount
    };

    match currency_chain.initiate_stake_unbonding(&uid, actual_amount) {
        Ok(unbonding_record) => {
            let cooldown_seconds =
                (unbonding_record.available_at - unbonding_record.initiated_at) as u64;
            let cooldown_days = cooldown_seconds / 86400;
            dchat_log::out!();
            dchat_log::out!("⏳ Unbonding period: {} days", cooldown_days);
            dchat_log::out!();
            dchat_log::out!("✅ Undelegation submitted!");
            dchat_log::out!("   Unbonding ID: {}", unbonding_record.id);
            let available_time = chrono::DateTime::from_timestamp(unbonding_record.available_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            dchat_log::out!("   Available: {}", available_time);
        }
        Err(e) => {
            dchat_log::out!();
            dchat_log::out!("❌ Undelegation failed: {}", e);
        }
    }

    Ok(())
}
