use std::{borrow::BorrowMut, collections::HashSet};

use once_cell::sync::Lazy;
use p256::{
    ecdsa::{Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};

use parking_lot::Mutex;
use wallet_common::{
    keys::{EcdsaKey, SecureEcdsaKey, SecureEncryptionKey, StoredByIdentifier, WithIdentifier},
    spawn,
};

use crate::bridge::hw_keystore::{get_encryption_key_bridge, get_signing_key_bridge};

use super::{HardwareKeyStoreError, KeyStoreError, PlatformEcdsaKey, PlatformEncryptionKey};

/// A static set that contains all the identifiers for `HardwareEcdsaKey` that are currently in use.
static ECDSA_KEY_IDENTIFIERS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
/// A static set that contains all the identifiers for `HardwareEncryptionKey` that are currently in use.
static ENCRYPTION_KEY_IDENTIFIERS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// This helper function claims a particular identifier by inserting it into a `HashSet` and
/// returning `true`, If the set already contains the identifier, it will return `false`.
fn claim_unique_identifier(identifiers: &mut HashSet<String>, identifier: &str) -> bool {
    let can_claim = !identifiers.contains(identifier);

    if can_claim {
        identifiers.insert(identifier.to_string());
    }

    can_claim
}

impl From<KeyStoreError> for p256::ecdsa::Error {
    // Wrap KeyStoreError in `p256::ecdsa::signature::error`,
    // as try_sign() has the latter as error type.
    fn from(value: KeyStoreError) -> Self {
        p256::ecdsa::Error::from_source(value)
    }
}

// HardwareSigningKey wraps SigningKeyBridge from native
#[derive(Clone)]
pub struct HardwareEcdsaKey {
    identifier: String,
}

impl Drop for HardwareEcdsaKey {
    // Remove our entry from the static `HashSet`.
    fn drop(&mut self) {
        ECDSA_KEY_IDENTIFIERS.lock().remove(&self.identifier);
    }
}

impl EcdsaKey for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let identifier = self.identifier.clone();

        spawn::blocking(|| {
            let public_key_bytes = get_signing_key_bridge().public_key(identifier)?;
            let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

            Ok::<_, Self::Error>(public_key)
        })
        .await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let identifier = self.identifier.clone();
        let payload = msg.to_vec();

        let signature_bytes = spawn::blocking(|| get_signing_key_bridge().sign(identifier, payload)).await?;

        // decode the DER encoded signature
        Ok(Signature::from_der(&signature_bytes)?)
    }
}

impl SecureEcdsaKey for HardwareEcdsaKey {}

impl WithIdentifier for HardwareEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl StoredByIdentifier for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    fn new_unique(identifier: &str) -> Option<Self> {
        // Only return a new `HardwareEcdsaKey` if we can claim the identifier.
        claim_unique_identifier(ECDSA_KEY_IDENTIFIERS.lock().borrow_mut(), identifier).then(|| HardwareEcdsaKey {
            identifier: identifier.to_string(),
        })
    }

    async fn delete(self) -> Result<(), Self::Error> {
        // Clone the identifier, as this type implements `Drop`.
        let identifier = self.identifier.clone();
        spawn::blocking(|| get_signing_key_bridge().delete(identifier)).await?;

        Ok(())
    }
}

impl PlatformEcdsaKey for HardwareEcdsaKey {}

// HardwareEncryptionKey wraps EncryptionKeyBridge from native
#[derive(Clone)]
pub struct HardwareEncryptionKey {
    identifier: String,
}

impl Drop for HardwareEncryptionKey {
    // Remove our entry from the static `HashSet`.
    fn drop(&mut self) {
        ENCRYPTION_KEY_IDENTIFIERS.lock().remove(&self.identifier);
    }
}

impl WithIdentifier for HardwareEncryptionKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl StoredByIdentifier for HardwareEncryptionKey {
    type Error = HardwareKeyStoreError;

    fn new_unique(identifier: &str) -> Option<Self> {
        // Only return a new `HardwareEncryptionKey` if we can claim the identifier.
        claim_unique_identifier(ENCRYPTION_KEY_IDENTIFIERS.lock().borrow_mut(), identifier).then(|| {
            HardwareEncryptionKey {
                identifier: identifier.to_string(),
            }
        })
    }

    async fn delete(self) -> Result<(), Self::Error> {
        // Clone the identifier, as this type implements `Drop`.
        let identifier = self.identifier.clone();
        spawn::blocking(|| get_encryption_key_bridge().delete(identifier)).await?;

        Ok(())
    }
}

impl SecureEncryptionKey for HardwareEncryptionKey {
    type Error = HardwareKeyStoreError;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let identifier = self.identifier.clone();
        let payload = msg.to_vec();
        let encrypted = spawn::blocking(|| get_encryption_key_bridge().encrypt(identifier, payload)).await?;
        Ok(encrypted)
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let identifier = self.identifier.clone();
        let payload = msg.to_vec();
        let decrypted = spawn::blocking(|| get_encryption_key_bridge().decrypt(identifier, payload)).await?;
        Ok(decrypted)
    }
}

impl PlatformEncryptionKey for HardwareEncryptionKey {}
