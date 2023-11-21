use async_trait::async_trait;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::{data::default_configuration, ConfigServerConfiguration, ConfigurationRepository};

pub struct RemoteConfigurationRepository {
    _config_server: ConfigServerConfiguration,
    config: WalletConfiguration,
}

impl RemoteConfigurationRepository {
    pub fn new(config_server: ConfigServerConfiguration) -> Self {
        Self {
            _config_server: config_server,
            config: default_configuration(),
        }
    }
}

#[async_trait]
impl ConfigurationRepository for RemoteConfigurationRepository {
    fn config(&self) -> &WalletConfiguration {
        &self.config
    }
}
