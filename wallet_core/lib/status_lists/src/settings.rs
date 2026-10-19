use std::time::Duration;

use derive_more::Debug;
use http_utils::urls::BaseUrl;
use serde::Deserialize;
use serde_with::serde_as;
use url::Url;
use utils::duration::DurationHours;
use utils::duration::DurationMinutes;
use utils::num::NonZeroU31;
use utils::num::Ratio;

use crate::config::StatusListConfig;
use crate::publish::PublishDir;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct StatusListsSettings {
    /// Optional storage url if different from rest of application
    pub storage_url: Option<Url>,
    /// List size
    pub list_size: NonZeroU31,
    /// Threshold relatively to `list_size` to start creating a new list in the background
    pub create_threshold_ratio: Ratio,
    /// Expiry duration after creation of the token (`exp` field). Note that this gets deserialized as
    /// `expiry_in_hours`, which is guaranteed to result in a non-zero duration.
    #[serde(rename = "expiry_in_hours")]
    #[serde_as(as = "DurationHours")]
    pub expiry: Duration,
    /// Threshold relatively to `expiry` to refresh the token
    pub refresh_threshold_ratio: Ratio,
    /// TTL that indicates how long verifiers can cache the status list locally. Note that this gets deserialized as
    /// `ttl_in_minutes`, which is guaranteed to result in a non-zero duration.
    #[serde(rename = "ttl_in_minutes")]
    #[serde_as(as = "Option<DurationMinutes>")]
    pub ttl: Option<Duration>,
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
        context_path: String,
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
            context_path,
            publish_dir,
            key_pair,
        };

        Ok(config)
    }

    fn expiry_ttl(&self) -> Result<(Duration, Option<Duration>), ExpiryLessThanTtl> {
        let ttl = self
            .ttl
            .map(|ttl| {
                if self.expiry < ttl {
                    return Err(ExpiryLessThanTtl {
                        expiry: self.expiry,
                        ttl,
                    });
                }
                Ok(ttl)
            })
            .transpose()?;

        Ok((self.expiry, ttl))
    }
}

fn default_serve() -> bool {
    true
}
