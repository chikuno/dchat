//! dchat-storage: Local data persistence layer
//!
//! This crate provides:
//! - SQLite database for messages, identities, and metadata
//! - Encrypted backup and restore
//! - Message deduplication via content addressing
//! - TTL-based data lifecycle management
//! - Storage economics (bonds, quotas)

pub mod backup;
pub mod database;
pub mod deduplication;
pub mod file_upload;
pub mod lifecycle;
pub mod schema;

pub use backup::{BackupManager, EncryptedBackup};
pub use database::{Database, DatabaseConfig, MessageRow};
pub use deduplication::{ContentAddressable, DeduplicationStore};
pub use file_upload::{
    FileUploadManager, MediaFileType, StorageStats, UploadConfig, UploadedFile,
};
pub use lifecycle::{LifecycleManager, TtlConfig};
pub use schema::Schema;
