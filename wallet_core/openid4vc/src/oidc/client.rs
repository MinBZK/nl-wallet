use base64::prelude::*;
pub use biscuit::errors::Error as BiscuitError;
pub use biscuit::jwa;
pub use biscuit::jwa::SignatureAlgorithm;
pub use biscuit::jwk::JWKSet;
pub use biscuit::ClaimsSet;
pub use biscuit::CompactJson;
pub use biscuit::CompactPart;
pub use biscuit::Empty;
pub use biscuit::ValidationOptions;
pub use biscuit::JWT;
use futures::TryFutureExt;
pub use josekit::jwe::alg;
pub use josekit::jwe::enc;
pub use josekit::jwe::JweContentEncryption;
pub use josekit::jwe::JweDecrypter;
pub use josekit::JoseError;
use reqwest::header;
use url::Url;

use error_category::ErrorCategory;
use wallet_common::reqwest::trusted_reqwest_client_builder;
use wallet_common::urls::BaseUrl;
use wallet_common::utils;

use crate::authorization::AuthorizationRequest;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PkceCodeChallenge;
use crate::authorization::ResponseType;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::token::AccessToken;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;
use crate::AuthBearerErrorCode;
use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::TokenErrorCode;

use super::Config;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum OidcError {
    #[error("transport error: {0}")]
    #[category(expected)]
    Http(#[from] reqwest::Error),
    #[error("url: path segments is cannot-be-a-base")]
    #[category(critical)]
    CannotBeABase,
    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
    #[error("URL decoding error: {0}")]
    UrlDecoding(#[from] serde::de::value::Error),
    #[error("error requesting authorization code: {0:?}")]
    RedirectUriError(ErrorResponse<AuthorizationErrorCode>),
    #[error("error requesting access token: {0:?}")]
    RequestingAccessToken(Box<ErrorResponse<TokenErrorCode>>),
    #[error("error requesting userinfo: {0:?}")]
    RequestingUserInfo(Box<ErrorResponse<AuthBearerErrorCode>>),
    #[error("invalid state token received in redirect URI")]
    #[category(critical)]
    StateTokenMismatch,
    #[error("no authorization code received in redirect URI")]
    #[category(critical)]
    NoAuthCode,
    #[error("invalid redirect URI received")]
    #[category(critical)]
    RedirectUriMismatch,
    #[error("JOSE error: {0}")]
    JoseKit(#[from] JoseError),
    #[error("JWE error: {0}")]
    Jwe(#[from] BiscuitError),
    #[error("JWE validation error: {0}")]
    JweValidation(#[from] biscuit::errors::ValidationError),
    #[error("config has no userinfo url")]
    #[category(critical)]
    NoUserinfoUrl,
}

const APPLICATION_JWT: &str = "application/jwt";

/// This trait is used to isolate the [`HttpOidcClient`], along with [`reqwest`] on which it depends.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OidcClient {
    /// Create a new instance by using OpenID discovery, and return an authorization URL.
    async fn start(
        trust_anchors: Vec<reqwest::Certificate>,
        issuer_url: BaseUrl,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<(Self, Url), OidcError>
    where
        Self: Sized;

    /// Create an OpenID Token Request based on the contents
    /// of the redirect URI received.
    ///
    /// Note that this consumes the [`OidcClient`], either on success or failure.
    /// Retrying this operation is entirely possible, but most likely not something
    /// that the UI will present to the user, instead they will have to start a new session.
    /// For the purpose of simplification, that means that this operation is transactional
    /// here as well.
    fn into_token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OidcError>;
}

/// An OpenID Connect client.
pub struct HttpOidcClient<P = S256PkcePair> {
    pub provider: Config,
    pub jwks: Option<JWKSet<Empty>>,

    client_id: String,
    redirect_uri: Url,

    pkce_pair: P,
    state: String,
    nonce: String,
}

impl<P: PkcePair> OidcClient for HttpOidcClient<P> {
    async fn start(
        trust_anchors: Vec<reqwest::Certificate>,
        issuer: BaseUrl,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<(Self, Url), OidcError> {
        let http_client = trusted_reqwest_client_builder(trust_anchors).build()?;
        let config = Config::discover(&http_client, &issuer).await?;
        let jwks = config.jwks(&http_client).await?;

        let client = Self::new(config, jwks, client_id, redirect_uri);

        let mut url = client.provider.authorization_endpoint.clone();
        url.set_query(Some(&client.url_encoded_auth_request()?));

        Ok((client, url))
    }

    fn into_token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OidcError> {
        let pre_authorized_code = self.authorization_code(received_redirect_uri)?;
        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            code_verifier: Some(self.pkce_pair.into_code_verifier()),
            client_id: Some(self.client_id),
            redirect_uri: Some(self.redirect_uri),
        };
        Ok(token_request)
    }
}

impl<P: PkcePair> HttpOidcClient<P> {
    pub fn new(config: Config, jwks: JWKSet<Empty>, client_id: String, redirect_uri: Url) -> Self {
        let csrf_token = BASE64_URL_SAFE_NO_PAD.encode(utils::random_bytes(16));
        let nonce = BASE64_URL_SAFE_NO_PAD.encode(utils::random_bytes(16));
        let pkce_pair = P::generate();

        HttpOidcClient {
            provider: config,
            client_id,
            redirect_uri,
            jwks: Some(jwks),
            pkce_pair,
            state: csrf_token,
            nonce,
        }
    }

    // If and when we support PAR (Pushed Authorization Requests, https://datatracker.ietf.org/doc/html/rfc9126)
    // we can make this method public.
    fn url_encoded_auth_request(&self) -> Result<String, OidcError> {
        let params = AuthorizationRequest {
            response_type: ResponseType::Code.into(),
            client_id: self.client_id.clone(),
            redirect_uri: Some(self.redirect_uri.clone()),
            state: Some(self.state.clone()),
            authorization_details: None,
            request_uri: None,
            code_challenge: Some(PkceCodeChallenge::S256 {
                code_challenge: self.pkce_pair.code_challenge().to_string(),
            }),
            scope: self.provider.scopes_supported.clone(),
            nonce: Some(self.nonce.clone()),
            response_mode: None,
        };

        Ok(serde_urlencoded::to_string(params)?)
    }

    fn matches_received_redirect_uri(&self, received_redirect_uri: &Url) -> bool {
        received_redirect_uri.as_str().starts_with(self.redirect_uri.as_str())
    }

    fn authorization_code(&self, received_redirect_uri: &Url) -> Result<AuthorizationCode, OidcError> {
        if !self.matches_received_redirect_uri(received_redirect_uri) {
            return Err(OidcError::RedirectUriMismatch);
        }

        let auth_response = received_redirect_uri.query().ok_or(OidcError::NoAuthCode)?;

        // First see if we received an error
        if received_redirect_uri.query_pairs().any(|(key, _)| key == "error") {
            let err_response: ErrorResponse<AuthorizationErrorCode> = serde_urlencoded::from_str(auth_response)?;
            return Err(OidcError::RedirectUriError(err_response));
        }

        let auth_response: AuthorizationResponse = serde_urlencoded::from_str(auth_response)?;
        if auth_response.state.as_ref() != Some(&self.state) {
            return Err(OidcError::StateTokenMismatch);
        }

        Ok(auth_response.code.into())
    }
}

pub async fn request_token(
    http_client: &reqwest::Client,
    issuer: &BaseUrl,
    token_request: TokenRequest,
) -> Result<TokenResponse, OidcError> {
    let config = Config::discover(http_client, issuer).await?;

    let response: TokenResponse = http_client
        .post(config.token_endpoint.clone())
        .form(&token_request)
        .send()
        .map_err(OidcError::from)
        .and_then(|response| async {
            // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                let error = response.json::<ErrorResponse<TokenErrorCode>>().await?;
                Err(OidcError::RequestingAccessToken(error.into()))
            } else {
                Ok(response.json().await?)
            }
        })
        .await?;

    Ok(response)
}

pub async fn request_userinfo<C, H>(
    http_client: &reqwest::Client,
    issuer: &BaseUrl,
    access_token: &AccessToken,
    expected_sig_alg: SignatureAlgorithm,
    encryption: Option<(&impl JweDecrypter, &impl JweContentEncryption)>,
) -> Result<JWT<C, H>, OidcError>
where
    ClaimsSet<C>: CompactPart,
    H: CompactJson,
{
    let config = Config::discover(http_client, issuer).await?;
    let jwks = config.jwks(http_client).await?;

    // Get userinfo endpoint from discovery, throw an error otherwise.
    let endpoint = config.userinfo_endpoint.clone().ok_or(OidcError::NoUserinfoUrl)?;

    // Use the access_token to retrieve the userinfo as a JWT.
    let jwt = http_client
        .post(endpoint)
        .header(header::ACCEPT, APPLICATION_JWT)
        .bearer_auth(access_token.as_ref())
        .send()
        .map_err(OidcError::from)
        .and_then(|response| async {
            // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                let error = response.json::<ErrorResponse<AuthBearerErrorCode>>().await?;
                Err(OidcError::RequestingUserInfo(error.into()))
            } else {
                let text = response.text().await?;
                Ok(text)
            }
        })
        .await?;

    let jws = match encryption {
        Some((decrypter, expected_enc_alg)) => decrypt_jwe(&jwt, decrypter, expected_enc_alg)?,
        None => jwt.as_bytes().to_vec(),
    };

    // Get a JWS from the (decrypted) JWT and decode it using the JWK set.
    let decoded_jws = JWT::<C, H>::from_bytes(&jws)?.decode_with_jwks(&jwks, Some(expected_sig_alg))?;

    decoded_jws
        .payload()?
        .registered
        .validate(ValidationOptions::default())?;

    Ok(decoded_jws)
}

fn decrypt_jwe(
    jwe_token: &str,
    decrypter: &impl JweDecrypter,
    expected_enc_alg: &impl JweContentEncryption,
) -> Result<Vec<u8>, OidcError> {
    // Unfortunately we need to use josekit to decrypt the JWE, as we wish to support the A128CBC_HS256 content
    // encryption which biscuit does not yet support.
    // See https://github.com/lawliet89/biscuit/issues/42
    let (jwe_payload, header) = josekit::jwe::deserialize_compact(jwe_token, decrypter)?;

    // Check the "enc" header to confirm that that the content is encoded with the expected algorithm.
    if header.content_encryption() == Some(expected_enc_alg.name()) {
        Ok(jwe_payload)
    } else {
        // This is the error that would have been returned, if the biscuit crate had done the algorithm checking.
        Err(biscuit::errors::ValidationError::WrongAlgorithmHeader)?
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use biscuit::jwk::JWKSet;
    use rstest::rstest;
    use url::Url;
    use wallet_common::urls::BaseUrl;

    use crate::oidc::tests::start_discovery_server;
    use crate::pkce::MockPkcePair;
    use crate::pkce::S256PkcePair;
    use crate::token::TokenRequestGrantType;
    use crate::AuthorizationErrorCode;

    use super::Config;
    use super::HttpOidcClient;
    use super::OidcClient;
    use super::OidcError;

    // These constants are used by multiple tests.
    const ISSUER_URL: &str = "http://example.com";
    const CLIENT_ID: &str = "client-1";
    const REDIRECT_URI: &str = "redirect://here";
    const CSRF_TOKEN: &str = "csrf_token";
    const CODE: &str = "code";
    const PARAM_ERROR: &str = "error";
    const PARAM_ERROR_DESCRIPTION: &str = "error_description";
    const PARAM_STATE: &str = "state";
    const PARAM_CODE: &str = "code";
    const PARAM_NONCE: &str = "nonce";
    const PARAM_PKCE_CHALLENGE: &str = "challenge";
    const PARAM_PKCE_VERIFIER: &str = "verifier";

    #[tokio::test]
    async fn test_start_and_into_token_request() {
        // Setup mock PKCE expectations
        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair
                .expect_code_challenge()
                .return_const(PARAM_PKCE_CHALLENGE.to_string());
            pkce_pair
                .expect_into_code_verifier()
                .return_const(PARAM_PKCE_VERIFIER.to_string());
            pkce_pair
        });

        // Create an OIDC client with start()
        let (_server, server_url) = start_discovery_server().await;
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let (client, auth_url) = HttpOidcClient::<MockPkcePair>::start(
            vec![],
            server_url.clone(),
            CLIENT_ID.to_string(),
            redirect_uri.clone(),
        )
        .await
        .unwrap();

        assert_eq!(&client.client_id, CLIENT_ID);
        assert_eq!(client.redirect_uri, redirect_uri);
        assert!(auth_url.as_str().starts_with(server_url.as_ref().as_str()));

        // Convert it to a token request
        let state = client.state.clone();
        let token_request = client
            .into_token_request(&redirect_uri.join(&format!("?code={}&state={}", CODE, state)).unwrap())
            .unwrap();

        assert_eq!(token_request.client_id, Some(CLIENT_ID.to_string()));
        assert_eq!(token_request.code_verifier, Some(PARAM_PKCE_VERIFIER.to_string()));
        assert_eq!(token_request.redirect_uri, Some(REDIRECT_URI.parse().unwrap()));
        assert_matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } if pre_authorized_code.as_ref() == CODE
        );
    }

    fn create_client() -> HttpOidcClient<MockPkcePair> {
        let server_url: BaseUrl = ISSUER_URL.parse().unwrap();

        let mut pkce_pair = MockPkcePair::new();
        pkce_pair.expect_code_challenge().return_const("challenge".to_string());

        HttpOidcClient {
            provider: Config::new_mock(&server_url),
            jwks: Some(JWKSet { keys: vec![] }),
            client_id: CLIENT_ID.to_string(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            pkce_pair,
            state: PARAM_STATE.to_string(),
            nonce: PARAM_NONCE.to_string(),
        }
    }

    #[tokio::test]
    async fn test_auth_url() {
        let client = create_client();

        // Generate authentication URL
        let auth_request = client.url_encoded_auth_request().unwrap();

        #[rustfmt::skip]
        assert_eq!(
            auth_request,
            "response_type=code\
                &client_id=client-1\
                &redirect_uri=redirect%3A%2F%2Fhere\
                &state=state\
                &code_challenge_method=S256\
                &code_challenge=challenge\
                &scope=openid\
                &nonce=nonce",
        );
    }

    #[test]
    fn test_matches_received_redirect_uri() {
        let client = create_client();

        // These URIs should match the `base_redirect_uri`.
        assert!(client.matches_received_redirect_uri(&Url::parse(REDIRECT_URI).unwrap()));
        assert!(client.matches_received_redirect_uri(&url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("foo", "bar"), ("bleh", "blah")]
        )));

        // These URIs should NOT match the `base_redirect_uri`.
        assert!(!client.matches_received_redirect_uri(&Url::parse("https://example.com").unwrap()));
        assert!(!client.matches_received_redirect_uri(&Url::parse("scheme://host/path").unwrap()));
    }

    // Helper function for testing `Client::token_request()` calls that should result in an error.
    fn parse_request_uri(uri: &Url) -> OidcError {
        let client = HttpOidcClient::<S256PkcePair>::new(
            Config::new_mock(&ISSUER_URL.parse().unwrap()),
            JWKSet { keys: vec![] },
            CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
        );

        client
            .into_token_request(uri)
            .expect_err("Getting access token should have failed")
    }

    #[tokio::test]
    async fn test_redirect_uri_mismatch() {
        // This URI does not match the `redirect_uri_base`.
        let uri = Url::parse("http://not-the-redirect-uri.com").unwrap();
        let error = parse_request_uri(&uri);

        assert_matches!(error, OidcError::RedirectUriMismatch);
    }

    #[tokio::test]
    async fn test_error() {
        // This URI contains an `error` query parameter, with an `error_description`.
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[
                (PARAM_ERROR, "invalid_request"),
                (PARAM_ERROR_DESCRIPTION, "this is the error description"),
            ],
        );

        let error = parse_request_uri(&uri);

        assert_matches!(
            error,
            OidcError::RedirectUriError(response) if matches!(response.error, AuthorizationErrorCode::InvalidRequest)
                && response.error_description == Some("this is the error description".to_string()
            )
        );
    }

    #[tokio::test]
    async fn test_state_mismatch() {
        // This URI contains an incorrect `state` query parameter.
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[(PARAM_CODE, CODE), (PARAM_STATE, "foobar")],
        );

        let error = parse_request_uri(&uri);

        assert_matches!(error, OidcError::StateTokenMismatch);
    }

    #[tokio::test]
    async fn test_missing_code() {
        // This URI is missing the `code` query parameter.
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[(PARAM_STATE, CSRF_TOKEN)]);

        let error = parse_request_uri(&uri);

        assert_matches!(error, OidcError::UrlDecoding(err) if err.to_string() == "missing field `code`");
    }

    #[test]
    fn test_into_token_request() {
        let mut client = create_client();

        client
            .pkce_pair
            .expect_into_code_verifier()
            .return_const("code_verifier".to_string());

        let redirect_uri = Url::parse(REDIRECT_URI).unwrap();
        let received_redirect_uri =
            url_with_query_pairs(redirect_uri.clone(), &[("state", &client.state), ("code", "123")]);

        let token_request = client.into_token_request(&received_redirect_uri).unwrap();

        assert_eq!(token_request.client_id, Some(CLIENT_ID.to_string()));
        assert_eq!(token_request.code_verifier, Some("code_verifier".to_string()));
        assert_eq!(token_request.redirect_uri, Some(redirect_uri));
        assert_matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } if pre_authorized_code.as_ref() == "123"
        );
    }

    #[test]
    fn test_get_authorization_url() {
        let client = create_client();

        let redirect_uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("state", &client.state), ("code", "123")],
        );

        assert_eq!(client.authorization_code(&redirect_uri).unwrap().as_ref(), "123");
    }

    pub fn url_with_query_pairs(mut url: Url, query_pairs: &[(&str, &str)]) -> Url {
        if !query_pairs.is_empty() {
            let mut query = url.query_pairs_mut();

            query_pairs.iter().for_each(|(name, value)| {
                query.append_pair(name, value);
            });
        }

        url
    }

    #[rstest]
    #[case("http://example.com", [], "http://example.com")]
    #[case("http://example.com", [("foo", "bar"), ("bleh", "blah")], "http://example.com?foo=bar&bleh=blah")]
    #[case("http://example.com", [("foo", ""), ("foo", "more_foo")], "http://example.com?foo=&foo=more_foo")]
    fn test_url_with_query_pairs<const N: usize>(
        #[case] url: Url,
        #[case] query_pairs: [(&str, &str); N],
        #[case] expected: Url,
    ) {
        let url = url_with_query_pairs(url, &query_pairs);

        assert_eq!(url, expected);
    }
}
