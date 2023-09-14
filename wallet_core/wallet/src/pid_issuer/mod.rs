use async_trait::async_trait;
use url::Url;

pub use client::PidIssuerClient;
use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, holder::TrustAnchor, utils::keys::KeyFactory};

mod client;

#[async_trait]
pub trait PidRetriever {
    async fn start_retrieve_pid(
        &mut self,
        base_url: &Url,
        access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidRetrieverError>;

    async fn accept_pid<'a>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a> + Sync),
    ) -> Result<(), PidRetrieverError>;

    async fn reject_pid_issuance(&mut self) -> Result<(), PidRetrieverError>;
}

#[cfg(any(test, feature = "mock"))]
pub struct MockPidRetriever {}

#[cfg(any(test, feature = "mock"))]
#[async_trait]
impl PidRetriever for MockPidRetriever {
    async fn start_retrieve_pid(
        &mut self,
        _base_url: &Url,
        _access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidRetrieverError> {
        Ok(Default::default())
    }

    async fn accept_pid<'a>(
        &mut self,
        _mdoc_trust_anchors: &[TrustAnchor<'_>],
        _key_factory: &'a (impl KeyFactory<'a> + Sync),
    ) -> Result<(), PidRetrieverError> {
        Ok(())
    }

    async fn reject_pid_issuance(&mut self) -> Result<(), PidRetrieverError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PidRetrieverError {
    #[error("could not get BSN from PID issuer: {0}")]
    PidIssuer(#[from] reqwest::Error),
    #[error("could not get BSN from PID issuer: {0} - Response body: {1}")]
    PidIssuerResponse(#[source] reqwest::Error, String),
    #[error("mdoc error: {0}")]
    MdocError(#[from] nl_wallet_mdoc::Error),
}
