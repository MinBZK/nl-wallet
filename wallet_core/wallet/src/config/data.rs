use std::sync::LazyLock;

use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;

const WALLET_CONFIG_JSON: &str = include_str!("../../wallet-config.json");
const CONFIG_SERVER_CONFIG_JSON: &str = include_str!("../../config-server-config.json");

pub static UNIVERSAL_LINK_BASE_URL: LazyLock<BaseUrl> = LazyLock::new(|| {
    option_env!("UNIVERSAL_LINK_BASE")
        .unwrap_or(DEFAULT_UNIVERSAL_LINK_BASE)
        .parse::<BaseUrl>()
        .expect("Could not parse universal link base url")
});

pub fn init_universal_link_base_url() {
    LazyLock::force(&UNIVERSAL_LINK_BASE_URL);
}

pub fn default_wallet_config() -> WalletConfiguration {
    // The JSON has already been parsed in build.rs, so unwrap is safe here
    serde_json::from_str(WALLET_CONFIG_JSON).unwrap()
}

pub fn default_config_server_config() -> ConfigServerConfiguration {
    // The JSON has already been parsed in build.rs, so unwrap is safe here
    serde_json::from_str(CONFIG_SERVER_CONFIG_JSON).unwrap()
}
