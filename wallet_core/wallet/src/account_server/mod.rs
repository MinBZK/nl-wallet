mod remote;

#[cfg(test)]
mod mock;

use async_trait::async_trait;
use reqwest::StatusCode;
use url::{ParseError, Url};

use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};

pub use self::remote::RemoteAccountServerClient;

#[derive(Debug, thiserror::Error)]
pub enum AccountServerClientError {
    #[error("server responded with {0}")]
    Response(#[from] AccountServerResponseError),
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
    /// This error variant only exist for the mock implementation of [`AccountServerClient`]
    /// by [`wallet_provider::account_server::AccountServer`].
    #[cfg(test)]
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountServerResponseError {
    #[error("status code {0}")]
    Status(StatusCode),
    #[error("status code {0} and contents: {1}")]
    Text(StatusCode, String),
}

#[async_trait]
pub trait AccountServerClient {
    fn new(base_url: &Url) -> Self
    where
        Self: Sized;

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError>;
    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError>;
}
