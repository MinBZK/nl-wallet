use once_cell::sync::Lazy;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use std::{collections::HashMap, sync::Mutex};
use std::io::Read;

pub use p256::ecdsa::SigningKey as SoftwareSigningKey;
use crate::hw_keystore::PlatformEncryptionKey;

use super::{HardwareKeyStoreError, PlatformSigningKey};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, p256::ecdsa::SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// SigningKey from p256::ecdsa conforms to the SigningKey trait
// if we provide an implementation for the signing_key and verifying_key methods.
impl PlatformSigningKey for SoftwareSigningKey {
    fn signing_key(identifier: &str) -> Result<Self, HardwareKeyStoreError> {
        // obtain lock on SIGNING_KEYS static hashmap
        let mut signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        // insert new random signing key, if the key is not present
        let key = signing_keys
            .entry(identifier.to_string())
            .or_insert_with(|| SoftwareSigningKey::random(&mut OsRng));

        // make a clone of the (mutable) signing key so we can
        // return (non-mutable) ownership to the caller
        Ok(key.clone())
    }

    fn verifying_key(&self) -> Result<VerifyingKey, HardwareKeyStoreError> {
        let verifying_key = *self.verifying_key();

        Ok(verifying_key)
    }
}

// static for storing identifier -> signing key mapping, will only every grow
static ENCRYPTION_KEYS: Lazy<Mutex<HashMap<String, SoftwareEncryptionKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct SoftwareEncryptionKey {
    cipher: Aes256Gcm,
}

impl PlatformEncryptionKey for SoftwareEncryptionKey {
    fn encryption_key(identifier: &str) -> Result<Self, HardwareKeyStoreError> where Self: Sized {
        // obtain lock on ENCRYPTION_KEYS static hashmap
        let mut encryption_keys = ENCRYPTION_KEYS.lock().expect("Could not get lock on ENCRYPTION_KEYS");

        // insert new random signing key, if the key is not present
        let key = encryption_keys
            .entry(identifier.to_string())
            .or_insert_with(|| {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                SoftwareEncryptionKey { cipher }
            });

        // make a clone of the (mutable) signing key so we can
        // return (non-mutable) ownership to the caller
        Ok(key.clone())
    }

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let cipher = &self.cipher;
        //TODO: Do we require a unique, per message, nonce?
        let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
        let encrypted_msg = cipher.encrypt(nonce, msg).expect("Could not encrypt message");
        Ok(encrypted_msg)
    }

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let cipher = &self.cipher;
        //TODO: Do we require a unique, per message, nonce?
        let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
        let decrypted_msg = cipher.decrypt(nonce, msg).expect("Could not decrypt message");
        Ok(decrypted_msg)
    }
}