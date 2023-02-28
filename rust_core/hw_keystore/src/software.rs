use p256::ecdsa::{SigningKey as ECDSASigningKey, VerifyingKey};
use rand_core::OsRng;
use std::collections::HashMap;

use crate::{Error, KeyStore, SigningKey};

#[derive(Default)]
pub struct InMemoryKeyStore {
    keys: HashMap<String, p256::ecdsa::SigningKey>,
}

impl InMemoryKeyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeyStore for InMemoryKeyStore {
    type SigningKeyType = ECDSASigningKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Result<ECDSASigningKey, Error> {
        let key = self
            .keys
            .entry(identifier.to_string())
            .or_insert_with(|| ECDSASigningKey::random(&mut OsRng));

        Ok(key.clone())
    }
}

impl SigningKey for ECDSASigningKey {
    fn verifying_key(&self) -> Result<VerifyingKey, Error> {
        Ok(self.verifying_key().clone())
    }
}
