use crate::{AsymmetricKey, KeyStore};

use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
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

    fn get_or_create_key(&mut self, identifier: &str) -> SoftwareKey {
        let key = self
            .keys
            .entry(identifier.to_string())
            .or_insert_with(|| Rc::new(SecretKey::random(&mut OsRng)));

        SoftwareKey::new(key)
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
    fn public_key(&self) -> Vec<u8> {
        self.key.public_key().to_public_key_der().unwrap().to_vec()
    }

    fn sign(&self, payload: &[u8]) -> [u8; 64] {
        let signature: Signature = SigningKey::from(self.key.as_ref()).sign(payload);

        signature.to_bytes().into()
    }
}
