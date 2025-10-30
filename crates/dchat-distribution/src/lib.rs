pub mod package;
pub mod gossip;

pub use package::{
    PackageMetadata, PackageType, DownloadSource, SourceType,
    PackageManager, AutoUpdateConfig, DistributionError, Result,
};

pub use gossip::{
    VersionAnnouncement, GossipDiscovery, UpdateScheduler,
};
