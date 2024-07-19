pub mod hardware;

#[cfg(feature = "software")]
pub mod software;
#[cfg(any(all(feature = "software", test), feature = "integration_test"))]
pub mod test;

use std::path::PathBuf;

use error_category::ErrorCategory;

// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)] // Not sure what get's into the `reason` fields
pub enum UtilitiesError {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

pub trait PlatformUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
