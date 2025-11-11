use url::Url;

use jwt::error::JwtError;

use crate::status_list_token::StatusListToken;

#[derive(Debug, thiserror::Error)]
pub enum StatusListClientError {
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),

    #[error("jwt parsing error: {0}")]
    JwtParsing(#[from] JwtError),
}

#[cfg_attr(test, mockall::automock)]
pub trait StatusListClient {
    async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError>;
}
