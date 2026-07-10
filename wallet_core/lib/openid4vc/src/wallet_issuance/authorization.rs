use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use crypto::trust_anchor::TrustAnchors;
use error_category::ErrorCategory;
use http_utils::reqwest::HttpJsonClient;
use itertools::Either;
use itertools::Itertools;
use jwt::wia::WIA_HEADER_NAME;
use jwt::wia::WIA_POP_HEADER_NAME;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;
use wscd::wscd::WiaClient;

use super::AuthorizationSession;
use super::WalletIssuanceError;
use super::authorization_endpoints::AuthorizationEndpoints;
use super::issuance_session::HttpIssuanceSession;
use super::issuance_session::HttpVcMessageClient;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::errors::AuthorizationErrorCode;
use crate::errors::ParErrorCode;
use crate::errors::RemoteErrorCode;
use crate::errors::RemoteErrorResponse;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum OAuthError {
    #[error("error encoding authorization request to URL: {0}")]
    AuthRequestUrlEncoding(#[source] serde_qs::Error),

    #[error("error decoding authorization response from URL: {0}")]
    AuthResponseUrlDecoding(#[source] serde_qs::Error),

    #[error("error decoding error response from URL: {0}")]
    ErrorResponseUrlDecoding(#[source] serde_qs::Error),

    #[error("error requesting authorization code: {0:?}")]
    RedirectUriError(Box<RemoteErrorResponse<AuthorizationErrorCode>>),

    #[error("invalid state token received in redirect URI")]
    #[category(critical)]
    StateTokenMismatch,

    #[error("no authorization code received in redirect URI")]
    #[category(critical)]
    NoAuthCode,

    #[error("invalid redirect URI received")]
    #[category(critical)]
    RedirectUriMismatch,

    #[error("pushed authorization request rejected: {0:?}")]
    #[category(expected)]
    PushedAuthorizationRequest(Box<RemoteErrorResponse<ParErrorCode>>),

    #[error("user denied authentication")]
    #[category(expected)]
    Denied,
}

/// The state of an in-progress OAuth authorization code flow.
#[derive(Debug)]
pub struct HttpAuthorizationSession<P = S256PkcePair> {
    credential_configurations: VecNonEmpty<(CredentialConfigurationId, CredentialConfiguration)>,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    token_endpoint: Url,
    authorization_server: IssuerIdentifier,
    http_client: HttpJsonClient,

    auth_url: Url,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HttpAuthorizationSessionData {
    credential_configurations: VecNonEmpty<(CredentialConfigurationId, CredentialConfiguration)>,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    token_endpoint: Url,
    authorization_server: IssuerIdentifier,
    auth_url: Url,
    redirect_uri: Url,
    code_verifier: String,
    state: String,
}

impl<P: PkcePair> HttpAuthorizationSession<P> {
    /// POST the authorization parameters to the PAR endpoint, then build the authorization URL
    /// using the returned `request_uri`. Returns an error if the provider has no PAR endpoint,
    /// the PAR request is rejected, or the URL cannot be constructed.
    #[expect(clippy::too_many_arguments, reason = "internal constructor")]
    pub(super) async fn create(
        http_client: HttpJsonClient,
        credential_configurations: VecNonEmpty<(CredentialConfigurationId, CredentialConfiguration)>,
        credential_issuer: IssuerIdentifier,
        issuer_endpoints: IssuerEndpoints,
        auth_endpoints: AuthorizationEndpoints,
        client_id: String,
        redirect_uri: Url,
        issuer_state: Option<String>,
        wia_client: &impl WiaClient,
        authorization_server: IssuerIdentifier,
    ) -> Result<Self, WalletIssuanceError> {
        // Include the `scope` values for each Credential Configuration with a supported credential format that was
        // present in the Credential Offer and return an error if any of them do not include a `scope`. Note that we
        // include the `scope` values in favour of using the `authorization_details` field. According to HAIP:
        //
        // "For Grant Type authorization_code, the Issuer MUST include a scope value in order to allow the Wallet to
        // identify the desired Credential Type. The Wallet MUST use that value in the scope Authorization parameter."
        //
        // Source: <https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-4.2>
        let (scope, no_scope_config_ids): (_, Vec<_>) = credential_configurations
            .iter()
            .filter(|(_id, config)| config.format.is_supported())
            .partition_map(|(id, config)| match &config.scope {
                Some(scope) => Either::Left(scope.clone()),
                None => Either::Right(id.clone()),
            });

        if !no_scope_config_ids.is_empty() {
            return Err(WalletIssuanceError::IssuerMetadataNoScope(no_scope_config_ids));
        }

        let pkce_pair = P::generate();
        let state = BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16));

        let par_request = VciAuthorizationRequest::for_auth_code(
            client_id.clone(),
            redirect_uri.clone(),
            state.clone(),
            issuer_state,
            scope,
            &pkce_pair,
        );

        let wia = wia_client
            .issue_wia(authorization_server.to_string(), None)
            .await
            .map_err(|e| WalletIssuanceError::WiaIssuance(e.into()))?;

        let response = http_client
            .post(auth_endpoints.par_endpoint, |builder| {
                builder
                    .form(&par_request)
                    .header(WIA_HEADER_NAME, wia.wia().serialization())
                    .header(WIA_POP_HEADER_NAME, wia.wia_pop().serialization())
            })
            .await
            .map_err(WalletIssuanceError::ParHttp)?;

        let par_response = if response.status().is_success() {
            response
                .json::<PushedAuthorizationResponse>()
                .await
                .map_err(WalletIssuanceError::ParHttp)?
        } else {
            let error = response
                .json::<RemoteErrorResponse<ParErrorCode>>()
                .await
                .map_err(WalletIssuanceError::ParHttp)?;
            return Err(OAuthError::PushedAuthorizationRequest(Box::new(error)).into());
        };

        let mut auth_url = auth_endpoints.authorization_endpoint;
        auth_url
            .query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("request_uri", &par_response.request_uri);

        Ok(Self {
            credential_configurations,
            credential_issuer,
            issuer_endpoints,
            token_endpoint: auth_endpoints.token_endpoint,
            authorization_server,
            http_client,
            auth_url,
            redirect_uri,
            pkce_pair,
            state,
        })
    }

    #[cfg(any(test, feature = "test"))]
    pub fn state(&self) -> &str {
        &self.state
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
            let err_response: RemoteErrorResponse<AuthorizationErrorCode> =
                serde_qs::from_str(auth_response).map_err(OAuthError::ErrorResponseUrlDecoding)?;

            return if err_response.error == RemoteErrorCode::Known(AuthorizationErrorCode::AccessDenied) {
                Err(OAuthError::Denied)
            } else {
                Err(OAuthError::RedirectUriError(Box::new(err_response)))
            };
        }

        let auth_response: AuthorizationResponse =
            serde_qs::from_str(auth_response).map_err(OAuthError::AuthResponseUrlDecoding)?;
        if auth_response.state.as_ref() != Some(&self.state) {
            return Err(OAuthError::StateTokenMismatch);
        }

        Ok(auth_response.code.into())
    }
}

impl HttpAuthorizationSession {
    pub fn restore(http_client: HttpJsonClient, data: HttpAuthorizationSessionData) -> Self {
        Self {
            credential_configurations: data.credential_configurations,
            credential_issuer: data.credential_issuer,
            issuer_endpoints: data.issuer_endpoints,
            token_endpoint: data.token_endpoint,
            authorization_server: data.authorization_server,
            http_client,
            auth_url: data.auth_url,
            redirect_uri: data.redirect_uri,
            pkce_pair: S256PkcePair::from_code_verifier(data.code_verifier),
            state: data.state,
        }
    }
}

impl AuthorizationSession for HttpAuthorizationSession {
    type Issuance = HttpIssuanceSession;
    type Persisted = HttpAuthorizationSessionData;

    fn auth_url(&self) -> &Url {
        &self.auth_url
    }

    fn state(&self) -> &str {
        &self.state
    }

    fn persist(&self) -> Self::Persisted {
        HttpAuthorizationSessionData {
            credential_configurations: self.credential_configurations.clone(),
            credential_issuer: self.credential_issuer.clone(),
            issuer_endpoints: self.issuer_endpoints.clone(),
            token_endpoint: self.token_endpoint.clone(),
            authorization_server: self.authorization_server.clone(),
            auth_url: self.auth_url.clone(),
            redirect_uri: self.redirect_uri.clone(),
            code_verifier: self.pkce_pair.code_verifier().to_string(),
            state: self.state.clone(),
        }
    }

    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        wia_client: &impl WiaClient,
        trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let authorization_code = self.authorization_code(received_redirect_uri)?;
        let message_client = HttpVcMessageClient::new(self.http_client);

        // Create the Token Request to be sent to the issuer with the minimal amount of information required. This does
        // not include either a `scope` or `authorization_details` field, as we have no need to further restrict the
        // credentials requested at this point.
        let token_request = TokenRequest::new_authorization_code(
            authorization_code,
            self.redirect_uri,
            self.pkce_pair.into_code_verifier(),
        );

        HttpIssuanceSession::create(
            message_client,
            self.credential_configurations,
            self.credential_issuer,
            self.issuer_endpoints,
            &self.token_endpoint,
            token_request,
            wia_client,
            &self.authorization_server,
            trust_anchors,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;

    use attestation_types::credential_format::Format;
    use http::header;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpJsonClient;
    use http_utils::reqwest::default_reqwest_client_builder;
    use http_utils::urls::BaseUrl;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use itertools::Itertools;
    use rstest::rstest;
    use serde_json::json;
    use serial_test::serial;
    use url::Url;
    use wscd::mock_remote::MockWiaClient;

    use super::super::AuthorizationSession;
    use super::super::WalletIssuanceError;
    use super::HttpAuthorizationSession;
    use super::HttpAuthorizationSessionData;
    use super::OAuthError;
    use crate::errors::AuthorizationErrorCode;
    use crate::errors::RemoteErrorCode;
    use crate::issuable_document::CredentialKind;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::pkce::MockPkcePair;
    use crate::pkce::S256PkcePair;
    use crate::wallet_issuance::authorization_endpoints::AuthorizationEndpoints;

    const ISSUER_URL: &str = "https://example.com";
    const TOKEN_ENDPOINT: &str = "/issuance/token";
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
        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            ISSUER_URL.parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let mut pkce_pair = MockPkcePair::new();
        pkce_pair.expect_code_challenge().return_const("challenge".to_string());
        HttpAuthorizationSession {
            credential_configurations: issuer_metadata
                .credential_configurations_supported
                .into_iter()
                .collect_vec()
                .try_into()
                .unwrap(),
            credential_issuer: issuer_metadata.credential_issuer,
            issuer_endpoints: issuer_metadata.endpoints,
            token_endpoint: ISSUER_URL.parse::<BaseUrl>().unwrap().join(TOKEN_ENDPOINT),
            authorization_server: ISSUER_URL.parse().unwrap(),
            http_client: HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap(),
            auth_url: ISSUER_URL.parse().unwrap(),
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

    fn authorization_endpoints(base_url: &BaseUrl) -> AuthorizationEndpoints {
        let authorization_endpoint = base_url.join("/authorize");
        let pushed_authorization_request_endpoint = base_url.join("/issuance/par");
        let token_endpoint = base_url.join("/issuance/token");

        AuthorizationEndpoints {
            authorization_endpoint,
            par_endpoint: pushed_authorization_request_endpoint,
            token_endpoint,
        }
    }

    #[rstest]
    #[case::without_issuer_state(None)]
    #[case::with_issuer_state(Some("foobar"))]
    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_auth_url(#[case] issuer_state: Option<&str>) {
        let server = MockServer::start_async().await;

        server
            .mock_async(|when, then| {
                let when = when
                    .method(POST)
                    .path("/issuance/par")
                    .form_urlencoded_tuple("client_id", MOCK_WALLET_CLIENT_ID)
                    .form_urlencoded_tuple("redirect_uri", REDIRECT_URI);

                match issuer_state {
                    None => {
                        when.form_urlencoded_tuple_missing("issuer_state");
                    }
                    Some(issuer_state) => {
                        when.form_urlencoded_tuple("issuer_state", issuer_state);
                    }
                }

                then.status(201)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(json!({
                        "request_uri": PAR_REQUEST_URI,
                        "expires_in": 60,
                    }));
            })
            .await;

        let auth_endpoints = authorization_endpoints(&server.base_url().parse().unwrap());

        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair.expect_code_challenge().return_const("challenge".to_string());
            pkce_pair
        });

        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            server.base_url().parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let session = HttpAuthorizationSession::<MockPkcePair>::create(
            HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            issuer_metadata
                .credential_configurations_supported
                .into_iter()
                .collect_vec()
                .try_into()
                .unwrap(),
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            auth_endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            issuer_state.map(str::to_string),
            &MockWiaClient::new(),
            issuer_metadata.credential_issuer,
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
        assert!(!params.contains_key("issuer_state"));
        assert!(!params.contains_key("scope"));
    }

    #[tokio::test]
    async fn test_http_authorization_session_create_issuer_metadata_no_scope_error() {
        let issuer_identifier = "https://example.com".parse::<IssuerIdentifier>().unwrap();
        let config_id = CredentialConfigurationId::from("config_id".to_string());

        let auth_endpoints = authorization_endpoints(issuer_identifier.as_base_url());

        let mut issuer_metadata = IssuerMetadata::new_mock(
            issuer_identifier,
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );

        for config in issuer_metadata.credential_configurations_supported.values_mut() {
            config.scope = None;
        }

        let credential_configurations = issuer_metadata
            .credential_configurations_supported
            .clone()
            .into_iter()
            .collect_vec()
            .try_into()
            .unwrap();

        let error = HttpAuthorizationSession::<MockPkcePair>::create(
            HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            credential_configurations,
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            auth_endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            None,
            &MockWiaClient::new(),
            issuer_metadata.credential_issuer,
        )
        .await
        .expect_err("creating authorization session should fail");

        assert_matches!(
            error,
            WalletIssuanceError::IssuerMetadataNoScope(config_ids) if config_ids == vec![config_id]
        );
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
                if matches!(response.error, RemoteErrorCode::Known(AuthorizationErrorCode::InvalidRequest))
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

    #[tokio::test]
    async fn test_persist_and_restore() {
        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            ISSUER_URL.parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let persisted = HttpAuthorizationSessionData {
            credential_configurations: issuer_metadata
                .credential_configurations_supported
                .into_iter()
                .collect_vec()
                .try_into()
                .unwrap(),
            credential_issuer: issuer_metadata.credential_issuer,
            issuer_endpoints: issuer_metadata.endpoints,
            token_endpoint: ISSUER_URL.parse::<BaseUrl>().unwrap().join(TOKEN_ENDPOINT),
            auth_url: ISSUER_URL.parse().unwrap(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            code_verifier: "verifier".to_string(),
            state: CSRF_TOKEN.to_string(),
            authorization_server: ISSUER_URL.parse().unwrap(),
        };

        let session = HttpAuthorizationSession {
            credential_configurations: persisted.credential_configurations,
            credential_issuer: persisted.credential_issuer,
            issuer_endpoints: persisted.issuer_endpoints,
            token_endpoint: persisted.token_endpoint,
            http_client: HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap(),
            auth_url: persisted.auth_url.clone(),
            redirect_uri: persisted.redirect_uri.clone(),
            pkce_pair: S256PkcePair::from_code_verifier(persisted.code_verifier.clone()),
            state: persisted.state.clone(),
            authorization_server: ISSUER_URL.parse().unwrap(),
        };

        let restored = HttpAuthorizationSession::restore(
            HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap(),
            session.persist(),
        );
        let restored_persisted = restored.persist();

        assert_eq!(restored_persisted.auth_url, persisted.auth_url);
        assert_eq!(restored_persisted.redirect_uri, persisted.redirect_uri);
        assert_eq!(restored_persisted.code_verifier, persisted.code_verifier);
        assert_eq!(restored_persisted.state, persisted.state);
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
