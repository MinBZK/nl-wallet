#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

use thiserror::Error;
use wallet_common::account::signing_key::SecureEcdsaKey;

#[derive(Debug, Error)]
pub enum HardwareKeyStoreError {
    #[error(transparent)]
    KeyStoreError(#[from] KeyStoreError),
    #[error("Error decoding public key from hardware: {0}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
}

// implementation of KeyStoreError from UDL, only with "hardware" flag
#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("Key error: {reason}")]
    KeyError { reason: String },
    #[error("Bridging error: {reason}")]
    BridgingError { reason: String },
}

/// Contract for ECDSA private keys suitable for use in the wallet, as the authentication key for the WP.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`PlatformSigningKey::signing_key()`].
pub trait PlatformEcdsaKey: SecureEcdsaKey {
    fn signing_key(identifier: &str) -> Result<Self, HardwareKeyStoreError>
    where
        Self: Sized;

    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

pub trait PlatformEncryptionKey {
    fn encryption_key(identifier: &str) -> Result<Self, HardwareKeyStoreError>
    where
        Self: Sized;

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError>;
    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError>;
}
