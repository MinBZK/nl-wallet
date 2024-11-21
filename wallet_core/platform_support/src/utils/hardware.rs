use std::path::PathBuf;

use wallet_common::spawn;

use crate::bridge::utils::get_utils_bridge;

use super::PlatformUtilities;
use super::UtilitiesError;

pub struct HardwareUtilities;

impl PlatformUtilities for HardwareUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError> {
        let path = spawn::blocking(|| get_utils_bridge().get_storage_path()).await?;

        Ok(path.into())
    }
}
