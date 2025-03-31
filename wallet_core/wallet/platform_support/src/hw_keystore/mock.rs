use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::LazyLock;

use aes_gcm::aead::KeyInit;
use aes_gcm::Aes256Gcm;
use derive_more::Debug;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use parking_lot::Mutex;
use rand_core::OsRng;

use crypto::keys::EcdsaKey;
use crypto::keys::EncryptionKey;
use crypto::keys::SecureEcdsaKey;
use crypto::keys::SecureEncryptionKey;
use crypto::keys::WithIdentifier;

use super::PlatformEcdsaKey;
use super::PlatformEncryptionKey;
use super::StoredByIdentifier;

// Static for storing identifier to signing key mapping.
static SIGNING_KEYS: LazyLock<Mutex<HashMap<String, Arc<SigningKey>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
// Static for storing identifier to AES cipher mapping.
static ENCRYPTION_CIPHERS: LazyLock<Mutex<HashMap<String, Arc<Aes256Gcm>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// To be used in tests in place of `HardwareEcdsaKey`. It implements the [`EcdsaKey`],
/// [`SecureEcdsaKey`], [`WithIdentifier`] and [`StoredByIdentifier`] traits, mocking
/// the behaviour of keys that are stored in secure hardware on a device.
#[derive(Debug)]
pub struct MockHardwareEcdsaKey {
    identifier: String,
    #[debug(skip)]
    key: Arc<SigningKey>,
}

impl MockHardwareEcdsaKey {
    /// Peek into the static hashmap to see if an instance of
    /// [`MockHardwareEcdsaKey`] with the specified identifier exists.
    pub fn identifier_exists(identifier: &str) -> bool {
        SIGNING_KEYS.lock().contains_key(identifier)
    }
}

impl EcdsaKey for MockHardwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let key = self.key.verifying_key();

        Ok(*key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Signer::try_sign(self.key.as_ref(), msg)
    }
}
impl SecureEcdsaKey for MockHardwareEcdsaKey {}

impl WithIdentifier for MockHardwareEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl StoredByIdentifier for MockHardwareEcdsaKey {
    type Error = Infallible;

    fn new_unique(identifier: &str) -> Option<Self> {
        // Obtain lock on SIGNING_KEYS static hashmap.
        let mut signing_keys = SIGNING_KEYS.lock();

        // Retrieve the signing key from the static hashmap.
        let maybe_key = signing_keys.get(identifier);

        // If there is a key and it has a reference count of more than 1, this means
        // means an instance already exists out there and we should return `None`.
        if maybe_key.map(|key| Arc::strong_count(key) > 1).unwrap_or_default() {
            return None;
        }

        // Otherwise, increment the reference count or create a new random key
        // and insert it into the static hashmap.
        let key = maybe_key.map(Arc::clone).unwrap_or_else(|| {
            let signing_key = SigningKey::random(&mut OsRng).into();

            signing_keys.insert(identifier.to_string(), Arc::clone(&signing_key));

            signing_key
        });

        Some(MockHardwareEcdsaKey {
            key,
            identifier: identifier.to_string(),
        })
    }

    async fn delete(self) -> Result<(), Self::Error> {
        // Simply remove the signing key from the static hashmap, if present.
        SIGNING_KEYS.lock().remove(&self.identifier);

        Ok(())
    }
}

impl PlatformEcdsaKey for MockHardwareEcdsaKey {}

/// To be used in tests in place of `HardwareEncryptionKey`. It implements the [`EncryptionKey`],
/// [`SecureEncryptionKey`], [`WithIdentifier`] and [`StoredByIdentifier`] traits, mocking
/// the behaviour of keys that are stored in secure hardware on a device.
#[derive(Debug)]
pub struct MockHardwareEncryptionKey {
    identifier: String,
    #[debug(skip)]
    cipher: Arc<Aes256Gcm>,
}

impl MockHardwareEncryptionKey {
    pub fn new_random(identifier: String) -> Self {
        Self {
            identifier,
            cipher: Arc::new(Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng))),
        }
    }

    /// Peek into the static hashmap to see if an instance of
    /// [`MockHardwareEncryptionKey`] with the specified identifier exists.
    pub fn identifier_exists(identifier: &str) -> bool {
        ENCRYPTION_CIPHERS.lock().contains_key(identifier)
    }
}

impl EncryptionKey for MockHardwareEncryptionKey {
    type Error = <Aes256Gcm as EncryptionKey>::Error;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        <Aes256Gcm as EncryptionKey>::encrypt(&self.cipher, msg).await
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        <Aes256Gcm as EncryptionKey>::decrypt(&self.cipher, msg).await
    }
}

impl SecureEncryptionKey for MockHardwareEncryptionKey {}

impl WithIdentifier for MockHardwareEncryptionKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl StoredByIdentifier for MockHardwareEncryptionKey {
    type Error = Infallible;

    fn new_unique(identifier: &str) -> Option<Self> {
        // Obtain lock on ENCRYPTION_CIPHERS static hashmap.
        let mut encryption_ciphers = ENCRYPTION_CIPHERS.lock();

        // Retrieve the cipher from the static hashmap.
        let maybe_cipher = encryption_ciphers.get(identifier);

        // If there is a cipher and it has a reference count of more than 1, this means
        // means an instance already exists out there and we should return `None`.
        if maybe_cipher.map(|key| Arc::strong_count(key) > 1).unwrap_or_default() {
            return None;
        }

        // Otherwise, increment the reference count or create a new random cipher
        // and insert it into the static hashmap.
        let cipher = maybe_cipher.map(Arc::clone).unwrap_or_else(|| {
            let encryption_cipher = Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng)).into();

            encryption_ciphers.insert(identifier.to_string(), Arc::clone(&encryption_cipher));

            encryption_cipher
        });

        Some(MockHardwareEncryptionKey {
            cipher,
            identifier: identifier.to_string(),
        })
    }

    async fn delete(self) -> Result<(), Self::Error> {
        // Simply remove the encryption cipher from the static hashmap, if present.
        ENCRYPTION_CIPHERS.lock().remove(&self.identifier);

        Ok(())
    }
}

impl PlatformEncryptionKey for MockHardwareEncryptionKey {}

#[cfg(test)]
mod tests {
    use super::super::test;
    use super::MockHardwareEcdsaKey;
    use super::MockHardwareEncryptionKey;

    #[tokio::test]
    async fn test_mock_hardware_signature() {
        let payload = b"This is a message that will be signed.";
        let identifier = "test_mock_hardware_signature";

        test::sign_and_verify_signature::<MockHardwareEcdsaKey>(payload, identifier).await;
    }

    #[tokio::test]
    async fn test_mock_hardware_encryption() {
        let payload = b"This message will be encrypted.";
        let identifier = "test_mock_hardware_encryption";

        test::encrypt_and_decrypt_message::<MockHardwareEncryptionKey>(payload, identifier).await;
    }
}
