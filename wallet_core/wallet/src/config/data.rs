use std::fmt::Debug;

use url::Url;

use wallet_common::account::jwt::EcdsaDecodingKey;

#[derive(Debug)]
pub struct Configuration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
}

#[derive(Debug)]
pub struct LockTimeoutConfiguration {
    pub inactive_timeout: u16,
    pub background_timeout: u16,
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
