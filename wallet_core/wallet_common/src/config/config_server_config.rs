use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;

use crate::account::serialization::DerVerifyingKey;
use crate::config::http::TlsPinningConfig;
use crate::config::EnvironmentSpecific;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerConfiguration {
    pub environment: String,
    pub http_config: TlsPinningConfig,
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
