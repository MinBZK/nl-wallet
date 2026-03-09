use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::info;

use http_utils::reqwest::IntoReqwestClient;
use jwt::DEFAULT_VALIDATIONS;
use jwt::EcdsaDecodingKey;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::config::ConfigurationError;
use crate::repository::EtagHttpClient;
use crate::repository::HttpClient;
use crate::repository::HttpResponse;
use crate::repository::Repository;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::WalletConfigJwt;
use super::file_repository::RawJwtProvider;

type ConfigState = (Arc<WalletConfiguration>, Option<Arc<WalletConfigJwt>>);

pub struct HttpConfigurationRepository<B> {
    client: EtagHttpClient<WalletConfigJwt, B, ConfigurationError>,
    signing_public_key: EcdsaDecodingKey,
    config: RwLock<ConfigState>,
}

impl<B> HttpConfigurationRepository<B> {
    pub async fn new(
        signing_public_key: EcdsaDecodingKey,
        storage_path: PathBuf,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let repo = Self {
            client: EtagHttpClient::new(
                "wallet-config".parse().expect("should be a valid filename"),
                storage_path,
            )
            .await?,
            signing_public_key,
            config: RwLock::new((Arc::new(initial_config), None)),
        };

        Ok(repo)
    }
}

impl<C> Repository<Arc<WalletConfiguration>> for HttpConfigurationRepository<C> {
    fn get(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config.read().0)
    }
}

impl<B> RawJwtProvider for HttpConfigurationRepository<B> {
    fn last_raw_jwt(&self) -> Option<Arc<WalletConfigJwt>> {
        self.config.read().1.as_ref().map(Arc::clone)
    }
}

/// Here we assume that lock poisoning is a programmer error and therefore
/// we just panic when that occurs.
impl<B> UpdateableRepository<Arc<WalletConfiguration>, B> for HttpConfigurationRepository<B>
where
    B: IntoReqwestClient + Clone + Hash + Send + Sync,
{
    type Error = ConfigurationError;

    async fn fetch(&self, client_builder: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
        let response = self.client.fetch(client_builder).await?;
        match response {
            HttpResponse::Parsed(parsed_response) => {
                let (_, new_config) =
                    parsed_response.parse_and_verify(&self.signing_public_key, &DEFAULT_VALIDATIONS)?;

                {
                    let current_config = self.config.read();
                    if new_config.version <= current_config.0.version {
                        info!(
                            "Received wallet configuration with version: {}, but we have version: {}",
                            new_config.version, current_config.0.version
                        );
                        return Ok(RepositoryUpdateState::Unmodified(Arc::clone(&current_config.0)));
                    }
                }

                info!("Received new wallet configuration with version: {}", new_config.version);

                let mut config = self.config.write();
                let from = Arc::clone(&config.0);
                *config = (Arc::new(new_config), Some(Arc::new(parsed_response)));

                Ok(RepositoryUpdateState::Updated {
                    from,
                    to: Arc::clone(&config.0),
                })
            }
            HttpResponse::NotModified => {
                info!("No new wallet configuration received");

                Ok(RepositoryUpdateState::Unmodified(self.get()))
            }
        }
    }
}
