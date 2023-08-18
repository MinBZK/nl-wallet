mod client;

use async_trait::async_trait;
use url::Url;

pub use client::RemotePidIssuerClient;

#[async_trait]
pub trait PidIssuerClient {
    async fn extract_bsn(&self, base_url: &Url, access_token: &str) -> Result<String, PidIssuerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PidIssuerError {
    #[error("could not get BSN from PID issuer: {0}")]
    PidIssuer(#[from] reqwest::Error),
    #[error("could not get BSN from PID issuer: {0} - Response body: {1}")]
    PidIssuerResponse(#[source] reqwest::Error, String),
}
