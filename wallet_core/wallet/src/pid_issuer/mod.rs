mod client;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::holder::TrustAnchor;

pub use client::PidIssuerClient;

#[async_trait]
pub trait PidRetriever {
    async fn retrieve_pid(
        &self,
        base_url: &Url,
        mdoc_trust_anchors: &[TrustAnchor],
        access_token: &str,
    ) -> Result<(), PidRetrieverError>;
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
