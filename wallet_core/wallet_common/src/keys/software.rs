use std::{collections::HashMap, sync::Mutex};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;

use crate::utils::random_bytes;

use super::{ConstructableWithIdentifier, EcdsaKey, SecureEcdsaKey, SecureEncryptionKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// static for storing identifier -> aes cipher mapping, will only ever grow
static ENCRYPTION_CIPHERS: Lazy<Mutex<HashMap<String, Aes256Gcm>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct SoftwareEcdsaKey {
    identifier: String,
}

#[cfg(feature = "mock")]
impl SoftwareEcdsaKey {
    /// Insert a given existing key in the map of [`SoftwareEcdsaKey`]s, for use in testing
    /// (e.g. with the keys in ISO 23220).
    pub fn insert(identifier: &str, key: SigningKey) {
        SIGNING_KEYS
            .lock()
            .expect("Could not get lock on SIGNING_KEYS")
            .insert(identifier.to_string(), key);
    }
}

#[async_trait]
impl EcdsaKey for SoftwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        let key = signing_keys.get(&self.identifier).unwrap().verifying_key();

        Ok(*key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        let key = signing_keys.get(&self.identifier).unwrap();
        p256::ecdsa::signature::Signer::try_sign(key, msg)
    }
}
impl SecureEcdsaKey for SoftwareEcdsaKey {}

impl ConstructableWithIdentifier for SoftwareEcdsaKey {
    fn new(identifier: &str) -> Self
    where
        Self: Sized,
    {
        // obtain lock on SIGNING_KEYS static hashmap
        let mut signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        // insert new random signing key, if the key is not present
        if !signing_keys.contains_key(identifier) {
            signing_keys.insert(identifier.to_string(), SigningKey::random(&mut OsRng));
        }

        SoftwareEcdsaKey {
            identifier: identifier.to_string(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[derive(Clone)]
pub struct SoftwareEncryptionKey {
    identifier: String,
}

#[async_trait]
impl ConstructableWithIdentifier for SoftwareEncryptionKey {
    fn new(identifier: &str) -> Self
    where
        Self: Sized,
    {
        // obtain lock on ENCRYPTION_KEYS static hashmap
        let mut encryption_ciphers = ENCRYPTION_CIPHERS
            .lock()
            .expect("Could not get lock on ENCRYPTION_CIPHERS");

        // insert new random encryption cipher, if the key is not present
        if !encryption_ciphers.contains_key(identifier) {
            encryption_ciphers.insert(
                identifier.to_string(),
                Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng)),
            );
        }

        SoftwareEncryptionKey {
            identifier: identifier.to_string(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[async_trait]
impl SecureEncryptionKey for SoftwareEncryptionKey {
    type Error = aes_gcm::Error;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Generate a random nonce
        let nonce_bytes = random_bytes(12);
        let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

        let encryption_ciphers = ENCRYPTION_CIPHERS
            .lock()
            .expect("Could not get lock on ENCRYPTION_CIPHERS");

        // Encrypt the provided message
        let encrypted_msg = encryption_ciphers
            .get(&self.identifier)
            .unwrap()
            .encrypt(nonce, msg)
            .expect("Could not encrypt message");

        // concatenate nonce with encrypted payload
        let result: Vec<_> = nonce_bytes.into_iter().chain(encrypted_msg).collect();

        Ok(result)
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Re-create the nonce from the first 12 bytes
        let nonce = Nonce::from_slice(&msg[..12]);

        let encryption_ciphers = ENCRYPTION_CIPHERS
            .lock()
            .expect("Could not get lock on ENCRYPTION_CIPHERS");

        // Decrypt the provided message with the retrieved nonce
        let decrypted_msg = encryption_ciphers
            .get(&self.identifier)
            .unwrap()
            .decrypt(nonce, &msg[12..])
            .expect("Could not decrypt message");

        Ok(decrypted_msg)
    }
}
