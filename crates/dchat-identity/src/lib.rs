//! dchat-identity: Identity management and verification
//! 
//! This crate provides identity management including:
//! - Identity registration and verification
//! - Multi-device synchronization
//! - Hierarchical key derivation
//! - Burner identities
//! - Guardian-based account recovery
//! - User profiles and status

pub mod identity;
pub mod device;
pub mod derivation;
pub mod sync;
pub mod guardian;
pub mod guardian_recovery; // Phase 2: Guardian-based account recovery
pub mod verification;
pub mod burner;
pub mod biometric; // Phase 7 Sprint 6: Keyless UX - Biometric authentication
pub mod enclave; // Phase 7 Sprint 6: Keyless UX - Secure enclave integration
pub mod mpc; // Phase 7 Sprint 6: Keyless UX - MPC threshold signing
pub mod profile; // User profiles, status, and privacy settings
pub mod storage; // Profile database storage

pub use identity::{Identity, IdentityManager};
pub use device::{Device, DeviceManager};
pub use derivation::{KeyPath, IdentityDerivation};
pub use guardian::{Guardian, GuardianManager, RecoveryRequest};
pub use guardian_recovery::{GuardianRecoveryManager, GuardianId, RecoveryStatus};
pub use verification::{VerifiedBadge, VerificationProof};
pub use burner::BurnerIdentity;
pub use biometric::{BiometricAuthenticator, BiometricConfig, BiometricType, BiometricAuthResult};
pub use enclave::{SecureEnclave, EnclaveConfig};
pub use mpc::{MpcSigner, MpcConfig, ThresholdSignature};
pub use profile::{
    UserProfile, ProfilePicture, UserStatus, StatusType, OnlineStatus,
    PrivacySettings, VisibilityLevel, ProfileManager, MusicProvider, MusicApiTrack
};
pub use storage::ProfileStorage;

