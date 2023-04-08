use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::{collections::HashMap, sync::Mutex};

use super::{HardwareKeyStoreError, PlatformSigningKey};

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, p256::ecdsa::SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct SoftwareSigningKey(p256::ecdsa::SigningKey);

impl From<SigningKey> for SoftwareSigningKey {
    fn from(value: SigningKey) -> Self {
        SoftwareSigningKey(value)
    }
}
impl Signer<Signature> for SoftwareSigningKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        Signer::try_sign(&self.0, msg)
    }
}
impl wallet_shared::account::signing_key::SigningKey for SoftwareSigningKey {
    type Error = p256::ecdsa::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.0.verifying_key())
    }
}

impl wallet_shared::account::signing_key::SecureSigningKey for SoftwareSigningKey {}

// SigningKey from p256::ecdsa conforms to the SigningKey trait
// if we provide an implementation for the signing_key and verifying_key methods.
impl PlatformSigningKey for SoftwareSigningKey {
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
