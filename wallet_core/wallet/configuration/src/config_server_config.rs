use std::time::Duration;

use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crypto::p256_der::DerVerifyingKey;
use http_utils::tls::pinning::TlsPinningConfig;

use crate::EnvironmentSpecific;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerConfiguration {
    pub environment: String,
    pub http_config: TlsPinningConfig,
    #[serde_as(as = "Base64")]
    pub signing_public_key: DerVerifyingKey,

    #[serde(rename = "update_frequency_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub update_frequency: Duration,
}

impl EnvironmentSpecific for ConfigServerConfiguration {
    fn environment(&self) -> &str {
        &self.environment
    }
}
