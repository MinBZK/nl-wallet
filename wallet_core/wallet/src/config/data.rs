use std::fmt::Debug;

use base64::prelude::*;
use once_cell::sync::Lazy;
use p256::{ecdsa::VerifyingKey, pkcs8::DecodePublicKey};
use url::{ParseError, Url};

use nl_wallet_mdoc::{holder::TrustAnchor, utils::x509::OwnedTrustAnchor};
use wallet_common::account::jwt::EcdsaDecodingKey;

// This should always equal the deep/universal link configured for the app.
static UNIVERSAL_LINK_BASE: Lazy<Url> =
    Lazy::new(|| Url::parse("walletdebuginteraction://wallet.edi.rijksoverheid.nl/").unwrap());

// Each of these values can be overridden from environment variables at compile time
// when the `env_config` feature is enabled. Additionally, environment variables can
// be added to using a file named `.env` in root directory of this crate.
const BASE_URL: &str = "http://localhost:3000/api/v1/";

// todo: this is now a random public_key to ensure the accountserver configuration contains legal values. Can we actually have a default for this?
const CERTIFICATE_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEW2zhAd/0VH7PzLdmAfDEmHpSWwbVRfr5H31fo2rQWtyU\
                                      oWZT/C5WSeVm5Ktp6nCwnOwhhJLLGb4K3LtUJeLKjA==";

const DIGID_CLIENT_ID: &str = "";
const DIGID_URL: &str = "https://localhost/8006/";

// todo: this is now a random public_key to ensure the accountserver configuration contains legal values. Can we actually have a default for this?
const INSTRUCTION_RESULT_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEpQqynmHM6Iey1gqLPtTi4T9PflzCDpttyk\
                                             oP/iW47jE1Ra6txPJEPq4FVQdqQJEXcJ7i8TErVQ3KNB823StXnA==";

const PID_ISSUER_URL: &str = "http://localhost:3003/";
// TODO: Remove this hardcoded CA that is necessary for tests.
const MDOC_TRUST_ANCHORS: &str = "MIIBgDCCASagAwIBAgIUA21zb+2cuU3O3IHdqIWQNWF6+fwwCgYIKoZIzj0EAwIwDzENMAsGA1UEAwwE\
                                  bXljYTAeFw0yMzA4MTAxNTEwNDBaFw0yNDA4MDkxNTEwNDBaMA8xDTALBgNVBAMMBG15Y2EwWTATBgcq\
                                  hkjOPQIBBggqhkjOPQMBBwNCAATHjlwqhDY6oe0hXL2n5jY1RjPboePKABhtItYpTwqi0MO6tTTIxdED\
                                  4IY60Qvu9DCBcW5C/jju+qMy/kFUiSuPo2AwXjAdBgNVHQ4EFgQUSjuvOcpIpcOrbq8sMjgMsk9IYyQw\
                                  HwYDVR0jBBgwFoAUSjuvOcpIpcOrbq8sMjgMsk9IYyQwDwYDVR0TAQH/BAUwAwEB/zALBgNVHQ8EBAMC\
                                  AQYwCgYIKoZIzj0EAwIDSAAwRQIgL1Gc3qKGIyiAyiL4WbeR1r22KbwoTfMk11kq6xWBpDACIQDfyPw+\
                                  qs2nh8R8WEFQzk+zJlz/4DNMXoT7M9cjFwg+Xg==";
// TODO: Remove this randomly generated CA.
const RP_TRUST_ANCHORS: &str = "MIIBlDCCATqgAwIBAgIUMmfPjx+jkrbY6twjDTCNHtnoPB4wCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwO\
                                Y2EuZXhhbXBsZS5jb20wHhcNMjMxMTA3MTA1NDEzWhcNMjQxMTA2MTA1NDEzWjAZMRcwFQYDVQQDDA5j\
                                YS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABPD39ZBr4/cNp76DturjGWRtjjqU\
                                qQgt+Xny57i2xFYHRSogzdQYQbqdUOzEfBZeW3GkBjzmbCmug2zHr5B54UKjYDBeMB0GA1UdDgQWBBR4\
                                cYdOOiKhp1xTmK4ZW8JMG4CggzAfBgNVHSMEGDAWgBR4cYdOOiKhp1xTmK4ZW8JMG4CggzAPBgNVHRMB\
                                Af8EBTADAQH/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNIADBFAiBpJ/sEsPeTm8A2XYwRmu6NOkoL\
                                NqhPN569XKLTR6rVdwIhANOMtj2LwDUG2YcLkSBPSdhh/i/iCgTeuZQpOI8y+kBw";

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

#[derive(Debug, Clone)]
pub struct Configuration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub disclosure: DisclosureConfiguration,
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
    pub digid_redirect_path: String,
}

#[derive(Debug, Clone)]
pub struct DisclosureConfiguration {
    pub uri_base_path: String,
    pub rp_trust_anchors: Vec<OwnedTrustAnchor>,
}

fn parse_trust_anchors(source: &str) -> Vec<OwnedTrustAnchor> {
    source
        .split('|')
        .map(|anchor| {
            base64::engine::general_purpose::STANDARD
                .decode(anchor.as_bytes())
                .expect("failed to base64-decode trust anchor certificate")
                .as_slice()
                .try_into()
                .expect("failed to parse trust anchor")
        })
        .collect()
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
                base_url: Url::parse(config_default!(BASE_URL)).unwrap(),
                certificate_public_key: VerifyingKey::from_public_key_der(
                    &BASE64_STANDARD.decode(config_default!(CERTIFICATE_PUBLIC_KEY)).unwrap(),
                )
                .unwrap()
                .into(),
                instruction_result_public_key: VerifyingKey::from_public_key_der(
                    &BASE64_STANDARD
                        .decode(config_default!(INSTRUCTION_RESULT_PUBLIC_KEY))
                        .unwrap(),
                )
                .unwrap()
                .into(),
            },
            pid_issuance: PidIssuanceConfiguration {
                pid_issuer_url: Url::parse(config_default!(PID_ISSUER_URL)).unwrap(),
                digid_url: Url::parse(config_default!(DIGID_URL)).unwrap(),
                digid_client_id: String::from(config_default!(DIGID_CLIENT_ID)),
                digid_redirect_path: "authentication".to_string(),
            },
            disclosure: DisclosureConfiguration {
                uri_base_path: "disclosure".to_string(),
                rp_trust_anchors: parse_trust_anchors(config_default!(RP_TRUST_ANCHORS)),
            },
            mdoc_trust_anchors: parse_trust_anchors(config_default!(MDOC_TRUST_ANCHORS)),
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

impl PidIssuanceConfiguration {
    pub fn digid_redirect_uri(&self) -> Result<Url, ParseError> {
        UNIVERSAL_LINK_BASE.join(&self.digid_redirect_path)
    }
}

impl DisclosureConfiguration {
    pub fn uri_base(&self) -> Result<Url, ParseError> {
        UNIVERSAL_LINK_BASE.join(&self.uri_base_path)
    }

    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.rp_trust_anchors.iter().map(|anchor| anchor.into()).collect()
    }
}
