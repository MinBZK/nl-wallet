use anyhow::Result;
use reqwest::Client;
use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    signed::SignedDouble,
};

use super::AccountServerClient;

pub struct RemoteAccountServer {
    url: String,
    client: Client,
}

impl RemoteAccountServer {
    pub fn new(url: String) -> RemoteAccountServer {
        RemoteAccountServer {
            url,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AccountServerClient for RemoteAccountServer {
    async fn registration_challenge(&self) -> Result<Vec<u8>> {
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

    async fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate> {
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
