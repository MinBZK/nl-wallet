pub mod remote;

use anyhow::Result;
use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};

pub trait AccountServerClient {
    fn registration_challenge(&self) -> Result<Vec<u8>>;
    fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate>;
}

#[cfg(test)]
mod mock {
    use wallet_provider::account_server::AccountServer;

    use super::*;

    impl AccountServerClient for AccountServer {
        fn registration_challenge(&self) -> Result<Vec<u8>> {
            AccountServer::registration_challenge(self)
        }
        fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate> {
            AccountServer::register(self, registration_message)
        }
    }
}
