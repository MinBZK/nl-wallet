mod client;
mod openid;
mod pkce;

use tokio::sync::{Mutex, OnceCell};

pub use client::DigidClient;

/// Global variable to hold the `DigidClient` singleton.
static DIGID_CLIENT: OnceCell<Mutex<DigidClient>> = OnceCell::const_new();

#[derive(Debug, thiserror::Error)]
pub enum DigidError {
    #[error("could not perform openid operation: {0}")]
    OpenId(#[from] ::openid::error::Error),
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
    #[error("no ID token received from DigiD connector")]
    NoIdToken,
}

pub async fn get_or_initialize_digid_client() -> Result<&'static Mutex<DigidClient>, DigidError> {
    DIGID_CLIENT
        .get_or_try_init(|| async {
            let connector = DigidClient::create().await?;

            Ok(Mutex::new(connector))
        })
        .await
}
