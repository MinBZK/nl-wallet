use std::num::NonZeroU16;
use std::time::Duration;

use derive_more::Debug;
use serde::Deserialize;
use url::Url;

use http_utils::urls::BaseUrl;
use utils::num::NonZeroU31;
use utils::num::Ratio;

use crate::config::StatusListConfig;
use crate::publish::PublishDir;

#[derive(Debug, Clone, Deserialize)]
pub struct StatusListsSettings {
    /// Optional storage url if different from rest of application
    pub storage_url: Option<Url>,
    /// List size
    pub list_size: NonZeroU31,
    /// Threshold relatively to `list_size` to start creating a new list in the background
    pub create_threshold_ratio: Ratio,
    /// Expiry duration in hours after creation of the token (`exp` field)
    pub expiry_in_hours: NonZeroU16,
    /// Threshold relatively to `expiry` to refresh the token
    pub refresh_threshold_ratio: Ratio,
    /// TTL in minutes that indicates how long verifiers can cache the status list locally
    pub ttl_in_minutes: Option<NonZeroU16>,
    /// Whether to serve the Status List Token it publishes
    #[serde(default = "default_serve")]
    pub serve: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("configured expiry is less than the TTL: {expiry:?} < {ttl:?}")]
pub struct ExpiryLessThanTtl {
    expiry: Duration,
    ttl: Duration,
}

impl StatusListsSettings {
    pub fn to_config<K>(
        &self,
        base_url: BaseUrl,
        publish_dir: PublishDir,
        key_pair: crypto::server_keys::KeyPair<K>,
    ) -> Result<StatusListConfig<K>, ExpiryLessThanTtl> {
        let (expiry, ttl) = self.expiry_ttl()?;

        let config = StatusListConfig {
            list_size: self.list_size,
            create_threshold: self.create_threshold_ratio.of_nonzero_u31(self.list_size),
            expiry,
            refresh_threshold: self.refresh_threshold_ratio.of_duration(expiry),
            ttl,
            base_url,
            publish_dir,
            key_pair,
        };

        Ok(config)
    }

    fn expiry_ttl(&self) -> Result<(Duration, Option<Duration>), ExpiryLessThanTtl> {
        let expiry = Duration::from_secs(u64::from(self.expiry_in_hours.get()) * 3600);
        let ttl = self
            .ttl()
            .map(|ttl| {
                if expiry < ttl {
                    return Err(ExpiryLessThanTtl { expiry, ttl });
                }
                Ok(ttl)
            })
            .transpose()?;
        Ok((expiry, ttl))
    }

    pub fn ttl(&self) -> Option<Duration> {
        self.ttl_in_minutes
            .map(|ttl| Duration::from_secs(u64::from(ttl.get()) * 60))
    }
}

fn default_serve() -> bool {
    true
}
