use once_cell::sync::Lazy;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub use p256::ecdsa::SigningKey as SoftwareSigningKey;

use crate::hw_keystore::{Error, KeyStore, SigningKey};

static KEY_STORE: Lazy<Arc<RwLock<InMemoryKeyStore>>> =
    Lazy::new(|| Arc::new(RwLock::new(InMemoryKeyStore::new())));

// Software implemenation of KeyStore, just stores SigningKey entities in a hash map.
pub struct InMemoryKeyStore {
    signing_keys: HashMap<String, p256::ecdsa::SigningKey>,
}

impl InMemoryKeyStore {
    fn new() -> Self {
        InMemoryKeyStore {
            signing_keys: HashMap::new(),
        }
    }

    // this mirrors the static in the hardware implementation
    pub fn key_store() -> Arc<RwLock<Self>> {
        Arc::clone(Lazy::force(&KEY_STORE))
    }
}

impl KeyStore for InMemoryKeyStore {
    type SigningKeyType = SoftwareSigningKey;

    fn create_key(&mut self, identifier: &str) -> Result<&mut SoftwareSigningKey, Error> {
        let key = self
            .signing_keys
            .entry(identifier.to_string())
            .or_insert_with(|| SoftwareSigningKey::random(&mut OsRng));

        Ok(key)
    }

    fn get_key(&self, identifier: &str) -> Option<&SoftwareSigningKey> {
        self.signing_keys.get(identifier)
    }
}

// SigningKey from p256::ecdsa conforms to the SigningKey trait
// if we provide an implementation for our verifying_key method.
impl SigningKey for SoftwareSigningKey {
    fn verifying_key(&self) -> Result<&VerifyingKey, Error> {
        Ok(self.verifying_key())
    }
}
