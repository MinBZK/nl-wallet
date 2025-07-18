use std::path::Path;
use std::string::FromUtf8Error;

use derive_more::Debug;
use futures::TryFutureExt;
use gcloud_auth::credentials::CredentialsFile;
use gcloud_auth::project::Config;
use gcloud_auth::project::create_token_source_from_credentials;
use gcloud_auth::token_source::TokenSource;
use http::HeaderValue;
use http::header;
use http::header::InvalidHeaderValue;
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use tokio::fs;
use tokio::sync::OnceCell;
use url::Url;

use super::integrity_verdict::IntegrityVerdict;

const GOOGLE_CLOUD_PLAY_INTEGRITY_SCOPE: &str = "https://www.googleapis.com/auth/playintegrity";

const URL_PREFIX: &str = "https://playintegrity.googleapis.com/v1/";
const URL_SUFFIX: &str = ":decodeIntegrityToken";

#[derive(Debug, thiserror::Error)]
pub enum ServiceAccountError {
    #[error("could not read service account credentials file: {0}")]
    Read(#[source] std::io::Error),
    #[error("service account credentials file contains invalid UTF-8: {0}")]
    Utf8(#[source] FromUtf8Error),
    #[error("could not parse service account credentials JSON: {0}")]
    Parse(#[source] gcloud_auth::error::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PlayIntegrityError {
    #[error("could not retrieve access token for Google Cloud API: {0}")]
    AccessToken(#[source] gcloud_auth::error::Error),
    #[error("could not format authorization header: {0}")]
    AuthorizationHeader(#[source] InvalidHeaderValue),
    #[error("package name leads to invalid URL: {0}")]
    PackageName(#[source] url::ParseError),
    #[error("could not send HTTP request: {0}")]
    Http(#[source] reqwest::Error),
    #[error("received HTTP error response \"{0}\" from API: {1}")]
    HttpResponse(StatusCode, String),
    #[error("could not decode integrity verdict JSON: {0}")]
    DecodeIntegrityVerdict(#[source] serde_json::Error),
}

#[derive(Debug, Serialize)]
struct IntegrityTokenRequest<'a> {
    pub integrity_token: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IntegrityVerdictResponse {
    token_payload_external: IntegrityVerdict,
}

pub trait PlayIntegrityAuthProvider {
    async fn authorization_header_value(&self) -> Result<HeaderValue, PlayIntegrityError>;
}

pub struct ServiceAccountAuthenticator {
    google_cloud_credentials: CredentialsFile,
    token_source: OnceCell<Box<dyn TokenSource>>,
}

impl ServiceAccountAuthenticator {
    pub async fn new(credentials_json_file: &Path) -> Result<Self, ServiceAccountError> {
        let credentials_json = String::from_utf8(
            fs::read(credentials_json_file)
                .await
                .map_err(ServiceAccountError::Read)?,
        )
        .map_err(ServiceAccountError::Utf8)?;

        let google_cloud_credentials = CredentialsFile::new_from_str(&credentials_json)
            .await
            .map_err(ServiceAccountError::Parse)?;

        let authenticator = Self {
            google_cloud_credentials,
            token_source: OnceCell::new(),
        };

        Ok(authenticator)
    }
}

impl PlayIntegrityAuthProvider for ServiceAccountAuthenticator {
    async fn authorization_header_value(&self) -> Result<HeaderValue, PlayIntegrityError> {
        // Initialize the token source from the credentials on first use.
        // Note that this immediately fetches an access token.
        let token_source = self
            .token_source
            .get_or_try_init(|| async {
                let config = Config::default().with_scopes(&[GOOGLE_CLOUD_PLAY_INTEGRITY_SCOPE]);

                create_token_source_from_credentials(&self.google_cloud_credentials, &config).await
            })
            .await
            .map_err(PlayIntegrityError::AccessToken)?;

        // Fetch the access token, which relies on caching behaviour inside the implementor of TokenSource.
        let token = token_source.token().await.unwrap();

        // Format the access token as a HTTP header value.
        let header_value = HeaderValue::from_str(&format!("Bearer {}", token.access_token))
            .map_err(PlayIntegrityError::AuthorizationHeader)?;

        Ok(header_value)
    }
}

#[derive(Debug)]
pub struct PlayIntegrityClient<A = ServiceAccountAuthenticator> {
    #[debug(skip)]
    auth_provider: A,
    client: Client,
    url: Url,
    package_name_offset: usize,
    package_name_len: usize,
}

impl<A> PlayIntegrityClient<A> {
    pub fn new(client: Client, auth_provider: A, package_name: &str) -> Result<Self, PlayIntegrityError> {
        let url = format!("{URL_PREFIX}{package_name}{URL_SUFFIX}")
            .parse()
            .map_err(PlayIntegrityError::PackageName)?;
        let package_name_offset = URL_PREFIX.len();
        let package_name_len = package_name.len();

        let client = Self {
            auth_provider,
            client,
            url,
            package_name_offset,
            package_name_len,
        };

        Ok(client)
    }

    pub fn package_name(&self) -> &str {
        &self.url.as_str()[self.package_name_offset..(self.package_name_offset + self.package_name_len)]
    }

    pub async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), PlayIntegrityError>
    where
        A: PlayIntegrityAuthProvider,
    {
        let auth_header = self.auth_provider.authorization_header_value().await?;

        let request_body = IntegrityTokenRequest { integrity_token };
        let json = self
            .client
            .post(self.url.clone())
            .header(header::AUTHORIZATION, auth_header)
            .json(&request_body)
            .send()
            .map_err(PlayIntegrityError::Http)
            .and_then(|response| async {
                let status = response.status();
                let body = response.text().await.map_err(PlayIntegrityError::Http)?;

                if status.is_client_error() || status.is_server_error() {
                    return Err(PlayIntegrityError::HttpResponse(status, body));
                }

                Ok(body)
            })
            .await?;

        let response = serde_json::from_str::<IntegrityVerdictResponse>(&json)
            .map_err(PlayIntegrityError::DecodeIntegrityVerdict)?;

        Ok((response.token_payload_external, json))
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use reqwest::ClientBuilder;
    use reqwest::StatusCode;
    use serde_json::Value;
    use serde_json::json;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::body_partial_json;
    use wiremock::matchers::header;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    use super::super::tests::EXAMPLE_VERDICT;
    use super::super::tests::EXAMPLE_VERDICT_JSON;
    use super::*;

    const INTEGRITY_TOKEN: &str = "example_integrity_token";
    const AUTH_HEADER: &str = "Bearer access_token";

    #[derive(Default)]
    struct MockPlayIntegrityAuthProvider {}

    impl PlayIntegrityAuthProvider for MockPlayIntegrityAuthProvider {
        async fn authorization_header_value(&self) -> Result<HeaderValue, PlayIntegrityError> {
            Ok(HeaderValue::from_static(AUTH_HEADER))
        }
    }

    /// Start a mock HTTP server and patch the client's URL to point to that mock server.
    async fn inject_play_integrity_server<A>(
        mut client: PlayIntegrityClient<A>,
    ) -> (PlayIntegrityClient<A>, MockServer) {
        let server = MockServer::start().await;

        // Set up a response for INTEGRITY_TOKEN, based on the parameters of the client.
        Mock::given(method("POST"))
            .and(path(client.url.path()))
            .and(header(header::AUTHORIZATION, AUTH_HEADER))
            .and(body_partial_json(json!({
                "integrity_token": INTEGRITY_TOKEN
            })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"tokenPayloadExternal": *EXAMPLE_VERDICT_JSON})),
            )
            .mount(&server)
            .await;

        // Replace the host and port within the URL, while keeping any other components the same.
        client.url = [
            &server.uri(),
            client.url.path(),
            client.url.query().unwrap_or_default(),
            client.url.fragment().unwrap_or_default(),
        ]
        .join("")
        .parse()
        .unwrap();

        (client, server)
    }

    #[tokio::test]
    async fn test_play_integrity_client() {
        let client = PlayIntegrityClient::new(
            ClientBuilder::default().build().unwrap(),
            MockPlayIntegrityAuthProvider::default(),
            "com.package.name",
        )
        .expect("package name should be valid in URL");

        assert_eq!(client.package_name(), "com.package.name");

        let (client, _server) = inject_play_integrity_server(client).await;

        let (verdict, json) = client
            .decode_token(INTEGRITY_TOKEN)
            .await
            .expect("request to decode integrity token should return integrity token and JSON source");

        assert_eq!(verdict, *EXAMPLE_VERDICT);

        let parsed_json = serde_json::from_str::<Value>(&json).expect("source should parse as json");

        assert_eq!(parsed_json.get("tokenPayloadExternal"), Some(&*EXAMPLE_VERDICT_JSON));
    }

    #[tokio::test]
    async fn test_play_integrity_http_response_error() {
        let client = PlayIntegrityClient::new(
            ClientBuilder::default().build().unwrap(),
            MockPlayIntegrityAuthProvider::default(),
            "com.package.name",
        )
        .expect("package name should be valid in URL");
        let (client, _server) = inject_play_integrity_server(client).await;

        let error = client
            .decode_token("does_not_exist")
            .await
            .expect_err("request to decode an unknown integrity token should return a error");

        assert_matches!(error, PlayIntegrityError::HttpResponse(status, _) if status == StatusCode::NOT_FOUND);
    }
}
