pub mod error;

#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

#[cfg(feature = "integration-test")]
pub mod integration_test;

use std::path::PathBuf;

use self::error::UtilitiesError;

pub trait PlatformUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError>;
}

// if the hardware feature is enabled, prefer HardwareUtilities
#[cfg(feature = "hardware")]
pub type PreferredPlatformUtilities = self::hardware::HardwareUtilities;

// otherwise if the software feature is enabled, prefer SoftwareUtilities
#[cfg(all(not(feature = "hardware"), feature = "software"))]
pub type PreferredPlatformUtilities = self::software::SoftwareUtilities;

// otherwise just just alias the Never type
#[cfg(not(any(feature = "hardware", feature = "software")))]
pub type PreferredPlatformUtilities = never::Never;
