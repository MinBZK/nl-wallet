use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use once_cell::sync::Lazy;
use openid::Options;
use url::Url;
use wallet_common::utils;

use crate::utils::url_find_first_query_value;

use super::{openid::Client, pkce, DigidError};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

const PARAM_ERROR: &str = "error";
const PARAM_ERROR_DESCRIPTION: &str = "error_description";
const PARAM_STATE: &str = "state";
const PARAM_CODE: &str = "code";

// TODO: Read from configuration.
static DIGID_ISSUER_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://example.com/digid-connector")
        .expect("Could not parse DigiD issuer URL")
});

// TODO: read the following values from configuration, and align with digid-connector configuration
const WALLET_CLIENT_ID: &str = "SSSS";
const WALLET_CLIENT_REDIRECT_URI: &str = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/authentication";

pub struct DigidClient {
    openid_client: Client,
    session_state: Option<DigidSessionState>,
}

struct DigidSessionState {
    /// The PKCE verifier used.
    pkce_verifier: String,
    /// CSRF token (stored in state parameter).
    csrf_token: String,
    /// The generated nonce that was used.
    nonce: String,
}

impl DigidClient {
    pub async fn create() -> Result<Self, DigidError> {
        let http_client = reqwest::Client::builder()
            .timeout(CLIENT_TIMEOUT)
            .build()
            .expect("Could not build reqwest HTTP client");
        let openid_client = Client::discover_with_client(
            http_client,
            WALLET_CLIENT_ID.to_string(),
            None,
            Some(WALLET_CLIENT_REDIRECT_URI.to_string()),
            DIGID_ISSUER_URL.clone(),
        )
        .await?;

        let client = DigidClient {
            openid_client,
            session_state: None,
        };

        Ok(client)
    }

    pub fn start_session(&mut self) -> Url {
        // Generate a random PKCE verifier, CSRF token and nonce.
        let (pkce_verifier, code_challenge) = pkce::generate_verifier_and_challenge();
        let csrf_token = URL_SAFE_NO_PAD.encode(utils::random_bytes(16));
        let nonce = URL_SAFE_NO_PAD.encode(utils::random_bytes(16));

        // Store the generated tokens as session state for when the redirect URI returns.
        self.session_state.replace(DigidSessionState {
            pkce_verifier,
            csrf_token: csrf_token.clone(),
            nonce: nonce.clone(),
        });

        // Get all scopes supported by the DigiD connector (populated during discovery),
        // the use these and the tokens generated above to create an authentication URL.
        let scopes_supported = self
            .openid_client
            .config()
            .scopes_supported
            .as_ref()
            .map(|scopes| scopes.join(" "))
            .unwrap_or_default();

        let options = Options {
            scope: Some(scopes_supported),
            state: Some(csrf_token),
            nonce: Some(nonce),
            ..Default::default()
        };

        self.openid_client.auth_url_pkce(&options, &code_challenge)
    }

    pub async fn get_access_token(&mut self, redirect_url: Url) -> Result<String, DigidError> {
        // Check if the redirect URL received actually belongs to us.
        if !redirect_url.as_str().starts_with(WALLET_CLIENT_REDIRECT_URI) {
            return Err(DigidError::RedirectUriMismatch);
        }

        // Pop the session state, return an error if we have none.
        let DigidSessionState {
            pkce_verifier,
            csrf_token,
            nonce,
        } = self.session_state.take().ok_or(DigidError::NoSession)?;

        // Check if the `error` query parameter is populated, if so create an
        // error from it and a potential `error_description` query parameter.
        let error = url_find_first_query_value(&redirect_url, PARAM_ERROR);
        if let Some(error) = error {
            let error_description = url_find_first_query_value(&redirect_url, PARAM_ERROR_DESCRIPTION);

            return Err(DigidError::RedirectUriError {
                error: error.into_owned(),
                error_description: error_description.map(|d| d.into_owned()),
            });
        }

        // Verify that the state query parameter matches the csrf_token.
        let state = url_find_first_query_value(&redirect_url, PARAM_STATE).ok_or(DigidError::StateTokenMismatch)?;

        if state != csrf_token {
            return Err(DigidError::StateTokenMismatch);
        }

        // Parse the authorization code from the redirect URL.
        let authorization_code = url_find_first_query_value(&redirect_url, PARAM_CODE).ok_or(DigidError::NoAuthCode)?;

        // Use the authorization code and the PKCE verifier to request the
        // access token and verify the result.
        let token = self
            .openid_client
            .authenticate_pkce(&authorization_code, &pkce_verifier, nonce.as_str(), None)
            .await?;

        // Double check if the received token had an ID token, otherwise
        // validation of the token will not actually have taken place.
        if token.id_token.is_none() {
            return Err(DigidError::NoIdToken);
        }

        let access_token = token.bearer.access_token;

        Ok(access_token)
    }
}
