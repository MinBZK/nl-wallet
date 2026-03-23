pub use josekit::jwe::alg;
pub use josekit::jwe::enc;
pub use josekit::jwe::JweContentEncryption;
pub use josekit::jwe::JweDecrypter;
pub use josekit::JoseError;
pub use jsonwebtoken::jwk::JwkSet;
pub use jsonwebtoken::Algorithm;

use base64::prelude::*;
use error_category::ErrorCategory;
use futures::try_join;
use futures::TryFutureExt;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use reqwest::header;
use serde::de::DeserializeOwned;
use url::Url;

use crate::authorization::AuthorizationRequest;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PkceCodeChallenge;
use crate::authorization::ResponseType;
use crate::issuer_identifier::IssuerIdentifier;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;
use crate::well_known;
use crate::well_known::WellKnownError;
use crate::AuthBearerErrorCode;
use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::TokenErrorCode;

use super::AuthorizationServerMetadata;
use super::Discover;
use super::HttpDiscover;
use super::HttpJsonClient;

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
    RedirectUriError(Box<ErrorResponse<AuthorizationErrorCode>>),

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
    #[error("JWE decryption error: {0}")]
    JweDecryption(#[from] JoseError),

    #[error("JWT error: {0}")]
    Jsonwebtoken(#[from] jsonwebtoken::errors::Error),

    #[error("unexpected JWE content encryption algorithm")]
    #[category(critical)]
    UnexpectedEncAlgorithm,

    #[error("decrypted JWE payload is not valid UTF-8")]
    #[category(critical)]
    JwePayloadNotUtf8,

    #[error("JWT header is missing key ID (kid)")]
    #[category(critical)]
    MissingKeyId,

    #[error("JWT key ID not found in JWKS")]
    #[category(critical)]
    KeyNotFound,

    #[error("config has no userinfo url")]
    #[category(critical)]
    NoUserinfoUrl,

    #[error("config has no authorization endpoint")]
    #[category(critical)]
    NoAuthorizationEndpoint,

    #[error("config has no JWKS URI")]
    #[category(critical)]
    NoJwksUri,

    #[error("user denied authentication")]
    #[category(expected)]
    Denied,

    #[error("error fetching well-known metadata: {0}")]
    #[category(critical)]
    WellKnown(#[from] WellKnownError),
}

const APPLICATION_JWT: &str = "application/jwt";

/// Produces an [`AuthorizationServer`] by performing OIDC discovery.
pub trait OidcDiscovery {
    type Server: AuthorizationServer;

    /// Perform OIDC discovery and start an authorization flow, returning an [`AuthorizationServer`]
    /// (for later token exchange) and the authorization URL to redirect the user to.
    async fn discover(
        &self,
        authorization_server: &IssuerIdentifier,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<(Self::Server, Url), OidcError>;
}

/// Returned by [`OidcDiscovery::discover`]. Holds the state of an in-progress authorization code flow
/// and is consumed by [`into_token_request`] once the user redirects back.
///
/// [`into_token_request`]: AuthorizationServer::into_token_request
pub trait AuthorizationServer {
    /// Create an OpenID Token Request based on the contents of the redirect URI received.
    ///
    /// Note that this consumes the [`AuthorizationServer`], either on success or failure.
    fn into_token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OidcError>;
}

/// The discovered authorization server state for an in-progress OIDC authorization code flow.
#[derive(Debug)]
pub struct HttpAuthorizationServer<P = S256PkcePair> {
    provider: AuthorizationServerMetadata,
    #[expect(dead_code)]
    jwks: Option<JwkSet>,
    client_id: String,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
    nonce: String,
}

impl<P: PkcePair> HttpAuthorizationServer<P> {
    pub fn new(
        config: AuthorizationServerMetadata,
        jwks: Option<JwkSet>,
        client_id: String,
        redirect_uri: Url,
    ) -> Self {
        Self {
            provider: config,
            jwks,
            client_id,
            redirect_uri,
            pkce_pair: P::generate(),
            state: BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16)),
            nonce: BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16)),
        }
    }

    /// Returns the authorization URL to redirect the user to, with all PKCE/CSRF/nonce parameters encoded.
    pub fn auth_url(&self) -> Result<Url, OidcError> {
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

        let mut url = self
            .provider
            .authorization_endpoint
            .clone()
            .ok_or(OidcError::NoAuthorizationEndpoint)?;
        url.set_query(Some(&serde_urlencoded::to_string(params)?));
        Ok(url)
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

            if err_response.error == AuthorizationErrorCode::AccessDenied {
                return Err(OidcError::Denied);
            } else {
                return Err(OidcError::RedirectUriError(Box::new(err_response)));
            }
        }

        let auth_response: AuthorizationResponse = serde_urlencoded::from_str(auth_response)?;
        if auth_response.state.as_ref() != Some(&self.state) {
            return Err(OidcError::StateTokenMismatch);
        }

        Ok(auth_response.code.into())
    }
}

