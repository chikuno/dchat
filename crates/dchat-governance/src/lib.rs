// dchat-governance: Decentralized governance and DAO infrastructure
//
// This crate implements voting, proposals, and decentralized moderation
// for the dchat protocol.

pub mod voting;
pub mod abuse_reporting;
pub mod moderation;
pub mod upgrade;

pub use voting::{Proposal, Vote, VoteManager, ProposalType};
pub use abuse_reporting::{AbuseReport, ReportManager, JurySelection};
pub use moderation::{ModerationAction, ModerationManager, SlashingVote};
pub use upgrade::{
    UpgradeProposal, UpgradeManager, UpgradeType, UpgradeStatus,
    Version, ValidatorSignature, ForkState,
};
