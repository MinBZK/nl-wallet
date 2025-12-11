use std::time::Duration;

use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct StatusListTokenCacheSettings {
    pub capacity: u64,

    #[serde(default, rename = "default_ttl_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub default_ttl: Duration,

    #[serde(default, rename = "error_ttl_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub error_ttl: Duration,
}

impl Default for StatusListTokenCacheSettings {
    fn default() -> Self {
        Self {
            capacity: 100,
            default_ttl: Duration::from_secs(0),
            error_ttl: Duration::from_secs(0),
        }
    }
}
