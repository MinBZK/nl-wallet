pub mod hardware;

#[cfg(feature = "mock_utils")]
pub mod mock;
#[cfg(any(all(feature = "mock_utils", test), feature = "integration_test"))]
pub mod test;

use std::path::PathBuf;

pub use crate::bridge::utils::UtilitiesError;

pub trait PlatformUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
