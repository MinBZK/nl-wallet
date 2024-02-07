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
pub trait OpenIdClient {
    /// Perform OpenID discovery and return a client instance on success.
    async fn discover(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, OpenIdError>
    where
        Self: Sized;

    /// Generate an authentication URL for the configured issuer This takes two
    /// generated tokens and a generated PKCE pair as parameters.
    fn auth_url<P>(&self, csrf_token: String, nonce: String, pkce_pair: &P) -> Url
    where
        P: PkcePair + 'static;
}

pub struct HttpOpenIdClient {
    openid_client: Client,
}

impl OpenIdClient for HttpOpenIdClient {
    async fn discover(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, OpenIdError> {
        // Configure a simple `reqwest` HTTP client with a timeout.
        let http_client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        // Perform OpenID discovery at the issuer, using our modified `Client`.
        let openid_client =
            Client::discover_with_client(http_client, client_id, None, Some(redirect_uri.into()), issuer_url).await?;
        // Wrap the newly created `Client` instance in our newtype.
        let client = HttpOpenIdClient { openid_client };

        Ok(client)
    }

    fn auth_url<P>(&self, csrf_token: String, nonce: String, pkce_pair: &P) -> Url
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
            state: Some(csrf_token),
            nonce: Some(nonce),
            ..Default::default()
        };

        self.openid_client.auth_url(&options, pkce_pair)
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
        let pkce_pair = {
            let mut pkce_pair = MockPkcePair::new();

            pkce_pair
                .expect_code_challenge()
                .return_const("pkcecodechallenge".to_string());

            pkce_pair
        };

        // Generate authentication URL
        let url = client.auth_url(csrf_token.to_string(), nonce.to_string(), &pkce_pair);

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

        // TODO: Add test for the authenticate() method by mocking an authentication
        //       response that can actually be verified by the client.
    }
}
