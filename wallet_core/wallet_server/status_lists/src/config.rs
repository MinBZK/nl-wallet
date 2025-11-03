use std::collections::HashMap;

use derive_more::AsRef;
use derive_more::From;
use derive_more::IntoIterator;
use futures::future::try_join_all;

use crypto::server_keys::KeyPair;
use hsm::service::Pkcs11Hsm;
use http_utils::urls::BaseUrl;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::keys::PrivateKeyVariant;
use utils::num::NonZeroU31;

use crate::publish::PublishDir;
use crate::settings::StatusListAttestationSettings;
use crate::settings::StatusListsSettings;

#[derive(Debug, Clone)]
pub struct StatusListConfig {
    pub list_size: NonZeroU31,
    pub create_threshold: NonZeroU31,

    pub base_url: BaseUrl,
    pub publish_dir: PublishDir,
    pub key_pair: KeyPair<PrivateKeyVariant>,
}

#[derive(Debug, Clone, From, IntoIterator, AsRef)]
pub struct StatusListConfigs(HashMap<String, StatusListConfig>);

impl StatusListConfigs {
    pub async fn from_settings(
        settings: &StatusListsSettings,
        pairs: impl IntoIterator<Item = (String, StatusListAttestationSettings)>,
        hsm: &Option<Pkcs11Hsm>,
    ) -> Result<Self, PrivateKeySettingsError> {
        let (types, attestation_settings): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        let configs = try_join_all(
            attestation_settings
                .into_iter()
                .map(|attestation| StatusListConfig::from_settings(settings, attestation, hsm.clone())),
        )
        .await?;

        let map = Self(types.into_iter().zip(configs.into_iter()).collect::<HashMap<_, _>>());
        Ok(map)
    }

    pub fn types(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect::<Vec<_>>()
    }
}

impl StatusListConfig {
    pub async fn from_settings(
        settings: &StatusListsSettings,
        attestation: StatusListAttestationSettings,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<Self, PrivateKeySettingsError> {
        Ok(StatusListConfig {
            list_size: settings.list_size,
            create_threshold: settings.create_threshold.of_nonzero_u31(settings.list_size),
            base_url: attestation.base_url,
            publish_dir: attestation.publish_dir,
            key_pair: attestation.keypair.parse(hsm).await?,
        })
    }
}
