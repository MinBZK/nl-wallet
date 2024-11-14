use std::env;
use std::path::PathBuf;

use tokio::task;

use super::PlatformUtilities;
use super::UtilitiesError;

pub struct SoftwareUtilities;

impl PlatformUtilities for SoftwareUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError> {
        // This should not panic and does not error,
        // so we don't need to use `spawn::blocking()`.
        let path = task::spawn_blocking(env::temp_dir)
            .await
            .expect("Could not join tokio task");

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test;
    use super::*;

    #[tokio::test]
    async fn test_storage_path() {
        test::get_and_verify_storage_path::<SoftwareUtilities>().await;
    }
}
