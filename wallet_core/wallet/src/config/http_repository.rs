use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use url::Url;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::{
    client::HttpConfigurationClient, ConfigurationError, ConfigurationRepository, UpdateableConfigurationRepository,
};

pub struct HttpConfigurationRepository {
    client: HttpConfigurationClient,
    config: RwLock<Arc<WalletConfiguration>>,
}

impl HttpConfigurationRepository {
    pub fn new(base_url: Url, initial_config: WalletConfiguration) -> Self {
        Self {
            client: HttpConfigurationClient::new(base_url),
            config: RwLock::new(Arc::new(initial_config)),
        }
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
    async fn fetch(&self) -> Result<(), ConfigurationError> {
        let new_config = self.client.get_wallet_config().await?;
        let mut config = self.config.write().unwrap();
        *config = Arc::new(new_config);
        Ok(())
    }
}
