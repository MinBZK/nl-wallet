pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

use std::path::PathBuf;

use async_trait::async_trait;

// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error)]
pub enum UtilitiesError {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

#[async_trait]
pub trait PlatformUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
