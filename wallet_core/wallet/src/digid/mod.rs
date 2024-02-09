mod client;
mod openid_client;
mod openid_pkce;

use url::Url;

use openid4vc::token::{AuthorizationCode, TokenRequest};

pub use self::client::HttpDigidSession;
pub use self::openid_client::OpenIdError;

#[cfg(feature = "wallet_deps")]
pub use {self::openid_client::HttpOpenIdClient, crate::pkce::S256PkcePair};

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
    #[error("invalid state token received in redirect URI")]
    StateTokenMismatch,
    #[error("no authorization code received in redirect URI")]
    NoAuthCode,
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DigidSession {
    /// Start a new DigiD session by performing OpenID discovery and returning
    /// an authorization URL that can be sent to the system browser.
    async fn start(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, DigidError>
    where
        Self: Sized;

    /// Generate an authentication URL for the session.
    fn auth_url(&self) -> Url;

    /// Check if the DigiD session matches the provided redirect URI.
    fn matches_received_redirect_uri(&self, received_redirect_uri: &Url) -> bool;

    /// Parse the authorization code from the redirect URI.
    fn get_authorization_code(&self, received_redirect_uri: &Url) -> Result<AuthorizationCode, DigidError>;

    fn into_pre_authorized_code_request(self, pre_authorized_code: AuthorizationCode) -> TokenRequest;
}
