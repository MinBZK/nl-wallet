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

use crate::settings::PublishDir;
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
            create_threshold: settings
                .create_threshold
                .unwrap_or_else(|| default_create_threshold(settings.list_size)),
            base_url: attestation.base_url,
            publish_dir: attestation.publish_dir,
            key_pair: attestation.keypair.parse(hsm).await?,
        })
    }
}

fn default_create_threshold(list_size: NonZeroU31) -> NonZeroU31 {
    let list_size = list_size.into_inner() as u32;
    // list_size is a larger NonZeroU31
    NonZeroU31::try_new(list_size.div_ceil(10) as i32).unwrap()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use utils::ints::NonZeroU31;

    use super::default_create_threshold;

    #[rstest]
    #[case(99, 10)]
    #[case(1, 1)]
    fn test_default_create_threshold(#[case] list_size: i32, #[case] expected_create_threshold: i32) {
        assert_eq!(
            default_create_threshold(list_size.try_into().unwrap()),
            NonZeroU31::try_from(expected_create_threshold).unwrap(),
        );
    }
}
