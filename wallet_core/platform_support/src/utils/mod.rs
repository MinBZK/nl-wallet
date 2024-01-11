pub mod hardware;

#[cfg(any(test, feature = "software"))]
pub mod software;
#[cfg(any(test, feature = "integration-test"))]
pub mod test;

use std::path::PathBuf;

// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error)]
pub enum UtilitiesError {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

pub trait PlatformUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
