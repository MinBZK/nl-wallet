mod remote;

#[cfg(test)]
mod mock;

use std::error::Error;

use async_trait::async_trait;

use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};

pub use self::remote::RemoteAccountServerClient;

#[async_trait]
pub trait AccountServerClient {
    type Error: Error + Send + Sync + 'static;

    async fn registration_challenge(&self) -> Result<Vec<u8>, Self::Error>;
    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, Self::Error>;
}
