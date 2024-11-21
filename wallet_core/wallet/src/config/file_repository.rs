use std::path::PathBuf;
use std::sync::Arc;

use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::reqwest::RequestBuilder;

use super::config_file;
use super::ConfigurationError;
use super::ConfigurationRepository;
use super::ConfigurationUpdateState;
use super::HttpConfigurationRepository;
use super::UpdateableConfigurationRepository;

pub struct FileStorageConfigurationRepository<T> {
    wrapped: T,
    storage_path: PathBuf,
}

impl<C> FileStorageConfigurationRepository<HttpConfigurationRepository<C>>
where
    C: RequestBuilder,
{
    pub async fn init(
        storage_path: PathBuf,
        http_config: C,
        signing_public_key: EcdsaDecodingKey,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let default_config = match config_file::get_config_file(storage_path.as_path()).await? {
            Some(stored_config) if initial_config.version > stored_config.version => {
                // When the initial configuration is newer than the stored configuration (e.g. due to an app update)
                // that version is used and the stored configuration is overwritten.
                config_file::update_config_file(storage_path.as_path(), &initial_config).await?;
                initial_config
            }
            Some(stored_config) => stored_config,
            None => initial_config,
        };

        Ok(Self::new(
            HttpConfigurationRepository::new(http_config, signing_public_key, storage_path.clone(), default_config)
                .await?,
            storage_path,
        ))
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

impl<T> ConfigurationRepository for FileStorageConfigurationRepository<T>
where
    T: ConfigurationRepository,
{
    fn config(&self) -> Arc<WalletConfiguration> {
        self.wrapped.config()
    }
}

impl<T> UpdateableConfigurationRepository for FileStorageConfigurationRepository<T>
where
    T: UpdateableConfigurationRepository + Sync,
{
    async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError> {
        let result = self.wrapped.fetch().await?;

        if let ConfigurationUpdateState::Updated = result {
            let wrapped_config = self.wrapped.config();
            config_file::update_config_file(self.storage_path.as_path(), wrapped_config.as_ref()).await?;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use p256::ecdsa::SigningKey;
    use parking_lot::RwLock;
    use rand_core::OsRng;

    use wallet_common::config::http::test::HttpConfig;
    use wallet_common::config::wallet_config::WalletConfiguration;
    use wallet_common::jwt::EcdsaDecodingKey;

    use crate::config::config_file;
    use crate::config::default_configuration;
    use crate::config::ConfigurationError;
    use crate::config::ConfigurationRepository;
    use crate::config::ConfigurationUpdateState;
    use crate::config::FileStorageConfigurationRepository;
    use crate::config::UpdateableConfigurationRepository;

    struct TestConfigRepo(RwLock<WalletConfiguration>);

    impl ConfigurationRepository for TestConfigRepo {
        fn config(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().clone())
        }
    }

    impl UpdateableConfigurationRepository for TestConfigRepo {
        async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError> {
            let mut config = self.0.write();
            config.lock_timeouts.background_timeout = 700;
            Ok(ConfigurationUpdateState::Updated)
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

        let file_config = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();

        let repo = FileStorageConfigurationRepository::new(TestConfigRepo(RwLock::new(file_config)), path);

        let config = repo.config();
        assert_eq!(
            700, config.lock_timeouts.background_timeout,
            "should return value read from filesystem"
        );
    }

    #[tokio::test]
    async fn should_use_newer_embedded_wallet_config() {
        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.into_path();
        let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
        let config_decoding_key: EcdsaDecodingKey = (&verifying_key).into();

        let mut initially_stored_wallet_config = default_configuration();
        initially_stored_wallet_config.version = 10;

        // store initial wallet config having version 10
        config_file::update_config_file(path.as_path(), &initially_stored_wallet_config)
            .await
            .unwrap();

        let repo = FileStorageConfigurationRepository::init(
            path.clone(),
            HttpConfig {
                base_url: "http://localhost".parse().unwrap(),
            },
            config_decoding_key.clone(),
            default_configuration(),
        )
        .await
        .unwrap();
        assert_eq!(10, repo.config().version, "should use stored config");

        let mut embedded_wallet_config = default_configuration();
        embedded_wallet_config.version = 20;

        let repo = FileStorageConfigurationRepository::init(
            path.clone(),
            HttpConfig {
                base_url: "http://localhost".parse().unwrap(),
            },
            config_decoding_key,
            embedded_wallet_config,
        )
        .await
        .unwrap();
        assert_eq!(20, repo.config().version, "should use newer embedded config");

        let stored_config = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();
        assert_eq!(
            20, stored_config.version,
            "newer embedded config should have been stored"
        );
    }
}
