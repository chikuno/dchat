// Rewards CLI Command Handlers
//
// Dedicated handler functions for rewards subcommands.
// Extracted from main.rs for better maintainability.

use dchat_blockchain::currency_chain::{CurrencyChainClient, CurrencyChainConfig};
use dchat_core::error::{Error, Result};
use dchat_core::UserId;
use uuid::Uuid;

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

/// Get currency chain RPC URL from environment
fn get_currency_rpc_url() -> String {
    std::env::var("DCHAT_CURRENCY_RPC_URL")
        .unwrap_or_else(|_| "https://currency.dchat.network/rpc".to_string())
}

/// Initialize currency chain client
fn init_currency_chain() -> Result<CurrencyChainClient> {
    let rpc_url = get_currency_rpc_url();
    let chain_config = CurrencyChainConfig {
        rpc_url,
        ..Default::default()
    };
    CurrencyChainClient::new(chain_config)
        .map_err(|e| Error::chain(format!("Failed to connect to currency chain: {}", e)))
}

/// Handle `dchat rewards claim` command
pub async fn handle_claim(user_id: String) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    dchat_log::out!("\n🎁 Claiming Rewards:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);

    match init_currency_chain() {
        Ok(currency_chain) => {
            // First check pending rewards
            match currency_chain.get_wallet(&uid) {
                Ok(Some(wallet)) => {
                    if wallet.rewards_pending == 0 {
                        dchat_log::out!();
                        dchat_log::out!("Pending Rewards: 0 DCHAT");
                        dchat_log::out!();
                        dchat_log::out!("No rewards to claim at this time.");
                        dchat_log::out!("💡 Earn rewards by staking tokens or running a relay node!");
                    } else {
                        dchat_log::out!();
                        dchat_log::out!(
                            "Pending Rewards: {} DCHAT",
                            format_tokens(wallet.rewards_pending)
                        );

                        match currency_chain.claim_rewards(&uid) {
                            Ok(tx_hash) => {
                                dchat_log::out!();
                                dchat_log::out!("✅ Rewards claimed successfully!");
                                dchat_log::out!("   Transaction: {}", tx_hash);
                                dchat_log::out!(
                                    "   Amount: {} DCHAT",
                                    format_tokens(wallet.rewards_pending)
                                );
                            }
                            Err(e) => {
                                dchat_log::out!();
                                dchat_log::out!("❌ Failed to claim rewards: {}", e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    dchat_log::out!();
                    dchat_log::out!("Pending Rewards: 0 DCHAT");
                    dchat_log::out!();
                    dchat_log::out!("No rewards to claim at this time.");
                    dchat_log::out!("💡 Earn rewards by staking tokens or running a relay node!");
                }
                Err(e) => {
                    dchat_log::out!();
                    dchat_log::out!("⚠️  Unable to query rewards: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to currency chain: {}", e);
            dchat_log::out!();
            dchat_log::out!("⚠️  Cannot connect to currency chain: {}", e);
        }
    }

    Ok(())
}

/// Handle `dchat rewards history` command
pub async fn handle_history(user_id: String, limit: u32) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    dchat_log::out!("\n📜 Reward History (last {}):", limit);
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("{:<24} {:<20} {:>15}", "Date", "Type", "Amount");
    dchat_log::out!("{}", "-".repeat(65));

    match init_currency_chain() {
        Ok(currency_chain) => {
            match currency_chain
                .get_reward_history(&uid, limit as usize)
                .await
            {
                Ok(history) if history.is_empty() => {
                    dchat_log::out!("No reward history found.");
                    dchat_log::out!();
                    dchat_log::out!("Rewards are distributed:");
                    dchat_log::out!("  • Staking rewards: Every epoch (~24 hours)");
                    dchat_log::out!("  • Relay rewards: Per message delivered");
                    dchat_log::out!("  • Validator rewards: Per block produced");
                }
                Ok(history) => {
                    for entry in history {
                        let date = chrono::DateTime::from_timestamp(entry.timestamp, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        dchat_log::out!(
                            "{:<24} {:<20} {:>12} DCHAT",
                            date,
                            entry.reward_type,
                            format_tokens(entry.amount)
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to query reward history: {}", e);
                    dchat_log::out!("⚠️  Unable to query history: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to currency chain: {}", e);
            dchat_log::out!("⚠️  Cannot connect to currency chain: {}", e);
        }
    }

    Ok(())
}

/// Handle `dchat rewards pending` command
pub async fn handle_pending(user_id: String) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    dchat_log::out!("\n⏳ Pending Rewards:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);
    dchat_log::out!();

    match init_currency_chain() {
        Ok(currency_chain) => {
            match currency_chain.get_pending_rewards_breakdown(&uid).await {
                Ok(breakdown) => {
                    dchat_log::out!(
                        "Staking Rewards:    {} DCHAT",
                        format_tokens(breakdown.staking_rewards)
                    );
                    dchat_log::out!(
                        "Relay Rewards:      {} DCHAT",
                        format_tokens(breakdown.relay_rewards)
                    );
                    dchat_log::out!(
                        "Referral Rewards:   {} DCHAT",
                        format_tokens(breakdown.referral_rewards)
                    );
                    dchat_log::out!("──────────────────────────");
                    let total = breakdown.staking_rewards
                        + breakdown.relay_rewards
                        + breakdown.referral_rewards;
                    dchat_log::out!("Total Pending:      {} DCHAT", format_tokens(total));
                    dchat_log::out!();
                    if total > 0 {
                        dchat_log::out!("Claim with: dchat rewards claim --user-id {}", user_id);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to query pending rewards: {}", e);
                    // Fallback to wallet query
                    match currency_chain.get_wallet(&uid) {
                        Ok(Some(wallet)) => {
                            dchat_log::out!(
                                "Staking Rewards:    {} DCHAT",
                                format_tokens(wallet.rewards_pending)
                            );
                            dchat_log::out!("Relay Rewards:      0 DCHAT");
                            dchat_log::out!("Referral Rewards:   0 DCHAT");
                            dchat_log::out!("──────────────────────────");
                            dchat_log::out!(
                                "Total Pending:      {} DCHAT",
                                format_tokens(wallet.rewards_pending)
                            );
                            dchat_log::out!();
                            if wallet.rewards_pending > 0 {
                                dchat_log::out!("Claim with: dchat rewards claim --user-id {}", user_id);
                            }
                        }
                        _ => {
                            dchat_log::out!("⚠️  Unable to query rewards: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to currency chain: {}", e);
            dchat_log::out!("⚠️  Cannot connect to currency chain: {}", e);
        }
    }

    Ok(())
}

/// Handle `dchat rewards breakdown` command
pub async fn handle_breakdown(user_id: String) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    dchat_log::out!("\n📊 Reward Breakdown:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);
    dchat_log::out!();

    match init_currency_chain() {
        Ok(currency_chain) => match currency_chain.get_all_time_rewards(&uid).await {
            Ok(breakdown) => {
                let total = breakdown.staking
                    + breakdown.relaying
                    + breakdown.referrals
                    + breakdown.governance;
                let staking_pct = if total > 0 {
                    breakdown.staking as f64 / total as f64 * 100.0
                } else {
                    0.0
                };
                let relaying_pct = if total > 0 {
                    breakdown.relaying as f64 / total as f64 * 100.0
                } else {
                    0.0
                };
                let referral_pct = if total > 0 {
                    breakdown.referrals as f64 / total as f64 * 100.0
                } else {
                    0.0
                };
                let governance_pct = if total > 0 {
                    breakdown.governance as f64 / total as f64 * 100.0
                } else {
                    0.0
                };

                dchat_log::out!("All-Time Earnings:");
                dchat_log::out!(
                    "  Staking:     {} DCHAT ({:.1}%)",
                    format_tokens(breakdown.staking),
                    staking_pct
                );
                dchat_log::out!(
                    "  Relaying:    {} DCHAT ({:.1}%)",
                    format_tokens(breakdown.relaying),
                    relaying_pct
                );
                dchat_log::out!(
                    "  Referrals:   {} DCHAT ({:.1}%)",
                    format_tokens(breakdown.referrals),
                    referral_pct
                );
                dchat_log::out!(
                    "  Governance:  {} DCHAT ({:.1}%)",
                    format_tokens(breakdown.governance),
                    governance_pct
                );
                dchat_log::out!("──────────────────────────");
                dchat_log::out!("  Total:       {} DCHAT", format_tokens(total));
                dchat_log::out!();
                dchat_log::out!("Current APY Estimate:");
                dchat_log::out!("  Staking APY:    {:.1}%", breakdown.current_staking_apy);
                dchat_log::out!(
                    "  Combined APY:   ~{:.1}% (with active relaying)",
                    breakdown.combined_apy_estimate
                );
            }
            Err(e) => {
                tracing::warn!("Failed to query reward breakdown: {}", e);
                dchat_log::out!("All-Time Earnings:");
                dchat_log::out!("  Staking:     0 DCHAT (0%)");
                dchat_log::out!("  Relaying:    0 DCHAT (0%)");
                dchat_log::out!("  Referrals:   0 DCHAT (0%)");
                dchat_log::out!("  Governance:  0 DCHAT (0%)");
                dchat_log::out!("──────────────────────────");
                dchat_log::out!("  Total:       0 DCHAT");
                dchat_log::out!();
                dchat_log::out!("⚠️  Could not retrieve full breakdown: {}", e);
            }
        },
        Err(e) => {
            tracing::warn!("Failed to connect to currency chain: {}", e);
            dchat_log::out!("⚠️  Cannot connect to currency chain: {}", e);
        }
    }

    Ok(())
}

/// Handle `dchat rewards compound` command
pub async fn handle_compound(user_id: String, enable: bool) -> Result<()> {
    let uid = UserId(Uuid::parse_str(&user_id).map_err(|_| Error::validation("Invalid user ID"))?);

    dchat_log::out!("\n🔄 Auto-Compound Settings:");
    dchat_log::out!("══════════════════════════════════════════════════════════");
    dchat_log::out!("User ID: {}", user_id);
    dchat_log::out!(
        "Auto-Compound: {}",
        if enable { "ENABLED" } else { "DISABLED" }
    );

    match init_currency_chain() {
        Ok(currency_chain) => match currency_chain.set_auto_compound(&uid, enable).await {
            Ok(_) => {
                if enable {
                    dchat_log::out!();
                    dchat_log::out!("✅ Auto-compounding enabled!");
                    dchat_log::out!("   Rewards will be automatically restaked for maximum returns.");
                } else {
                    dchat_log::out!();
                    dchat_log::out!("✅ Auto-compounding disabled!");
                    dchat_log::out!("   Rewards will accumulate as claimable balance.");
                }
            }
            Err(e) => {
                dchat_log::out!();
                dchat_log::out!("❌ Failed to update auto-compound setting: {}", e);
            }
        },
        Err(e) => {
            tracing::warn!("Failed to connect to currency chain: {}", e);
            dchat_log::out!("⚠️  Cannot connect to currency chain: {}", e);
        }
    }

    Ok(())
}
