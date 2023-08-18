use std::fmt::Debug;

use base64::prelude::*;
use url::Url;

use wallet_common::account::jwt::EcdsaDecodingKey;

#[derive(Debug)]
pub struct Configuration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
}

#[derive(Debug)]
pub struct LockTimeoutConfiguration {
    /// App inactivity lock timeout in seconds
    pub inactive_timeout: u16,
    /// App background lock timeout in seconds
    pub background_timeout: u16,
}

pub struct AccountServerConfiguration {
    // The base URL for the Account Server API
    pub base_url: Url,
    // The known public key for the Wallet Provider
    pub certificate_public_key: EcdsaDecodingKey,
    pub instruction_result_public_key: EcdsaDecodingKey,
}

#[derive(Debug, Clone)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: Url,
    pub digid_url: Url,
    pub digid_client_id: String,
    pub digid_redirect_uri: Url,
}

impl Debug for AccountServerConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountServerConfiguration")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            lock_timeouts: LockTimeoutConfiguration {
                inactive_timeout: 5 * 60,
                background_timeout: 5 * 60,
            },
            account_server: AccountServerConfiguration {
                base_url: Url::parse("http://localhost:3000/api/v1/").unwrap(),
                certificate_public_key: EcdsaDecodingKey::from_sec1(&BASE64_STANDARD.decode("").unwrap()),
                instruction_result_public_key: EcdsaDecodingKey::from_sec1(&BASE64_STANDARD.decode("").unwrap()),
            },
            pid_issuance: PidIssuanceConfiguration {
                pid_issuer_url: Url::parse("http://10.0.2.2:3003/").unwrap(),
                digid_url: Url::parse(
                    "https://example.com/digid-connector",
                )
                .unwrap(),
                digid_client_id: "SSSS".to_string(),
                digid_redirect_uri: Url::parse("walletdebuginteraction://wallet.edi.rijksoverheid.nl/authentication")
                    .unwrap(),
            },
        }
    }
}
