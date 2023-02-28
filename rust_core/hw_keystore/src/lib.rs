#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

#[cfg(feature = "integration-test")]
pub mod integration_test;

use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "hardware")]
    #[error(transparent)]
    KeyStoreError(#[from] hardware::KeyStoreError),
    #[cfg(feature = "hardware")]
    #[error(transparent)]
    PKCS8Error(#[from] p256::pkcs8::spki::Error),
}

pub trait KeyStore {
    type SigningKeyType: SigningKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Result<Self::SigningKeyType, Error>;
}

pub trait SigningKey: Signer<Signature> {
    fn verifying_key(&self) -> Result<VerifyingKey, Error>;
}
