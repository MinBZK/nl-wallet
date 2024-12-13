use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;

use base64::prelude::*;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;

use wallet_common::config::http::TlsPinningConfig;
use wallet_common::config::wallet_config::AccountServerConfiguration;
use wallet_common::config::wallet_config::DigidConfiguration;
use wallet_common::config::wallet_config::DisclosureConfiguration;
use wallet_common::config::wallet_config::LockTimeoutConfiguration;
use wallet_common::config::wallet_config::PidIssuanceConfiguration;
use wallet_common::config::wallet_config::UpdatePolicyServerConfiguration;
use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::trust_anchor::DerTrustAnchor;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;

// Each of these values can be overridden from environment variables at compile time
// when the `env_config` feature is enabled. Additionally, environment variables can
// be added to using a file named `.env` in root directory of this crate.
const WALLET_CONFIG_VERSION: &str = "1";

const CONFIG_SERVER_BASE_URL: &str = "https://localhost:3000/config/v1/";

const CONFIG_SERVER_TRUST_ANCHORS: &str = "\
                                   MIIBlTCCATqgAwIBAgIURlVkuYVVlqtiuecbOwVySS9jdFwwCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwO\
                                   Y2EuZXhhbXBsZS5jb20wHhcNMjMxMTI3MDc1NDMyWhcNMjQxMTI2MDc1NDMyWjAZMRcwFQYDVQQDDA5j\
                                   YS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABFPE9hj71n7dNJpV1lCBBExbCK1B\
                                   8KYu8q22Z5sPWzzzuUfRAM+K7NgsfQVprob1rR6U+pvemR1e992K8rua5gGjYDBeMB0GA1UdDgQWBBQv\
                                   7ArBe8g9qs+S0QVagvo1xhFd7TAfBgNVHSMEGDAWgBQv7ArBe8g9qs+S0QVagvo1xhFd7TAPBgNVHRMB\
                                   Af8EBTADAQH/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNJADBGAiEAuITZR9Rbj5zfzN39+PEymrnk\
                                   K8WVHjOID8jeajR4DC0CIQD9XnpbZLDYMCWqkVVeBMphwv8R3P1t3NSpXRQyLRIO2w==";

const CONFIG_SERVER_SIGNING_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEW2zhAd/\
                                                0VH7PzLdmAfDEmHpSWwbVRfr5H31fo2rQWtyUoWZT/\
                                                C5WSeVm5Ktp6nCwnOwhhJLLGb4K3LtUJeLKjA==";

const CONFIG_SERVER_UPDATE_FREQUENCY_IN_SEC: &str = "3600";
const WALLET_PROVIDER_BASE_URL: &str = "https://localhost:3000/api/v1/";

const WALLET_PROVIDER_TRUST_ANCHORS: &str = "\
                                  MIIBlTCCATqgAwIBAgIURlVkuYVVlqtiuecbOwVySS9jdFwwCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwO\
                                  Y2EuZXhhbXBsZS5jb20wHhcNMjMxMTI3MDc1NDMyWhcNMjQxMTI2MDc1NDMyWjAZMRcwFQYDVQQDDA5j\
                                  YS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABFPE9hj71n7dNJpV1lCBBExbCK1B\
                                  8KYu8q22Z5sPWzzzuUfRAM+K7NgsfQVprob1rR6U+pvemR1e992K8rua5gGjYDBeMB0GA1UdDgQWBBQv\
                                  7ArBe8g9qs+S0QVagvo1xhFd7TAfBgNVHSMEGDAWgBQv7ArBe8g9qs+S0QVagvo1xhFd7TAPBgNVHRMB\
                                  Af8EBTADAQH/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNJADBGAiEAuITZR9Rbj5zfzN39+PEymrnk\
                                  K8WVHjOID8jeajR4DC0CIQD9XnpbZLDYMCWqkVVeBMphwv8R3P1t3NSpXRQyLRIO2w==";

const CERTIFICATE_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEW2zhAd/\
                                      0VH7PzLdmAfDEmHpSWwbVRfr5H31fo2rQWtyUoWZT/\
                                      C5WSeVm5Ktp6nCwnOwhhJLLGb4K3LtUJeLKjA==";

const DIGID_CLIENT_ID: &str = "";
const DIGID_URL: &str = "https://localhost:8006/";
const DIGID_TRUST_ANCHORS: &str = "MIIBlTCCATqgAwIBAgIURlVkuYVVlqtiuecbOwVySS9jdFwwCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwO\
                                   Y2EuZXhhbXBsZS5jb20wHhcNMjMxMTI3MDc1NDMyWhcNMjQxMTI2MDc1NDMyWjAZMRcwFQYDVQQDDA5j\
                                   YS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABFPE9hj71n7dNJpV1lCBBExbCK1B\
                                   8KYu8q22Z5sPWzzzuUfRAM+K7NgsfQVprob1rR6U+pvemR1e992K8rua5gGjYDBeMB0GA1UdDgQWBBQv\
                                   7ArBe8g9qs+S0QVagvo1xhFd7TAfBgNVHSMEGDAWgBQv7ArBe8g9qs+S0QVagvo1xhFd7TAPBgNVHRMB\
                                   Af8EBTADAQH/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNJADBGAiEAuITZR9Rbj5zfzN39+PEymrnk\
                                   K8WVHjOID8jeajR4DC0CIQD9XnpbZLDYMCWqkVVeBMphwv8R3P1t3NSpXRQyLRIO2w==";

