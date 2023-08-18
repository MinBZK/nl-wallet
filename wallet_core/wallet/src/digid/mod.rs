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
    /// Start a new DigiD session by performing OpenID discovery and returning
    /// an authorization URL that can be sent to the system browser.
    async fn start_session(
        &mut self,
        issuer_url: &Url,
        client_id: &str,
        redirect_uri: &Url,
    ) -> Result<Url, DigidClientError>;

    /// Check if the DigiD client would currently accept the provided redirect URI.
    fn accepts_redirect_uri(&self, redirect_uri: &Url) -> bool;

    /// Retrieve the access token from DigiD, based on the contents
    /// of the redirect URI received.
    async fn get_access_token(&mut self, received_redirect_uri: &Url) -> Result<String, DigidClientError>;
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
