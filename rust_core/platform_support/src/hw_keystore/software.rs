use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};
use rand_core::OsRng;
use std::{collections::HashMap, sync::Mutex};
use wallet_shared::account::signing_key::SecureEcdsaKey;

use super::{HardwareKeyStoreError, PlatformEcdsaKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, p256::ecdsa::SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct SoftwareEcdsaKey(p256::ecdsa::SigningKey);

impl From<p256::ecdsa::SigningKey> for SoftwareEcdsaKey {
    fn from(value: p256::ecdsa::SigningKey) -> Self {
        SoftwareEcdsaKey(value)
    }
}
impl Signer<Signature> for SoftwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        Signer::try_sign(&self.0, msg)
    }
}
impl wallet_shared::account::signing_key::EcdsaKey for SoftwareEcdsaKey {
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
            .or_insert_with(|| p256::ecdsa::SigningKey::random(&mut OsRng));

        // make a clone of the (mutable) signing key so we can
        // return (non-mutable) ownership to the caller
        Ok(key.clone().into())
    }
}
