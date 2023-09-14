use std::fmt::Debug;

use base64::prelude::*;
use url::Url;

use nl_wallet_mdoc::{holder::TrustAnchor, utils::x509::OwnedTrustAnchor};
use wallet_common::account::jwt::EcdsaDecodingKey;

// Configuration parameters, these MUST be on a single line for the setup-devenv.sh script to work.
const BASE_URL: &str = "http://localhost:3000/api/v1/";
const CERTIFICATE_PUBLIC_KEY: &str = "";
const DIGID_CLIENT_ID: &str = "";
const DIGID_URL: &str = "https://localhost/8006/";
const INSTRUCTION_RESULT_PUBLIC_KEY: &str = "";
const PID_ISSUER_URL: &str = "http://localhost:3003/";
const TRUST_ANCHOR_CERTS: [&str; 1] = ["MIIBgDCCASagAwIBAgIUA21zb+2cuU3O3IHdqIWQNWF6+fwwCgYIKoZIzj0EAwIwDzENMAsGA1UEAwwEbXljYTAeFw0yMzA4MTAxNTEwNDBaFw0yNDA4MDkxNTEwNDBaMA8xDTALBgNVBAMMBG15Y2EwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATHjlwqhDY6oe0hXL2n5jY1RjPboePKABhtItYpTwqi0MO6tTTIxdED4IY60Qvu9DCBcW5C/jju+qMy/kFUiSuPo2AwXjAdBgNVHQ4EFgQUSjuvOcpIpcOrbq8sMjgMsk9IYyQwHwYDVR0jBBgwFoAUSjuvOcpIpcOrbq8sMjgMsk9IYyQwDwYDVR0TAQH/BAUwAwEB/zALBgNVHQ8EBAMCAQYwCgYIKoZIzj0EAwIDSAAwRQIgL1Gc3qKGIyiAyiL4WbeR1r22KbwoTfMk11kq6xWBpDACIQDfyPw+qs2nh8R8WEFQzk+zJlz/4DNMXoT7M9cjFwg+Xg=="];
const WALLET_REDIRECT_URI: &str = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/authentication";

#[derive(Debug, Clone)]
pub struct Configuration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub mdoc_trust_anchors: Vec<OwnedTrustAnchor>,
}

#[derive(Debug, Clone)]
pub struct LockTimeoutConfiguration {
    /// App inactivity lock timeout in seconds
    pub inactive_timeout: u16,
    /// App background lock timeout in seconds
    pub background_timeout: u16,
}

#[derive(Clone)]
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

impl Configuration {
    pub fn mdoc_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.mdoc_trust_anchors.iter().map(|anchor| anchor.into()).collect()
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
                base_url: Url::parse(BASE_URL).unwrap(),
                certificate_public_key: EcdsaDecodingKey::from_sec1(
                    &BASE64_STANDARD.decode(CERTIFICATE_PUBLIC_KEY).unwrap(),
                ),
                instruction_result_public_key: EcdsaDecodingKey::from_sec1(
                    &BASE64_STANDARD.decode(INSTRUCTION_RESULT_PUBLIC_KEY).unwrap(),
                ),
            },
            pid_issuance: PidIssuanceConfiguration {
                pid_issuer_url: Url::parse(PID_ISSUER_URL).unwrap(),
                digid_url: Url::parse(DIGID_URL).unwrap(),
                digid_client_id: DIGID_CLIENT_ID.to_string(),
                digid_redirect_uri: Url::parse(WALLET_REDIRECT_URI).unwrap(),
            },
            mdoc_trust_anchors: TRUST_ANCHOR_CERTS
                .iter()
                .map(|anchor| {
                    base64::engine::general_purpose::STANDARD
                        .decode(anchor.as_bytes())
                        .expect("failed to base64-decode trust anchor certificate")
                        .as_slice()
                        .try_into()
                        .expect("failed to parse trust anchor")
                })
                .collect(),
        }
    }
}

impl Debug for AccountServerConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountServerConfiguration")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}
