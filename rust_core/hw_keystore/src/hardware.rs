use crate::{AsymmetricKey, KeyStore, KeyStoreError};

use lazy_static::lazy_static;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

uniffi::include_scaffolding!("hw_keystore");

impl From<uniffi::UnexpectedUniFFICallbackError> for KeyStoreError {
    fn from(error: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::InternalError {
            message: error.reason,
        }
    }
}

pub struct HardwareKeyStore {
    bridge: Arc<dyn KeyStoreBridge>,
}

impl HardwareKeyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for HardwareKeyStore {
    fn default() -> Self {
        let bridge = KEY_STORE_BRIDGE
            .read()
            .expect("Could not acquire read lock to KEY_STORE_BRIDGE")
            .clone()
            .expect("KEY_STORE_BRIDGE used before init_hw_keystore() was called");

        HardwareKeyStore { bridge }
    }
}

impl KeyStore for HardwareKeyStore {
    type KeyType = HardwareAsymmetricKey;

    fn get_or_create_key(
        &mut self,
        identifier: &str,
    ) -> Result<HardwareAsymmetricKey, KeyStoreError> {
        let bridge = self.bridge.get_or_create_key(identifier.to_string())?;

        Ok(HardwareAsymmetricKey::new(bridge))
    }
}

pub struct HardwareAsymmetricKey {
    bridge: Box<dyn AsymmetricKeyBridge>,
}

impl HardwareAsymmetricKey {
    fn new(bridge: Box<dyn AsymmetricKeyBridge>) -> Self {
        HardwareAsymmetricKey { bridge }
    }
}

impl AsymmetricKey for HardwareAsymmetricKey {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError> {
        let public_key = self.bridge.public_key()?;

        Ok(public_key)
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        let signature = self.bridge.sign(payload.to_vec())?;

        Ok(signature)
    }
}

trait KeyStoreBridge: Send + Sync + Debug {
    fn get_or_create_key(
        &self,
        identifier: String,
    ) -> Result<Box<dyn AsymmetricKeyBridge>, KeyStoreError>;
}

trait AsymmetricKeyBridge: Send + Sync + Debug {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

lazy_static! {
    static ref KEY_STORE_BRIDGE: RwLock<Option<Arc<dyn KeyStoreBridge>>> = RwLock::new(None);
}

fn init_hw_keystore(bridge: Box<dyn KeyStoreBridge>) {
    let mut static_bridge = KEY_STORE_BRIDGE
        .write()
        .expect("Could not acquire write lock to KEY_STORE_BRIDGE");
    assert!(
        static_bridge.is_none(),
        "Cannot call init_hw_keystore() more than once"
    );

    static_bridge.replace(Arc::from(bridge));
}
