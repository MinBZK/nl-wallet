use base64::prelude::*;
use p256::{ecdsa::VerifyingKey, pkcs8::DecodePublicKey};
use url::Url;

use wallet_common::{
    config::wallet_config::{
        AccountServerConfiguration, DisclosureConfiguration, LockTimeoutConfiguration, PidIssuanceConfiguration,
        WalletConfiguration,
    },
    trust_anchor::DerTrustAnchor,
};

// Each of these values can be overridden from environment variables at compile time
// when the `env_config` feature is enabled. Additionally, environment variables can
// be added to using a file named `.env` in root directory of this crate.
const CONFIG_SERVER_BASE_URL: &str = "http://localhost:3000/config/v1/";

const WALLET_PROVIDER_BASE_URL: &str = "http://localhost:3000/api/v1/";

const CERTIFICATE_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEW2zhAd/0VH7PzLdmAfDEmHpSWwbVRfr5H31fo2rQWtyU\
                                      oWZT/C5WSeVm5Ktp6nCwnOwhhJLLGb4K3LtUJeLKjA==";

const DIGID_CLIENT_ID: &str = "";
const DIGID_URL: &str = "https://localhost/8006/";

const INSTRUCTION_RESULT_PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEpQqynmHM6Iey1gqLPtTi4T9PflzCDpttyk\
                                             oP/iW47jE1Ra6txPJEPq4FVQdqQJEXcJ7i8TErVQ3KNB823StXnA==";

const PID_ISSUER_URL: &str = "http://localhost:3003/";

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
pub struct ConfigServerConfiguration {
    pub base_url: Url,
}

impl Default for ConfigServerConfiguration {
    fn default() -> Self {
        Self {
            base_url: Url::parse(config_default!(CONFIG_SERVER_BASE_URL)).unwrap(),
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
        lock_timeouts: LockTimeoutConfiguration::default(),
        account_server: AccountServerConfiguration {
            base_url: Url::parse(config_default!(WALLET_PROVIDER_BASE_URL)).unwrap(),
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
