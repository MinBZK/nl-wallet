use std::path::PathBuf;
use std::sync::Arc;

use crypto::kid::VerifyingKeyWithKid;
use derive_more::Constructor;
use http_utils::reqwest::IntoReqwestClient;
use jwt::error::JwtVerifyError;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::ConfigurationError;
use super::HttpConfigurationRepository;
use super::WALLET_CONFIG_VALIDATION;
use super::WalletConfigJwt;
use super::config_file;
use crate::repository::Repository;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

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
        config_key: VerifyingKeyWithKid,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let etag_path = config_file::path_for_config_etag(&storage_path);
        let default_config = match config_file::get_config_file(storage_path.as_path()).await? {
            Some(jwt) => {
                match jwt.parse_and_verify_by_kid(&config_key, WALLET_CONFIG_VALIDATION.clone()) {
                    Ok((_, stored_config)) if stored_config.version >= initial_config.version => stored_config,

                    // Stored configuration is signed by an older key identifier
                    Err(JwtVerifyError::KeyNotFound(kid)) => {
                        tracing::info!("Configuration JWT signed by different key identifier: {kid}");

                        // Clear ETag file to ensure a fresh fetch even when ETags collide.
                        // Collisions can happen e.g. on nginx when configs are of same size and have same mtime.
                        if let Err(err) = tokio::fs::remove_file(&etag_path).await
                            && err.kind() != std::io::ErrorKind::NotFound
                        {
                            tracing::warn!("Failed to remove ETag file: {err}");
                        }
                        initial_config
                    }

                    // Initial config is newer or JWT is tampered/invalid: fall back to the embedded config.
                    // We do not write the embedded config to disk since it has no corresponding JWT.
                    // The next successful HTTP fetch will populate the file.
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse and verify wallet configuration JWT, falling back to embedded config: {e}"
                        );
                        initial_config
                    }

                    _ => initial_config,
                }
            }
            None => initial_config,
        };

        Ok(Self::new(
            HttpConfigurationRepository::new(
                config_key,
                config_file::path_for_config_etag(&storage_path),
                default_config,
            ),
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

    use http_utils::client::InternalHttpConfig;
    use http_utils::client::TlsPinningConfig;
    use jwt::SignedJwt;
    use parking_lot::RwLock;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use super::RawJwtProvider;
    use crate::config::ConfigSigningKey;
    use crate::config::ConfigurationError;
    use crate::config::FileStorageConfigurationRepository;
    use crate::config::HttpConfigurationRepository;
    use crate::config::WALLET_CONFIG_VALIDATION;
    use crate::config::WalletConfigJwt;
    use crate::config::config_file;
    use crate::config::default_wallet_config;
    use crate::repository::Repository;
    use crate::repository::RepositoryUpdateState;
    use crate::repository::UpdateableRepository;

    struct TestConfigRepo {
        signing_key: ConfigSigningKey,
        config: RwLock<(WalletConfiguration, Option<Arc<WalletConfigJwt>>)>,
    }

    impl TestConfigRepo {
        fn new(signing_key: ConfigSigningKey, config: WalletConfiguration) -> Self {
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
            let jwt = SignedJwt::sign_with_kid(&new_config, &self.signing_key).await.unwrap();
            self.config.write().1 = Some(Arc::new(jwt.into_unverified()));
            Ok(RepositoryUpdateState::Updated {
                from: Arc::new(from),
                to: Arc::new(new_config),
            })
        }
    }

    fn config_with_version(version: u64) -> WalletConfiguration {
        let mut config = default_wallet_config();
        config.version = version;
        config
    }

    #[tokio::test]
    async fn should_store_config_to_filesystem() {
        let signing_key = ConfigSigningKey::new_with_kid("0");
        let config_key = signing_key.to_config_key_with_kid();

        let mut initial_wallet_config = default_wallet_config();
        initial_wallet_config.lock_timeouts.background_timeout = 500;

        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.path().to_path_buf();

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
        let (_, file_config) = jwt
            .parse_and_verify_by_kid(&config_key, WALLET_CONFIG_VALIDATION.clone())
            .unwrap();

        assert_eq!(
            700, file_config.lock_timeouts.background_timeout,
            "should return value read from filesystem"
        );
    }

    #[tokio::test]
    async fn should_use_newer_embedded_wallet_get() {
        let signing_key = ConfigSigningKey::new_with_kid("0");
        let config_key = signing_key.to_config_key_with_kid();

        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.path().to_path_buf();

        let initially_stored_wallet_config = config_with_version(10);

        // Store a signed JWT with version 10 on disk
        let jwt = SignedJwt::sign_with_kid(&initially_stored_wallet_config, &signing_key)
            .await
            .unwrap();
        config_file::update_config_file(path.as_path(), &jwt.into_unverified())
            .await
            .unwrap();

        let repo: FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>> =
            FileStorageConfigurationRepository::init(path.clone(), config_key.clone(), config_with_version(0))
                .await
                .unwrap();
        assert_eq!(10, repo.get().version, "should use stored config");

        let repo: FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>> =
            FileStorageConfigurationRepository::init(path.clone(), config_key.clone(), config_with_version(20))
                .await
                .unwrap();
        assert_eq!(20, repo.get().version, "should use newer embedded config");

        // The JWT on disk still has version 10; the embedded config (v20) is not written back
        // since it has no corresponding JWT. The next HTTP fetch will update the file.
        let jwt = config_file::get_config_file(path.as_path()).await.unwrap().unwrap();
        let (_, stored_config) = jwt
            .parse_and_verify_by_kid(&config_key, WALLET_CONFIG_VALIDATION.clone())
            .unwrap();
        assert_eq!(
            10, stored_config.version,
            "disk should still contain the original version-10 JWT"
        );
    }

    #[tokio::test]
    async fn should_clear_etag_on_newer_kid() {
        let config_dir = tempfile::tempdir().unwrap();
        let path = config_dir.path().to_path_buf();

        // Store a signed JWT with kid v0
        let signing_key_v0 = ConfigSigningKey::new_with_kid("0");
        let jwt = SignedJwt::sign_with_kid(&config_with_version(20), &signing_key_v0)
            .await
            .unwrap();
        config_file::update_config_file(&path, &jwt.into_unverified())
            .await
            .unwrap();

        // Create an ETag file
        tokio::fs::write(config_file::path_for_config_etag(&path), "V0")
            .await
            .unwrap();

        // Create repo with kid v1 simulating config key roll over
        let config_key_v1 = ConfigSigningKey::new_with_kid("1").to_config_key_with_kid();

        let repo: FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>> =
            FileStorageConfigurationRepository::init(path.clone(), config_key_v1, config_with_version(10))
                .await
                .unwrap();
        assert_eq!(
            10,
            repo.get().version,
            "should use embedded config even though version is older"
        );

        // The ETag file should be cleared
        assert!(!config_file::path_for_config_etag(&path).exists());
    }
}
