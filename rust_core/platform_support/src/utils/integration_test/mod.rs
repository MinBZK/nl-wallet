use crate::bridge::utils::UTILITIES;
use crate::utils::PlatformUtilities;

#[cfg(all(feature = "hardware-integration-test"))]
pub mod hardware;

pub fn get_and_verify_storage_path<K: PlatformUtilities>() -> bool {
    let path = UTILITIES.get().expect("Could not get utilities")
        .get_storage_path().expect("Could not get storage path");
    path.len() > 0 && path.starts_with("/")
}
