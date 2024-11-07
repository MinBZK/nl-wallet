use std::fmt::Debug;

use uniffi::UnexpectedUniFFICallbackError;

use super::get_platform_support;

// implementation of KeyStoreError from UDL
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("key error: {reason}")]
    KeyError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

// this is required to catch UnexpectedUniFFICallbackError
impl From<UnexpectedUniFFICallbackError> for KeyStoreError {
    fn from(value: UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
pub trait SigningKeyBridge: Send + Sync + Debug {
    fn public_key(&self, identifier: String) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
    fn delete(&self, identifier: String) -> Result<(), KeyStoreError>;
}

pub trait EncryptionKeyBridge: Send + Sync + Debug {
    fn encrypt(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
    fn decrypt(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
    fn delete(&self, identifier: String) -> Result<(), KeyStoreError>;
}

pub fn get_signing_key_bridge() -> &'static dyn SigningKeyBridge {
    get_platform_support().signing_key.as_ref()
}

pub fn get_encryption_key_bridge() -> &'static dyn EncryptionKeyBridge {
    get_platform_support().encryption_key.as_ref()
}
