use std::marker::PhantomData;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use once_cell::sync::Lazy;
use url::Url;
use wallet_common::utils;

use crate::utils::url_find_first_query_value;

use super::{openid_client::OpenIdClient, pkce::PkceSource, DigidClientError};

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
static WALLET_CLIENT_REDIRECT_URI: Lazy<Url> = Lazy::new(|| {
    Url::parse("walletdebuginteraction://wallet.edi.rijksoverheid.nl/authentication")
        .expect("Could not parse DigiD redirect URI")
});

#[derive(Debug)]
pub struct DigidClient<C, P> {
    // Only one session may be active at a time. A potential improvement would be
    // to support multiple sessions and to persist these sessions, so that they may
    // be resumed after app termination.
    session_state: Option<DigidSessionState<C>>,
    _pkce_source: PhantomData<P>,
}

#[derive(Debug)]
struct DigidSessionState<C> {
    // The discovered OpenID client.
    openid_client: C,
    /// CSRF token (stored in state parameter).
    csrf_token: String,
    /// The generated nonce that was used.
    nonce: String,
    /// The PKCE verifier used.
    pkce_verifier: String,
}

impl<C, P> DigidClient<C, P> {
    fn new() -> Self {
        DigidClient {
            session_state: None,
            _pkce_source: PhantomData,
        }
    }
}

impl<C, P> Default for DigidClient<C, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, P> DigidClient<C, P>
where
    P: PkceSource,
    C: OpenIdClient,
{
    pub async fn start_session(&mut self) -> Result<Url, DigidClientError> {
        // TODO: This performs discovery every time a session is started and an authentication URL
        //       is generated. An improvement would be to cache the OpenIdClient and only perform
        //       discovery again when the configuration parameters change.

        // Perform OpenID discovery at the issuer.
        let openid_client = C::discover(
            DIGID_ISSUER_URL.clone(),
            WALLET_CLIENT_ID.to_string(),
            WALLET_CLIENT_REDIRECT_URI.clone(),
        )
        .await?;

        // Generate a random PKCE verifier, CSRF token and nonce.
        let (pkce_verifier, pkce_challenge) = P::generate_verifier_and_challenge();
        let csrf_token = URL_SAFE_NO_PAD.encode(utils::random_bytes(16));
        let nonce = URL_SAFE_NO_PAD.encode(utils::random_bytes(16));

        let url = openid_client.auth_url(&csrf_token, &nonce, &pkce_challenge);

        // Store the client and generated tokens as session state for when the redirect URI returns.
        self.session_state.replace(DigidSessionState {
            openid_client,
            pkce_verifier,
            csrf_token,
            nonce,
        });

        Ok(url)
    }

    pub async fn get_access_token(&mut self, redirect_url: Url) -> Result<String, DigidClientError> {
        // Check if the redirect URL received actually belongs to us.
        if !redirect_url.as_str().starts_with(WALLET_CLIENT_REDIRECT_URI.as_str()) {
            return Err(DigidClientError::RedirectUriMismatch);
        }

        // Pop the session state, return an error if we have none.
        let DigidSessionState {
            openid_client,
            pkce_verifier,
            csrf_token,
            nonce,
        } = self.session_state.take().ok_or(DigidClientError::NoSession)?;

        // Check if the `error` query parameter is populated, if so create an
        // error from it and a potential `error_description` query parameter.
        let error = url_find_first_query_value(&redirect_url, PARAM_ERROR);
        if let Some(error) = error {
            let error_description = url_find_first_query_value(&redirect_url, PARAM_ERROR_DESCRIPTION);

            return Err(DigidClientError::RedirectUriError {
                error: error.into_owned(),
                error_description: error_description.map(|d| d.into_owned()),
            });
        }

        // Verify that the state query parameter matches the csrf_token.
        let state =
            url_find_first_query_value(&redirect_url, PARAM_STATE).ok_or(DigidClientError::StateTokenMismatch)?;

        if state != csrf_token {
            return Err(DigidClientError::StateTokenMismatch);
        }

        // Parse the authorization code from the redirect URL.
        let authorization_code =
            url_find_first_query_value(&redirect_url, PARAM_CODE).ok_or(DigidClientError::NoAuthCode)?;

        // Use the authorization code and the PKCE verifier to request the
        // access token and verify the result.
        let access_token = openid_client
            .authenticate(&authorization_code, &nonce, &pkce_verifier)
            .await?;

        Ok(access_token)
    }
}
