use http::{header, HeaderMap, HeaderValue, StatusCode};
use reqwest::{Client, Request};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use wallet_common::{
    account::{
        messages::{
            auth::{Certificate, Challenge, Registration, WalletCertificate},
            errors::{AccountError, AccountErrorType},
            instructions::{
                Instruction, InstructionChallengeRequestMessage, InstructionEndpoint, InstructionResult,
                InstructionResultMessage,
            },
        },
        signed::SignedDouble,
    },
    http_error::HttpJsonErrorBody,
    reqwest::{default_reqwest_client_builder, parse_content_type},
    urls::BaseUrl,
};

use super::{AccountProviderClient, AccountProviderError, AccountProviderResponseError};

pub struct HttpAccountProviderClient {
    http_client: Client,
}

impl AccountProviderResponseError {
    fn from_json_body(status: StatusCode, body: String) -> Self {
        serde_json::from_str::<HttpJsonErrorBody<AccountErrorType>>(&body)
            .and_then(|error_body| {
                AccountError::try_from_type_and_data(error_body.r#type, error_body.extra)
                    .map(|account_error| Self::Account(account_error, error_body.detail))
            })
            .unwrap_or(Self::Text(status, body))
    }
}

impl HttpAccountProviderClient {
    fn new() -> Self {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static("application/json"),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        HttpAccountProviderClient { http_client }
    }

    async fn send_json_post_request<S, T>(&self, url: Url, json: &S) -> Result<T, AccountProviderError>
    where
        S: Serialize,
        T: DeserializeOwned,
    {
        let request = self.http_client.post(url).json(json).build()?;
        self.send_json_request::<T>(request).await
    }

    async fn send_json_request<T>(&self, request: Request) -> Result<T, AccountProviderError>
    where
        T: DeserializeOwned,
    {
        let response = self.http_client.execute(request).await?;
        let status = response.status();

        // In case of a 4xx or 5xx response...
        if status.is_client_error() || status.is_server_error() {
            let content_length = response.content_length();
            // Parse any `Content-Type` header that is present to a Mime type...
            let content_type = parse_content_type(&response);
            let content_type_components = content_type
                .as_ref()
                .map(|content_type| (content_type.type_(), content_type.subtype(), content_type.suffix()));

            // Return the correct `AccountServerResponseError` based on all of these.
            let error = match (content_length, content_type_components) {
                // If we know there is an empty body,
                // we can stop early and return `AccountServerResponseError::Status`.
                (Some(0), _) => AccountProviderResponseError::Status(status),
                // When the `Content-Type` header is either `application/json` or `application/???+json`,
                // attempt to parse the body as `HttpJsonErrorBody<AccountErrorType>`. If this fails,
                // fall back on either`AccountServerResponseError::Text` or `AccountProviderResponseError::Status`
                (_, Some((mime::APPLICATION, mime::JSON, _))) | (_, Some((mime::APPLICATION, _, Some(mime::JSON)))) => {
                    response
                        .text()
                        .await
                        .map(|body| AccountProviderResponseError::from_json_body(status, body))
                        .unwrap_or_else(|_| AccountProviderResponseError::Status(status))
                }
                // When the `Content-Type` header is `text/plain`, attempt to get the body as text
                // and return `AccountServerResponseError::Text`. If this fails or the body is empty,
                // just return `AccountServerResponseError::Status`.
                (_, Some((mime::TEXT, mime::PLAIN, _))) => match response.text().await {
                    Ok(text) if !text.is_empty() => AccountProviderResponseError::Text(status, text),
                    _ => AccountProviderResponseError::Status(status),
                },
                // The fallback is to return `AccountServerResponseError::Status`.
                _ => AccountProviderResponseError::Status(status),
            };

            return Err(AccountProviderError::Response(error));
        }

        let body = response.json().await?;

        Ok(body)
    }
}

impl Default for HttpAccountProviderClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountProviderClient for HttpAccountProviderClient {
    async fn registration_challenge(&self, base_url: &BaseUrl) -> Result<Vec<u8>, AccountProviderError> {
        let url = base_url.join("enroll");
        let request = self.http_client.post(url).build()?;
        let challenge: Challenge = self.send_json_request::<Challenge>(request).await?;

        Ok(challenge.challenge)
    }

