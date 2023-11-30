use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::{
    client::HttpConfigurationClient, ConfigServerConfiguration, ConfigurationError, ConfigurationRepository,
};

pub struct HttpConfigurationRepository {
    client: HttpConfigurationClient,
    config: RwLock<Arc<WalletConfiguration>>,
}

impl HttpConfigurationRepository {
    pub fn new(config_server: ConfigServerConfiguration, initial_config: WalletConfiguration) -> Self {
        Self {
            client: HttpConfigurationClient::new(config_server.base_url),
            config: RwLock::new(Arc::new(initial_config)),
        }
    }
}

#[async_trait]
/// Here we assume that lock poisoning is a programmer error and therefore
/// we just panic when that occurs.
impl ConfigurationRepository for HttpConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config.read().unwrap())
    }

    async fn fetch(&self) -> Result<(), ConfigurationError> {
        let new_config = self.client.get_wallet_config().await?;
        let mut config = self.config.write().unwrap();
        *config = Arc::new(new_config);
        Ok(())
    }
}
