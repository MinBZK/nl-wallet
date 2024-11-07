pub mod hardware;

#[cfg(feature = "software")]
pub mod software;
#[cfg(any(all(feature = "software", test), feature = "integration_test"))]
pub mod test;

use std::path::PathBuf;

pub use crate::bridge::utils::UtilitiesError;

pub trait PlatformUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError>;
}
