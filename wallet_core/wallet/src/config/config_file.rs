use std::path::Path;
use std::path::PathBuf;

use tokio::fs;

use wallet_common::config::wallet_config::WalletConfiguration;

use super::FileStorageError;

pub async fn get_config_file(storage_path: &Path) -> Result<Option<WalletConfiguration>, FileStorageError> {
    let path = path_for_config_file(storage_path);

    if !fs::try_exists(&path).await? {
        return Ok(None);
    }

    let config = read_config(path.as_path()).await?;
    Ok(Some(config))
}

pub async fn update_config_file(storage_path: &Path, config: &WalletConfiguration) -> Result<(), FileStorageError> {
    let path = path_for_config_file(storage_path);
    write_config(path.as_path(), config).await
}

async fn write_config(path: &Path, config: &WalletConfiguration) -> Result<(), FileStorageError> {
    let contents = serde_json::to_vec(config)?;
    fs::write(path, contents).await?;
    Ok(())
}

async fn read_config(path: &Path) -> Result<WalletConfiguration, FileStorageError> {
    let content = fs::read(path).await?;
    let config = serde_json::from_slice(&content)?;
    Ok(config)
}

fn path_for_config_file(storage_path: &Path) -> PathBuf {
    storage_path.join("configuration.json")
}

#[cfg(test)]
mod tests {
    use crate::config::config_file::get_config_file;
    use crate::config::config_file::update_config_file;
    use crate::config::default_configuration;

    #[tokio::test]
    async fn should_read_and_update_config() {
        let tempdir = tempfile::tempdir().unwrap();

        assert!(get_config_file(tempdir.path()).await.unwrap().is_none());

        let mut config = default_configuration();
        config.lock_timeouts.background_timeout = 1500;
        update_config_file(tempdir.path(), &config).await.unwrap();

        let updated = get_config_file(tempdir.path()).await.unwrap().unwrap();

        assert_ne!(&default_configuration(), &updated);
        assert_eq!(1500, updated.lock_timeouts.background_timeout);
    }
}
