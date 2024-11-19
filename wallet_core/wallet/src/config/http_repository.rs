use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use reqwest::Certificate;
use tracing::info;

use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::urls::BaseUrl;

use crate::config::http_client::HttpConfigurationClient;
use crate::config::ConfigurationError;
use crate::config::ConfigurationRepository;
use crate::config::ConfigurationUpdateState;
use crate::config::UpdateableConfigurationRepository;

pub struct HttpConfigurationRepository {
    client: HttpConfigurationClient,
    config: RwLock<Arc<WalletConfiguration>>,
}

impl HttpConfigurationRepository {
    pub async fn new(
        base_url: BaseUrl,
        trust_anchors: Vec<Certificate>,
        signing_public_key: EcdsaDecodingKey,
        storage_path: PathBuf,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        Ok(Self {
            client: HttpConfigurationClient::new(base_url, trust_anchors, signing_public_key, storage_path).await?,
            config: RwLock::new(Arc::new(initial_config)),
        })
    }
}

impl ConfigurationRepository for HttpConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config.read())
    }
}

/// Here we assume that lock poisoning is a programmer error and therefore
/// we just panic when that occurs.
impl UpdateableConfigurationRepository for HttpConfigurationRepository {
    async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError> {
        if let Some(new_config) = self.client.get_wallet_config().await? {
            {
                let current_config = self.config.read();
                if new_config.version <= current_config.version {
                    info!(
                        "Received wallet configuration with version: {}, but we have version: {}",
                        new_config.version, current_config.version
                    );
                    return Ok(ConfigurationUpdateState::Unmodified);
                }
            }

            info!("Received new wallet configuration with version: {}", new_config.version);

            let mut config = self.config.write();
            *config = Arc::new(new_config);

            Ok(ConfigurationUpdateState::Updated)
        } else {
            info!("No new wallet configuration received");

            Ok(ConfigurationUpdateState::Unmodified)
        }
    }
}
