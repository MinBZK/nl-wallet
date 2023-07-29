use std::{collections::HashMap, sync::Mutex};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;

use crate::{
    account::signing_key::{EcdsaKey, SecureEcdsaKey},
    utils::random_bytes,
};

use super::signing_key::{ConstructableWithIdentifier, PlatformEncryptionKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// static for storing identifier -> aes cipher mapping, will only ever grow
static ENCRYPTION_CIPHERS: Lazy<Mutex<HashMap<String, Aes256Gcm>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct SoftwareEcdsaKey {
    identifier: String,
}

// SigningKey from p256::ecdsa almost conforms to the EcdsaKey trait,
// so we can forward the try_sign method and verifying_key methods.
impl Signer<Signature> for SoftwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        let signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        signing_keys.get(&self.identifier).unwrap().try_sign(msg)
    }
}
impl EcdsaKey for SoftwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        let key = signing_keys.get(&self.identifier).unwrap().verifying_key();

        Ok(*key)
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

impl PlatformEncryptionKey for SoftwareEncryptionKey {
    type Error = aes_gcm::Error;

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
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

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
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
