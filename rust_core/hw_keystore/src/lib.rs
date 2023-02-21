#[cfg(feature = "software")]
pub mod software;

use std::rc::Rc;

pub trait KeyStore {
    type KeyType: AsymmetricKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Rc<Self::KeyType>;
}

pub trait AsymmetricKey {
    fn public_key(&self) -> Vec<u8>;
    fn sign(&self, message: &[u8]) -> [u8; 64];
}
