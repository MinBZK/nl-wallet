use async_trait::async_trait;

use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};
use wallet_provider::account_server::AccountServer;

use super::{AccountServerClient, AccountServerClientError};

/// Mock implementation of [`AccountServerClient`] that is bound directly to
/// [`AccountServer`], skipping JSON encoding and HTTP(S).
#[async_trait]
impl AccountServerClient for AccountServer {
    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError> {
        AccountServer::registration_challenge(self).map_err(|e| AccountServerClientError(e.into()))
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError> {
        AccountServer::register(self, registration_message).map_err(|e| AccountServerClientError(e.into()))
    }
}
