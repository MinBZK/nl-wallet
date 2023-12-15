use async_trait::async_trait;
use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{MdocCopies, TrustAnchor},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};
use url::Url;

use crate::digid::DigidSession;

use super::{OpenidPidIssuerClient, PidIssuerClient, PidIssuerError};

#[derive(Default)]
pub struct MockPidIssuerClient {
    pub has_session: bool,
    pub unsigned_mdocs: Vec<UnsignedMdoc>,
    pub mdoc_copies: Vec<MdocCopies>,
    pub next_error: Option<PidIssuerError>,
}

#[async_trait]
impl OpenidPidIssuerClient for MockPidIssuerClient {
    fn has_session(&self) -> bool {
        self.has_session
    }

    async fn start_retrieve_pid<DGS: DigidSession + Send + Sync>(
        &mut self,
        _digid_session: DGS,
        _base_url: &Url,
        _pre_authorized_code: String,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError> {
        match self.next_error.take() {
            None => Ok(self.unsigned_mdocs.clone()),
            Some(error) => Err(error),
        }
    }

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        _mdoc_trust_anchors: &[TrustAnchor<'_>],
        _key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
        _credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, PidIssuerError> {
        match self.next_error.take() {
            None => Ok(self.mdoc_copies.clone()),
            Some(error) => Err(error),
        }
    }

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        match self.next_error.take() {
            None => Ok(()),
            Some(error) => Err(error),
        }
    }
}

#[async_trait]
impl PidIssuerClient for MockPidIssuerClient {
    fn has_session(&self) -> bool {
        self.has_session
    }

    async fn start_retrieve_pid(
        &mut self,
        _base_url: &Url,
        _access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError> {
        match self.next_error.take() {
            None => Ok(self.unsigned_mdocs.clone()),
            Some(error) => Err(error),
        }
    }

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        _mdoc_trust_anchors: &[TrustAnchor<'_>],
        _key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
    ) -> Result<Vec<MdocCopies>, PidIssuerError> {
        match self.next_error.take() {
            None => Ok(self.mdoc_copies.clone()),
            Some(error) => Err(error),
        }
    }

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        match self.next_error.take() {
            None => Ok(()),
            Some(error) => Err(error),
        }
    }
}
