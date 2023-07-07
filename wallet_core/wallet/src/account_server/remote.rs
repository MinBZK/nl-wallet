use std::time::Duration;

use async_trait::async_trait;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Request,
};
use serde::de::DeserializeOwned;
use url::{ParseError, Url};

use wallet_common::account::{
    messages::{
        auth::{Certificate, Challenge, Registration, WalletCertificate},
        errors::ErrorData,
    },
    signed::SignedDouble,
};

use super::{AccountServerClient, AccountServerClientError, AccountServerResponseError};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct RemoteAccountServerClient {
    base_url: Url,
    client: Client,
}

impl RemoteAccountServerClient {
    fn new(base_url: Url) -> Self {
        let client = Client::builder()
            .timeout(CLIENT_TIMEOUT)
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static("application/json"),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        RemoteAccountServerClient { base_url, client }
    }

    fn url(&self, path: &str) -> Result<Url, ParseError> {
        self.base_url.join(path)
    }

    async fn send_json_request<T>(&self, request: Request) -> Result<T, AccountServerClientError>
    where
        T: DeserializeOwned,
    {
        let response = self.client.execute(request).await?;
        let status = response.status();

        // In case of a 4xx or 5xx response...
        if status.is_client_error() || status.is_server_error() {
            // ...try to get the response body as a string with the appropriate encoding.
            // If that doesn't work or the body is empty, just wrap the status code in an error.
            let error = response.text().await.ok().filter(|text| !text.is_empty()).map_or_else(
                || AccountServerResponseError::Status(status),
                |text| {
                    // If it does work, try to decode the body as an ErrorData struct in order to wrap
                    // that data in an error along with the status code. Otherwise, fall back to just
                    // wrapping the body text in an error, again with the status code.
                    serde_json::from_str::<ErrorData>(&text).ok().map_or_else(
                        || AccountServerResponseError::Text(status, text),
                        |error_data| AccountServerResponseError::Data(status, error_data),
                    )
                },
            );

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
        let request = self
            .client
            .post(self.url("createwallet")?)
            .json(&registration_message)
            .build()?;
        let certificate = self.send_json_request::<Certificate>(request).await?.certificate;

        Ok(certificate)
    }
}

#[cfg(test)]
/// Ceci n'est pas une unit test.
///
/// This test sets up a mock HTTP server and by definition also tests the `reqwest` dependency.
/// Its goal is mostly to validate that HTTP error responses get converted to the right variant
/// of `RemoteAccountServerClient` and `AccountServerResponseError`.
mod tests {
    use std::collections::HashMap;

    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use wallet_common::account::messages::errors::{DataValue, ErrorType};

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
            .respond_with(ResponseTemplate::new(400).set_body_json(ErrorData {
                typ: ErrorType::ChallengeValidation,
                title: "Error title".to_string(),
                data: Some(HashMap::from([
                    ("foo".to_string(), DataValue::String("bar".to_string())),
                    ("bleh".to_string(), DataValue::String("blah".to_string())),
                ])),
            }))
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

            if let Some(data) = data.data {
                assert!(matches!(data.get("foo"), Some(DataValue::String(string)) if string == "bar"));
                assert!(matches!(data.get("bleh"), Some(DataValue::String(string)) if string == "blah"));
            } else {
                panic!("Error has no additional data")
            }
        } else {
            panic!("No error data received")
        }
    }
}
