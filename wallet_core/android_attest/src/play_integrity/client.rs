use std::error::Error;

use reqwest::Client;
use serde::Serialize;
use url::Url;

use super::integrity_verdict::IntegrityVerdict;

const URL_PREFIX: &str = "https://playintegrity.googleapis.com/v1/";
const URL_SUFFIX: &str = ":decodeIntegrityToken";

#[trait_variant::make(Send)]
pub trait IntegrityTokenDecoder {
    type Error: Error + Send + Sync + 'static;

    async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum PlayIntegrityClientError {
    #[error("package name leads to invalid URL: {0}")]
    PackageName(#[from] url::ParseError),
    #[error("could not send HTTP request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("could not decode integrity verdict JSON: {0}")]
    DecodeIntegrityVerdict(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize)]
struct IntegrityTokenRequest<'a> {
    pub integrity_token: &'a str,
}

#[derive(Debug, Clone)]
pub struct PlayIntegrityClient {
    client: Client,
    url: Url,
    package_name_offset: usize,
    package_name_len: usize,
}

impl PlayIntegrityClient {
    pub fn new(client: Client, package_name: &str) -> Result<Self, PlayIntegrityClientError> {
        let url = format!("{URL_PREFIX}{package_name}{URL_SUFFIX}").parse()?;
        let package_name_offset = URL_PREFIX.len();
        let package_name_len = package_name.len();

        let client = Self {
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
}

impl IntegrityTokenDecoder for PlayIntegrityClient {
    type Error = PlayIntegrityClientError;

    async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), Self::Error> {
        let request_body = IntegrityTokenRequest { integrity_token };
        let json = self
            .client
            .get(self.url.clone())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let integrity_verdict = serde_json::from_str(&json)?;

        Ok((integrity_verdict, json))
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use reqwest::ClientBuilder;
    use reqwest::StatusCode;
    use serde_json::json;
    use serde_json::Value;
    use wiremock::matchers::body_partial_json;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    use super::super::tests::EXAMPLE_VERDICT;
    use super::super::tests::EXAMPLE_VERDICT_JSON;
    use super::*;

    const INTEGRITY_TOKEN: &str = "example_integrity_token";

    /// Start a mock HTTP server and patch the client's URL to point to that mock server.
    async fn inject_play_integrity_server(mut client: PlayIntegrityClient) -> (PlayIntegrityClient, MockServer) {
        let server = MockServer::start().await;

        // Set up a response for INTEGRITY_TOKEN, based on the parameters of the client.
        Mock::given(method("GET"))
            .and(path(client.url.path()))
            .and(body_partial_json(json!({
                "integrity_token": INTEGRITY_TOKEN
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(&*EXAMPLE_VERDICT_JSON))
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
        let client = PlayIntegrityClient::new(ClientBuilder::default().build().unwrap(), "com.package.name")
            .expect("package name should be valid in URL");

        assert_eq!(client.package_name(), "com.package.name");

        let (client, _server) = inject_play_integrity_server(client).await;

        let (verdict, json) = client
            .decode_token(INTEGRITY_TOKEN)
            .await
            .expect("request to decode integrity token should return integrity token and JSON source");

        assert_eq!(verdict, *EXAMPLE_VERDICT);

        let parsed_json = serde_json::from_str::<Value>(&json).expect("source should parse as json");

        assert_eq!(parsed_json, *EXAMPLE_VERDICT_JSON);
    }

    #[tokio::test]
    async fn test_play_integrity_http_error() {
        let client = PlayIntegrityClient::new(ClientBuilder::default().build().unwrap(), "com.package.name")
            .expect("package name should be valid in URL");
        let (client, _server) = inject_play_integrity_server(client).await;

        let error = client
            .decode_token("does_not_exist")
            .await
            .expect_err("request to decode an unknown integrity token should return a error");

        assert_matches!(error, PlayIntegrityClientError::Http(error) if error.status() == Some(StatusCode::NOT_FOUND));
    }
}

#[cfg(feature = "mock_play_integrity")]
pub mod mock {
    use base64::prelude::*;

    use super::super::verification::VerifyPlayStore;
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("mock play integrity client error to be used in tests")]
    pub struct MockPlayIntegrityClientError {}

    pub struct MockPlayIntegrityClient {
        pub package_name: String,
        pub verify_play_store: VerifyPlayStore,
        pub has_error: bool,
    }

    impl MockPlayIntegrityClient {
        pub fn new(package_name: String, verify_play_store: VerifyPlayStore) -> Self {
            Self {
                package_name,
                verify_play_store,
                has_error: false,
            }
        }
    }

    impl IntegrityTokenDecoder for MockPlayIntegrityClient {
        type Error = MockPlayIntegrityClientError;

        async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), Self::Error> {
            if self.has_error {
                return Err(MockPlayIntegrityClientError {});
            }

            // For testing, assume the integrity token simply contains the Base64 encoded request hash.
            let request_hash = BASE64_STANDARD_NO_PAD.decode(integrity_token).unwrap();

            let verdict =
                IntegrityVerdict::new_mock(self.package_name.clone(), request_hash, self.verify_play_store.clone());
            let json = serde_json::to_string(&verdict).unwrap();

            Ok((verdict, json))
        }
    }
}
