// dchat-privacy: Privacy-preserving cryptographic primitives
//
// This crate implements zero-knowledge proofs, blind tokens, and stealth payloads
// for metadata resistance and anonymous operations in dchat.

pub mod zk_proofs;
pub mod blind_tokens;
pub mod stealth;

pub use zk_proofs::{ZkProof, ContactProof, ReputationProof};
pub use blind_tokens::{BlindToken, BlindSigner, TokenIssuer};
pub use stealth::{StealthPayload, StealthAddress};
