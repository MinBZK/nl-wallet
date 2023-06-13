use std::{collections::HashMap, sync::Mutex};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;

use wallet_common::{
    account::signing_key::{EcdsaKey, SecureEcdsaKey},
    utils::random_bytes,
};

use super::{ConstructableWithIdentifier, HardwareKeyStoreError, PlatformEcdsaKey, PlatformEncryptionKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// static for storing identifier -> aes cipher mapping, will only ever grow
static ENCRYPTION_CIPHERS: Lazy<Mutex<HashMap<String, Aes256Gcm>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct SoftwareEcdsaKey {
    identifier: String,
    signing_key: SigningKey,
}

// SigningKey from p256::ecdsa almost conforms to the EcdsaKey trait,
// so we can forward the try_sign method and verifying_key methods.
impl Signer<Signature> for SoftwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        self.signing_key.try_sign(msg)
    }
}
impl EcdsaKey for SoftwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.signing_key.verifying_key())
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
        let signing_key = signing_keys
            .entry(identifier.to_string())
            .or_insert_with(|| SigningKey::random(&mut OsRng));

        SoftwareEcdsaKey {
            identifier: identifier.to_string(),
            signing_key: signing_key.clone(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}
impl PlatformEcdsaKey for SoftwareEcdsaKey {}

#[derive(Clone)]
pub struct SoftwareEncryptionKey {
    identifier: String,
    cipher: Aes256Gcm,
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
        let cipher = encryption_ciphers
            .entry(identifier.to_string())
            .or_insert_with(|| Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng)));

        SoftwareEncryptionKey {
            identifier: identifier.to_string(),
            cipher: cipher.clone(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}
impl PlatformEncryptionKey for SoftwareEncryptionKey {
    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        // Generate a random nonce
        let nonce_bytes = random_bytes(12);
        let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

        // Encrypt the provided message
        let encrypted_msg = self.cipher.encrypt(nonce, msg).expect("Could not encrypt message");

        // concatenate nonce with encrypted payload
        let result: Vec<_> = nonce_bytes.into_iter().chain(encrypted_msg).collect();

        Ok(result)
    }

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        // Re-create the nonce from the first 12 bytes
        let nonce = Nonce::from_slice(&msg[..12]);

        // Decrypt the provided message with the retrieved nonce
        let decrypted_msg = self
            .cipher
            .decrypt(nonce, &msg[12..])
            .expect("Could not decrypt message");

        Ok(decrypted_msg)
    }
}
