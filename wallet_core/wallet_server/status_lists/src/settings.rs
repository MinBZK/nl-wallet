use serde::Deserialize;
use url::Url;
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
