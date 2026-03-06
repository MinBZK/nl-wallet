use std::path::Path;
use std::path::PathBuf;

use tokio::fs;

use super::ConfigurationError;
use super::FileStorageError;
use super::WalletConfigJwt;

pub async fn get_config_file(
    storage_path: &Path,
) -> Result<Option<WalletConfigJwt>, ConfigurationError> {
    let path = path_for_config_file(storage_path);

    if !fs::try_exists(&path).await.map_err(FileStorageError::from)? {
        return Ok(None);
    }

    let jwt_string = fs::read_to_string(path).await.map_err(FileStorageError::from)?;
    Ok(Some(jwt_string.parse()?))
}

pub async fn update_config_file(
    storage_path: &Path,
    jwt: &WalletConfigJwt,
) -> Result<(), FileStorageError> {
    let path = path_for_config_file(storage_path);
    fs::write(path, jwt.serialization()).await?;
    Ok(())
}

fn path_for_config_file(storage_path: &Path) -> PathBuf {
    storage_path.join("configuration.jwt")
}

#[cfg(test)]
mod tests {
    use jwt::SignedJwt;
    use rand_core::OsRng;

    use crate::config::config_file::get_config_file;
    use crate::config::config_file::update_config_file;
    use crate::config::default_wallet_config;

    #[tokio::test]
    async fn should_read_and_update_config() {
        let signing_key = p256::ecdsa::SigningKey::random(&mut OsRng);
        let mut config = default_wallet_config();
        config.lock_timeouts.background_timeout = 1500;
        let jwt = SignedJwt::sign(&config, &signing_key).await.unwrap();
        let unverified = jwt.into_unverified();

        let tempdir = tempfile::tempdir().unwrap();

        assert!(get_config_file(tempdir.path()).await.unwrap().is_none());

        update_config_file(tempdir.path(), &unverified).await.unwrap();

        let stored = get_config_file(tempdir.path()).await.unwrap().unwrap();
        assert_eq!(unverified.serialization(), stored.serialization());
    }
}
