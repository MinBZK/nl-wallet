use std::sync::LazyLock;

use wallet_common::config::config_server_config::ConfigServerConfiguration;
use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;

const UNIVERSAL_LINK_BASE: &str = DEFAULT_UNIVERSAL_LINK_BASE;

const WALLET_CONFIG_JSON: &str = include_str!("../../wallet-config.json");
const CONFIG_SERVER_CONFIG_JSON: &str = include_str!("../../config-server-config.json");

macro_rules! config_default {
    ($name:ident) => {
        if cfg!(feature = "env_config") {
            // If the `env_config` feature is enabled, try to get the config default from
            // the environment variable of the same name, otherwise fall back to the constant.
            option_env!(stringify!($name)).unwrap_or($name)
        } else {
            $name
        }
    };
}

pub static UNIVERSAL_LINK_BASE_URL: LazyLock<BaseUrl> = LazyLock::new(|| {
    config_default!(UNIVERSAL_LINK_BASE)
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
