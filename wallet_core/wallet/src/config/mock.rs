use std::sync::Arc;

use async_trait::async_trait;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::data::default_configuration;

use super::{ConfigurationError, ConfigurationRepository};

pub struct LocalConfigurationRepository {
    config: Arc<WalletConfiguration>,
}

impl LocalConfigurationRepository {
    pub fn new(config: WalletConfiguration) -> Self {
        LocalConfigurationRepository {
            config: Arc::new(config),
        }
    }
}

impl Default for LocalConfigurationRepository {
    fn default() -> Self {
        Self::new(default_configuration())
    }
}

#[async_trait]
impl ConfigurationRepository for LocalConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config)
    }

    async fn fetch(&self) -> Result<(), ConfigurationError> {
        Ok(())
    }
}
