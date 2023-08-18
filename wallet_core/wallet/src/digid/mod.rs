mod client;
mod openid;
mod openid_client;
mod pkce;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use self::openid_client::OpenIdClientError;

pub use self::client::DigidClient;

/// Global variable to hold the `DigidClient` singleton.
pub static DIGID_CLIENT: Lazy<Mutex<DigidClient>> = Lazy::new(|| Mutex::new(DigidClient::default()));

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
