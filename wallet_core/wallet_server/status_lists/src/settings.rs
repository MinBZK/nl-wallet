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
    /// Threshold to start creating a new list in the background
    pub create_threshold: NonZeroU31,
}

#[derive(Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// Base url for the status list
    pub base_url: BaseUrl,

    /// Key pair to sign status list
    #[serde(flatten)]
    pub keypair: KeyPair,
}
