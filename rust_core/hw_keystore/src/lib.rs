#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

use std::sync::Arc;

pub trait KeyStore {
    type KeyType: AsymmetricKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Arc<Self::KeyType>;
}

pub trait AsymmetricKey {
    fn public_key(&self) -> Vec<u8>;
    fn sign(&self, payload: &[u8]) -> [u8; 64];
}
