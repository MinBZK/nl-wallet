use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use url::Url;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::{
    http_client::HttpConfigurationClient, ConfigurationError, ConfigurationRepository, ConfigurationUpdateState,
    UpdateableConfigurationRepository,
};

pub struct HttpConfigurationRepository {
    client: HttpConfigurationClient,
    config: RwLock<Arc<WalletConfiguration>>,
}

impl HttpConfigurationRepository {
    pub async fn new(
        base_url: Url,
        storage_path: PathBuf,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        Ok(Self {
            client: HttpConfigurationClient::new(base_url, storage_path).await?,
            config: RwLock::new(Arc::new(initial_config)),
        })
    }
}

impl ConfigurationRepository for HttpConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config.read().unwrap())
    }
}

#[async_trait]
/// Here we assume that lock poisoning is a programmer error and therefore
/// we just panic when that occurs.
impl UpdateableConfigurationRepository for HttpConfigurationRepository {
    async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError> {
        if let Some(new_config) = self.client.get_wallet_config().await? {
            let mut config = self.config.write().unwrap();
            *config = Arc::new(new_config);
            Ok(ConfigurationUpdateState::Updated)
        } else {
            Ok(ConfigurationUpdateState::Unmodified)
        }
    }
}
