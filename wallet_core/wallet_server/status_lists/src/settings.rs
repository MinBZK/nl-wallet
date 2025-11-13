use std::collections::HashMap;
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
    pub create_threshold: Ratio,
    /// TTL that indicates how long verifiers can cache the status list locally
    pub ttl: Option<Duration>,
    /// Publish directories to serve by context path
    #[serde(default)]
    pub serve_directories: HashMap<String, PublishDir>,
}

#[derive(Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// Base url for the status list
    pub base_url: BaseUrl,

    /// Path to directory for the published status list
    pub publish_dir: PublishDir,

    /// Key pair to sign status list
    #[serde(flatten)]
    pub keypair: KeyPair,
}
