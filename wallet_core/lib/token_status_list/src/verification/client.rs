use std::sync::Arc;

use url::Url;

use jwt::error::JwtError;

use crate::status_list_token::StatusListToken;

#[derive(Debug, Clone, thiserror::Error)]
pub enum StatusListClientError {
    #[error("networking error: {0}")]
    Networking(#[from] Arc<reqwest::Error>),

    #[error("jwt parsing error: {0}")]
    JwtParsing(#[from] Arc<JwtError>),
}

#[cfg_attr(test, mockall::automock)]
pub trait StatusListClient {
    async fn fetch(&self, url: Url) -> Result<StatusListToken, StatusListClientError>;
}
