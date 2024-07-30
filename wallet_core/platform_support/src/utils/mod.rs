pub mod hardware;

#[cfg(feature = "software")]
pub mod software;
#[cfg(any(all(feature = "software", test), feature = "integration_test"))]
pub mod test;

use std::path::PathBuf;

use error_category::ErrorCategory;

// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)] // reason field might leak sensitive data
pub enum UtilitiesError {
    #[error("platform error: {reason}")]
    PlatformError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

pub trait PlatformUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