#[cfg(any(test, feature = "mock"))]
impl<P: PkcePair> HttpAuthorizationServer<P> {
    /// Returns the CSRF state token. Available in test/mock builds to allow constructing valid redirect URIs.
    pub fn csrf_state(&self) -> &str {
        &self.state
    }
}

impl<P: PkcePair> AuthorizationServer for HttpAuthorizationServer<P> {
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

/// Performs OIDC discovery to produce an [`HttpAuthorizationServer`].
pub struct HttpOidcDiscovery {
    http_client: HttpJsonClient,
}

impl HttpOidcDiscovery {
    pub fn new(http_client: HttpJsonClient) -> Self {
        Self { http_client }
    }

    pub async fn start<D, P>(
        &self,
        authorization_server: &IssuerIdentifier,
        client_id: String,
        redirect_uri: Url,
        discovery: &D,
    ) -> Result<(HttpAuthorizationServer<P>, Url), OidcError>
    where
        D: Discover<AuthorizationServerMetadata, OidcError>,
        P: PkcePair,
    {
        let config = discovery.discover(authorization_server).await?;
        let jwks = config.jwks(&self.http_client).await?;

        let server = HttpAuthorizationServer::new(config, Some(jwks), client_id, redirect_uri);
        let url = server.auth_url()?;

        Ok((server, url))
    }
}

impl OidcDiscovery for HttpOidcDiscovery {
    type Server = HttpAuthorizationServer;

    async fn discover(
        &self,
        authorization_server: &IssuerIdentifier,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<(HttpAuthorizationServer, Url), OidcError> {
        let discovery = HttpDiscover::new(self.http_client.clone());
        self.start(authorization_server, client_id, redirect_uri, &discovery)
            .await
    }
}

async fn request_userinfo_jwt(
    http_client: &HttpJsonClient,
    config: &AuthorizationServerMetadata,
    token_request: TokenRequest,
) -> Result<String, OidcError> {
    // Get userinfo endpoint from discovery, throw an error otherwise.
    let endpoint = config.userinfo_endpoint.clone().ok_or(OidcError::NoUserinfoUrl)?;

    let token_response = http_client
        .post(config.token_endpoint.clone(), |request| request.form(&token_request))
        .map_err(OidcError::from)
        .and_then(|response| async {
            // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                let error = response.json::<ErrorResponse<TokenErrorCode>>().await?;
                Err(OidcError::RequestingAccessToken(error.into()))
            } else {
                let token_response = response.json::<TokenResponse>().await?;

                Ok(token_response)
            }
        })
        .await?;

    // Use the access_token to retrieve the userinfo as a JWT.
    let jwt = http_client
        .post(endpoint, |request| {
            request
                .header(header::ACCEPT, APPLICATION_JWT)
                .bearer_auth(token_response.access_token.as_ref())
        })
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

