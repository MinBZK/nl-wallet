use reqwest::Client;
use serde::Serialize;
use url::Url;

use super::integrity_verdict::IntegrityVerdict;

const URL_PREFIX: &str = "https://playintegrity.googleapis.com/v1/";
const URL_SUFFIX: &str = ":decodeIntegrityToken";

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

    pub async fn decode_token(
        &self,
        integrity_token: &str,
    ) -> Result<(IntegrityVerdict, String), PlayIntegrityClientError> {
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
