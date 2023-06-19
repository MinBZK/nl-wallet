use std::fmt::Debug;

use url::Url;

use wallet_common::account::jwt::EcdsaDecodingKey;

#[derive(Debug)]
pub struct Configuration {
    pub account_server: AccountServerConfiguration,
}

pub struct AccountServerConfiguration {
    pub base_url: Url,
    pub public_key: EcdsaDecodingKey,
}

impl Debug for AccountServerConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountServerConfiguration")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}
