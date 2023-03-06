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

    fn create_key(&mut self, identifier: &str) -> Result<&mut Self::SigningKeyType, Error>;
    fn get_key(&self, identifier: &str) -> Option<&Self::SigningKeyType>;
}

pub trait SigningKey: Signer<Signature> {
    fn verifying_key(&self) -> Result<&VerifyingKey, Error>;
    // from Signer: try_sign() and sign() methods
}
