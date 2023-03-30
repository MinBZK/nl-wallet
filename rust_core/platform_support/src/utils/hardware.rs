use std::{path::PathBuf, sync::MutexGuard};

use super::{error::UtilitiesError, PlatformUtilities};
use crate::bridge::utils::{UtilitiesBridge, UTILITIES};

pub struct HardwareUtilities;

impl PlatformUtilities for HardwareUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError> {
        let utils = lock_utils_bridge();

        utils.get_storage_path().map(PathBuf::from)
    }
}

fn lock_utils_bridge() -> MutexGuard<'static, Box<dyn UtilitiesBridge>> {
    // crash if UTILITIES is not yet set, then wait for bridge mutex lock
    UTILITIES
        .get()
        .expect("UTILITIES used before init_utilities() was called")
        .lock()
        .expect("Could not get lock on UTILITIES")
}
