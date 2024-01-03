use std::sync::Arc;

use async_trait::async_trait;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::data::default_configuration;

use super::{
    ConfigurationError, ConfigurationRepository, ConfigurationUpdateState, ObservableConfigurationRepository,
    UpdateableConfigurationRepository,
};

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

impl ConfigurationRepository for LocalConfigurationRepository {
    fn config(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config)
    }
}

#[async_trait]
impl UpdateableConfigurationRepository for LocalConfigurationRepository {
    async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError> {
        Ok(ConfigurationUpdateState::Updated)
    }
}

#[async_trait]
impl ObservableConfigurationRepository for LocalConfigurationRepository {
    fn register_callback_on_update<F>(&self, _callback: F)
    where
        F: Fn(Arc<WalletConfiguration>) + Send + Sync,
    {
    }

    fn clear_callback(&self) {}
}
