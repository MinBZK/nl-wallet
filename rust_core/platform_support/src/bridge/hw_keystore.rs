use once_cell::sync::OnceCell;
use std::{fmt::Debug, sync::Mutex};

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

// protect key store with mutex, so creating or fetching keys is done atomically
pub static KEY_STORE: OnceCell<Mutex<Box<dyn KeyStoreBridge>>> = OnceCell::new();

pub fn init_hw_keystore(bridge: Box<dyn KeyStoreBridge>) {
    let key_store = Mutex::new(bridge);
    // crash if KEY_STORE was already set
    assert!(
        KEY_STORE.set(key_store).is_ok(),
        "Cannot call init_hw_keystore() more than once"
    )
}
