pub mod hardware;

#[cfg(feature = "mock_hw_keystore")]
pub mod mock;
#[cfg(any(all(feature = "mock_hw_keystore", test), feature = "hardware_integration_test"))]
pub mod test;

use std::error::Error;

use wallet_common::keys::SecureEcdsaKey;
use wallet_common::keys::SecureEncryptionKey;
use wallet_common::keys::WithIdentifier;

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

/// This trait is implemented on keys that are stored in a particular backing store,
/// such as Android's TEE/StrongBox or Apple's SE. These keys can be constructed by
/// an identifier, with the guarantee that only one instance can exist per identifier
/// in the entire process. If the key exists within the backing store, it will be
/// retrieved on first use, otherwise a random key will be created.
///
/// The key can be deleted from the backing store by a method that consumes the type.
/// If the type is simply dropped, it will remain in the backing store.
///
/// The limitation of having only one instance per identifier codifies that there is
/// only ever one owner of this key. If multiple instances with the same identifier
/// could be created, this could lead to undefined behaviour when the owner of one
/// of the types deletes the backing store key.
///
/// NB: Any type that implements `StoredByIdentifier` should probably not implement
///     `Clone`, as this would circumvent the uniqueness of the instance.
pub trait StoredByIdentifier: WithIdentifier {
    type Error: Error + Send + Sync + 'static;

    /// Creates a unique instance with the specified identifier. If an instance
    /// already exist with this identifier, `None` will be returned.
    fn new_unique(identifier: &str) -> Option<Self>
    where
        Self: Sized;

    /// Delete the key from the backing store and consume the type.
    async fn delete(self) -> Result<(), Self::Error>;
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
