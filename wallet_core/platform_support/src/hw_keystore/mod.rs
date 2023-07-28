#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

use std::error::Error;

use wallet_common::account::signing_key::SecureEcdsaKey;

#[derive(Debug, thiserror::Error)]
pub enum HardwareKeyStoreError {
    #[error(transparent)]
    KeyStoreError(#[from] KeyStoreError),
    #[error("error decoding public key from hardware: {0}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
}

// implementation of KeyStoreError from UDL, only with "hardware" flag
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("key error: {reason}")]
    KeyError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

/// The contract of this trait includes that a constructed type with the same
/// identifier behaves exactly the same, i.e. has the same key material backing it.
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
    type Error: Error + Send + Sync + 'static;

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
