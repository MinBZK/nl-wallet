use async_trait::async_trait;

use wallet_common::account::{
    auth::{Registration, WalletCertificate},
    signed::SignedDouble,
};
use wallet_provider::account_server::{AccountServer, ChallengeError, RegistrationError};

use super::AccountServerClient;

/// Combines [`AccountServer`] method errors into one type.
#[derive(Debug, thiserror::Error)]
pub enum MockAccountServerClientError {
    #[error(transparent)]
    Challenge(#[from] ChallengeError),
    #[error(transparent)]
    Registration(#[from] RegistrationError),
}

/// Mock implementation of [`AccountServerClient`] that is bound directly to
/// [`AccountServer`], skipping JSON encoding and HTTP(S).
#[async_trait]
impl AccountServerClient for AccountServer {
    type Error = MockAccountServerClientError;

    async fn registration_challenge(&self) -> Result<Vec<u8>, Self::Error> {
        let challenge = AccountServer::registration_challenge(self)?;

        Ok(challenge)
    }

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, Self::Error> {
        let cert = AccountServer::register(self, registration_message)?;

        Ok(cert)
    }
}
