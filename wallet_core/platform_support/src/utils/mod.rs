#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

use std::path::PathBuf;

// implementation of UtilitiesError from UDL, only with "hardware" flag
#[derive(Debug, thiserror::Error)]
pub enum UtilitiesError {
    #[error("Platform error: {reason}")]
    PlatformError { reason: String },
    #[error("Bridging error: {reason}")]
    BridgingError { reason: String },
}

pub trait PlatformUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
