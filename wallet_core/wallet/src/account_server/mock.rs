use async_trait::async_trait;
use url::Url;

use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};
use wallet_provider::{
    stub::{self, TestDeps},
    AccountServer,
};

use super::{AccountServerClient, AccountServerClientError};

/// Mock implementation of [`AccountServerClient`] that is bound directly to
/// [`AccountServer`], skipping JSON encoding and HTTP(S).
#[async_trait]
impl AccountServerClient for AccountServer {
    fn new(_: &Url) -> Self
    where
        Self: Sized,
    {
        stub::account_server()
    }

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        AccountServer::registration_challenge(self).map_err(|e| AccountServerClientError::Other(e.into()))
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        AccountServer::register(self, &TestDeps, registration_message)
            .await
            .map_err(|e| AccountServerClientError::Other(e.into()))
    }
}
