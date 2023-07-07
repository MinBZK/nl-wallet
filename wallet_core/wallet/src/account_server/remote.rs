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