const INSTRUCTION_RESULT_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEpQqynmHM6Iey1gqLPtTi4T9PflzCDpttykoP/\
                                             iW47jE1Ra6txPJEPq4FVQdqQJEXcJ7i8TErVQ3KNB823StXnA==";

const WTE_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEpQqynmHM6Iey1gqLPtTi4T9PflzCDpttykoP/\
                              iW47jE1Ra6txPJEPq4FVQdqQJEXcJ7i8TErVQ3KNB823StXnA==";

const PID_ISSUER_URL: &str = "http://localhost:3001/issuance/";

const MDOC_TRUST_ANCHORS: &str = "MIIBlTCCATqgAwIBAgIURlVkuYVVlqtiuecbOwVySS9jdFwwCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwO\
                                  Y2EuZXhhbXBsZS5jb20wHhcNMjMxMTI3MDc1NDMyWhcNMjQxMTI2MDc1NDMyWjAZMRcwFQYDVQQDDA5j\
                                  YS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABFPE9hj71n7dNJpV1lCBBExbCK1B\
                                  8KYu8q22Z5sPWzzzuUfRAM+K7NgsfQVprob1rR6U+pvemR1e992K8rua5gGjYDBeMB0GA1UdDgQWBBQv\
                                  7ArBe8g9qs+S0QVagvo1xhFd7TAfBgNVHSMEGDAWgBQv7ArBe8g9qs+S0QVagvo1xhFd7TAPBgNVHRMB\
                                  Af8EBTADAQH/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNJADBGAiEAuITZR9Rbj5zfzN39+PEymrnk\
                                  K8WVHjOID8jeajR4DC0CIQD9XnpbZLDYMCWqkVVeBMphwv8R3P1t3NSpXRQyLRIO2w==";

const RP_TRUST_ANCHORS: &str = "MIIBlDCCATqgAwIBAgIUMmfPjx+jkrbY6twjDTCNHtnoPB4wCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwO\
                                Y2EuZXhhbXBsZS5jb20wHhcNMjMxMTA3MTA1NDEzWhcNMjQxMTA2MTA1NDEzWjAZMRcwFQYDVQQDDA5j\
                                YS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABPD39ZBr4/cNp76DturjGWRtjjqU\
                                qQgt+Xny57i2xFYHRSogzdQYQbqdUOzEfBZeW3GkBjzmbCmug2zHr5B54UKjYDBeMB0GA1UdDgQWBBR4\
                                cYdOOiKhp1xTmK4ZW8JMG4CggzAfBgNVHSMEGDAWgBR4cYdOOiKhp1xTmK4ZW8JMG4CggzAPBgNVHRMB\
                                Af8EBTADAQH/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNIADBFAiBpJ/sEsPeTm8A2XYwRmu6NOkoL\
                                NqhPN569XKLTR6rVdwIhANOMtj2LwDUG2YcLkSBPSdhh/i/iCgTeuZQpOI8y+kBw";

const UNIVERSAL_LINK_BASE: &str = DEFAULT_UNIVERSAL_LINK_BASE;

const UPDATE_POLICY_SERVER_BASE_URL: &str = "https://localhost:3009/update/v1/";

