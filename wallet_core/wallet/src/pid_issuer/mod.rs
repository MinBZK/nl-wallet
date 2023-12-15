mod client;

#[cfg(any(test, feature = "mock"))]
mod mock;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{MdocCopies, TrustAnchor},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};

pub use client::{HttpOpenidPidIssuerClient, HttpPidIssuerClient};

use crate::digid::DigidSession;

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockPidIssuerClient;

#[derive(Debug, thiserror::Error)]
pub enum PidIssuerError {
    #[error("could not get BSN from PID issuer: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not get BSN from PID issuer: {0} - Response body: {1}")]
    Response(#[source] reqwest::Error, String),
    #[error("mdoc error: {0}")]
    MdocError(#[from] nl_wallet_mdoc::Error),
}

#[async_trait]
pub trait OpenidPidIssuerClient {
    fn has_session(&self) -> bool;

    async fn start_retrieve_pid<DGS: DigidSession + Send + Sync>(
        &mut self,
        digid_session: DGS,
        base_url: &Url,
        pre_authorized_code: String,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError>;

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, PidIssuerError>;

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError>;
}

#[async_trait]
pub trait PidIssuerClient {
    fn has_session(&self) -> bool;

    async fn start_retrieve_pid(
        &mut self,
        base_url: &Url,
        access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError>;

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
    ) -> Result<Vec<MdocCopies>, PidIssuerError>;

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError>;
}
