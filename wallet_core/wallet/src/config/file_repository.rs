use std::path::PathBuf;
use std::sync::Arc;

use derive_more::Constructor;

use http_utils::reqwest::IntoReqwestClient;
use jwt::DEFAULT_VALIDATIONS;
use jwt::EcdsaDecodingKey;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::repository::Repository;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::ConfigurationError;
use super::HttpConfigurationRepository;
use super::WalletConfigJwt;
use super::config_file;

pub trait RawJwtProvider {
    fn last_raw_jwt(&self) -> Option<Arc<WalletConfigJwt>>;
}

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
            Some(jwt) => {
                match jwt.parse_and_verify(&signing_public_key, &DEFAULT_VALIDATIONS) {
                    Ok((_, stored_config)) if stored_config.version >= initial_config.version => stored_config,
                    // Initial config is newer or JWT is tampered/invalid: fall back to the embedded config.
                    // We do not write the embedded config to disk since it has no corresponding JWT.
                    // The next successful HTTP fetch will populate the file.
                    _ => initial_config,
                }
            }
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
    T: UpdateableRepository<Arc<WalletConfiguration>, B, Error = ConfigurationError> + RawJwtProvider + Sync,
    B: IntoReqwestClient + Send + Sync,
{
    type Error = ConfigurationError;

    async fn fetch(&self, client_builder: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
        let result = self.wrapped.fetch(client_builder).await?;

        if let RepositoryUpdateState::Updated { .. } = result
            && let Some(jwt) = self.wrapped.last_raw_jwt()
        {
            config_file::update_config_file(self.storage_path.as_path(), &jwt).await?;
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

    use http_utils::client::InternalHttpConfig;
    use http_utils::client::TlsPinningConfig;
    use jwt::DEFAULT_VALIDATIONS;
    use jwt::EcdsaDecodingKey;
    use jwt::SignedJwt;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use crate::config::ConfigurationError;
    use crate::config::FileStorageConfigurationRepository;
    use crate::config::HttpConfigurationRepository;
    use crate::config::WalletConfigJwt;
    use crate::config::config_file;
    use crate::config::default_wallet_config;
    use crate::repository::Repository;
    use crate::repository::RepositoryUpdateState;
    use crate::repository::UpdateableRepository;

    use super::RawJwtProvider;

    struct TestConfigRepo {
        signing_key: SigningKey,
        config: RwLock<(WalletConfiguration, Option<Arc<WalletConfigJwt>>)>,
    }

    impl TestConfigRepo {
        fn new(signing_key: SigningKey, config: WalletConfiguration) -> Self {
            Self {
                signing_key,
                config: RwLock::new((config, None)),
            }
        }
    }

    impl Repository<Arc<WalletConfiguration>> for TestConfigRepo {
        fn get(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.config.read().0.clone())
        }
    }

    impl RawJwtProvider for TestConfigRepo {
        fn last_raw_jwt(&self) -> Option<Arc<WalletConfigJwt>> {
            self.config.read().1.as_ref().map(Arc::clone)
        }
    }

    impl<B> UpdateableRepository<Arc<WalletConfiguration>, B> for TestConfigRepo
    where
        B: Send + Sync,
    {
        type Error = ConfigurationError;

        async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
            let (from, new_config) = {
                let mut guard = self.config.write();
                let from = guard.0.clone();
                guard.0.lock_timeouts.background_timeout = 700;
                (from, guard.0.clone())
            };
            let jwt = SignedJwt::sign(&new_config, &self.signing_key).await.unwrap();
            self.config.write().1 = Some(Arc::new(jwt.into_unverified()));
            Ok(RepositoryUpdateState::Updated {
                from: Arc::new(from),
                to: Arc::new(new_config),
            })
        }
    }

    #[tokio::test]
    async fn should_store_config_to_filesystem() {
        let signing_key = SigningKey::random(&mut OsRng);
        let decoding_key = EcdsaDecodingKey::from(signing_key.verifying_key());

        let mut initial_wallet_config = default_wallet_config();
        initial_wallet_config.lock_timeouts.background_timeout = 500;

        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.keep();

        let repo = FileStorageConfigurationRepository::new(
            TestConfigRepo::new(signing_key, initial_wallet_config.clone()),
            path.clone(),
        );

        let config = repo.get();
        assert_eq!(
            500, config.lock_timeouts.background_timeout,
            "should return initial_wallet_config"
        );

        repo.fetch(&InternalHttpConfig::try_new("http://localhost".parse().unwrap()).unwrap())
            .await
            .unwrap();

        let config = repo.get();
        assert_eq!(
            700, config.lock_timeouts.background_timeout,
            "should return value set by TestConfigRepo.fetch()"
        );

        // Verify the JWT was written to disk and can be parsed back
        let jwt = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();
        let (_, file_config) = jwt.parse_and_verify(&decoding_key, &DEFAULT_VALIDATIONS).unwrap();

        assert_eq!(
            700, file_config.lock_timeouts.background_timeout,
            "should return value read from filesystem"
        );
    }

    #[tokio::test]
    async fn should_use_newer_embedded_wallet_get() {
        let signing_key = SigningKey::random(&mut OsRng);
        let config_decoding_key = EcdsaDecodingKey::from(signing_key.verifying_key());

        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.keep();

        let mut initially_stored_wallet_config = default_wallet_config();
        initially_stored_wallet_config.version = 10;

        // Store a signed JWT with version 10 on disk
        let jwt = SignedJwt::sign(&initially_stored_wallet_config, &signing_key)
            .await
            .unwrap();
        config_file::update_config_file(path.as_path(), &jwt.into_unverified())
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
            FileStorageConfigurationRepository::init(path.clone(), config_decoding_key.clone(), embedded_wallet_config)
                .await
                .unwrap();
        assert_eq!(20, repo.get().version, "should use newer embedded config");

        // The JWT on disk still has version 10; the embedded config (v20) is not written back
        // since it has no corresponding JWT. The next HTTP fetch will update the file.
        let jwt = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();
        let (_, stored_config) = jwt
            .parse_and_verify(&config_decoding_key, &DEFAULT_VALIDATIONS)
            .unwrap();
        assert_eq!(
            10, stored_config.version,
            "disk should still contain the original version-10 JWT"
        );
    }
}
