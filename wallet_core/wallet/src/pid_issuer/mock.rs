use async_trait::async_trait;
use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::TrustAnchor,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        mdocs_map::MdocsMap,
    },
};
use url::Url;

use super::{PidIssuerClient, PidIssuerError};

#[derive(Default)]
pub struct MockPidIssuerClient {}

#[async_trait]
impl PidIssuerClient for MockPidIssuerClient {
    async fn start_retrieve_pid(
        &mut self,
        _base_url: &Url,
        _access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError> {
        Ok(Default::default())
    }

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        _mdoc_trust_anchors: &[TrustAnchor<'_>],
        _key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
    ) -> Result<MdocsMap, PidIssuerError> {
        Ok(MdocsMap::new())
    }

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        Ok(())
    }
}
