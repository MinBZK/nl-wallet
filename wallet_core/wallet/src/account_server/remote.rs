use async_trait::async_trait;
use reqwest::Client;

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    signed::SignedDouble,
};

use super::{AccountServerClient, AccountServerClientError};

pub struct RemoteAccountServerClient {
    url: String,
    client: Client,
}

impl RemoteAccountServerClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AccountServerClient for RemoteAccountServerClient {
    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        let challenge = self
            .client
            .post(format!("{}/api/v1/enroll", self.url))
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
            .post(format!("{}/api/v1/createwallet", self.url))
            .json(&registration_message)
            .send()
            .await?
            .json::<Certificate>()
            .await?
            .certificate;

        Ok(cert)
    }
}
