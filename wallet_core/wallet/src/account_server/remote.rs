use async_trait::async_trait;
use reqwest::Client;
use url::Url;

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    signed::SignedDouble,
};

use super::{AccountServerClient, AccountServerClientError};

pub struct RemoteAccountServerClient {
    base_url: Url,
    client: Client,
}

impl RemoteAccountServerClient {
    fn new(base_url: Url) -> Self {
        RemoteAccountServerClient {
            base_url,
            client: Client::new(),
        }
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
        let challenge = self
            .client
            .post(self.base_url.join("enroll")?)
            .send()
            .await?
            .json::<Challenge>()
            .await?
            .challenge
            .0;

        Ok(challenge)
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        let cert = self
            .client
            .post(self.base_url.join("createwallet")?)
            .json(&registration_message)
            .send()
            .await?
            .json::<Certificate>()
            .await?
            .certificate;

        Ok(cert)
    }
}
