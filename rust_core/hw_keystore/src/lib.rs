#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

#[cfg(feature = "integration-test")]
pub mod integration_test;

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("KeyError")]
    KeyError { message: Option<String> },
    #[error("InternalError")]
    InternalError { message: String },
}

pub trait KeyStore {
    type KeyType: AsymmetricKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Result<Self::KeyType, KeyStoreError>;
}

pub trait AsymmetricKey {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, KeyStoreError>;
}