    Ok(jwt)
}

fn decrypt_jwe(
    jwe_token: &str,
    decrypter: &impl JweDecrypter,
    expected_enc_alg: &impl JweContentEncryption,
) -> Result<Vec<u8>, OidcError> {
    let (jwe_payload, header) = josekit::jwe::deserialize_compact(jwe_token, decrypter)?;

    // Check the "enc" header to confirm that that the content is encoded with the expected algorithm.
    if header.content_encryption() == Some(expected_enc_alg.name()) {
        Ok(jwe_payload)
    } else {
        Err(OidcError::UnexpectedEncAlgorithm)
    }
}

pub async fn request_userinfo<C>(
    http_client: &HttpJsonClient,
    authorization_server: &IssuerIdentifier,
    token_request: TokenRequest,
    client_id: &str,
    expected_sig_alg: Algorithm,
    encryption: Option<(&impl JweDecrypter, &impl JweContentEncryption)>,
) -> Result<C, OidcError>
where
    C: DeserializeOwned,
{
    let config: AuthorizationServerMetadata = well_known::fetch_well_known_unvalidated(
        http_client,
        authorization_server,
        well_known::WellKnownPath::OpenidConfiguration,
    )
    .await?;

    let (jwt, jwks) = try_join!(
        request_userinfo_jwt(http_client, &config, token_request),
        config.jwks(http_client)
    )?;

    let jws = match encryption {
        Some((decrypter, expected_enc_alg)) => String::from_utf8(decrypt_jwe(&jwt, decrypter, expected_enc_alg)?)
            .map_err(|_| OidcError::JwePayloadNotUtf8)?,
        None => jwt,
    };

    verify_against_keys(&jws, &jwks, client_id, expected_sig_alg)
}

// We can't use our own `Jwt` types here because they only support ECDSA/P256.
fn verify_against_keys<C: DeserializeOwned>(
    token: &str,
    jwks: &JwkSet,
    audience: &str,
    algorithm: Algorithm,
) -> Result<C, OidcError> {
    let header = jsonwebtoken::decode_header(token)?;

    let kid = header.kid.as_deref().ok_or(OidcError::MissingKeyId)?;
    let jwk = jwks.find(kid).ok_or(OidcError::KeyNotFound)?;
    let key = DecodingKey::from_jwk(jwk)?;

    let mut validation = Validation::new(algorithm);
    validation.required_spec_claims.clear(); // don't require exp
    validation.set_audience(&[audience]);

    let verified = jsonwebtoken::decode(token, &key, &validation)?;

    Ok(verified.claims)
}

#[cfg(any(test, feature = "mock"))]
mockall::mock! {
    #[derive(Debug)]
    pub AuthorizationServer {
        pub fn token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OidcError>;
    }
}

#[cfg(any(test, feature = "mock"))]
impl AuthorizationServer for MockAuthorizationServer {
    fn into_token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OidcError> {
        self.token_request(received_redirect_uri)
    }
}

#[cfg(any(test, feature = "mock"))]
mockall::mock! {
    #[derive(Debug)]
    pub OidcDiscovery {
        pub fn start_sync(
            &self,
            authorization_server: &IssuerIdentifier,
            client_id: String,
            redirect_uri: Url,
        ) -> Result<(MockAuthorizationServer, Url), OidcError>;
    }
}

#[cfg(any(test, feature = "mock"))]
impl OidcDiscovery for MockOidcDiscovery {
    type Server = MockAuthorizationServer;

