//! Integration tests for insurance fund and token-gated channels

use dchat_chain::{InsuranceFund, FundConfiguration, ClaimType, FundTransactionType};
use dchat_messaging::{ChannelAccessManager, AccessPolicy};
use dchat_core::types::UserId;

#[test]
fn test_insurance_fund_relay_failure_claim() {
    let config = FundConfiguration::default();
    let mut fund = InsuranceFund::new(1_000_000, config);
    
    // Simulate relay failure
    let relay_id = UserId::new();
    let affected_user = UserId::new();
    
    let claim_id = fund.submit_claim(
        affected_user,
        ClaimType::RelayFailure {
            relay_id,
            affected_users: vec![affected_user],
            message_count: 50,
        },
        5_000, // 100 tokens per message
        vec!["proof_of_non_delivery".to_string()],
    ).unwrap();
    
    // Governance votes
    for _ in 0..3 {
        fund.vote_on_claim(claim_id, UserId::new(), true).unwrap();
    }
    
    fund.approve_claim(claim_id).unwrap();
    fund.payout_claim(claim_id, "tx_123".to_string()).unwrap();
    
    let stats = fund.get_statistics();
    assert_eq!(stats.total_paid_out, 5_000);
    assert_eq!(stats.approved_claims, 1);
}

#[test]
fn test_token_gated_channel_integration() {
    let mut access_manager = ChannelAccessManager::new();
    let channel_id = dchat_core::types::ChannelId::new();
    let user = UserId::new();
    
    // Create token-gated channel requiring 1000 tokens
    let policy = AccessPolicy::TokenGated {
        token_address: "DCHAT".to_string(),
        minimum_balance: 1000,
    };
    
    access_manager.create_channel(channel_id, policy.clone()).unwrap();
    
    // User doesn't have tokens - should fail
    assert!(!access_manager.can_access(&channel_id, &user).unwrap());
    
    // Grant tokens
    access_manager.update_user_tokens(user, "DCHAT".to_string(), 1500);
    
    // Now user should have access
    assert!(access_manager.can_access(&channel_id, &user).unwrap());
    
    // Grant explicit access
    access_manager.grant_access(&channel_id, user).unwrap();
    assert!(access_manager.is_member(&channel_id, &user));
}

#[test]
fn test_insurance_fund_with_channel_slashing() {
    let config = FundConfiguration::default();
    let mut fund = InsuranceFund::new(2_000_000, config);
    
    // Simulate a malicious channel operator being slashed
    let operator_id = UserId::new();
    let affected_users = vec![UserId::new(), UserId::new(), UserId::new()];
    
    // Operator's stake was insufficient to cover damage
    let claim_id = fund.submit_claim(
        affected_users[0],
        ClaimType::SlashingOverflow {
            node_id: operator_id,
            deficit_amount: 10_000,
        },
        10_000,
        vec!["slash_proof".to_string()],
    ).unwrap();
    
    // Fast-track approval for slashing overflow
    for _ in 0..5 {
        fund.vote_on_claim(claim_id, UserId::new(), true).unwrap();
    }
    
    fund.approve_claim(claim_id).unwrap();
    fund.payout_claim(claim_id, "slash_tx_456".to_string()).unwrap();
    
    assert_eq!(fund.balance(), 1_990_000);
}

#[test]
fn test_nft_gated_channel_with_marketplace() {
    let mut access_manager = ChannelAccessManager::new();
    let channel_id = dchat_core::types::ChannelId::new();
    let user = UserId::new();
    
    // Create NFT-gated channel
    let policy = AccessPolicy::NftGated {
        collection: "PremiumAccess".to_string(),
        token_ids: None, // Any NFT from collection
    };
    
    access_manager.create_channel(channel_id, policy).unwrap();
    
    // User doesn't own NFT - denied
    assert!(!access_manager.can_access(&channel_id, &user).unwrap());
    
    // User acquires NFT
    access_manager.update_user_nfts(
        user,
        "PremiumAccess".to_string(),
        vec!["token_001".to_string()],
    );
    
    // Now has access
    assert!(access_manager.can_access(&channel_id, &user).unwrap());
}

#[test]
fn test_combined_policy_channel() {
    let mut access_manager = ChannelAccessManager::new();
    let channel_id = dchat_core::types::ChannelId::new();
    let user = UserId::new();
    
    // Requires both tokens AND reputation
    let policy = AccessPolicy::Combined {
        policies: vec![
            AccessPolicy::TokenGated {
                token_address: "DCHAT".to_string(),
                minimum_balance: 500,
            },
            AccessPolicy::ReputationGated {
                minimum_reputation: 0.7,
            },
        ],
    };
    
    access_manager.create_channel(channel_id, policy).unwrap();
    
    // Has tokens but not reputation
    access_manager.update_user_tokens(user, "DCHAT".to_string(), 600);
    assert!(!access_manager.can_access(&channel_id, &user).unwrap());
    
    // Now has both
    access_manager.update_user_reputation(user, 0.8);
    assert!(access_manager.can_access(&channel_id, &user).unwrap());
}

#[test]
fn test_insurance_fund_replenishment_from_fees() {
    let config = FundConfiguration {
        fee_allocation_percent: 15, // 15% of fees
        ..Default::default()
    };
    
    let mut fund = InsuranceFund::new(500_000, config);
    
    // Simulate transaction fees being collected
    for _ in 0..100 {
        let transaction_fee = 100;
        let fund_share = transaction_fee * 15 / 100; // 15 tokens per tx
        fund.deposit(fund_share, FundTransactionType::FeeDeposit, format!("fee_tx_{}", _));
    }
    
    // Fund should have grown
    assert_eq!(fund.balance(), 500_000 + (15 * 100));
    
    let stats = fund.get_statistics();
    assert!(stats.health_ratio > 1.0);
}

#[test]
fn test_stake_gated_channel_for_moderation() {
    let mut access_manager = ChannelAccessManager::new();
    let channel_id = dchat_core::types::ChannelId::new();
    
    // Moderator channel requires staking
    let policy = AccessPolicy::StakeGated {
        minimum_stake: 10_000,
    };
    
    access_manager.create_channel(channel_id, policy).unwrap();
    
    let moderator = UserId::new();
    access_manager.update_user_stake(moderator, 15_000);
    
    assert!(access_manager.can_access(&channel_id, &moderator).unwrap());
    access_manager.grant_access(&channel_id, moderator).unwrap();
    
    // If stake drops below threshold, access should be revoked
    access_manager.update_user_stake(moderator, 8_000);
    assert!(!access_manager.can_access(&channel_id, &moderator).unwrap());
}

#[test]
fn test_emergency_insurance_payout() {
    let config = FundConfiguration::default();
    let mut fund = InsuranceFund::new(5_000_000, config);
    
    // Emergency governance decision due to critical bug
    let affected_users = vec![UserId::new(); 50]; // 50 affected users
    
    let claim_id = fund.submit_claim(
        affected_users[0],
        ClaimType::EmergencyCompensation {
            proposal_id: "emergency_001".to_string(),
            affected_users: affected_users.clone(),
            reason: "Critical consensus bug caused message loss".to_string(),
        },
        100_000, // 2k per user
        vec!["bug_report".to_string(), "audit_log".to_string()],
    ).unwrap();
    
    // Emergency fast-track
    for _ in 0..7 {
        fund.vote_on_claim(claim_id, UserId::new(), true).unwrap();
    }
    
    fund.approve_claim(claim_id).unwrap();
    fund.payout_claim(claim_id, "emergency_tx".to_string()).unwrap();
    
    assert_eq!(fund.balance(), 4_900_000);
}
