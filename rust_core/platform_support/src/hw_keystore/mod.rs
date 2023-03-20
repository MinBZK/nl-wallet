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
    #[error("Error decoding public key from hardware: {0:?}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
}
pub trait PlatformSigningKey: Signer<Signature> {
    fn signing_key(identifier: &str) -> Result<Self, Error>
    where
        Self: Sized;

    fn verifying_key(&self) -> Result<&VerifyingKey, Error>;
    // from Signer: try_sign() and sign() methods
}
