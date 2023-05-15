pub mod remote;

use anyhow::Result;
use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};

#[async_trait::async_trait]
pub trait AccountServerClient {
    async fn registration_challenge(&self) -> Result<Vec<u8>>;
    async fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate>;
}

#[cfg(test)]
mod mock {
    use wallet_provider::account_server::AccountServer;

    use super::*;

    #[async_trait::async_trait]
    impl AccountServerClient for AccountServer {
        async fn registration_challenge(&self) -> Result<Vec<u8>> {
            AccountServer::registration_challenge(self)
        }

        async fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate> {
            AccountServer::register(self, registration_message)
        }
    }
}
