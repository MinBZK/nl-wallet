use http_utils::reqwest::HttpClient;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::metadata::oauth_metadata::OidcProviderMetadata;
use oauth::metadata::well_known::WellKnownError;
use oauth::metadata::well_known::WellKnownMetadata;

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
        let metadata = OidcProviderMetadata::fetch_well_known_json(&self.http_client, &self.expected_issuer)
            .await
            .map_err(OidcHttpClientError::WellKnown)?;
        let oauth_metadata = &metadata.oauth_metadata;

        if oauth_metadata.authorization_endpoint.is_none() {
            return Err(OidcHttpClientError::AuthorizationEndpointMissing);
        }

        if oauth_metadata.jwks_uri.is_none() {
            return Err(OidcHttpClientError::JwksUriMissing);
        }

        if !oauth_metadata.response_types_supported.contains("code") {
            return Err(OidcHttpClientError::AuthorizationCodeNotSupported);
        }

        if !oauth_metadata
            .code_challenge_methods_supported
            .as_ref()
            .is_some_and(|methods| methods.contains("S256"))
        {
            return Err(OidcHttpClientError::S256NotSupported);
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use indexmap::IndexSet;
    use oauth::metadata::oauth_metadata::AuthorizationServerMetadata;
    use serde_json::json;

    use super::*;

    fn metadata(issuer: &IssuerIdentifier) -> OidcProviderMetadata {
        let issuer_url = issuer.as_base_url();
        OidcProviderMetadata::new(
            AuthorizationServerMetadata {
                authorization_endpoint: Some(issuer_url.join("/authorize")),
                jwks_uri: Some(issuer_url.join("/jwks")),
                response_types_supported: IndexSet::from_iter(["code".to_string()]),
                code_challenge_methods_supported: Some(IndexSet::from_iter(["S256".to_string()])),
                ..AuthorizationServerMetadata::new(issuer.clone(), issuer_url.join("/token"))
            },
            Default::default(),
        )
    }

    async fn client(modify: impl FnOnce(&mut OidcProviderMetadata)) -> OidcHttpClient {
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
        OidcHttpClient::new(http_client, issuer)
    }

    #[tokio::test]
    async fn fetch_metadata_returns_valid_metadata() {
        let client = client(|_| {}).await;
        let metadata = client.fetch_metadata().await.unwrap();
        assert_eq!(&metadata.oauth_metadata.issuer, &client.expected_issuer);
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_authorization_endpoint() {
        let client = client(|metadata| metadata.oauth_metadata.authorization_endpoint = None).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::AuthorizationEndpointMissing));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_issuer_mismatch() {
        let client =
            client(|metadata| metadata.oauth_metadata.issuer = "https://other.example.com".parse().unwrap()).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(
            error,
            OidcHttpClientError::WellKnown(WellKnownError::IssuerIdentifierMismatch { .. })
        ));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_jwks_uri() {
        let client = client(|metadata| metadata.oauth_metadata.jwks_uri = None).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::JwksUriMissing));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_code_response_type() {
        let client = client(|metadata| metadata.oauth_metadata.response_types_supported = IndexSet::new()).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::AuthorizationCodeNotSupported));
    }

    #[tokio::test]
    async fn fetch_metadata_rejects_missing_s256() {
        let client = client(|metadata| metadata.oauth_metadata.code_challenge_methods_supported = None).await;
        let error = client.fetch_metadata().await.expect_err("should fail");
        assert!(matches!(error, OidcHttpClientError::S256NotSupported));
    }
}
