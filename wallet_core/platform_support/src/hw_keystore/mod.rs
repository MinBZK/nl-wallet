pub mod hardware;

use wallet_common::keys::{SecureEcdsaKey, SecureEncryptionKey, StoredByIdentifier};

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
/// Handles to private keys are requested through [`ConstructibleWithIdentifier::new()`].
pub trait PlatformEcdsaKey: StoredByIdentifier + SecureEcdsaKey {
    // from StoredByIdentifier: new_unique(), delete(), identifier()
    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

pub trait PlatformEncryptionKey: StoredByIdentifier + SecureEncryptionKey {
    // from StoredByIdentifier: new_unique(), delete(), identifier()
    // from SecureEncryptionKey: encrypt(), decrypt()
}

#[cfg(feature = "software")]
mod software {
    use wallet_common::keys::software::{SoftwareEcdsaKey, SoftwareEncryptionKey};

    use super::{PlatformEcdsaKey, PlatformEncryptionKey};

    impl PlatformEcdsaKey for SoftwareEcdsaKey {}

    impl PlatformEncryptionKey for SoftwareEncryptionKey {}
}
