use async_trait::async_trait;
use openid::Options;
use url::Url;

use crate::{pkce::PkcePair, utils::reqwest::default_reqwest_client_builder};

use super::openid_pkce::Client;

#[derive(Debug, thiserror::Error)]
pub enum OpenIdError {
    #[error("could not perform openid operation: {0}")]
    OpenId(#[from] openid::error::Error),
    #[error("no ID token received during authentication")]
    NoIdToken,
}

/// This trait is used to isolate the [`openid`] dependency, along with
/// [`reqwest`] on which [`openid`] depends.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OpenIdClient {
    /// Perform OpenID discovery and return a client instance on success.
    async fn discover(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, OpenIdError>
    where
        Self: Sized;

    /// Return the `redirect_uri` provided during discovery as a string slice.
    fn redirect_uri(&self) -> &str;

    /// Generate an authentication URL and PKCE pair for the configured issuer.
    /// This takes two generated tokens as parameters.
    fn auth_url_and_pkce<P>(&self, csrf_token: &str, nonce: &str) -> (Url, P)
    where
        P: PkcePair + 'static;

    /// Use an authentication code received in the redirect URI to fetch and validate an access token
    /// from the issuer. This requires both the nonce provided when generating the authentication URL
    /// and the PKCE verifier string that matches the PKCE challenge provided in the authentication URL.
    async fn authenticate<P>(&self, auth_code: &str, nonce: &str, pkce_pair: &P) -> Result<String, OpenIdError>
    where
        P: PkcePair + Send + Sync + 'static;
}

pub struct HttpOpenIdClient {
    openid_client: Client,
}

#[async_trait]
impl OpenIdClient for HttpOpenIdClient {
    async fn discover(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, OpenIdError> {
        // Configure a simple `reqwest` HTTP client with a timeout.
        let http_client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        // Perform OpenID discovery at the issuer, using our modified `Client`.
        let openid_client =
            Client::discover_with_client(http_client, client_id, None, redirect_uri.to_string(), issuer_url).await?;
        // Wrap the newly created `Client` instance in our newtype.
        let client = HttpOpenIdClient { openid_client };

        Ok(client)
    }

    fn redirect_uri(&self) -> &str {
        self.openid_client.redirect_url()
    }

    fn auth_url_and_pkce<P>(&self, csrf_token: &str, nonce: &str) -> (Url, P)
    where
        P: PkcePair,
    {
        // Collect all scopes supported by the issuer (as populated during discovery)
        // and join them together, separated by spaces.
        let scopes_supported = self
            .openid_client
            .config()
            .scopes_supported
            .as_ref()
            .map(|scopes| scopes.join(" "))
            .unwrap_or_default();

        // Generate the authentication URL containing these scopes and the provided tokens.
        let options = Options {
            scope: Some(scopes_supported),
            state: Some(csrf_token.to_string()),
            nonce: Some(nonce.to_string()),
            ..Default::default()
        };

        self.openid_client.auth_url_and_pkce(&options)
    }

    async fn authenticate<P>(&self, auth_code: &str, nonce: &str, pkce_pair: &P) -> Result<String, OpenIdError>
    where
        P: PkcePair + Send + Sync,
    {
        // Forward the received method parameters to our `Client` instance.
        let token = self
            .openid_client
            .authenticate_pkce(auth_code, pkce_pair, nonce, None)
            .await?;

        // Double check if the received token had an ID token, otherwise
        // validation of the token will not actually have taken place.
        if token.id_token.is_none() {
            return Err(OpenIdError::NoIdToken);
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

    use crate::pkce::MockPkcePair;

    use super::*;

    #[tokio::test]
    async fn test_http_open_id_client() {
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

        // Perform OpenID discovery
        let client = HttpOpenIdClient::discover(server_url.clone(), client_id.to_string(), redirect_uri.clone())
            .await
            .expect("Could not perform OpenID discovery");

        let pkce_pair_generate_context = MockPkcePair::generate_context();
        pkce_pair_generate_context.expect().returning(|| {
            let mut pkce_pair = MockPkcePair::new();

            pkce_pair
                .expect_code_challenge()
                .return_const("pkcecodechallenge".to_string());

            pkce_pair
        });

        // Generate authentication URL
        let (url, _) = client.auth_url_and_pkce::<MockPkcePair>(csrf_token, nonce);

        assert_eq!(
            url,
            server_url
                .join(
                    "/oauth2/auth?response_type=code&client_id=client-1&redirect_uri=\
                    http%3A%2F%2Fexample-client.com%2Foauth2%2Fcallback&scope=openid+&state=csrftoken&nonce=\
                    thisisthenonce&code_challenge=pkcecodechallenge&code_challenge_method=INVALID"
                )
                .unwrap(),
        );

        pkce_pair_generate_context.checkpoint();

        // TODO: Add test for the authenticate() method by mocking an authentication
        //       response that can actually be verified by the client.
    }
}
