use std::{fmt::Debug, sync::Arc};

use base64::prelude::*;
use nl_wallet_mdoc::holder::TrustAnchor;
use once_cell::sync::Lazy;
use url::Url;

use wallet_common::account::jwt::EcdsaDecodingKey;

#[derive(Debug)]
pub struct Configuration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub mdoc_trust_anchors: Lazy<Arc<Vec<TrustAnchor<'static>>>>,
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
            },
            mdoc_trust_anchors: Lazy::new(|| {
                TRUST_ANCHOR_CERTS
                    .iter()
                    .map(|anchor| {
                        let der = base64::engine::general_purpose::STANDARD
                            .decode(anchor.as_bytes())
                            .expect("failed to base64-decode trust anchor certificate");

                        // "Leak" the bytes, meaning they get lifetime 'static: the duration of the program.
                        let static_ref: &'static [u8] = Box::leak(Box::new(der));

                        TrustAnchor::try_from_cert_der(static_ref).expect("failed to parse trust anchor")
                    })
                    .collect::<Vec<TrustAnchor<'static>>>()
                    .into()
            }),
        }
    }
}

const TRUST_ANCHOR_CERTS: [&str; 1] = ["MIIBgDCCASagAwIBAgIUA21zb+2cuU3O3IHdqIWQNWF6+fwwCgYIKoZIzj0EAwIwDzENMAsGA1UEAwwEbXljYTAeFw0yMzA4MTAxNTEwNDBaFw0yNDA4MDkxNTEwNDBaMA8xDTALBgNVBAMMBG15Y2EwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATHjlwqhDY6oe0hXL2n5jY1RjPboePKABhtItYpTwqi0MO6tTTIxdED4IY60Qvu9DCBcW5C/jju+qMy/kFUiSuPo2AwXjAdBgNVHQ4EFgQUSjuvOcpIpcOrbq8sMjgMsk9IYyQwHwYDVR0jBBgwFoAUSjuvOcpIpcOrbq8sMjgMsk9IYyQwDwYDVR0TAQH/BAUwAwEB/zALBgNVHQ8EBAMCAQYwCgYIKoZIzj0EAwIDSAAwRQIgL1Gc3qKGIyiAyiL4WbeR1r22KbwoTfMk11kq6xWBpDACIQDfyPw+qs2nh8R8WEFQzk+zJlz/4DNMXoT7M9cjFwg+Xg=="];
