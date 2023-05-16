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

pub trait ConstructableWithIdentifier {
    fn new(identifier: &str) -> Self
    where
        Self: Sized;

    fn identifier(&self) -> &str;
}

/// Contract for ECDSA private keys suitable for use in the wallet, e.g. as the authentication key for the WP.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`ConstructableWithIdentifier::new()`].
pub trait PlatformEcdsaKey: ConstructableWithIdentifier + SecureEcdsaKey {
    // from ConstructableWithIdentifier: new(), identifier()
    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

/// Contract for encryption keys suitable for use in the wallet, e.g. for securely storing the database key.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`ConstructableWithIdentifier::new()`].
pub trait PlatformEncryptionKey: ConstructableWithIdentifier {
    // from ConstructableWithIdentifier: new(), identifier()

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError>;
    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError>;
}
