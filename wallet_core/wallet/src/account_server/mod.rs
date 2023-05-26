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

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AccountServerClientError(#[from] pub Box<dyn Error + Send + Sync>);

#[async_trait]
pub trait AccountServerClient {
    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError>;
    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError>;
}
