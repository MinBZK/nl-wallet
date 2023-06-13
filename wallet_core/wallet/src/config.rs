use base64::{engine::general_purpose::STANDARD, Engine};
use once_cell::sync::Lazy;
use url::Url;

use wallet_common::account::jwt::EcdsaDecodingKey;

pub struct Configuration {
    pub account_server: AccountServerConfiguration,
}

pub struct AccountServerConfiguration {
    pub base_url: Url,
    pub public_key: EcdsaDecodingKey,
}

pub static CONFIGURATION: Lazy<Configuration> = Lazy::new(|| Configuration {
    account_server: AccountServerConfiguration {
        base_url: Url::parse("http://localhost:3000").unwrap(),
        public_key: EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap()),
    },
});
