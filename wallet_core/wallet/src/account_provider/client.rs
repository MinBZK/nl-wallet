use std::path::Path;

use http::StatusCode;
use reqwest::Client;
use reqwest::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;

use wallet_account::messages::errors::AccountError;
use wallet_account::messages::errors::AccountErrorType;
use wallet_account::messages::instructions::Instruction;
use wallet_account::messages::instructions::InstructionAndResult;
use wallet_account::messages::instructions::InstructionChallengeRequest;
use wallet_account::messages::instructions::InstructionResult;
use wallet_account::messages::instructions::InstructionResultMessage;
use wallet_account::messages::registration::Certificate;
use wallet_account::messages::registration::Challenge;
use wallet_account::messages::registration::Registration;
use wallet_account::messages::registration::WalletCertificate;
use wallet_account::signed::ChallengeResponse;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::http_error::HttpJsonErrorBody;
use wallet_common::reqwest::parse_content_type;
use wallet_common::reqwest::RequestBuilder;

use super::AccountProviderClient;
use super::AccountProviderError;
use super::AccountProviderResponseError;

#[derive(Default)]
pub struct HttpAccountProviderClient {}

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
    async fn send_json_post_request<S, T, C>(
        &self,
        endpoint: &str,
        client_config: &C,
        json: &S,
    ) -> Result<T, AccountProviderError>
    where
        S: Serialize,
        T: DeserializeOwned,
        C: RequestBuilder,
    {
        let (http_client, request) = client_config.post(Path::new(endpoint));
        self.send_json_request::<T>(http_client, request.json(json).build()?)
            .await
    }

    async fn send_json_request<T>(&self, http_client: Client, request: Request) -> Result<T, AccountProviderError>
    where
        T: DeserializeOwned,
    {
        let response = http_client.execute(request).await?;
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

impl AccountProviderClient for HttpAccountProviderClient {
    async fn registration_challenge(&self, client_config: &TlsPinningConfig) -> Result<Vec<u8>, AccountProviderError> {
        let (http_client, request) = client_config.post("enroll");

        let challenge: Challenge = self.send_json_request(http_client, request.build()?).await?;

        Ok(challenge.challenge)
    }

    async fn register(
        &self,
        client_config: &TlsPinningConfig,
        registration_message: ChallengeResponse<Registration>,
    ) -> Result<WalletCertificate, AccountProviderError> {
        let cert: Certificate = self
            .send_json_post_request("createwallet", client_config, &registration_message)
            .await?;

        Ok(cert.certificate)
    }

    async fn instruction_challenge(
        &self,
        client_config: &TlsPinningConfig,
        challenge_request: InstructionChallengeRequest,
    ) -> Result<Vec<u8>, AccountProviderError> {
        let challenge: Challenge = self
            .send_json_post_request("instructions/challenge", client_config, &challenge_request)
            .await?;

        Ok(challenge.challenge)
    }

    async fn instruction<I>(
        &self,
        client_config: &TlsPinningConfig,
        instruction: Instruction<I>,
    ) -> Result<InstructionResult<I::Result>, AccountProviderError>
    where
        I: InstructionAndResult,
    {
        let message: InstructionResultMessage<I::Result> = self
            .send_json_post_request(&format!("instructions/{}", I::NAME), client_config, &instruction)
            .await?;

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
    use http::header;
    use http::HeaderValue;
    use reqwest::StatusCode;
    use serde::Deserialize;
    use serde::Serialize;
    use serde_json::json;
    use serde_json::Value;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    use wallet_common::config::http::test::HttpConfig;
    use wallet_common::reqwest::JsonReqwestBuilder;
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
        endpoint: impl AsRef<Path>,
        client_config: &impl JsonReqwestBuilder,
    ) -> Result<ExampleBody, AccountProviderError> {
        let (http_client, request) = client_config.post(endpoint);
        client.send_json_request(http_client, request.build()?).await
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
        let body = post_example_request(&client, "foobar", &HttpConfig { base_url })
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
        let error = post_example_request(&client, "foobar_404", &HttpConfig { base_url })
            .await
            .expect_err("No error received from server");

        assert_matches!(
            error,
            AccountProviderError::Response(AccountProviderResponseError::Status(StatusCode::NOT_FOUND))
        );
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
        let error = post_example_request(&client, "foobar_502", &HttpConfig { base_url })
            .await
            .expect_err("No error received from server");

        assert_matches!(
            error,
            AccountProviderError::Response(
                AccountProviderResponseError::Text(StatusCode::BAD_GATEWAY, body)
            ) if body == "Your gateway is bad and you should feel bad!"
        );
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
        let error = post_example_request(&client, "foobar_400", &HttpConfig { base_url })
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
        let error = post_example_request(&client, "foobar_503", &HttpConfig { base_url })
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
