use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::{collections::HashMap, sync::Mutex};
use wallet_common::{
    account::signing_key::{EcdsaKey, SecureEcdsaKey},
    utils::random_bytes,
};

use super::{HardwareKeyStoreError, PlatformEcdsaKey, PlatformEncryptionKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// static for storing identifier -> aes cipher mapping, will only ever grow
static ENCRYPTION_CIPHERS: Lazy<Mutex<HashMap<String, Aes256Gcm>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct SoftwareEcdsaKey(SigningKey);

impl From<SigningKey> for SoftwareEcdsaKey {
    fn from(value: SigningKey) -> Self {
        SoftwareEcdsaKey(value)
    }
}
impl Signer<Signature> for SoftwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        Signer::try_sign(&self.0, msg)
    }
}
impl EcdsaKey for SoftwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.0.verifying_key())
    }
}
impl SecureEcdsaKey for SoftwareEcdsaKey {}

// SigningKey from p256::ecdsa conforms to the SigningKey trait
// if we provide an implementation for the signing_key and verifying_key methods.
impl PlatformEcdsaKey for SoftwareEcdsaKey {
    fn signing_key(identifier: &str) -> Result<Self, HardwareKeyStoreError> {
        // obtain lock on SIGNING_KEYS static hashmap
        let mut signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        // insert new random signing key, if the key is not present
        let key = signing_keys
            .entry(identifier.to_string())
            .or_insert_with(|| SigningKey::random(&mut OsRng));

        // make a clone of the (mutable) signing key so we can
        // return (non-mutable) ownership to the caller
        Ok(key.clone().into())
    }
}

#[derive(Clone)]
pub struct SoftwareEncryptionKey(Aes256Gcm);

impl From<Aes256Gcm> for SoftwareEncryptionKey {
    fn from(value: Aes256Gcm) -> Self {
        SoftwareEncryptionKey(value)
    }
}
impl PlatformEncryptionKey for SoftwareEncryptionKey {
    fn encryption_key(identifier: &str) -> Result<Self, HardwareKeyStoreError>
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

        // make a clone of the (mutable) cipher so we can
        // return (non-mutable) ownership to the caller
        Ok(cipher.clone().into())
    }

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let cipher = &self.0;

        // Generate a random nonce
        let nonce_bytes = random_bytes(12);
        let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

        // Encrypt the provided message
        let encrypted_msg = cipher.encrypt(nonce, msg).expect("Could not encrypt message");

        // concatenate nonce with encrypted payload
        let result: Vec<_> = nonce_bytes.into_iter().chain(encrypted_msg).collect();

        Ok(result)
    }

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let cipher = &self.0;

        // Re-create the nonce from the first 12 bytes
        let nonce = Nonce::from_slice(&msg[..12]);

        // Decrypt the provided message with the retrieved nonce
        let decrypted_msg = cipher.decrypt(nonce, &msg[12..]).expect("Could not decrypt message");

        Ok(decrypted_msg)
    }
}
