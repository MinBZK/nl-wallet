use std::{env, path::PathBuf};

use config::{Config, ConfigError, Environment, File};

use wallet_common::config::wallet_config::WalletConfiguration;

pub fn wallet_configuration() -> Result<WalletConfiguration, ConfigError> {
    // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
    // otherwise look in the current working directory.
    let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

    Config::builder()
        .add_source(File::from(config_path.join("wallet.toml")).required(false))
        .add_source(
            Environment::with_prefix("wallet")
                .separator("__")
                .prefix_separator("_")
                .list_separator("|"),
        )
        .build()?
        .try_deserialize()
}
