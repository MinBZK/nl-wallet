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
    keys: HashMap<String, Rc<SoftwareKey>>,
}

impl KeyStore for InMemoryKeyStore {
    type KeyType = SoftwareKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Rc<SoftwareKey> {
        let key = self
            .keys
            .entry(identifier.to_string())
            .or_insert_with(|| Rc::new(SoftwareKey::new()));

        Rc::clone(key)
    }
}

pub struct SoftwareKey {
    key: SecretKey,
}

impl SoftwareKey {
    fn new() -> Self {
        SoftwareKey {
            key: SecretKey::random(&mut OsRng),
        }
    }
}

impl AsymmetricKey for SoftwareKey {
    fn public_key(&self) -> Vec<u8> {
        self.key.public_key().to_public_key_der().unwrap().to_vec()
    }

    fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature: Signature = SigningKey::from(&self.key).sign(message);

        signature.to_bytes().into()
    }
}
