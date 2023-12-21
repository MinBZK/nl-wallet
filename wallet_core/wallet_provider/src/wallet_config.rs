use std::{env, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use wallet_common::{
    account::serialization::DerVerifyingKey,
    config::wallet_config::{LockTimeoutConfiguration, WalletConfiguration},
};

pub fn wallet_configuration() -> Result<WalletConfiguration, ConfigError> {
    // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
    // otherwise look in the current working directory.
    let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

    let default_certificate_public_key: DerVerifyingKey = (*SigningKey::random(&mut OsRng).verifying_key()).into();
    let default_instruction_result_public_key: DerVerifyingKey =
        (*SigningKey::random(&mut OsRng).verifying_key()).into();

    Config::builder()
        .set_default("account_server.certificate_public_key", default_certificate_public_key)?
        .set_default(
            "account_server.instruction_result_public_key",
            default_instruction_result_public_key,
        )?
        .set_default(
            "lock_timeouts.inactive_timeout",
            LockTimeoutConfiguration::default().inactive_timeout,
        )?
        .set_default(
            "lock_timeouts.background_timeout",
            LockTimeoutConfiguration::default().background_timeout,
        )?
        .add_source(File::from(config_path.join("wallet.toml")).required(false))
        .add_source(
            Environment::with_prefix("wallet")
                .separator("__")
                .prefix_separator("_")
                .ignore_empty(true)
                .try_parsing(true)
                .with_list_parse_key("disclosure.rp_trust_anchors")
                .with_list_parse_key("mdoc_trust_anchors")
                .list_separator("|"),
        )
        .build()?
        .try_deserialize()
}
