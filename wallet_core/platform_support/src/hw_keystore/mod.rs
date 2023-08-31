pub mod hardware;

use wallet_common::keys::{ConstructableWithIdentifier, SecureEcdsaKey};

#[derive(Debug, thiserror::Error)]
pub enum HardwareKeyStoreError {
    #[error(transparent)]
    KeyStoreError(#[from] KeyStoreError),
    #[error("error decoding public key from hardware: {0}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
    #[error("error signing with hardware key: {0}")]
    SigningError(#[from] p256::ecdsa::Error),
}

// implementation of KeyStoreError from UDL
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("key error: {reason}")]
    KeyError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

/// Contract for ECDSA private keys suitable for use in the wallet, e.g. as the authentication key for the WP.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`ConstructableWithIdentifier::new()`].
pub trait PlatformEcdsaKey: ConstructableWithIdentifier + SecureEcdsaKey {
    // from ConstructableWithIdentifier: new(), identifier()
    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

#[cfg(feature = "software")]
impl PlatformEcdsaKey for wallet_common::keys::software::SoftwareEcdsaKey {}
