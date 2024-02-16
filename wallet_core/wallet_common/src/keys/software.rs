use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{self, Debug},
    sync::Mutex,
};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;

use crate::{keys::WithIdentifier, utils::random_bytes};

use super::{ConstructibleWithIdentifier, DeletableWithIdentifier, EcdsaKey, SecureEcdsaKey, SecureEncryptionKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// static for storing identifier -> aes cipher mapping, will only ever grow
static ENCRYPTION_CIPHERS: Lazy<Mutex<HashMap<String, Aes256Gcm>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// This is a software-based counterpart of `HardwareEcdsaKey` that should be used exclusively for testing.
/// Please note that its behaviour differs from the Android and iOS backed implementations of `HardwareEcdsaKey`,
/// in that it initializes the actual keys eagerly (on `new()`) instead of lazily. This should not matter during
/// testing if the keys are used consistently.
#[derive(Clone)]
pub struct SoftwareEcdsaKey {
    identifier: String,
    key: SigningKey,
}

impl SoftwareEcdsaKey {
    pub fn new(identifier: String, key: SigningKey) -> Self {
        SoftwareEcdsaKey { identifier, key }
    }

    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, SigningKey::random(&mut OsRng))
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
        Signer::try_sign(&self.key, msg)
    }
}
impl SecureEcdsaKey for SoftwareEcdsaKey {}

impl ConstructibleWithIdentifier for SoftwareEcdsaKey {
    fn get_or_create(identifier: String) -> Self
    where
        Self: Sized,
    {
        // Obtain lock on SIGNING_KEYS static hashmap.
        let mut signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");

        // Insert new random signing key, if the key is not present.
        let key = signing_keys
            .entry(identifier.clone())
            .or_insert_with(|| SigningKey::random(&mut OsRng))
            .clone();

        SoftwareEcdsaKey { identifier, key }
    }
}

impl DeletableWithIdentifier for SoftwareEcdsaKey {
    type Error = Infallible;

    async fn delete(self) -> Result<(), Self::Error> {
        let mut signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        signing_keys.remove(&self.identifier);

        Ok(())
    }
}

impl WithIdentifier for SoftwareEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

/// This is a software-based counterpart of `HardwareEncryptionKey` that should be used exclusively for testing.
/// Please note that its behaviour differs from the Android and iOS backed implementations of `HardwareEncryptionKey`,
/// in that it initializes the actual keys eagerly (on `new()`) instead of lazily. This should not matter during
/// testing if the keys are used consistently.
#[derive(Clone)]
pub struct SoftwareEncryptionKey {
    identifier: String,
    cipher: Aes256Gcm,
}

impl SoftwareEncryptionKey {
    pub fn new(identifier: String, cipher: Aes256Gcm) -> Self {
        SoftwareEncryptionKey { identifier, cipher }
    }

    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng)))
    }

    // Peek into the static hashmap to see if an identifier / cipher pair exists.
    pub fn has_identifier(identifier: &str) -> bool {
        ENCRYPTION_CIPHERS
            .lock()
            .expect("Could not get lock on SIGNING_KEYS")
            .contains_key(identifier)
    }
}

impl Debug for SoftwareEncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftwareEncryptionKey")
            .field("identifier", &self.identifier)
            .finish_non_exhaustive()
    }
}

impl ConstructibleWithIdentifier for SoftwareEncryptionKey {
    fn get_or_create(identifier: String) -> Self
    where
        Self: Sized,
    {
        // obtain lock on ENCRYPTION_KEYS static hashmap
        let mut encryption_ciphers = ENCRYPTION_CIPHERS
            .lock()
            .expect("Could not get lock on ENCRYPTION_CIPHERS");

        // Insert new random encryption cipher, if the key is not present.
        let cipher = encryption_ciphers
            .entry(identifier.clone())
            .or_insert_with(|| Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng)))
            .clone();

        SoftwareEncryptionKey { identifier, cipher }
    }
}

impl DeletableWithIdentifier for SoftwareEncryptionKey {
    type Error = Infallible;

    async fn delete(self) -> Result<(), Self::Error> {
        let mut encryption_ciphers = ENCRYPTION_CIPHERS
            .lock()
            .expect("Could not get lock on ENCRYPTION_CIPHERS");
        encryption_ciphers.remove(&self.identifier);

        Ok(())
    }
}

impl WithIdentifier for SoftwareEncryptionKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl SecureEncryptionKey for SoftwareEncryptionKey {
    type Error = aes_gcm::Error;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Generate a random nonce
        let nonce_bytes = random_bytes(12);
        let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

        // Encrypt the provided message
        let encrypted_msg = self.cipher.encrypt(nonce, msg)?;

        // concatenate nonce with encrypted payload
        let result = nonce_bytes.into_iter().chain(encrypted_msg).collect();

        Ok(result)
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Re-create the nonce from the first 12 bytes
        let nonce = Nonce::from_slice(&msg[..12]);

        // Decrypt the provided message with the retrieved nonce
        self.cipher.decrypt(nonce, &msg[12..])
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test, *};

    #[tokio::test]
    async fn test_software_signature() {
        let payload = b"This is a message that will be signed.";
        let identifier = "key";

        assert!(test::sign_and_verify_signature::<SoftwareEcdsaKey>(payload, identifier.to_string()).await);
    }

    #[tokio::test]
    async fn test_software_encryption() {
        let payload = b"This message will be encrypted.";
        let identifier = "key";

        assert!(test::encrypt_and_decrypt_message::<SoftwareEncryptionKey>(payload, identifier.to_string()).await);
    }
}
