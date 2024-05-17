mod app2app;

use url::Url;

use openid4vc::{oidc::OidcError, token::TokenRequest};
use wallet_common::config::wallet_config::PidIssuanceConfiguration;

pub use app2app::HttpDigidSession;

#[derive(Debug, thiserror::Error)]
pub enum DigidSessionError {
    #[error("OIDC error: {0}")]
    Oidc(#[from] OidcError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("missing location header")]
    MissingLocation,
    #[error("cannot parse location header to str: {0}")]
    HeaderNotAStr(#[from] http::header::ToStrError),
    #[error("cannot parse location header to URL: {0}")]
    NotAUrl(#[from] url::ParseError),
    #[error("missing query in location header")]
    MissingLocationQuery,
    #[error("expected HTTP 307 Temporary Redirect, got: {0}")]
    ExpectedRedirect(http::StatusCode),
    #[error("cannot deserialize from URL query: {0}")]
    UrlDeserialize(#[from] serde_urlencoded::de::Error),
    #[error("cannot serialize to URL query: {0}")]
    UrlSerialize(#[from] serde_urlencoded::ser::Error),
    #[error("error in app2app response: {0}")]
    App2AppError(String),
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DigidSession {
    async fn start(config: PidIssuanceConfiguration, redirect_uri: Url) -> Result<(Self, Url), DigidSessionError>
    where
        Self: Sized;

    async fn into_token_request(self, received_redirect_uri: Url) -> Result<TokenRequest, DigidSessionError>;
}
