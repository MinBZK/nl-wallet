use crate::{AsymmetricKey, KeyStore, KeyStoreError};

use p256::{
    ecdsa::{
        signature::{SignatureEncoding, Signer},
        Signature, SigningKey,
    },
    pkcs8::EncodePublicKey,
    SecretKey,
};
use rand_core::OsRng;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub struct InMemoryKeyStore {
    keys: HashMap<String, Rc<SecretKey>>,
}

impl InMemoryKeyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeyStore for InMemoryKeyStore {
    type KeyType = SoftwareKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Result<SoftwareKey, KeyStoreError> {
        let key = self
            .keys
            .entry(identifier.to_string())
            .or_insert_with(|| Rc::new(SecretKey::random(&mut OsRng)));

        Ok(SoftwareKey::new(key))
    }
}

pub struct SoftwareKey {
    key: Rc<SecretKey>,
}

impl SoftwareKey {
    fn new(key: &Rc<SecretKey>) -> Self {
        SoftwareKey {
            key: Rc::clone(key),
        }
    }
}

impl AsymmetricKey for SoftwareKey {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError> {
        Ok(self.key.public_key().to_public_key_der().unwrap().to_vec())
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        let signature: Signature = SigningKey::from(self.key.as_ref()).sign(payload);

        Ok(signature.to_der().to_vec())
    }
}
