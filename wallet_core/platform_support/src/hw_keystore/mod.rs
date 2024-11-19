pub mod hardware;

use wallet_common::keys::SecureEcdsaKey;
use wallet_common::keys::SecureEncryptionKey;
use wallet_common::keys::StoredByIdentifier;

pub use crate::bridge::hw_keystore::KeyStoreError;

#[derive(Debug, thiserror::Error)]
pub enum HardwareKeyStoreError {
    #[error(transparent)]
    KeyStoreError(#[from] KeyStoreError),
    #[error("error decoding public key from hardware: {0}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
    #[error("error signing with hardware key: {0}")]
    SigningError(#[from] p256::ecdsa::Error),
}

/// Contract for ECDSA private keys suitable for use in the wallet, e.g. as the authentication key for the WP.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`StoredByIdentifier::new_unique()`].
pub trait PlatformEcdsaKey: StoredByIdentifier + SecureEcdsaKey {
    // from StoredByIdentifier: new_unique(), delete(), identifier()
    // from EcdsaKey: verifying_key(), try_sign(), sign()
}

pub trait PlatformEncryptionKey: StoredByIdentifier + SecureEncryptionKey {
    // from StoredByIdentifier: new_unique(), delete(), identifier()
    // from EncryptionKey: encrypt(), decrypt()
}

#[cfg(feature = "mock")]
mod software {
    use wallet_common::keys::mock_hardware::MockHardwareEcdsaKey;
    use wallet_common::keys::mock_hardware::MockHardwareEncryptionKey;

    use super::PlatformEcdsaKey;
    use super::PlatformEncryptionKey;

    impl PlatformEcdsaKey for MockHardwareEcdsaKey {}

    impl PlatformEncryptionKey for MockHardwareEncryptionKey {}
}
