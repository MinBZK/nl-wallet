use http_utils::reqwest::HttpClient;
use jwt::JwtTyp;
use jwt::jwk::JwkSet;
use oauth::authorization::OidcAuthorizationRequest;
use oauth::errors::TokenErrorCode;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::jwks::HttpJwksClient;
use oauth::jwks::JwksError;
use oauth::metadata::oauth_metadata::OidcProviderMetadata;
use oauth::metadata::oauth_metadata::WellKnownOpenIdConfiguration;
use oauth::metadata::well_known::WellKnownError;
use oauth::token::AccessToken;
use oauth::token::TokenEndpointError;
use oauth::token::TokenRequest;
use oauth::token::TokenResponse;
use oauth::token::request_token;
use oauth::userinfo::UserInfoError;
use oauth::userinfo::request_userinfo;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::skip_serializing_none;
use url::Url;

/// Token response for the admin portal OIDC flow. Extends the standard OAuth 2.0 [`oauth::token::TokenResponse`]
/// with the `id_token` field that OpenID Connect includes in authorization code grant responses.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminTokenResponse {
    #[serde(flatten)]
    pub token_response: TokenResponse,
    pub id_token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OidcHttpClientError {
    #[error("error fetching well-known OIDC metadata: {0}")]
    WellKnown(#[source] WellKnownError),

    #[error("upstream OIDC metadata does not define the authorization endpoint")]
    AuthorizationEndpointMissing,

    #[error("upstream OIDC metadata does not define the jwks URI")]
    JwksUriMissing,

    #[error("upstream OIDC metadata does not support the code response type")]
    AuthorizationCodeNotSupported,

    #[error("upstream OIDC metadata does not support the S256 PKCE code challenge method")]
    S256NotSupported,

    #[error("error fetching OIDC provider JWKS: {0}")]
    Jwks(#[source] JwksError),

    #[error("error exchanging authorization code for tokens: {0}")]
    TokenEndpoint(#[source] TokenEndpointError<TokenErrorCode>),

    #[error("error requesting userinfo: {0}")]
    UserInfo(#[source] UserInfoError),

    #[error("error encoding request as query string: {0}")]
    Encode(#[source] serde_qs::Error),
}

pub struct OidcHttpClient {
    http_client: HttpClient,
    expected_issuer: IssuerIdentifier,
}

impl OidcHttpClient {
    pub fn new(http_client: HttpClient, expected_issuer: IssuerIdentifier) -> Self {
        Self {
            http_client,
            expected_issuer,
        }
    }

    pub async fn fetch_metadata(&self) -> Result<OidcProviderMetadata, OidcHttpClientError> {
        let metadata = OidcProviderMetadata::fetch_openid_configuration(&self.http_client, &self.expected_issuer)
            .await
            .map_err(OidcHttpClientError::WellKnown)?;
        if metadata.oauth_metadata.authorization_endpoint.is_none() {
            return Err(OidcHttpClientError::AuthorizationEndpointMissing);
        }

        if metadata.oauth_metadata.jwks_uri.is_none() {
            return Err(OidcHttpClientError::JwksUriMissing);
        }

        if !metadata.oauth_metadata.response_types_supported.contains("code") {
            return Err(OidcHttpClientError::AuthorizationCodeNotSupported);
        }

        if !metadata
            .oauth_metadata
            .code_challenge_methods_supported
            .as_ref()
            .is_some_and(|methods| methods.contains("S256"))
        {
            return Err(OidcHttpClientError::S256NotSupported);
        }

        Ok(metadata)
    }

    pub async fn fetch_jwks(&self, metadata: &OidcProviderMetadata) -> Result<JwkSet, OidcHttpClientError> {
        let jwks_uri = metadata
            .oauth_metadata
            .jwks_uri
            .clone()
            .ok_or(OidcHttpClientError::JwksUriMissing)?;

        HttpJwksClient::new(self.http_client.clone())
            .jwks(jwks_uri)
            .await
            .map_err(OidcHttpClientError::Jwks)
    }

    /// Build the authorization URL for the OIDC authorization code flow, based on the provider's discovery
    /// metadata. The caller is responsible for persisting `state`, `nonce` and the PKCE pair's code verifier before
    /// redirecting the user agent.
    pub fn authorization_url(
        &self,
        oidc_request: &OidcAuthorizationRequest,
        metadata: &OidcProviderMetadata,
    ) -> Result<Url, OidcHttpClientError> {
        let mut auth_url = metadata
            .oauth_metadata
            .authorization_endpoint
            .clone()
            .ok_or(OidcHttpClientError::AuthorizationEndpointMissing)?;

        let query_string = serde_qs::to_string(&oidc_request).map_err(OidcHttpClientError::Encode)?;
        auth_url.set_query(Some(&query_string));

        Ok(auth_url)
    }

    /// Exchange an authorization code for tokens at the provider's token endpoint.
    pub async fn exchange_code<T>(
        &self,
        token_request: TokenRequest<T>,
        metadata: &OidcProviderMetadata,
    ) -> Result<AdminTokenResponse, OidcHttpClientError>
    where
        T: Serialize,
    {
        let response: AdminTokenResponse = request_token(&self.http_client, metadata, token_request)
            .await
            .map_err(OidcHttpClientError::TokenEndpoint)?;
        Ok(response)
    }

    /// Request the UserInfo claims for a previously obtained `access_token`.
    pub async fn request_userinfo<C>(
        &self,
        client_id: &str,
        access_token: &AccessToken,
        metadata: &OidcProviderMetadata,
    ) -> Result<C, OidcHttpClientError>
    where
        C: DeserializeOwned + JwtTyp,
    {
        let claims = request_userinfo(&self.http_client, metadata, access_token, client_id, None)
            .await
            .map_err(OidcHttpClientError::UserInfo)?;
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use indexmap::IndexSet;
    use jwt::error::JwtVerifyError;
    use jwt::jwk::JwkSet;
    use jwt::nonce::Nonce;
    use oauth::authorization::AuthorizationCodeRequest;
    use oauth::metadata::oauth_metadata::AuthorizationServerMetadata;
    use oauth::metadata::well_known::WellKnownMetadata;
    use oauth::pkce::S256PkcePair;
    use oauth::scope::Scope;
    use oauth::token::AuthorizationCode;
    use oauth::token::AuthorizationCodeGrantType;
    use serde_json::json;
    use url::Url;

    use super::*;

    fn metadata(issuer: &IssuerIdentifier) -> OidcProviderMetadata {
        let issuer_url = issuer.as_base_url();
        OidcProviderMetadata {
            oauth_metadata: AuthorizationServerMetadata {
                authorization_endpoint: Some(issuer_url.join("/authorize")),
                jwks_uri: Some(issuer_url.join("/jwks")),
                response_types_supported: IndexSet::from_iter(["code".to_string()]),
                code_challenge_methods_supported: Some(IndexSet::from_iter(["S256".to_string()])),
                ..AuthorizationServerMetadata::new(issuer.clone(), issuer_url.join("/token"))
            },
            oidc_metadata_extension: Default::default(),
        }
    }

    async fn client(modify: impl FnOnce(&mut OidcProviderMetadata)) -> (MockServer, OidcHttpClient) {
        let server = MockServer::start_async().await;
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let mut metadata = metadata(&issuer);
        modify(&mut metadata);
        server
            .mock_async(|when, then| {
                when.method(GET).path("/.well-known/openid-configuration");
                then.status(200).json_body(json!(metadata));
            })
            .await;
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        (server, OidcHttpClient::new(http_client, issuer))
    }

    async fn jwks_client(status: u16) -> (MockServer, OidcHttpClient, OidcProviderMetadata) {
        let server = MockServer::start_async().await;
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let metadata = metadata(&issuer);
        server
            .mock_async(move |when, then| {
                when.method(GET).path("/jwks");
                then.status(status).json_body(json!(JwkSet { keys: vec![] }));
            })
            .await;
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        (server, OidcHttpClient::new(http_client, issuer), metadata)
    }

    #[tokio::test]
    async fn fetch_metadata_returns_valid_metadata() {
        let (_server, client) = client(|_| {}).await;
        let metadata = client.fetch_metadata().await.expect("should succeed");
        assert_eq!(metadata.issuer_identifier(), &client.expected_issuer);
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_authorization_endpoint() {
        let (_server, client) = client(|metadata| metadata.oauth_metadata.authorization_endpoint = None).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::AuthorizationEndpointMissing));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_issuer_mismatch() {
        let (_server, client) =
            client(|metadata| metadata.oauth_metadata.issuer = "https://other.example.com".parse().unwrap()).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(
            error,
            OidcHttpClientError::WellKnown(WellKnownError::IssuerIdentifierMismatch { .. })
        ));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_jwks_uri() {
        let (_server, client) = client(|metadata| metadata.oauth_metadata.jwks_uri = None).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::JwksUriMissing));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_code_response_type() {
        let (_server, client) =
            client(|metadata| metadata.oauth_metadata.response_types_supported = IndexSet::new()).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::AuthorizationCodeNotSupported));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_s256() {
        let (_server, client) =
            client(|metadata| metadata.oauth_metadata.code_challenge_methods_supported = None).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::S256NotSupported));
    }

    #[tokio::test]
    async fn fetch_jwks_returns_provider_keys() {
        let (_server, client, metadata) = jwks_client(200).await;
        let jwks = client.fetch_jwks(&metadata).await.expect("should succeed");
        assert_eq!(jwks, JwkSet { keys: vec![] });
    }

    #[tokio::test]
    async fn fetch_jwks_rejects_missing_jwks_uri() {
        let (_server, client, mut metadata) = jwks_client(200).await;
        metadata.oauth_metadata.jwks_uri = None;
        let error = client.fetch_jwks(&metadata).await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::JwksUriMissing));
    }

    #[tokio::test]
    async fn fetch_jwks_reports_provider_errors() {
        let (_server, client, metadata) = jwks_client(500).await;
        let error = client.fetch_jwks(&metadata).await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::Jwks(_)));
    }

    fn redirect_uri() -> Url {
        "https://portal.example.org/callback".parse().unwrap()
    }

    fn token_request(redirect_uri: Url) -> TokenRequest<AuthorizationCodeGrantType> {
        TokenRequest {
            grant_type: AuthorizationCodeGrantType::AuthorizationCode {
                code: AuthorizationCode::from("auth-code".to_string()),
            },
            client_id: Some("test-client".to_string()),
            redirect_uri: Some(redirect_uri),
            scope: None,
            code_verifier: Some("test-verifier".to_string()),
        }
    }

    async fn token_client(status: u16, body: serde_json::Value) -> (OidcHttpClient, OidcProviderMetadata, MockServer) {
        let server = MockServer::start_async().await;
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let metadata = metadata(&issuer);
        server
            .mock_async(move |when, then| {
                when.method(POST).path("/token");
                then.status(status).json_body(body.clone());
            })
            .await;
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        (OidcHttpClient::new(http_client, issuer), metadata, server)
    }

    #[tokio::test]
    async fn exchange_code_returns_tokens() {
        let (client, metadata, _server) = token_client(
            200,
            json!({
                "id_token": "the-id-token",
                "access_token": "the-access-token",
                "token_type": "Bearer"
            }),
        )
        .await;

        let response = client
            .exchange_code(token_request(redirect_uri()), &metadata)
            .await
            .expect("should succeed");
        assert_eq!(response.id_token, "the-id-token");
        assert_eq!(response.token_response.access_token.as_ref(), "the-access-token");
    }

    #[tokio::test]
    async fn exchange_code_reports_provider_error() {
        let (client, metadata, _server) = token_client(500, json!({ "error": "server_error" })).await;

        let error = client
            .exchange_code(token_request(redirect_uri()), &metadata)
            .await
            .expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::TokenEndpoint(_)));
    }

    fn userinfo_claims() -> serde_json::Value {
        json!({
            "sub": "subject-1",
            "name": "Jane Doe"
        })
    }

    async fn userinfo_client(
        status: u16,
        body: serde_json::Value,
    ) -> (OidcHttpClient, OidcProviderMetadata, MockServer) {
        let server = MockServer::start_async().await;
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let mut metadata = metadata(&issuer);
        metadata.oidc_metadata_extension.userinfo_endpoint = Some(issuer.as_base_url().join("/userinfo"));
        server
            .mock_async(move |when, then| {
                when.method(POST).path("/userinfo");
                then.status(status).json_body(body.clone());
            })
            .await;
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        (OidcHttpClient::new(http_client, issuer), metadata, server)
    }

    #[derive(Debug, Deserialize)]
    #[serde(transparent)]
    struct TestUserInfo(serde_json::Value);

    impl jwt::JwtTyp for TestUserInfo {
        fn is_valid_typ(_header_typ: Option<&str>) -> Result<(), JwtVerifyError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn request_userinfo_returns_claims() {
        let (client, metadata, _server) = userinfo_client(200, userinfo_claims()).await;
        let access_token = AccessToken::from("the-access-token".to_string());
        let claims: TestUserInfo = client
            .request_userinfo("test-client", &access_token, &metadata)
            .await
            .expect("should succeed");
        assert_eq!(claims.0["name"], "Jane Doe");
    }

    #[tokio::test]
    async fn request_userinfo_reports_provider_error() {
        let (client, metadata, _server) = userinfo_client(401, json!({ "error": "invalid_token" })).await;
        let access_token = AccessToken::from("the-access-token".to_string());
        let error = client
            .request_userinfo::<TestUserInfo>("test-client", &access_token, &metadata)
            .await
            .expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::UserInfo(_)));
    }

    #[tokio::test]
    async fn request_userinfo_rejects_missing_userinfo_endpoint() {
        let server = MockServer::start_async().await;
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let metadata = metadata(&issuer);
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let client = OidcHttpClient::new(http_client, issuer);
        let access_token = AccessToken::from("the-access-token".to_string());
        let error = client
            .request_userinfo::<TestUserInfo>("test-client", &access_token, &metadata)
            .await
            .expect_err("should fail");
        assert!(matches!(
            error,
            OidcHttpClientError::UserInfo(UserInfoError::NoUserinfoUrl)
        ));
    }

    fn authorization_pkce_pair() -> S256PkcePair {
        S256PkcePair::from_code_verifier("test-code-verifier".to_string())
    }

    fn oidc_authorization_request(
        scope: HashSet<Scope>,
        pkce_pair: &S256PkcePair,
        login_hint: Option<String>,
    ) -> OidcAuthorizationRequest {
        OidcAuthorizationRequest {
            auth_request: AuthorizationCodeRequest::new(
                "test-client".to_string(),
                redirect_uri(),
                "session-state".to_string(),
                scope,
                pkce_pair,
            ),
            nonce: Some(Nonce::new_random()),
            login_hint,
        }
    }

    #[test]
    fn authorization_url_builds_keycloak_auth_url() {
        let server = MockServer::start();
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let metadata = metadata(&issuer);
        let client = OidcHttpClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(), issuer);
        let redirect_uri: Url = redirect_uri();
        let scope = HashSet::from(["openid".parse::<Scope>().unwrap(), "profile".parse().unwrap()]);
        let pkce_pair = authorization_pkce_pair();

        let oidc_request = oidc_authorization_request(scope, &pkce_pair, Some("user@example.org".to_string()));
        let url = client
            .authorization_url(&oidc_request, &metadata)
            .expect("should build the URL");

        assert!(
            url.as_str().starts_with(
                &(metadata
                    .oauth_metadata
                    .authorization_endpoint
                    .clone()
                    .unwrap()
                    .to_string()
                    + "?")
            )
        );
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["response_type"], "code");
        assert_eq!(query["client_id"], "test-client");
        assert_eq!(query["redirect_uri"], redirect_uri.to_string());
        assert_eq!(
            query["scope"].split(' ').collect::<HashSet<_>>(),
            HashSet::from(["openid", "profile"])
        );
        assert_eq!(query["state"], "session-state");
        assert!(!query["nonce"].is_empty());
        assert_eq!(
            query["code_challenge"],
            S256PkcePair::challenge_for("test-code-verifier")
        );
        assert_eq!(query["code_challenge_method"], "S256");
        assert_eq!(query["login_hint"], "user@example.org");
    }

    #[test]
    fn authorization_url_omits_login_hint_when_absent() {
        let server = MockServer::start();
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let metadata = metadata(&issuer);
        let client = OidcHttpClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(), issuer);
        let oidc_request = oidc_authorization_request(HashSet::new(), &authorization_pkce_pair(), None);
        let url = client
            .authorization_url(&oidc_request, &metadata)
            .expect("should build the URL");

        let query = url.query().unwrap();
        assert!(!query.contains("login_hint"));
    }

    #[test]
    fn authorization_url_rejects_missing_authorization_endpoint() {
        let server = MockServer::start();
        let issuer: IssuerIdentifier = server.base_url().parse().unwrap();
        let mut metadata = metadata(&issuer);
        metadata.oauth_metadata.authorization_endpoint = None;
        let client = OidcHttpClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(), issuer);
        let oidc_request = oidc_authorization_request(HashSet::new(), &authorization_pkce_pair(), None);
        let error = client
            .authorization_url(&oidc_request, &metadata)
            .expect_err("should fail");

        assert!(matches!(error, OidcHttpClientError::AuthorizationEndpointMissing));
    }
}
