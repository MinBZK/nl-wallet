use std::collections::HashMap;
use std::num::NonZeroU16;
use std::time::Duration;

use derive_more::Debug;
use futures::future::try_join_all;
use itertools::Itertools;
use serde::Deserialize;
use url::Url;

use hsm::service::Pkcs11Hsm;
use http_utils::urls::BaseUrl;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::keys::PrivateKeyVariant;
use server_utils::settings::KeyPair;
use utils::num::NonZeroU31;
use utils::num::Ratio;

use crate::config::StatusListConfig;
use crate::config::StatusListConfigs;
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
        let expiry = Duration::from_secs(self.expiry_in_hours.get() as u64 * 3600);
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
            .map(|ttl| Duration::from_secs(ttl.get() as u64 * 60))
    }
}

fn default_serve() -> bool {
    true
}

#[derive(Debug, thiserror::Error)]
pub enum StatusListAttestationSettingsError {
    #[error("incorrectly configured asttestation status list expiration: {0}")]
    ExpiryLessThanTtl(#[from] ExpiryLessThanTtl),

    #[error("incorrectly configured asttestation status list private key or certificate: {0}")]
    PrivateKey(#[from] PrivateKeySettingsError),
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// Base url for the status list if different from public url of the server
    pub base_url: Option<BaseUrl>,

    /// Context path for the status list joined with base_url, also used for serving
    pub context_path: String,

    /// Path to directory for the published status list
    pub publish_dir: PublishDir,

    /// Key pair to sign status list
    #[serde(flatten)]
    #[debug(skip)]
    pub keypair: KeyPair,
}

impl StatusListAttestationSettings {
    pub async fn settings_into_configs(
        attestation_settings_pairs: impl IntoIterator<Item = (String, StatusListAttestationSettings)>,
        status_list_settings: &StatusListsSettings,
        public_url: &BaseUrl,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<StatusListConfigs<PrivateKeyVariant>, StatusListAttestationSettingsError> {
        let (types, attestation_settings): (Vec<_>, Vec<_>) = attestation_settings_pairs.into_iter().unzip();

        let attestation_count = attestation_settings.len();
        let configs = try_join_all(
            attestation_settings
                .into_iter()
                .zip_eq(itertools::repeat_n(hsm, attestation_count))
                .map(|(attestation, hsm)| attestation.into_config(status_list_settings, public_url, hsm)),
        )
        .await?;

        let map = types.into_iter().zip_eq(configs.into_iter()).collect::<HashMap<_, _>>();

        Ok(map.into())
    }

    async fn into_config(
        self,
        status_list_settings: &StatusListsSettings,
        public_url: &BaseUrl,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<StatusListConfig<PrivateKeyVariant>, StatusListAttestationSettingsError> {
        let base_url = self
            .base_url
            .as_ref()
            .unwrap_or(public_url)
            .join_base_url(&self.context_path);
        let key_pair = self.keypair.parse(hsm).await?;

        let config = status_list_settings.to_config(base_url, self.publish_dir, key_pair)?;

        Ok(config)
    }
}
