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

/// This is actually an integration test over the [`openid`] crate and our own code.
/// HTTP responses are mocked using the [`wiremock`] crate.
#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;

    #[tokio::test]
    async fn test_remote_open_id_client() {
        let server = MockServer::start().await;
        let server_url = Url::parse(&server.uri()).unwrap();

        // Mock OpenID configuration endpoint
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "issuer": server_url,
                "authorization_endpoint": server_url.join("/oauth2/auth").unwrap(),
                "token_endpoint": server_url.join("/oauth2/token").unwrap(),
                "jwks_uri": server_url.join("/.well-known/jwks.json").unwrap(),
                "response_types_supported": ["code", "id_token", "token id_token"]
            })))
            .expect(1)
            .mount(&server)
            .await;

        // Mock JWKS endpoint
        Mock::given(method("GET"))
            .and(path("/.well-known/jwks.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "keys": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        // All variables used
        let client_id = "client-1";
        let redirect_uri = Url::parse("http://example-client.com/oauth2/callback").unwrap();
        let csrf_token = "csrftoken";
        let nonce = "thisisthenonce";
        let pkce_challenge = "pkcecodechallenge";

        // Perform OpenID discovery
        let client = RemoteOpenIdClient::discover(server_url.clone(), client_id.to_string(), redirect_uri.clone())
            .await
            .expect("Could not perform OpenID discovery");

        // Generate authentication URL
        let url = client.auth_url(csrf_token, nonce, pkce_challenge);

        assert_eq!(
            url,
            server_url
                .join(
                    "/oauth2/auth?response_type=code&client_id=client-1&redirect_uri=\
                    http%3A%2F%2Fexample-client.com%2Foauth2%2Fcallback&scope=openid+&state=csrftoken&nonce=\
                    thisisthenonce&code_challenge=pkcecodechallenge&code_challenge_method=S256"
                )
                .unwrap(),
        );

        // TODO: Add test for the authenticate() method by mocking an authentication
        //       response that can actually be verified by the client.
    }
}
