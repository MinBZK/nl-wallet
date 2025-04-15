use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::LazyLock;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use parking_lot::Mutex;

use crypto::keys::EcdsaKey;
use crypto::keys::EncryptionKey;
use crypto::keys::SecureEcdsaKey;
use crypto::keys::SecureEncryptionKey;
use crypto::keys::WithIdentifier;
use utils::spawn;

use crate::bridge::hw_keystore::get_encryption_key_bridge;
use crate::bridge::hw_keystore::get_signing_key_bridge;

use super::HardwareKeyStoreError;
use super::KeyStoreError;
use super::PlatformEcdsaKey;
use super::PlatformEncryptionKey;
use super::StoredByIdentifier;

/// A static hash map of sets that contains all the identifiers for which an instance
/// of that type currently exists within the application, keyed by the type's `TypeId`.
static UNIQUE_IDENTIFIERS: LazyLock<Mutex<HashMap<TypeId, HashSet<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl From<KeyStoreError> for p256::ecdsa::Error {
    // Wrap KeyStoreError in `p256::ecdsa::Error`,
    // as try_sign() has the latter as error type.
    fn from(value: KeyStoreError) -> Self {
        p256::ecdsa::Error::from_source(value)
    }
}

/// Helper type that encapsulates the behaviour of a unique key
/// with a particular identifier and `TypeId`.
struct UniqueKey {
    identifier: String,
    type_id: TypeId,
}

impl UniqueKey {
    fn new(type_id: TypeId, identifier: &str) -> Option<Self> {
        let mut key_identifiers = UNIQUE_IDENTIFIERS.lock();
        let identifiers = key_identifiers.entry(type_id).or_default();

        // If this identifier exists within the `HashSet` for the `TypeId`, return None.
        // Otherwise, claim the identifier by inserting it into the `HashSet` and return
        // a new `UniqueKey`.
        (!identifiers.contains(identifier)).then(|| {
            let identifier = identifier.to_string();
            identifiers.insert(identifier.clone());

            UniqueKey { type_id, identifier }
        })
    }
}

impl Drop for UniqueKey {
    // Remove our entry from the static `HashSet`.
    fn drop(&mut self) {
        UNIQUE_IDENTIFIERS
            .lock()
            .get_mut(&self.type_id)
            .map(|identifiers| identifiers.remove(&self.identifier));
    }
}

// HardwareSigningKey wraps SigningKeyBridge from native.
pub struct HardwareEcdsaKey {
    unique_key: UniqueKey,
}

impl EcdsaKey for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let identifier = self.unique_key.identifier.clone();

        spawn::blocking(|| {
            let public_key_bytes = get_signing_key_bridge().public_key(identifier)?;
            let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

            Ok::<_, Self::Error>(public_key)
        })
        .await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let identifier = self.unique_key.identifier.clone();
        let payload = msg.to_vec();

        let signature_bytes = spawn::blocking(|| get_signing_key_bridge().sign(identifier, payload)).await?;

        // Decode the DER encoded signature.
        Ok(Signature::from_der(&signature_bytes)?)
    }
}

impl SecureEcdsaKey for HardwareEcdsaKey {}

impl WithIdentifier for HardwareEcdsaKey {
    fn identifier(&self) -> &str {
        &self.unique_key.identifier
    }
}

impl StoredByIdentifier for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    fn new_unique(identifier: &str) -> Option<Self> {
        UniqueKey::new(TypeId::of::<Self>(), identifier).map(|unique_key| HardwareEcdsaKey { unique_key })
    }

    async fn delete(self) -> Result<(), Self::Error> {
        // Clone the identifier, as `UniqueKey` implements `Drop`.
        // Note that this `Drop` implementation will remove the identifier from `UNIQUE_IDENTIFIERS`.
        let identifier = self.unique_key.identifier.clone();
        spawn::blocking(|| get_signing_key_bridge().delete(identifier)).await?;

        Ok(())
    }
}

impl PlatformEcdsaKey for HardwareEcdsaKey {}

// HardwareEncryptionKey wraps EncryptionKeyBridge from native.
pub struct HardwareEncryptionKey {
    unique_key: UniqueKey,
}

impl WithIdentifier for HardwareEncryptionKey {
    fn identifier(&self) -> &str {
        &self.unique_key.identifier
    }
}

impl StoredByIdentifier for HardwareEncryptionKey {
    type Error = HardwareKeyStoreError;

    fn new_unique(identifier: &str) -> Option<Self> {
        UniqueKey::new(TypeId::of::<Self>(), identifier).map(|unique_key| HardwareEncryptionKey { unique_key })
    }

    async fn delete(self) -> Result<(), Self::Error> {
        // Clone the identifier, as `UniqueKey` implements `Drop`.
        // Note that this `Drop` implementation will remove the identifier from `UNIQUE_IDENTIFIERS`.
        let identifier = self.unique_key.identifier.clone();
        spawn::blocking(|| get_encryption_key_bridge().delete(identifier)).await?;

        Ok(())
    }
}

impl EncryptionKey for HardwareEncryptionKey {
    type Error = HardwareKeyStoreError;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let identifier = self.unique_key.identifier.clone();
        let payload = msg.to_vec();
        let encrypted = spawn::blocking(|| get_encryption_key_bridge().encrypt(identifier, payload)).await?;
        Ok(encrypted)
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let identifier = self.unique_key.identifier.clone();
        let payload = msg.to_vec();
        let decrypted = spawn::blocking(|| get_encryption_key_bridge().decrypt(identifier, payload)).await?;
        Ok(decrypted)
    }
}

impl SecureEncryptionKey for HardwareEncryptionKey {}

impl PlatformEncryptionKey for HardwareEncryptionKey {}
