use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;
use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::HttpJsonClient;
use jwt::nonce::Nonce;

use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::authorization::AuthorizationRequest;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PkceCodeChallenge;
use crate::authorization::ResponseType;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::wallet_issuance::AuthorizationSession;
use crate::wallet_issuance::WalletIssuanceError;
use crate::wallet_issuance::issuance_session::HttpIssuanceSession;
use crate::wallet_issuance::issuance_session::HttpVcMessageClient;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum OAuthError {
    #[error("error encoding authorization request to URL: {0}")]
    AuthRequestUrlEncoding(#[source] serde_urlencoded::ser::Error),

    #[error("error decoding authorization response from URL: {0}")]
    AuthResponseUrlDecoding(#[source] serde::de::value::Error),

    #[error("error decoding error response from URL: {0}")]
    ErrorResponseUrlDecoding(#[source] serde::de::value::Error),

    #[error("error requesting authorization code: {0:?}")]
    RedirectUriError(Box<ErrorResponse<AuthorizationErrorCode>>),

    #[error("invalid state token received in redirect URI")]
    #[category(critical)]
    StateTokenMismatch,

    #[error("no authorization code received in redirect URI")]
    #[category(critical)]
    NoAuthCode,

    #[error("invalid redirect URI received")]
    #[category(critical)]
    RedirectUriMismatch,

    #[error("config has no authorization endpoint")]
    #[category(critical)]
    NoAuthorizationEndpoint,

    #[error("user denied authentication")]
    #[category(expected)]
    Denied,
}

/// The state of an in-progress OAuth authorization code flow.
#[derive(Debug)]
pub struct HttpAuthorizationSession<P = S256PkcePair> {
    issuer_metadata: IssuerMetadata,
    oauth_metadata: AuthorizationServerMetadata,
    http_client: HttpJsonClient,

    pub auth_url: Url,
    client_id: String,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
}

impl<P: PkcePair> HttpAuthorizationSession<P> {
    /// Create a new authorization server session and compute the authorization URL.
    /// Returns an error if the provider has no authorization endpoint or the URL cannot be encoded.
    pub fn try_new(
        http_client: HttpJsonClient,
        issuer_metadata: IssuerMetadata,
        oauth_metadata: AuthorizationServerMetadata,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self, OAuthError> {
        let pkce_pair = P::generate();
        let state = BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16));
        let nonce = Nonce::new_random();

        let mut scopes = IndexSet::with_capacity(1);
        scopes.insert("openid".to_string());

        let params = AuthorizationRequest {
            response_type: ResponseType::Code.into(),
            client_id: client_id.clone(),
            redirect_uri: Some(redirect_uri.clone()),
            state: Some(state.clone()),
            authorization_details: None,
            request_uri: None,
            code_challenge: Some(PkceCodeChallenge::S256 {
                code_challenge: pkce_pair.code_challenge().to_string(),
            }),
            scope: Some(scopes), /* TODO (PVW-5572): remove. "openid" scope should be an implementation detail of the
                                  * pid_issuer. Currently necessary because nl-rdo-max requires it. */
            nonce: Some(nonce), // TODO (PVW-5572): remove. Is not part of openid4vci spec
            response_mode: None,
        };

        let mut auth_url = oauth_metadata
            .authorization_endpoint
            .clone()
            .ok_or(OAuthError::NoAuthorizationEndpoint)?;

        auth_url.set_query(Some(
            &serde_urlencoded::to_string(params).map_err(OAuthError::AuthRequestUrlEncoding)?,
        ));

        Ok(Self {
            issuer_metadata,
            oauth_metadata,
            http_client,
            auth_url,
            client_id,
            redirect_uri,
            pkce_pair,
            state,
        })
    }

    fn matches_received_redirect_uri(&self, received_redirect_uri: &Url) -> bool {
        received_redirect_uri.as_str().starts_with(self.redirect_uri.as_str())
    }

    fn authorization_code(&self, received_redirect_uri: &Url) -> Result<AuthorizationCode, OAuthError> {
        if !self.matches_received_redirect_uri(received_redirect_uri) {
            return Err(OAuthError::RedirectUriMismatch);
        }

        let auth_response = received_redirect_uri.query().ok_or(OAuthError::NoAuthCode)?;

        // First see if we received an error
        if received_redirect_uri.query_pairs().any(|(key, _)| key == "error") {
            let err_response: ErrorResponse<AuthorizationErrorCode> =
                serde_urlencoded::from_str(auth_response).map_err(OAuthError::ErrorResponseUrlDecoding)?;

            return if err_response.error == AuthorizationErrorCode::AccessDenied {
                Err(OAuthError::Denied)
            } else {
                Err(OAuthError::RedirectUriError(Box::new(err_response)))
            };
        }

        let auth_response: AuthorizationResponse =
            serde_urlencoded::from_str(auth_response).map_err(OAuthError::AuthResponseUrlDecoding)?;
        if auth_response.state.as_ref() != Some(&self.state) {
            return Err(OAuthError::StateTokenMismatch);
        }

        Ok(auth_response.code.into())
    }
}

impl AuthorizationSession for HttpAuthorizationSession {
    type Issuance = HttpIssuanceSession;

    fn auth_url(&self) -> &Url {
        &self.auth_url
    }

    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let authorization_code = self.authorization_code(received_redirect_uri)?;
        let message_client = HttpVcMessageClient::new(self.http_client);

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: authorization_code,
            },
            code_verifier: Some(self.pkce_pair.into_code_verifier()),
            client_id: Some(self.client_id),
            redirect_uri: Some(self.redirect_uri),
        };

        HttpIssuanceSession::create(
            message_client,
            self.issuer_metadata,
            self.oauth_metadata,
            token_request,
            trust_anchors,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use rstest::rstest;
    use serial_test::serial;
    use url::Url;

    use http_utils::reqwest::HttpJsonClient;
    use http_utils::reqwest::default_reqwest_client_builder;

    use crate::AuthorizationErrorCode;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::pkce::MockPkcePair;

    use super::HttpAuthorizationSession;
    use super::OAuthError;

    const ISSUER_URL: &str = "https://example.com";
    const CLIENT_ID: &str = "client-1";
    const REDIRECT_URI: &str = "redirect://here";
    const CSRF_TOKEN: &str = "csrf_token";
    const CODE: &str = "code";

    pub fn url_with_query_pairs(mut url: Url, query_pairs: &[(&str, &str)]) -> Url {
        if !query_pairs.is_empty() {
            let mut query = url.query_pairs_mut();
            query_pairs.iter().for_each(|(name, value)| {
                query.append_pair(name, value);
            });
        }
        url
    }

    fn try_new_mock_session() -> HttpAuthorizationSession<MockPkcePair> {
        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair.expect_code_challenge().return_const("challenge".to_string());
            pkce_pair
        });
        HttpAuthorizationSession::<MockPkcePair>::try_new(
            HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap(),
            IssuerMetadata::new_mock(ISSUER_URL.parse().unwrap(), "test"),
            AuthorizationServerMetadata::new_mock(ISSUER_URL.parse().unwrap()),
            CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
        )
        .unwrap()
    }

    fn create_session() -> HttpAuthorizationSession<MockPkcePair> {
        let mut pkce_pair = MockPkcePair::new();
        pkce_pair.expect_code_challenge().return_const("challenge".to_string());
        HttpAuthorizationSession {
            issuer_metadata: IssuerMetadata::new_mock(ISSUER_URL.parse().unwrap(), "test"),
            oauth_metadata: AuthorizationServerMetadata::new_mock(ISSUER_URL.parse().unwrap()),
            http_client: HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap(),
            auth_url: ISSUER_URL.parse().unwrap(),
            client_id: CLIENT_ID.to_string(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            pkce_pair,
            state: CSRF_TOKEN.to_string(),
        }
    }

    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_start_and_into_token_request() {
        let session = try_new_mock_session();

        let state = session.state.clone();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let received = url_with_query_pairs(redirect_uri, &[("code", CODE), ("state", &state)]);

        let code = session.authorization_code(&received).unwrap();

        assert_eq!(code.as_ref(), CODE);
    }

    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_user_cancels() {
        let session = try_new_mock_session();

        let state = session.state.clone();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let received = url_with_query_pairs(redirect_uri, &[("error", "access_denied"), ("state", &state)]);

        let error = session.authorization_code(&received).unwrap_err();

        assert_matches!(error, OAuthError::Denied);
    }

    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_auth_url() {
        let session = try_new_mock_session();

        let params: std::collections::HashMap<_, _> = session.auth_url.query_pairs().collect();

        assert_eq!(params.get("response_type").map(|v| v.as_ref()), Some("code"));
        assert_eq!(params.get("client_id").map(|v| v.as_ref()), Some(CLIENT_ID));
        assert_eq!(params.get("redirect_uri").map(|v| v.as_ref()), Some(REDIRECT_URI));
        assert_eq!(params.get("code_challenge_method").map(|v| v.as_ref()), Some("S256"));
        assert_eq!(params.get("code_challenge").map(|v| v.as_ref()), Some("challenge"));
        assert_eq!(params.get("scope").map(|v| v.as_ref()), Some("openid"));
        assert!(params.contains_key("state"));
        assert!(params.contains_key("nonce"));
    }

    #[test]
    fn test_matches_received_redirect_uri() {
        let session = create_session();

        assert!(session.matches_received_redirect_uri(&Url::parse(REDIRECT_URI).unwrap()));
        assert!(session.matches_received_redirect_uri(&url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("foo", "bar"), ("bleh", "blah")]
        )));

        assert!(!session.matches_received_redirect_uri(&Url::parse("https://example.com").unwrap()));
        assert!(!session.matches_received_redirect_uri(&Url::parse("scheme://host/path").unwrap()));
    }

    #[test]
    fn test_redirect_uri_mismatch() {
        let session = create_session();
        let uri = Url::parse("http://not-the-redirect-uri.com").unwrap();

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::RedirectUriMismatch);
    }

    #[test]
    fn test_error() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[
                ("error", "invalid_request"),
                ("error_description", "this is the error description"),
            ],
        );

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(
            error,
            OAuthError::RedirectUriError(response)
                if matches!(response.error, AuthorizationErrorCode::InvalidRequest)
                && response.error_description == Some("this is the error description".to_string())
        );
    }

    #[test]
    fn test_state_mismatch() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("code", CODE), ("state", "foobar")],
        );

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::StateTokenMismatch);
    }

    #[test]
    fn test_missing_code() {
        let session = create_session();
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[("state", CSRF_TOKEN)]);

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::AuthResponseUrlDecoding(err) if err.to_string() == "missing field `code`");
    }

    #[test]
    fn test_get_authorization_url() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("state", CSRF_TOKEN), ("code", "123")],
        );

        assert_eq!(session.authorization_code(&uri).unwrap().as_ref(), "123");
    }

    #[test]
    fn test_into_token_request() {
        let mut session = create_session();
        session
            .pkce_pair
            .expect_into_code_verifier()
            .return_const("code_verifier".to_string());

        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("state", CSRF_TOKEN), ("code", "123")],
        );

        let code = session.authorization_code(&uri).unwrap();

        assert_eq!(code.as_ref(), "123");
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
