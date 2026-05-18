use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use crypto::trust_anchor::BorrowingTrustAnchor;
use error_category::ErrorCategory;
use http_utils::reqwest::HttpJsonClient;
use serde::Deserialize;
use url::Url;

use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
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

#[derive(Deserialize, Debug, Clone, PartialEq, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ParErrorCode {
    InvalidClient,
    ServerError,

    // Catch-all variant, in case the issuer sends an error code that the holder is not aware of.
    // Note that this is never to be used by the issuer, as this will lead to a panic.
    #[strum(default)]
    Other(String),
}

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

    #[error("config has no pushed authorization request endpoint")]
    #[category(critical)]
    NoPushedAuthorizationEndpoint,

    #[error("pushed authorization request rejected: {0:?}")]
    #[category(expected)]
    PushedAuthorizationRequest(Box<ErrorResponse<ParErrorCode>>),

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

    auth_url: Url,
    client_id: String,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
}

impl<P: PkcePair> HttpAuthorizationSession<P> {
    /// POST the authorization parameters to the PAR endpoint, then build the authorization URL
    /// using the returned `request_uri`. Returns an error if the provider has no PAR endpoint,
    /// the PAR request is rejected, or the URL cannot be constructed.
    pub(super) async fn create(
        http_client: HttpJsonClient,
        issuer_metadata: IssuerMetadata,
        oauth_metadata: AuthorizationServerMetadata,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self, WalletIssuanceError> {
        let pkce_pair = P::generate();
        let state = BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16));

        let par_request =
            VciAuthorizationRequest::for_par(client_id.clone(), redirect_uri.clone(), state.clone(), &pkce_pair);

        let par_endpoint = oauth_metadata
            .pushed_authorization_request_endpoint
            .as_ref()
            .ok_or(OAuthError::NoPushedAuthorizationEndpoint)?;

        let response = http_client
            .post(par_endpoint.as_str(), |builder| builder.form(&par_request))
            .await
            .map_err(WalletIssuanceError::Network)?;

        let par_response = if response.status().is_success() {
            response
                .json::<PushedAuthorizationResponse>()
                .await
                .map_err(WalletIssuanceError::Network)?
        } else {
            let error = response
                .json::<ErrorResponse<ParErrorCode>>()
                .await
                .map_err(WalletIssuanceError::Network)?;
            return Err(OAuthError::PushedAuthorizationRequest(Box::new(error)).into());
        };

        let mut auth_url = oauth_metadata
            .authorization_endpoint
            .clone()
            .ok_or(OAuthError::NoAuthorizationEndpoint)?;

        auth_url
            .query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("request_uri", &par_response.request_uri);

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

    fn state(&self) -> &str {
        &self.state
    }

    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        trust_anchors: &[BorrowingTrustAnchor],
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
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use http::header;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpJsonClient;
    use http_utils::reqwest::default_reqwest_client_builder;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use rstest::rstest;
    use serde_json::json;
    use serial_test::serial;
    use url::Url;

    use super::HttpAuthorizationSession;
    use super::OAuthError;
    use crate::AuthorizationErrorCode;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::pkce::MockPkcePair;

    const ISSUER_URL: &str = "https://example.com";
    const CLIENT_ID: &str = "client-1";
    const REDIRECT_URI: &str = "redirect://here";
    const CSRF_TOKEN: &str = "csrf_token";
    const CODE: &str = "code";
    const PAR_REQUEST_URI: &str = "urn:ietf:params:oauth:request_uri:test-12345";

    pub fn url_with_query_pairs(mut url: Url, query_pairs: &[(&str, &str)]) -> Url {
        if !query_pairs.is_empty() {
            let mut query = url.query_pairs_mut();
            query_pairs.iter().for_each(|(name, value)| {
                query.append_pair(name, value);
            });
        }
        url
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
    async fn test_authorization_code() {
        let session = create_session();

        let state = session.state.clone();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let received = url_with_query_pairs(redirect_uri, &[("code", CODE), ("state", &state)]);

        let code = session.authorization_code(&received).unwrap();

        assert_eq!(code.as_ref(), CODE);
    }

    #[tokio::test]
    async fn test_user_cancels() {
        let session = create_session();

        let state = session.state.clone();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let received = url_with_query_pairs(redirect_uri, &[("error", "access_denied"), ("state", &state)]);

        let error = session.authorization_code(&received).unwrap_err();

        assert_matches!(error, OAuthError::Denied);
    }

    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_auth_url() {
        let server = MockServer::start_async().await;

        server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/par");
                then.status(201)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(json!({
                        "request_uri": PAR_REQUEST_URI,
                        "expires_in": 60,
                    }));
            })
            .await;

        let issuer_identifier = server.base_url().parse().unwrap();
        let mut oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_identifier);
        oauth_metadata.pushed_authorization_request_endpoint = Some(server.url("/issuance/par").parse().unwrap());
        oauth_metadata.authorization_endpoint = Some(server.url("/issuance/authorize").parse().unwrap());

        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair.expect_code_challenge().return_const("challenge".to_string());
            pkce_pair
        });

        let session = HttpAuthorizationSession::<MockPkcePair>::create(
            HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            IssuerMetadata::new_mock(server.base_url().parse().unwrap(), "test"),
            oauth_metadata,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
        )
        .await
        .unwrap();

        let params: HashMap<_, _> = session.auth_url.query_pairs().collect();

        // Auth URL after PAR carries only client_id + request_uri (RFC 9126 §4)
        assert_eq!(params.get("client_id").map(|v| v.as_ref()), Some(MOCK_WALLET_CLIENT_ID));
        assert_eq!(params.get("request_uri").map(|v| v.as_ref()), Some(PAR_REQUEST_URI));
        assert!(!params.contains_key("code_challenge"));
        assert!(!params.contains_key("state"));
        assert!(!params.contains_key("redirect_uri"));
    }

    #[tokio::test]
    async fn test_matches_received_redirect_uri() {
        let session = create_session();

        assert!(session.matches_received_redirect_uri(&Url::parse(REDIRECT_URI).unwrap()));
        assert!(session.matches_received_redirect_uri(&url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("foo", "bar"), ("bleh", "blah")]
        )));

        assert!(!session.matches_received_redirect_uri(&Url::parse("https://example.com").unwrap()));
        assert!(!session.matches_received_redirect_uri(&Url::parse("scheme://host/path").unwrap()));
    }

    #[tokio::test]
    async fn test_redirect_uri_mismatch() {
        let session = create_session();
        let uri = Url::parse("http://not-the-redirect-uri.com").unwrap();

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::RedirectUriMismatch);
    }

    #[tokio::test]
    async fn test_error() {
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

    #[tokio::test]
    async fn test_state_mismatch() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("code", CODE), ("state", "foobar")],
        );

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::StateTokenMismatch);
    }

    #[tokio::test]
    async fn test_missing_code() {
        let session = create_session();
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[("state", CSRF_TOKEN)]);

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::AuthResponseUrlDecoding(err) if err.to_string() == "missing field `code`");
    }

    #[tokio::test]
    async fn test_get_authorization_url() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("state", CSRF_TOKEN), ("code", "123")],
        );

        assert_eq!(session.authorization_code(&uri).unwrap().as_ref(), "123");
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
