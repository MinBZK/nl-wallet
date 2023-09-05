use async_trait::async_trait;
use http::{header, HeaderMap, HeaderValue};
use mime::Mime;
use reqwest::{Client, Request};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use wallet_common::account::{
    messages::{
        auth::{Certificate, Challenge, Registration, WalletCertificate},
        errors::ErrorData,
        instructions::{
            CheckPin, Instruction, InstructionChallengeRequestMessage, InstructionResult, InstructionResultMessage,
        },
    },
    signed::SignedDouble,
};

use crate::utils::reqwest::default_reqwest_client_builder;

use super::{AccountProvider, AccountProviderError, AccountProviderResponseError};

pub struct AccountServerClient {
    http_client: Client,
}

impl AccountServerClient {
    fn new() -> Self {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static("application/json"),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        AccountServerClient { http_client }
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
            let content_type = response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|content_type| content_type.to_str().ok())
                .and_then(|content_type| content_type.parse::<Mime>().ok());
            // ...and get the media type, subtype and optional suffix.
            let content_type_components = content_type
                .as_ref()
                .map(|content_type| (content_type.type_(), content_type.subtype(), content_type.suffix()));

            // Return the correct `AccountServerResponseError` based on all of these.
            let error = match (content_length, content_type_components) {
                // If we know there is an empty body,
                // we can stop early and return `AccountServerResponseError::Status`.
                (Some(content_length), _) if content_length == 0 => AccountProviderResponseError::Status(status),
                // When the `Content-Type` header is either `application/json` or `application/???+json`,
                // attempt to parse the body as `ErrorData`. If this fails, just return
                // `AccountServerResponseError::Status`.
                (_, Some((mime::APPLICATION, mime::JSON, _))) | (_, Some((mime::APPLICATION, _, Some(mime::JSON)))) => {
                    match response.json::<ErrorData>().await {
                        Ok(error_data) => AccountProviderResponseError::Data(status, error_data),
                        Err(_) => AccountProviderResponseError::Status(status),
                    }
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

impl Default for AccountServerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountProvider for AccountServerClient {
    async fn registration_challenge(&self, base_url: &Url) -> Result<Vec<u8>, AccountProviderError> {
        let url = base_url.join("enroll")?;
        let request = self.http_client.post(url).build()?;
        let challenge: Challenge = self.send_json_request::<Challenge>(request).await?;

        Ok(challenge.challenge.0)
    }

    async fn register(
        &self,
        base_url: &Url,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountProviderError> {
        let url = base_url.join("createwallet")?;
        let cert: Certificate = self.send_json_post_request(url, &registration_message).await?;

        Ok(cert.certificate)
    }

    async fn instruction_challenge(
        &self,
        base_url: &Url,
        challenge_request: InstructionChallengeRequestMessage,
    ) -> Result<Vec<u8>, AccountProviderError> {
        let url = base_url.join("instructions/challenge")?;
        let challenge: Challenge = self.send_json_post_request(url, &challenge_request).await?;

        Ok(challenge.challenge.0)
    }

    async fn check_pin(
        &self,
        base_url: &Url,
        instruction: Instruction<CheckPin>,
    ) -> Result<InstructionResult<()>, AccountProviderError> {
        let url = base_url.join("instructions/check_pin")?;
        let message: InstructionResultMessage<()> = self.send_json_post_request(url, &instruction).await?;

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
    use http::HeaderValue;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use wallet_common::account::messages::errors::ErrorType;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct ExampleBody {
        pub foo: String,
        pub bar: u64,
    }

    async fn create_mock_server() -> (MockServer, Url) {
        let server = MockServer::start().await;
        let base_url = Url::parse(&server.uri()).expect("Could not parse mock server URI");

        (server, base_url)
    }

    async fn post_example_request(client: &AccountServerClient, url: Url) -> Result<ExampleBody, AccountProviderError> {
        let request = client.http_client.post(url).build().expect("Could not create request");

        client.send_json_request::<ExampleBody>(request).await
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_ok() {
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

        let client = AccountServerClient::default();
        let body = post_example_request(&client, base_url.join("foobar").unwrap())
            .await
            .expect("Could not get succesful response from server");

        assert_eq!(body.foo, "blah");
        assert_eq!(body.bar, 1234);
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_error_status() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar_404"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let client = AccountServerClient::default();
        let error = post_example_request(&client, base_url.join("foobar_404").unwrap())
            .await
            .expect_err("No error received from server");

        assert!(matches!(
            error,
            AccountProviderError::Response(AccountProviderResponseError::Status(StatusCode::NOT_FOUND))
        ))
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_error_text() {
        let (server, base_url) = create_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/foobar_502"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Your gateway is bad and you should feel bad!"))
            .expect(1)
            .mount(&server)
            .await;

        let client = AccountServerClient::default();
        let error = post_example_request(&client, base_url.join("foobar_502").unwrap())
            .await
            .expect_err("No error received from server");

        assert!(matches!(
            error,
            AccountProviderError::Response(AccountProviderResponseError::Text(StatusCode::BAD_GATEWAY, body))
       if body == "Your gateway is bad and you should feel bad!"))
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_error_data() {
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
                            "type": "ChallengeValidation",
                            "title": "Error title",
                        }))
                        .unwrap(),
                    ),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = AccountServerClient::default();
        let error = post_example_request(&client, base_url.join("foobar_400").unwrap())
            .await
            .expect_err("No error received from server");

        if let AccountProviderError::Response(AccountProviderResponseError::Data(StatusCode::BAD_REQUEST, data)) = error
        {
            assert!(matches!(data.typ, ErrorType::ChallengeValidation));
            assert_eq!(data.title, "Error title");
        }
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_other_json() {
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

        let client = AccountServerClient::default();
        let error = post_example_request(&client, base_url.join("foobar_503").unwrap())
            .await
            .expect_err("No error received from server");

        assert!(matches!(
            error,
            AccountProviderError::Response(AccountProviderResponseError::Status(StatusCode::SERVICE_UNAVAILABLE))
        ));
    }
}
