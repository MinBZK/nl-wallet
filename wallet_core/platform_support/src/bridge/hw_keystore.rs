use std::{fmt::Debug, sync::MutexGuard};

use super::get_bridge_collection;
pub use crate::hw_keystore::KeyStoreError;

// this is required to catch UnexpectedUniFFICallbackError
impl From<uniffi::UnexpectedUniFFICallbackError> for KeyStoreError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
pub trait KeyStoreBridge: Send + Sync + Debug {
    fn get_or_create_signing_key(&self, identifier: String) -> Result<Box<dyn SigningKeyBridge>, KeyStoreError>;
    fn get_or_create_encryption_key(&self, identifier: String) -> Result<Box<dyn EncryptionKeyBridge>, KeyStoreError>;
}

pub trait SigningKeyBridge: Send + Sync + Debug {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

pub trait EncryptionKeyBridge: Send + Sync + Debug {
    fn encrypt(&self, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
    fn decrypt(&self, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

pub fn get_key_store() -> MutexGuard<'static, Box<dyn KeyStoreBridge>> {
    get_bridge_collection()
        .key_store
        .lock()
        .expect("Could not get lock on key store")
}
