use crate::{AsymmetricKey, KeyStore};

use lazy_static::lazy_static;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

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

    fn get_or_create_key(&mut self, identifier: &str) -> HardwareAsymmetricKey {
        let bridge = self.bridge.get_or_create_key(identifier.to_string());

        HardwareAsymmetricKey::new(bridge)
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
    fn public_key(&self) -> Vec<u8> {
        self.bridge.public_key()
    }

    fn sign(&self, payload: &[u8]) -> Vec<u8> {
        self.bridge.sign(payload.to_vec())
    }
}

uniffi::include_scaffolding!("hw_keystore");

pub trait KeyStoreBridge: Send + Sync + Debug {
    fn get_or_create_key(&self, identifier: String) -> Box<dyn AsymmetricKeyBridge>;
}

pub trait AsymmetricKeyBridge: Send + Sync + Debug {
    fn public_key(&self) -> Vec<u8>;
    fn sign(&self, payload: Vec<u8>) -> Vec<u8>;
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
