use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{self, Debug},
    sync::{Arc, LazyLock},
};

use aes_gcm::{aead::KeyInit, Aes256Gcm};
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use parking_lot::Mutex;
use rand_core::OsRng;

use crate::keys::WithIdentifier;

use super::{EcdsaKey, EncryptionKey, SecureEcdsaKey, SecureEncryptionKey, StoredByIdentifier};

// Static for storing identifier to signing key mapping.
static SIGNING_KEYS: LazyLock<Mutex<HashMap<String, Arc<SigningKey>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
// Static for storing identifier to AES cipher mapping.
static ENCRYPTION_CIPHERS: LazyLock<Mutex<HashMap<String, Arc<Aes256Gcm>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct SoftwareEcdsaKey {
    identifier: String,
    key: Arc<SigningKey>,
}

impl SoftwareEcdsaKey {
    pub fn new(identifier: String, key: SigningKey) -> Self {
        SoftwareEcdsaKey {
            identifier,
            key: key.into(),
        }
    }

    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, SigningKey::random(&mut OsRng))
    }

    // Peek into the static hashmap to see if an instance of
    // `SoftwareEcdsaKey` with the specified identifier exists.
    pub fn identifier_exists(identifier: &str) -> bool {
        SIGNING_KEYS.lock().contains_key(identifier)
    }
}

impl Debug for SoftwareEcdsaKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftwareEcdsaKey")
            .field("identifier", &self.identifier)
            .finish_non_exhaustive()
    }
}

impl EcdsaKey for SoftwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let key = self.key.verifying_key();

        Ok(*key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Signer::try_sign(self.key.as_ref(), msg)
    }
}
impl SecureEcdsaKey for SoftwareEcdsaKey {}

impl WithIdentifier for SoftwareEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl StoredByIdentifier for SoftwareEcdsaKey {
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

        Some(SoftwareEcdsaKey {
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

pub struct SoftwareEncryptionKey {
    identifier: String,
    cipher: Arc<Aes256Gcm>,
}

impl SoftwareEncryptionKey {
    pub fn new(identifier: String, cipher: Aes256Gcm) -> Self {
        SoftwareEncryptionKey {
            identifier,
            cipher: cipher.into(),
        }
    }

    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng)))
    }

    // Peek into the static hashmap to see if an instance of
    // `SoftwareEncryptionKey` with the specified identifier exists.
    pub fn identifier_exists(identifier: &str) -> bool {
        ENCRYPTION_CIPHERS.lock().contains_key(identifier)
    }
}

impl Debug for SoftwareEncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftwareEncryptionKey")
            .field("identifier", &self.identifier)
            .finish_non_exhaustive()
    }
}

impl EncryptionKey for SoftwareEncryptionKey {
    type Error = <Aes256Gcm as EncryptionKey>::Error;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        <Aes256Gcm as EncryptionKey>::encrypt(&self.cipher, msg).await
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        <Aes256Gcm as EncryptionKey>::decrypt(&self.cipher, msg).await
    }
}

impl SecureEncryptionKey for SoftwareEncryptionKey {}

impl WithIdentifier for SoftwareEncryptionKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl StoredByIdentifier for SoftwareEncryptionKey {
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

        Some(SoftwareEncryptionKey {
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

#[cfg(test)]
mod tests {
    use crate::keys::{
        software::{SoftwareEcdsaKey, SoftwareEncryptionKey},
        test,
    };

    #[tokio::test]
    async fn test_software_signature() {
        let payload = b"This is a message that will be signed.";
        let identifier = "test_software_signature";

        test::sign_and_verify_signature::<SoftwareEcdsaKey>(payload, identifier).await;
    }

    #[tokio::test]
    async fn test_software_encryption() {
        let payload = b"This message will be encrypted.";
        let identifier = "test_software_encryption";

        test::encrypt_and_decrypt_message::<SoftwareEncryptionKey>(payload, identifier).await;
    }
}
