use std::path::PathBuf;
use std::sync::Arc;

use derive_more::Constructor;

use http_utils::reqwest::IntoPinnedReqwestClient;
use jwt::EcdsaDecodingKey;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::repository::Repository;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::ConfigurationError;
use super::HttpConfigurationRepository;
use super::config_file;

#[derive(Constructor)]
pub struct FileStorageConfigurationRepository<T> {
    wrapped: T,
    storage_path: PathBuf,
}

impl<B> FileStorageConfigurationRepository<HttpConfigurationRepository<B>> {
    pub async fn init(
        storage_path: PathBuf,
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
            HttpConfigurationRepository::new(signing_public_key, storage_path.clone(), default_config).await?,
            storage_path,
        ))
    }
}

impl<T> Repository<Arc<WalletConfiguration>> for FileStorageConfigurationRepository<T>
where
    T: Repository<Arc<WalletConfiguration>>,
{
    fn get(&self) -> Arc<WalletConfiguration> {
        self.wrapped.get()
    }
}

impl<T, B> UpdateableRepository<Arc<WalletConfiguration>, B> for FileStorageConfigurationRepository<T>
where
    T: UpdateableRepository<Arc<WalletConfiguration>, B, Error = ConfigurationError> + Sync,
    B: IntoPinnedReqwestClient + Send + Sync,
{
    type Error = ConfigurationError;

    async fn fetch(&self, client_builder: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
        let result = self.wrapped.fetch(client_builder).await?;

        if let RepositoryUpdateState::Updated { ref to, .. } = result {
            config_file::update_config_file(self.storage_path.as_path(), to).await?;
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

    use http_utils::tls::pinning::TlsPinningConfig;
    use jwt::EcdsaDecodingKey;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use crate::config::ConfigurationError;
    use crate::config::FileStorageConfigurationRepository;
    use crate::config::HttpConfigurationRepository;
    use crate::config::config_file;
    use crate::config::default_wallet_config;
    use crate::repository::Repository;
    use crate::repository::RepositoryUpdateState;
    use crate::repository::UpdateableRepository;

    struct TestConfigRepo(RwLock<WalletConfiguration>);

    impl Repository<Arc<WalletConfiguration>> for TestConfigRepo {
        fn get(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().clone())
        }
    }

    impl<B> UpdateableRepository<Arc<WalletConfiguration>, B> for TestConfigRepo
    where
        B: Send + Sync,
    {
        type Error = ConfigurationError;

        async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
            let mut config = self.0.write();
            let from = config.clone();
            config.lock_timeouts.background_timeout = 700;
            Ok(RepositoryUpdateState::Updated {
                from: Arc::new(from),
                to: Arc::new(config.clone()),
            })
        }
    }

    #[tokio::test]

    async fn should_store_config_to_filesystem() {
        let mut initial_wallet_config = default_wallet_config();
        initial_wallet_config.lock_timeouts.background_timeout = 500;

        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.into_path();

        let repo = FileStorageConfigurationRepository::new(
            TestConfigRepo(RwLock::new(initial_wallet_config.clone())),
            path.clone(),
        );

        let config = repo.get();
        assert_eq!(
            500, config.lock_timeouts.background_timeout,
            "should return initial_wallet_config"
        );

        repo.fetch(&TlsPinningConfig {
            base_url: "http://localhost".parse().unwrap(),
            trust_anchors: vec![],
        })
        .await
        .unwrap();

        let config = repo.get();
        assert_eq!(
            700, config.lock_timeouts.background_timeout,
            "should return value set by TestConfigRepo.fetch()"
        );

        let file_config = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();

        let repo = FileStorageConfigurationRepository::new(TestConfigRepo(RwLock::new(file_config)), path);

        let config = repo.get();
        assert_eq!(
            700, config.lock_timeouts.background_timeout,
            "should return value read from filesystem"
        );
    }

    #[tokio::test]
    async fn should_use_newer_embedded_wallet_get() {
        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.into_path();
        let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
        let config_decoding_key: EcdsaDecodingKey = (&verifying_key).into();

        let mut initially_stored_wallet_config = default_wallet_config();
        initially_stored_wallet_config.version = 10;

        // store initial wallet config having version 10
        config_file::update_config_file(path.as_path(), &initially_stored_wallet_config)
            .await
            .unwrap();

        let repo: FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>> =
            FileStorageConfigurationRepository::init(
                path.clone(),
                config_decoding_key.clone(),
                default_wallet_config(),
            )
            .await
            .unwrap();
        assert_eq!(10, repo.get().version, "should use stored config");

        let mut embedded_wallet_config = default_wallet_config();
        embedded_wallet_config.version = 20;

        let repo: FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>> =
            FileStorageConfigurationRepository::init(path.clone(), config_decoding_key, embedded_wallet_config)
                .await
                .unwrap();
        assert_eq!(20, repo.get().version, "should use newer embedded config");

        let stored_config = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();
        assert_eq!(
            20, stored_config.version,
            "newer embedded config should have been stored"
        );
    }
}