const UPDATE_POLICY_SERVER_TRUST_ANCHORS: &str = "MIIDTTCCAjWgAwIBAgIUeMgUF0TsrJXCGw5cqmPUw5llilswDQYJKoZIhvcNAQELBQAw\
                                                  NjELMAkGA1UEBhMCTkwxJzAlBgNVBAMMHm5sLXdhbGxldC11cGRhdGUtcG9saWN5LXNl\
                                                  cnZlcjAeFw0yNDEwMDExODIzMDdaFw0yOTA5MzAxODIzMDdaMDYxCzAJBgNVBAYTAk5M\
                                                  MScwJQYDVQQDDB5ubC13YWxsZXQtdXBkYXRlLXBvbGljeS1zZXJ2ZXIwggEiMA0GCSqG\
                                                  SIb3DQEBAQUAA4IBDwAwggEKAoIBAQC+3t5V4fOtQyP9X5/r/q9SmMbZMZzvSuOG14Ri\
                                                  7bF0KFYSsvOF208tQs1NNgZul4UmZaFZVp9tehVAL3fHtrfCjV91fM04EIELgT4FHUb8\
                                                  DDF6GClfP+4TyHn9y6ETtbgbUDgp6oJZo9nXW7Zfw+L6Ws3BIgSSDCqsOhlRX3LNNhb+\
                                                  k/PQf1HR7ChdbdotNCWu+EMpwjtsYHE7lB7Sa5KWL3yVlqIyBbGccnroNQ/+1WBo0/YK\
                                                  0vlVT3CFF/VMas8vUxdwE7qCs6AyhYa28/8sI5uPahFaTciTnrBs4srdXFtpzRrYM9qw\
                                                  IvIrcZ9e0IwTn9TMbK5KbQAiloAfomVJAgMBAAGjUzBRMB0GA1UdDgQWBBRB17Lzkw2v\
                                                  YffZcTypFdIjv/4C0DAfBgNVHSMEGDAWgBRB17Lzkw2vYffZcTypFdIjv/4C0DAPBgNV\
                                                  HRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQCwxsu6okRIlTyNXXoelpUkLxaI\
                                                  NvInCrAUwqxecQn3GVxc2gj416DCTiCDq+wZZKNgLmIsc4UBHGBWfpGzhL5HqE7XR/z+\
                                                  OPRL7ySTJRgigqU5sWkew32zzoJFSA6h4jQcFItE0gwxyma/1SvFMjmb+2+kPR7vT51L\
                                                  EhurwKFVn9oQ9+Xgs2hynu3UIjLB0MgB8ob/HwB11pVcEryFgQYvFdETHOyE0juseYKK\
                                                  ZQ05rKQSCsX9UGAviSeLRwLaJKPqdMC/wwi5Nva8coVVzVkXvtfdnelWhZl7R0YWY5J1\
                                                  iV7i3tbb5T7+bpB/Z9GB3aiq1+Qz/RSunrx3Bto00M4I";

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

#[derive(Debug, Clone)]
pub struct ConfigServerConfiguration {
    pub http_config: TlsPinningConfig,
    pub signing_public_key: VerifyingKey,
    pub update_frequency: Duration,
}

impl Default for ConfigServerConfiguration {
    fn default() -> Self {
        Self {
            http_config: TlsPinningConfig {
                base_url: config_default!(CONFIG_SERVER_BASE_URL).parse().unwrap(),
                trust_anchors: parse_trust_anchors(config_default!(CONFIG_SERVER_TRUST_ANCHORS)),
            },
            signing_public_key: VerifyingKey::from_public_key_der(
                &BASE64_STANDARD
                    .decode(config_default!(CONFIG_SERVER_SIGNING_PUBLIC_KEY))
                    .unwrap(),
            )
            .unwrap(),
            update_frequency: Duration::from_secs(
                config_default!(CONFIG_SERVER_UPDATE_FREQUENCY_IN_SEC).parse().unwrap(),
            ),
        }
    }
}

fn parse_trust_anchors(source: &str) -> Vec<DerTrustAnchor> {
    source
        .split('|')
        .map(|anchor| serde_json::from_str(format!("\"{}\"", anchor).as_str()).expect("failed to parse trust anchor"))
        .collect()
}

pub fn default_configuration() -> WalletConfiguration {
    WalletConfiguration {
        version: u64::from_str(config_default!(WALLET_CONFIG_VERSION)).unwrap(),
        lock_timeouts: LockTimeoutConfiguration::default(),
        account_server: AccountServerConfiguration {
            http_config: TlsPinningConfig {
                base_url: config_default!(WALLET_PROVIDER_BASE_URL).parse().unwrap(),
                trust_anchors: parse_trust_anchors(config_default!(WALLET_PROVIDER_TRUST_ANCHORS)),
            },
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
            wte_public_key: VerifyingKey::from_public_key_der(
                &BASE64_STANDARD.decode(config_default!(WTE_PUBLIC_KEY)).unwrap(),
            )
            .unwrap()
            .into(),
        },
        pid_issuance: PidIssuanceConfiguration {
            pid_issuer_url: config_default!(PID_ISSUER_URL).parse().unwrap(),
            digid: DigidConfiguration {
                client_id: String::from(config_default!(DIGID_CLIENT_ID)),
                app2app: None,
            },
            digid_http_config: TlsPinningConfig {
                base_url: config_default!(DIGID_URL).parse().unwrap(),
                trust_anchors: parse_trust_anchors(config_default!(DIGID_TRUST_ANCHORS)),
            },
        },
        disclosure: DisclosureConfiguration {
            rp_trust_anchors: parse_trust_anchors(config_default!(RP_TRUST_ANCHORS)),
        },
        mdoc_trust_anchors: parse_trust_anchors(config_default!(MDOC_TRUST_ANCHORS)),
        update_policy_server: UpdatePolicyServerConfiguration {
            http_config: TlsPinningConfig {
                base_url: config_default!(UPDATE_POLICY_SERVER_BASE_URL).parse().unwrap(),
                trust_anchors: parse_trust_anchors(config_default!(UPDATE_POLICY_SERVER_TRUST_ANCHORS)),
            },
        },
    }
}
