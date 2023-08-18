mod client;
mod openid;
mod openid_client;
mod pkce;

use async_trait::async_trait;
use url::Url;

use self::openid_client::OpenIdClientError;

pub use self::client::RemoteDigidClient;

#[async_trait]
pub trait DigidClient {
    fn is_redirect_uri(url: &Url) -> bool;

    async fn start_session(&mut self) -> Result<Url, DigidClientError>;
    async fn get_access_token(&mut self, redirect_url: &Url) -> Result<String, DigidClientError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DigidClientError {
    #[error(transparent)]
    OpenId(#[from] OpenIdClientError),
    #[error("invalid redirect URI received")]
    RedirectUriMismatch,
    #[error("unsuccessful DigiD stepout: {}", .error_description.as_ref().unwrap_or(.error))]
    RedirectUriError {
        error: String,
        error_description: Option<String>,
    },
    #[error("no DigiD session was found")]
    NoSession,
    #[error("invalid state token received in redirect URI")]
    StateTokenMismatch,
    #[error("no authorization code received in redirect URI")]
    NoAuthCode,
}
