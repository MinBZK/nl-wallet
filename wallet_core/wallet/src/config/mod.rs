mod data;
mod local;
mod preload;
mod remote;

use wallet_common::config::wallet_config::WalletConfiguration;

pub use self::{
    data::{default_configuration, ConfigServerConfiguration},
    local::LocalConfigurationRepository,
    preload::PreloadConfigurationRepository,
    remote::RemoteConfigurationRepository,
};

pub trait ConfigurationRepository {
    fn config(&self) -> &WalletConfiguration;
}
