use nl_wallet_mdoc::{
    holder::{MdocCopies, TrustAnchor},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};
use openid4vc::{
    issuance_client::IssuanceClient,
    token::{AttestationPreview, TokenRequest},
};
use url::Url;

#[derive(Default)]
pub struct MockPidIssuerClient {
    pub has_session: bool,
    pub attestation_previews: Vec<AttestationPreview>,
    pub mdoc_copies: Vec<MdocCopies>,
    pub next_error: Option<openid4vc::Error>,
}

impl IssuanceClient for MockPidIssuerClient {
    fn has_session(&self) -> bool {
        self.has_session
    }

    async fn start_issuance(
        &mut self,
        _base_url: &Url,
        _token_request: TokenRequest,
    ) -> Result<Vec<AttestationPreview>, openid4vc::Error> {
        match self.next_error.take() {
            None => Ok(self.attestation_previews.clone()),
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
