use std::path::PathBuf;

use super::{PlatformUtilities, UtilitiesError};
use crate::bridge::utils::{UtilitiesBridge, UTILITIES};

pub struct HardwareUtilities;

impl PlatformUtilities for HardwareUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError> {
        get_utils_bridge().get_storage_path().map(PathBuf::from)
    }
}

fn get_utils_bridge() -> &'static dyn UtilitiesBridge {
    // crash if UTILITIES is not yet set
    UTILITIES
        .get()
        .expect("UTILITIES used before init_utilities() was called")
        .as_ref()
}
