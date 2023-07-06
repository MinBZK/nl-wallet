use std::time::Duration;

use async_trait::async_trait;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Request,
};
use serde::de::DeserializeOwned;
use url::{ParseError, Url};

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    signed::SignedDouble,
};

use super::{AccountServerClient, AccountServerClientError};

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

    async fn send_json_request<T>(&self, request: Request) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let body = self.client.execute(request).await?.error_for_status()?.json().await?;

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
