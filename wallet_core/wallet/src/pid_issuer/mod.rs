mod client;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::holder::TrustAnchor;

pub use client::HttpPidIssuerClient;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait PidIssuerClient {
    async fn retrieve_pid<'a>(
        &self,
        base_url: &Url,
        mdoc_trust_anchors: &[TrustAnchor<'a>],
        access_token: &str,
    ) -> Result<(), PidIssuerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PidIssuerError {
    #[error("could not get BSN from PID issuer: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not get BSN from PID issuer: {0} - Response body: {1}")]
    Response(#[source] reqwest::Error, String),
    #[error("mdoc error: {0}")]
    MdocError(#[from] nl_wallet_mdoc::Error),
}
