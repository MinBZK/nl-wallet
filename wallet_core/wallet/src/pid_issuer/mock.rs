use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{MdocCopies, TrustAnchor},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};
use openid4vc::{
    issuance_client::IssuanceClientTrait,
    token::{AttestationPreview, TokenRequest},
};
use url::Url;

use super::{PidIssuerClient, PidIssuerError};

#[derive(Default)]
pub struct MockPidIssuerClient {
    pub has_session: bool,
    pub unsigned_mdocs: Vec<AttestationPreview>,
    pub mdoc_copies: Vec<MdocCopies>,
    pub next_error: Option<openid4vc::Error>,
}

impl IssuanceClientTrait for MockPidIssuerClient {
    fn has_issuance_session(&self) -> bool {
        self.has_session
    }

    async fn start_issuance(
        &mut self,
        _base_url: &Url,
        _token_request: TokenRequest,
    ) -> Result<Vec<AttestationPreview>, openid4vc::Error> {
        match self.next_error.take() {
            None => Ok(self.unsigned_mdocs.clone()),
            Some(error) => Err(error),
        }
    }

    async fn finish_issuance<K: MdocEcdsaKey>(
        &mut self,
        _mdoc_trust_anchors: &[TrustAnchor<'_>],
        _key_factory: impl KeyFactory<Key = K>,
        _credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, openid4vc::Error> {
        match self.next_error.take() {
            None => Ok(self.mdoc_copies.clone()),
            Some(error) => Err(error),
        }
    }

    async fn reject_issuance(&mut self) -> Result<(), openid4vc::Error> {
        match self.next_error.take() {
            None => Ok(()),
            Some(error) => Err(error),
        }
    }
}

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
            None => Ok(self
                .unsigned_mdocs
                .iter()
                .map(|preview| preview.into())
                .cloned()
                .collect()),
            Some(error) => Err(PidIssuerError::Openid(error)),
        }
    }

    async fn accept_pid<K: MdocEcdsaKey>(
        &mut self,
        _mdoc_trust_anchors: &[TrustAnchor<'_>],
        _key_factory: &impl KeyFactory<Key = K>,
    ) -> Result<Vec<MdocCopies>, PidIssuerError> {
        match self.next_error.take() {
            None => Ok(self.mdoc_copies.clone()),
            Some(error) => Err(PidIssuerError::Openid(error)),
        }
    }

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        match self.next_error.take() {
            None => Ok(()),
            Some(error) => Err(PidIssuerError::Openid(error)),
        }
    }
}
