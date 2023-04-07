use crate::utils::PlatformUtilities;

#[cfg(all(feature = "hardware-integration-test"))]
pub mod hardware;

pub fn get_and_verify_storage_path<K: PlatformUtilities>() -> bool {
    let path =  K::storage_path().expect("Could not get storage path")
        .into_os_string().into_string().expect("Could not convert PathBuf to String");
    path.len() > 0 && path.starts_with("/")
}
