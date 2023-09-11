mod client;
mod openid_client;
mod openid_pkce;

use async_trait::async_trait;
use url::Url;

pub use self::openid_client::OpenIdError;

pub use self::client::HttpDigidClient;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait DigidClient {
    /// Start a new DigiD session by performing OpenID discovery and returning
    /// an authorization URL that can be sent to the system browser.
    async fn start_session(&mut self, issuer_url: Url, client_id: String, redirect_uri: Url)
        -> Result<Url, DigidError>;

    /// Check if the DigiD client would currently accept the provided redirect URI.
    fn accepts_redirect_uri(&self, redirect_uri: &Url) -> bool;

    /// Cancel a DigiD session, if one is in progress.
    fn cancel_session(&mut self);

    /// Retrieve the access token from DigiD, based on the contents
    /// of the redirect URI received.
    async fn get_access_token(&mut self, received_redirect_uri: &Url) -> Result<String, DigidError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DigidError {
    #[error("{0}")]
    OpenId(#[from] OpenIdError),
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
