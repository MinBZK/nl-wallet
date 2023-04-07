#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

#[cfg(feature = "integration-test")]
pub mod integration_test;

use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HardwareKeyStoreError {
    #[error(transparent)]
    KeyStoreError(#[from] KeyStoreError),
    #[error("Error decoding public key from hardware: {0}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
}

// implementation of KeyStoreError from UDL
#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("Key error: {reason}")]
    KeyError { reason: String },
    #[error("Bridging error: {reason}")]
    BridgingError { reason: String },
}

pub trait PlatformSigningKey: Signer<Signature> {
    fn signing_key(identifier: &str) -> Result<Self, HardwareKeyStoreError>
    where
        Self: Sized;

    fn verifying_key(&self) -> Result<VerifyingKey, HardwareKeyStoreError>;
    // from Signer: try_sign() and sign() methods
}

// if the hardware feature is enabled, prefer HardwareSigningKey
#[cfg(feature = "hardware")]
pub type PreferredPlatformSigningKey = self::hardware::HardwareSigningKey;

// otherwise if the software feature is enabled, prefer SoftwareSigningKey
#[cfg(all(not(feature = "hardware"), feature = "software"))]
pub type PreferredPlatformSigningKey = self::software::SoftwareSigningKey;

// otherwise just just alias the Never type
#[cfg(not(any(feature = "hardware", feature = "software")))]
pub type PreferredPlatformSigningKey = never::Never;

pub trait PlatformEncryptionKey {
    fn encryption_key(identifier: &str) -> Result<Self, HardwareKeyStoreError>
    where
        Self: Sized;

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError>;

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError>;
}
