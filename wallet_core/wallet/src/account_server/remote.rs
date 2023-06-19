use async_trait::async_trait;
use reqwest::Client;
use url::Url;

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration, WalletCertificate},
    signed::SignedDouble,
};

use super::{AccountServerClient, AccountServerClientError};

pub trait AccountServerConfigurationProvider {
    fn base_url(&self) -> &Url;
}

pub struct RemoteAccountServerClient<'a, C> {
    config: &'a C,
    client: Client,
}

impl<'a, C> RemoteAccountServerClient<'a, C> {
    pub fn new(config: &'a C) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl<C> AccountServerClient for RemoteAccountServerClient<'_, C>
where
    C: AccountServerConfigurationProvider + Send + Sync,
{
    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        let challenge = self
            .client
            .post(self.config.base_url().join("/api/v1/enroll")?)
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
            .post(self.config.base_url().join("/api/v1/createwallet")?)
            .json(&registration_message)
            .send()
            .await?
            .json::<Certificate>()
            .await?
            .certificate;

        Ok(cert)
    }
}
