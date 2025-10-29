use std::collections::HashMap;

use derive_more::AsRef;
use derive_more::From;
use derive_more::Into;

use http_utils::urls::BaseUrl;
use utils::ints::NonZeroU31;

use crate::settings::StatusListAttestationSettings;
use crate::settings::StatusListsSettings;

#[derive(Debug, Clone)]
pub struct StatusListConfig {
    pub list_size: NonZeroU31,
    pub create_threshold: NonZeroU31,

    pub base_url: BaseUrl,
}

#[derive(Debug, Clone, From, Into, AsRef)]
pub struct StatusListConfigs(HashMap<String, StatusListConfig>);

impl StatusListConfigs {
    pub fn types(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect::<Vec<_>>()
    }
}

impl StatusListConfig {
    pub fn from_settings(settings: &StatusListsSettings, attestation: &StatusListAttestationSettings) -> Self {
        StatusListConfig {
            list_size: settings.list_size,
            create_threshold: settings.create_threshold,
            base_url: attestation.base_url.clone(),
        }
    }
}
