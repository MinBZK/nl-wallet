#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

pub trait KeyStore {
    type KeyType: AsymmetricKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Self::KeyType;
}

pub trait AsymmetricKey {
    fn public_key(&self) -> Vec<u8>;
    fn sign(&self, payload: &[u8]) -> [u8; 64];
}
