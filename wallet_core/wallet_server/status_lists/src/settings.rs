use std::path::PathBuf;

use serde::Deserialize;
use url::Url;

use http_utils::urls::BaseUrl;
use server_utils::settings::KeyPair;
use utils::ints::NonZeroU31;

#[derive(Clone, Deserialize)]
pub struct StatusListsSettings {
    /// Optional storage url if different from rest of application
    pub storage_url: Option<Url>,
    /// List size
    pub list_size: NonZeroU31,
    /// Threshold to start creating a new list in the background, defaults to 10% of list_size
    pub create_threshold: Option<NonZeroU31>,
}

#[derive(Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// Base url for the status list
    pub base_url: BaseUrl,

    /// Path to directory for the published status list
    pub publish_dir: PathBuf,

    /// Key pair to sign status list
    #[serde(flatten)]
    pub keypair: KeyPair,
}
