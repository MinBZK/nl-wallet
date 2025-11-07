use std::path::Path;
use std::path::PathBuf;

use nutype::nutype;
use serde::Deserialize;
use url::Url;

use http_utils::urls::BaseUrl;
use server_utils::settings::KeyPair;
use utils::num::NonZeroU31;
use utils::num::Ratio;
use utils::path::prefix_local_path;

#[derive(Clone, Deserialize)]
pub struct StatusListsSettings {
    /// Optional storage url if different from rest of application
    pub storage_url: Option<Url>,
    /// List size
    pub list_size: NonZeroU31,
    /// Threshold relatively to `list_size` to start creating a new list in the background
    pub create_threshold: Ratio,
}

#[derive(Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// Base url for the status list
    pub base_url: BaseUrl,

    /// Path to directory for the published status list
    pub publish_dir: PublishDir,

    /// Key pair to sign status list
    #[serde(flatten)]
    pub keypair: KeyPair,
}

#[nutype(derive(Debug, Clone, Into, Deserialize), validate(with=PublishDir::validate, error=PublishDirError))]
pub struct PublishDir(PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum PublishDirError {
    #[error("publish dir IO error: {0}")]
    IO(std::io::Error),

    #[error("publish dir is not a directory")]
    NotADirectory,
}

impl PublishDir {
    fn validate(path: &Path) -> Result<(), PublishDirError> {
        let path = prefix_local_path(path);
        let metadata = std::fs::metadata(&path).map_err(PublishDirError::IO)?;
        if !metadata.is_dir() {
            return Err(PublishDirError::NotADirectory);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn publish_dir_should_succeed_on_dir() {
        let tempdir = tempfile::tempdir().unwrap();
        let result = PublishDir::try_new(tempdir.path().to_path_buf());
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn publish_dir_should_fail_on_non_existing_dir() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().to_path_buf();
        drop(tempdir);

        let result = PublishDir::try_new(path);
        assert_matches!(result, Err(PublishDirError::IO(_)));
    }

    #[test]
    fn publish_dir_should_fail_on_non_dir() {
        let tempfile = NamedTempFile::new().unwrap();
        let result = PublishDir::try_new(tempfile.path().to_path_buf());
        assert_matches!(result, Err(PublishDirError::NotADirectory));
    }
}
