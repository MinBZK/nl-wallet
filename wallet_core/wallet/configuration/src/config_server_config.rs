use std::time::Duration;

use crypto::p256_der::DerVerifyingKey;
use http_utils::client::TlsPinningConfig;
use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::base64::Base64;
use serde_with::serde_as;

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

    /// How long to wait between attempts to replace an expired wallet configuration.
    #[serde(rename = "expired_retry_interval_in_sec", default = "default_expired_retry_interval")]
    #[serde_as(as = "DurationSeconds")]
    pub expired_retry_interval: Duration,
}

fn default_expired_retry_interval() -> Duration {
    Duration::from_secs(10)
}

impl EnvironmentSpecific for ConfigServerConfiguration {
    fn environment(&self) -> &str {
        &self.environment
    }
}
