use std::num::NonZeroU16;
use std::time::Duration;

use serde::Deserialize;
use url::Url;

use http_utils::urls::BaseUrl;
use server_utils::settings::KeyPair;
use utils::num::NonZeroU31;
use utils::num::Ratio;

use crate::publish::PublishDir;

#[derive(Clone, Deserialize)]
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
    pub fn expiry_ttl(&self) -> Result<(Duration, Option<Duration>), ExpiryLessThanTtl> {
        let expiry = Duration::from_secs(self.expiry_in_hours.get() as u64 * 3600);
        let ttl = self
            .ttl_in_minutes
            .map(|ttl| {
                let ttl = Duration::from_secs(ttl.get() as u64 * 60);
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
            .map(|ttl| Duration::from_secs(ttl.get() as u64 * 60))
    }
}

fn default_serve() -> bool {
    true
}

#[derive(Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// Base url for the status list if different from public url of the server
    pub base_url: Option<BaseUrl>,
    /// Context path for the status list joined with base_url, also used for serving
    pub context_path: String,
    /// Path to directory for the published status list
    pub publish_dir: PublishDir,
    /// Key pair to sign status list
    #[serde(flatten)]
    pub keypair: KeyPair,
}