    async fn register(
        &self,
        base_url: &BaseUrl,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountProviderError> {
        let url = base_url.join("createwallet");
        let cert: Certificate = self.send_json_post_request(url, &registration_message).await?;

        Ok(cert.certificate)
    }

    async fn instruction_challenge(
        &self,
        base_url: &BaseUrl,
        challenge_request: InstructionChallengeRequestMessage,
    ) -> Result<Vec<u8>, AccountProviderError> {
        let url = base_url.join("instructions/challenge");
        let challenge: Challenge = self.send_json_post_request(url, &challenge_request).await?;

        Ok(challenge.challenge)
    }

    async fn instruction<I>(
        &self,
        base_url: &BaseUrl,
        instruction: Instruction<I>,
    ) -> Result<InstructionResult<I::Result>, AccountProviderError>
    where
        I: InstructionEndpoint,
    {
        let url = base_url.join(&format!("instructions/{}", I::ENDPOINT));
        let message: InstructionResultMessage<I::Result> = self.send_json_post_request(url, &instruction).await?;

        Ok(message.result)
    }
}

#[cfg(test)]
/// Ceci n'est pas une unit test.
///
/// This test sets up a mock HTTP server and by definition also tests the `reqwest` dependency.
/// Its goal is mostly to validate that HTTP error responses get converted to the right variant
/// of `RemoteAccountServerClient` and `AccountServerResponseError`.
mod tests {
    use assert_matches::assert_matches;
    use http::HeaderValue;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use wallet_common::urls::BaseUrl;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct ExampleBody {
        pub foo: String,
        pub bar: u64,
    }

    async fn create_mock_server() -> (MockServer, BaseUrl) {
        let server = MockServer::start().await;
        let base_url = server.uri().parse().unwrap();

        (server, base_url)
    }

    async fn post_example_request(
        client: &HttpAccountProviderClient,
        url: Url,
    ) -> Result<ExampleBody, AccountProviderError> {
        let request = client.http_client.post(url).build().expect("Could not create request");

        client.send_json_request::<ExampleBody>(request).await
    }

    #[tokio::test]
    async fn test_http_account_server_client_send_json_request_ok() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ExampleBody {
                foo: "blah".to_string(),
                bar: 1234,
            }))
            .expect(1)
            .mount(&server)
            .await;

        let client = HttpAccountProviderClient::default();
        let body = post_example_request(&client, base_url.join("foobar"))
            .await
            .expect("Could not get succesful response from server");

        assert_eq!(body.foo, "blah");
        assert_eq!(body.bar, 1234);
    }

    #[tokio::test]
    async fn test_http_account_server_client_send_json_request_error_status() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar_404"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let client = HttpAccountProviderClient::default();
        let error = post_example_request(&client, base_url.join("foobar_404"))
            .await
            .expect_err("No error received from server");

        assert_matches!(
            error,
            AccountProviderError::Response(AccountProviderResponseError::Status(StatusCode::NOT_FOUND))
        )
    }

    #[tokio::test]
    async fn test_http_account_server_client_send_json_request_error_text() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar_502"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Your gateway is bad and you should feel bad!"))
            .expect(1)
            .mount(&server)
            .await;

        let client = HttpAccountProviderClient::default();
        let error = post_example_request(&client, base_url.join("foobar_502"))
            .await
            .expect_err("No error received from server");

        assert_matches!(
            error,
            AccountProviderError::Response(
                AccountProviderResponseError::Text(StatusCode::BAD_GATEWAY, body)
            ) if body == "Your gateway is bad and you should feel bad!"
        )
    }

    #[tokio::test]
    async fn test_http_account_server_client_send_json_request_error_type() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar_400"))
            .respond_with(
                ResponseTemplate::new(400)
                    .insert_header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str("application/problem+json").unwrap(),
                    )
                    .set_body_bytes(
                        serde_json::to_vec(&json!({
                            "type": "challenge_validation",
                            "detail": "Error description.",
                        }))
                        .unwrap(),
                    ),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = HttpAccountProviderClient::default();
        let error = post_example_request(&client, base_url.join("foobar_400"))
            .await
            .expect_err("No error received from server");

        assert_matches!(
            error,
            AccountProviderError::Response(
                AccountProviderResponseError::Account(AccountError::ChallengeValidation, detail)
            ) if detail == Some("Error description.".to_string())
        );
    }

    #[tokio::test]
    async fn test_http_account_server_client_send_json_request_other_json() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar_503"))
            .respond_with(ResponseTemplate::new(503).set_body_json(json!({
                "status": "503",
                "text": "Service Unavailable",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = HttpAccountProviderClient::default();
        let error = post_example_request(&client, base_url.join("foobar_503"))
            .await
            .expect_err("No error received from server");

        let expected_json = json!({
            "status": "503",
            "text": "Service Unavailable",
        });

        match error {
            AccountProviderError::Response(AccountProviderResponseError::Text(
                StatusCode::SERVICE_UNAVAILABLE,
                body,
            )) => {
                assert_eq!(serde_json::from_str::<Value>(&body).unwrap(), expected_json);
            }
            _ => panic!("should have received expected error"),
        }
    }
}
