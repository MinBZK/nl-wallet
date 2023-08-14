use std::time::Duration;

use async_trait::async_trait;
use openid::Options;
use url::Url;

use super::openid::Client;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum OpenIdClientError {
    #[error("could not perform openid operation: {0}")]
    OpenId(#[from] openid::error::Error),
    #[error("no ID token received during authentication")]
    NoIdToken,
}

/// This trait is used to isolate the [`openid`] dependency, along with
/// [`reqwest`] on which [`openid`] depends.
#[async_trait]
pub trait OpenIdClient {
    /// Perform OpenID discovery and return a client instance on success.
    async fn discover(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, OpenIdClientError>
    where
        Self: Sized;

    /// Generate an authentication URL for the configured issuer.
    /// This takes several generated tokens as parameters.
    fn auth_url(&self, csrf_token: &str, nonce: &str, pkce_challenge: &str) -> Url;
    /// Use an authentication code received in the redirect URI to fetch and validate an access token
    /// from the issuer. This requires both the nonce provided when generating the authentication URL
    /// and the PKCE verifier string that matches the PKCE challenge provided in the authentication URL.
    async fn authenticate(
        &self,
        auth_code: &str,
        nonce: &str,
        pkce_verifier: &str,
    ) -> Result<String, OpenIdClientError>;
}

pub struct RemoteOpenIdClient(Client);

#[async_trait]
impl OpenIdClient for RemoteOpenIdClient {
    async fn discover(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, OpenIdClientError> {
        // Configure a simple `reqwest` HTTP client with a timeout.
        let http_client = reqwest::Client::builder()
            .timeout(CLIENT_TIMEOUT)
            .build()
            .expect("Could not build reqwest HTTP client");

        // Perform OpenID discovery at the issuer, using our modified `Client`.
        let openid_client =
            Client::discover_with_client(http_client, client_id, None, redirect_uri.to_string(), issuer_url).await?;
        // Wrap the newly created `Client` instance in our newtype.
        let client = RemoteOpenIdClient(openid_client);

        Ok(client)
    }

    fn auth_url(&self, csrf_token: &str, nonce: &str, pkce_challenge: &str) -> Url {
        // Collect all scopes supported by the issuer (as populated during discovery)
        // and join them together, separated by spaces.
        let scopes_supported = self
            .0
            .config()
            .scopes_supported
            .as_ref()
            .map(|scopes| scopes.join(" "))
            .unwrap_or_default();

        // Generate the authetication URL containing these scopes and the provided tokens.
        let options = Options {
            scope: Some(scopes_supported),
            state: Some(csrf_token.to_string()),
            nonce: Some(nonce.to_string()),
            ..Default::default()
        };

        self.0.auth_url_pkce(&options, pkce_challenge)
    }

    async fn authenticate(
        &self,
        auth_code: &str,
        nonce: &str,
        pkce_verifier: &str,
    ) -> Result<String, OpenIdClientError> {
        // Forward the received method parameters to our `Client` instance.
        let token = self.0.authenticate_pkce(auth_code, pkce_verifier, nonce, None).await?;

        // Double check if the received token had an ID token, otherwise
        // validation of the token will not actually have taken place.
        if token.id_token.is_none() {
            return Err(OpenIdClientError::NoIdToken);
        }

        // Extract the resulting access token and return it.
        let access_token = token.bearer.access_token;

        Ok(access_token)
    }
}
