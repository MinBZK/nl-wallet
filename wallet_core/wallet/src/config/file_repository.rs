use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use wallet_common::config::wallet_config::WalletConfiguration;

use super::{
    config_file, default_configuration, ConfigServerConfiguration, ConfigurationError, ConfigurationRepository,
    HttpConfigurationRepository,
};

pub struct FileStorageConfigurationRepository<T> {
    wrapped: T,
    storage_path: PathBuf,
}

impl FileStorageConfigurationRepository<HttpConfigurationRepository> {
    pub async fn init(storage_path: PathBuf) -> Result<Self, ConfigurationError> {
        let default_config = Self::read_config_file(storage_path.as_path())
            .await?
            .unwrap_or(default_configuration());

        Ok(Self::new(
            HttpConfigurationRepository::new(ConfigServerConfiguration::default(), default_config),
            storage_path,
        ))
    }
}

impl<T> FileStorageConfigurationRepository<T> {
    async fn read_config_file(storage_path: &Path) -> Result<Option<WalletConfiguration>, ConfigurationError> {
        Ok(config_file::get_config_file(storage_path).await?)
    }
}

impl<T> FileStorageConfigurationRepository<T>
where
    T: ConfigurationRepository,
{
    fn new(wrapped: T, storage_path: PathBuf) -> FileStorageConfigurationRepository<T> {
        Self { wrapped, storage_path }
    }
}

#[async_trait]
impl<T> ConfigurationRepository for FileStorageConfigurationRepository<T>
where
    T: ConfigurationRepository + Send + Sync,
{
    fn config(&self) -> Arc<WalletConfiguration> {
        self.wrapped.config()
    }

    async fn fetch(&self) -> Result<(), ConfigurationError> {
        self.wrapped.fetch().await?;
        let wrapped_config = self.wrapped.config();
        config_file::update_config_file(self.storage_path.as_path(), wrapped_config.as_ref()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use async_trait::async_trait;

    use wallet_common::config::wallet_config::WalletConfiguration;

    use crate::config::{
        default_configuration, ConfigurationError, ConfigurationRepository, FileStorageConfigurationRepository,
    };

    struct TestConfigRepo(RwLock<WalletConfiguration>);

    #[async_trait]
    impl ConfigurationRepository for TestConfigRepo {
        fn config(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().unwrap().clone())
        }
        async fn fetch(&self) -> Result<(), ConfigurationError> {
            let mut config = self.0.write().unwrap();
            config.lock_timeouts.background_timeout = 700;
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_store_config_to_filesystem() {
        let mut initial_wallet_config = default_configuration();
        initial_wallet_config.lock_timeouts.background_timeout = 500;

        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.into_path();

        let repo = FileStorageConfigurationRepository::new(
            TestConfigRepo(RwLock::new(initial_wallet_config.clone())),
            path.clone(),
        );

        let config = repo.config();
        assert_eq!(
            500, config.lock_timeouts.background_timeout,
            "should return initial_wallet_config"
        );

        repo.fetch().await.unwrap();

        let config = repo.config();
        assert_eq!(
            700, config.lock_timeouts.background_timeout,
            "should return value set by TestConfigRepo.fetch()"
        );

        let file_config = FileStorageConfigurationRepository::<TestConfigRepo>::read_config_file(path.as_path())
            .await
            .unwrap()
            .unwrap();

        let repo = FileStorageConfigurationRepository::new(TestConfigRepo(RwLock::new(file_config)), path);

        let config = repo.config();
        assert_eq!(
            700, config.lock_timeouts.background_timeout,
            "should return value read from filesystem"
        );
    }
}