    async fn discover(
        &self,
        authorization_server: &IssuerIdentifier,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<(MockAuthorizationServer, Url), OidcError> {
        self.start_sync(authorization_server, client_id, redirect_uri)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
    use josekit::jwe::enc::aescbc_hmac::AescbcHmacJweEncryption;
    use josekit::jwe::JweHeader;
    use josekit::jwe::ECDH_ES_A256KW;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use josekit::jwk::Jwk;
    use josekit::jwk::KeyPair;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::EncodingKey;
    use jsonwebtoken::Header;
    use rstest::rstest;
    use serde_json::json;
    use serial_test::serial;
    use url::Url;

    use http_utils::urls::BaseUrl;

    use crate::issuer_identifier::IssuerIdentifier;
    use crate::oauth::tests::start_discovery_server;
    use crate::oauth::Discover;
    use crate::pkce::MockPkcePair;
    use crate::token::TokenRequestGrantType;
    use crate::AuthorizationErrorCode;

    use super::decrypt_jwe;
    use super::verify_against_keys;
    use super::AuthorizationServer;
    use super::AuthorizationServerMetadata;
    use super::HttpAuthorizationServer;
    use super::HttpJsonClient;
    use super::HttpOidcDiscovery;
    use super::JwkSet;
    use super::OidcError;

    /// A test discoverer that bypasses `IssuerIdentifier` HTTPS requirement by using a `BaseUrl` directly.
    struct TestDiscover(BaseUrl);

    impl Discover<AuthorizationServerMetadata, OidcError> for TestDiscover {
        async fn discover(&self, _identifier: &IssuerIdentifier) -> Result<AuthorizationServerMetadata, OidcError> {
            let client = HttpJsonClient::try_new().unwrap();
            let url = self.0.join("/.well-known/openid-configuration");
            client.get(url).await.map_err(OidcError::Http)
        }
    }

    // These constants are used by multiple tests.
    const ISSUER_URL: &str = "https://example.com";
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
    #[serial(MockPkcePair)]
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

        // Create an OIDC discovery with start_with_discover()
        let (_server, server_url) = start_discovery_server().await;
        let http_client = HttpJsonClient::try_new().unwrap();
        let authorization_server: IssuerIdentifier = "https://example.com/".parse().unwrap();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let discovery = HttpOidcDiscovery::new(http_client);
        let (server, auth_url) = discovery
            .start::<_, MockPkcePair>(
                &authorization_server,
                CLIENT_ID.to_string(),
                redirect_uri.clone(),
                &TestDiscover(server_url.clone()),
            )
            .await
            .unwrap();

        assert_eq!(&server.client_id, CLIENT_ID);
        assert_eq!(server.redirect_uri, redirect_uri);
        assert!(auth_url.as_str().starts_with(server_url.as_ref().as_str()));

        // Convert it to a token request
        let state = server.state.clone();
        let token_request = server
            .into_token_request(&redirect_uri.join(&format!("?code={CODE}&state={state}")).unwrap())
            .unwrap();

        assert_eq!(token_request.client_id, Some(CLIENT_ID.to_string()));
        assert_eq!(token_request.code_verifier, Some(PARAM_PKCE_VERIFIER.to_string()));
        assert_eq!(token_request.redirect_uri, Some(REDIRECT_URI.parse().unwrap()));
        assert_matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } if pre_authorized_code.as_ref() == CODE
        );
    }

    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_user_cancels() {
        // Setup mock PKCE expectations
        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair
                .expect_code_challenge()
                .times(1)
                .return_const(PARAM_PKCE_CHALLENGE.to_string());
            pkce_pair
        });

        let (_server, server_url) = start_discovery_server().await;
        let http_client = HttpJsonClient::try_new().unwrap();
        let authorization_server: IssuerIdentifier = "https://example.com/".parse().unwrap();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let discovery = HttpOidcDiscovery::new(http_client);
        let (server, _) = discovery
            .start::<_, MockPkcePair>(
                &authorization_server,
                CLIENT_ID.to_string(),
                redirect_uri.clone(),
                &TestDiscover(server_url.clone()),
            )
            .await
            .unwrap();

        // Convert it to a token request
        let state = server.state.clone();
        let error = server
            .into_token_request(
                &redirect_uri
                    .join(&format!("?error=access_denied&state={state}"))
                    .unwrap(),
            )
            .unwrap_err();

        assert_matches!(error, OidcError::Denied);
    }

    fn create_server() -> HttpAuthorizationServer<MockPkcePair> {
        let issuer_identifier = ISSUER_URL.parse::<IssuerIdentifier>().unwrap();

        let mut pkce_pair = MockPkcePair::new();
        pkce_pair.expect_code_challenge().return_const("challenge".to_string());

        HttpAuthorizationServer {
            provider: AuthorizationServerMetadata::new_mock(issuer_identifier),
            jwks: Some(JwkSet { keys: vec![] }),
            client_id: CLIENT_ID.to_string(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            pkce_pair,
            state: PARAM_STATE.to_string(),
            nonce: PARAM_NONCE.to_string(),
        }
    }

    #[tokio::test]
    async fn test_auth_url() {
        let server = create_server();

        let auth_url = server.auth_url().unwrap();

        #[rustfmt::skip]
        assert_eq!(
            auth_url.query().unwrap(),
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
        let server = create_server();

        // These URIs should match the `base_redirect_uri`.
        assert!(server.matches_received_redirect_uri(&Url::parse(REDIRECT_URI).unwrap()));
        assert!(server.matches_received_redirect_uri(&url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("foo", "bar"), ("bleh", "blah")]
        )));

        // These URIs should NOT match the `base_redirect_uri`.
        assert!(!server.matches_received_redirect_uri(&Url::parse("https://example.com").unwrap()));
        assert!(!server.matches_received_redirect_uri(&Url::parse("scheme://host/path").unwrap()));
    }

    // Helper function for testing `AuthorizationServer::into_token_request()` calls that should result in an error.
    fn parse_request_uri(uri: &Url) -> OidcError {
        let server = HttpAuthorizationServer::<MockPkcePair> {
            provider: AuthorizationServerMetadata::new_mock(ISSUER_URL.parse().unwrap()),
            jwks: Some(JwkSet { keys: vec![] }),
            client_id: CLIENT_ID.to_string(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            pkce_pair: MockPkcePair::new(),
            state: CSRF_TOKEN.to_string(),
            nonce: "nonce".to_string(),
        };

        server
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
        let mut server = create_server();

        server
            .pkce_pair
            .expect_into_code_verifier()
            .return_const("code_verifier".to_string());

        let redirect_uri = Url::parse(REDIRECT_URI).unwrap();
        let state = server.state.clone();
        let received_redirect_uri = url_with_query_pairs(redirect_uri.clone(), &[("state", &state), ("code", "123")]);

        let token_request = server.into_token_request(&received_redirect_uri).unwrap();

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
        let server = create_server();

        let redirect_uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("state", &server.state), ("code", "123")],
        );

        assert_eq!(server.authorization_code(&redirect_uri).unwrap().as_ref(), "123");
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

    // This value was captured from nl-rdo-max in a local dev environment.
    static JWS_PAYLOAD: LazyLock<serde_json::Value> = LazyLock::new(|| {
        json!({
            "aud": "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            "bsn": "999991772",
            "iss": "https://localhost:8006",
            "loa_authn": "http://eidas.europa.eu/LoA/substantial",
            "session_id": "oKir-PwoC36a5TxX5vwIIPAU7WXoGVEsTkUwGSAv9ZM",
            "sub": "ff5a4850ab665a3196ec4311d187a24d615d164787b38c89b98f6144855ddcfe"
        })
    });

    fn make_jws(include_kid: bool) -> (String, JwkSet) {
        let algoritm = Algorithm::HS256;
        let kid = "hmac_key_id";

        let mut header = Header::new(algoritm);
        if include_kid {
            header.kid = Some(kid.to_string());
        }
        let encoding_key = EncodingKey::from_secret(b"secret hmac key");
        let jws = jsonwebtoken::encode(&header, LazyLock::force(&JWS_PAYLOAD), &encoding_key).unwrap();

        let mut jwk = jsonwebtoken::jwk::Jwk::from_encoding_key(&encoding_key, algoritm).unwrap();
        jwk.common.key_id = Some(kid.to_string());
        let jwks = JwkSet { keys: vec![jwk] };

        (jws, jwks)
    }

    #[test]
    fn test_verify_against_keys_success() {
        let (jws, jwks) = make_jws(true);

        let payload = verify_against_keys::<serde_json::Value>(
            &jws,
            &jwks,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Algorithm::HS256,
        )
        .expect("verifying JWS should succeed");

        assert_eq!(
            payload
                .as_object()
                .and_then(|payload| payload.get("bsn"))
                .and_then(serde_json::Value::as_str),
            Some("999991772")
        );
    }

    #[test]
    fn test_verify_against_keys_error_missing_key_id() {
        let (jws, jwks) = make_jws(false);

        let error = verify_against_keys::<serde_json::Value>(
            &jws,
            &jwks,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Algorithm::HS256,
        )
        .expect_err("verifying JWS should fail");

        assert_matches!(error, OidcError::MissingKeyId);
    }

    #[test]
    fn test_verify_against_keys_error_key_not_found() {
        let (jws, mut jwks) = make_jws(true);

        jwks.keys.first_mut().unwrap().common.key_id = Some("wrong_kid".to_string());

        let error = verify_against_keys::<serde_json::Value>(
            &jws,
            &jwks,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Algorithm::HS256,
        )
        .expect_err("verifying JWS should fail");

        assert_matches!(error, OidcError::KeyNotFound);
    }

    #[test]
    fn test_verify_against_keys_error_wrong_aud() {
        let (jws, jwks) = make_jws(true);

        let error = verify_against_keys::<serde_json::Value>(&jws, &jwks, "wrong_aud", Algorithm::HS256)
            .expect_err("verifying JWS should fail");

        assert_matches!(error, OidcError::Jsonwebtoken(_));
    }

    #[test]
    fn test_verify_against_keys_error_wrong_alg() {
        let (jws, jwks) = make_jws(true);

        let error = verify_against_keys::<serde_json::Value>(&jws, &jwks, "wrong_aud", Algorithm::HS512)
            .expect_err("verifying JWS should fail");

        assert_matches!(error, OidcError::Jsonwebtoken(_));
    }

    const JWE_ENC: AescbcHmacJweEncryption = AescbcHmacJweEncryption::A128cbcHs256;
    const JWE_ALG: EcdhEsJweAlgorithm = ECDH_ES_A256KW;

    fn make_jwe(payload: &[u8]) -> (String, Jwk) {
        let key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();
        let jwk = key_pair.to_jwk_key_pair();

        let mut header = JweHeader::new();
        header.set_content_encryption(JWE_ENC.name());

        let encrypter = JWE_ALG.encrypter_from_jwk(&jwk).unwrap();
        let jwe = josekit::jwe::serialize_compact(payload, &header, &encrypter).unwrap();

        (jwe, jwk)
    }

    #[test]
    fn test_decrypt_jwe_success() {
        let payload = b"hello world";
        let (jwe, jwk) = make_jwe(payload);
        let decrypter = JWE_ALG.decrypter_from_jwk(&jwk).unwrap();

        let result = decrypt_jwe(&jwe, &decrypter, &JWE_ENC).unwrap();

        assert_eq!(result, payload);
    }

    #[test]
    fn test_decrypt_jwe_wrong_enc_algorithm() {
        let wrong_enc = AescbcHmacJweEncryption::A256cbcHs512;
        let (jwe, jwk) = make_jwe(b"payload");
        let decrypter = JWE_ALG.decrypter_from_jwk(&jwk).unwrap();

        let result = decrypt_jwe(&jwe, &decrypter, &wrong_enc);

        assert_matches!(result, Err(OidcError::UnexpectedEncAlgorithm));
    }

    #[test]
    fn test_decrypt_jwe_wrong_key() {
        let (jwe, _) = make_jwe(b"payload");

        let other_jwk = EcKeyPair::generate(EcCurve::P256).unwrap().to_jwk_key_pair();
        let decrypter = JWE_ALG.decrypter_from_jwk(&other_jwk).unwrap();

        let result = decrypt_jwe(&jwe, &decrypter, &JWE_ENC);

        assert_matches!(result, Err(OidcError::JweDecryption(_)));
    }
}
