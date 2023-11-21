use async_trait::async_trait;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::data::default_configuration;

use super::ConfigurationRepository;

// TODO: This will become HttpConfigurationRepository in the near future.
pub struct LocalConfigurationRepository {
    config: WalletConfiguration,
}

impl LocalConfigurationRepository {
    pub fn new(config: WalletConfiguration) -> Self {
        LocalConfigurationRepository { config }
    }
}

impl Default for LocalConfigurationRepository {
    fn default() -> Self {
        Self::new(default_configuration())
    }
}

#[async_trait]
impl ConfigurationRepository for LocalConfigurationRepository {
    fn config(&self) -> &WalletConfiguration {
        &self.config
    }
}
