use async_trait::async_trait;
use mime::Mime;
use reqwest::{
    header::{self},
    Client, Request,
};
use serde::{de::DeserializeOwned, Serialize};
use url::{ParseError, Url};

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

use crate::utils::reqwest::build_reqwest_client;

use super::{AccountServerClient, AccountServerClientError, AccountServerResponseError};

pub struct RemoteAccountServerClient {
    base_url: Url,
    client: Client,
}

impl RemoteAccountServerClient {
    fn new(base_url: Url) -> Self {
        let client = build_reqwest_client();

        RemoteAccountServerClient { base_url, client }
    }

    fn url(&self, path: &str) -> Result<Url, ParseError> {
        self.base_url.join(path)
    }

    async fn send_json_post_request<S, T>(&self, path: &str, json: &S) -> Result<T, AccountServerClientError>
    where
        S: Serialize,
        T: DeserializeOwned,
    {
        let request = self.client.post(self.url(path)?).json(json).build()?;
        self.send_json_request::<T>(request).await
    }

    async fn send_json_request<T>(&self, request: Request) -> Result<T, AccountServerClientError>
    where
        T: DeserializeOwned,
    {
        let response = self.client.execute(request).await?;
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
                (Some(content_length), _) if content_length == 0 => AccountServerResponseError::Status(status),
                // When the `Content-Type` header is either `application/json` or `application/???+json`,
                // attempt to parse the body as `ErrorData`. If this fails, just return
                // `AccountServerResponseError::Status`.
                (_, Some((mime::APPLICATION, mime::JSON, _))) | (_, Some((mime::APPLICATION, _, Some(mime::JSON)))) => {
                    match response.json::<ErrorData>().await {
                        Ok(error_data) => AccountServerResponseError::Data(status, error_data),
                        Err(_) => AccountServerResponseError::Status(status),
                    }
                }
                // When the `Content-Type` header is `text/plain`, attempt to get the body as text
                // and return `AccountServerResponseError::Text`. If this fails or the body is empty,
                // just return `AccountServerResponseError::Status`.
                (_, Some((mime::TEXT, mime::PLAIN, _))) => match response.text().await {
                    Ok(text) if !text.is_empty() => AccountServerResponseError::Text(status, text),
                    _ => AccountServerResponseError::Status(status),
                },
                // The fallback is to return `AccountServerResponseError::Status`.
                _ => AccountServerResponseError::Status(status),
            };

            return Err(AccountServerClientError::Response(error));
        }

        let body = response.json().await?;

        Ok(body)
    }
}

#[async_trait]
impl AccountServerClient for RemoteAccountServerClient {
    fn new(base_url: &Url) -> Self
    where
        Self: Sized,
    {
        Self::new(base_url.clone())
    }

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        let request = self.client.post(self.url("enroll")?).build()?;
        let challenge = self.send_json_request::<Challenge>(request).await?.challenge.0;

        Ok(challenge)
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        let cert: Certificate = self
            .send_json_post_request("createwallet", &registration_message)
            .await?;

        Ok(cert.certificate)
    }

    async fn instruction_challenge(
        &self,
        challenge_request: InstructionChallengeRequestMessage,
    ) -> Result<Vec<u8>, AccountServerClientError> {
        let challenge: Challenge = self
            .send_json_post_request("instructions/challenge", &challenge_request)
            .await?;

        Ok(challenge.challenge.0)
    }

    async fn check_pin(
        &self,
        instruction: Instruction<CheckPin>,
    ) -> Result<InstructionResult<()>, AccountServerClientError> {
        let message: InstructionResultMessage<()> = self
            .send_json_post_request("instructions/check_pin", &instruction)
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

    async fn create_mock_server_and_client() -> (MockServer, RemoteAccountServerClient) {
        let server = MockServer::start().await;
        let client =
            RemoteAccountServerClient::new(Url::parse(&server.uri()).expect("Could not parse mock server URI"));

        (server, client)
    }

    async fn post_example_request(
        client: &RemoteAccountServerClient,
        path: &str,
    ) -> Result<ExampleBody, AccountServerClientError> {
        let request = client
            .client
            .post(client.url(path).expect("Could not build URL"))
            .build()
            .expect("Could not create request");

        client.send_json_request::<ExampleBody>(request).await
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_ok() {
        let (server, client) = create_mock_server_and_client().await;

        Mock::given(method("POST"))
            .and(path("/foobar"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ExampleBody {
                foo: "blah".to_string(),
                bar: 1234,
            }))
            .expect(1)
            .mount(&server)
            .await;

        let body = post_example_request(&client, "foobar")
            .await
            .expect("Could not get succesful response from server");

        assert_eq!(body.foo, "blah");
        assert_eq!(body.bar, 1234);
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_error_status() {
        let (server, client) = create_mock_server_and_client().await;

        Mock::given(method("POST"))
            .and(path("/foobar_404"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let error = post_example_request(&client, "foobar_404")
            .await
            .expect_err("No error received from server");

        assert!(matches!(
            error,
            AccountServerClientError::Response(AccountServerResponseError::Status(StatusCode::NOT_FOUND))
        ))
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_error_text() {
        let (server, client) = create_mock_server_and_client().await;

        Mock::given(method("POST"))
            .and(path("/foobar_502"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Your gateway is bad and you should feel bad!"))
            .expect(1)
            .mount(&server)
            .await;

        let error = post_example_request(&client, "foobar_502")
            .await
            .expect_err("No error received from server");

        assert!(matches!(
            error,
            AccountServerClientError::Response(AccountServerResponseError::Text(StatusCode::BAD_GATEWAY, body))
       if body == "Your gateway is bad and you should feel bad!"))
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_error_data() {
        let (server, client) = create_mock_server_and_client().await;

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

        let error = post_example_request(&client, "foobar_400")
            .await
            .expect_err("No error received from server");

        if let AccountServerClientError::Response(AccountServerResponseError::Data(StatusCode::BAD_REQUEST, data)) =
            error
        {
            assert!(matches!(data.typ, ErrorType::ChallengeValidation));
            assert_eq!(data.title, "Error title");
        }
    }

    #[tokio::test]
    async fn test_remote_account_server_client_send_json_request_other_json() {
        let (server, client) = create_mock_server_and_client().await;

        Mock::given(method("POST"))
            .and(path("/foobar_503"))
            .respond_with(ResponseTemplate::new(503).set_body_json(json!({
                "status": "503",
                "text": "Service Unavailable",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let error = post_example_request(&client, "foobar_503")
            .await
            .expect_err("No error received from server");

        assert!(matches!(
            error,
            AccountServerClientError::Response(AccountServerResponseError::Status(StatusCode::SERVICE_UNAVAILABLE))
        ));
    }
}
