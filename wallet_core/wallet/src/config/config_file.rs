use std::path::{Path, PathBuf};

use tokio::{fs, io};

use wallet_common::config::wallet_config::WalletConfiguration;

#[derive(Debug, thiserror::Error)]
pub enum ConfigFileError {
    #[error("config file I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub async fn get_config_file(storage_path: &Path) -> Result<Option<WalletConfiguration>, ConfigFileError> {
    let path = path_for_config_file(storage_path);

    if !path.try_exists()? {
        return Ok(None);
    }

    let config = read_config(path.as_path()).await?;
    Ok(Some(config))
}

pub async fn update_config_file(storage_path: &Path, config: &WalletConfiguration) -> Result<(), ConfigFileError> {
    let path = path_for_config_file(storage_path);
    write_config(path.as_path(), config).await
}

async fn write_config(path: &Path, config: &WalletConfiguration) -> Result<(), ConfigFileError> {
    let contents = serde_json::to_vec(config)?;
    fs::write(path, contents).await?;
    Ok(())
}

async fn read_config(path: &Path) -> Result<WalletConfiguration, ConfigFileError> {
    let content = fs::read(path).await?;
    let config = serde_json::from_slice(&content)?;
    Ok(config)
}

fn path_for_config_file(storage_path: &Path) -> PathBuf {
    storage_path.join("configuration.json")
}

#[cfg(test)]
mod tests {
    use crate::config::{
        config_file::{get_config_file, update_config_file},
        default_configuration,
    };

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
