use std::path::PathBuf;

use super::{PlatformUtilities, UtilitiesError};
use crate::bridge::utils::get_utils;

pub struct HardwareUtilities;

impl PlatformUtilities for HardwareUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError> {
        get_utils().get_storage_path().map(PathBuf::from)
    }
}
