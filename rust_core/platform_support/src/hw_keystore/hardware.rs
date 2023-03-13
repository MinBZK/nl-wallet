use once_cell::sync::OnceCell;
use p256::{
    ecdsa::{
        signature::{Error as SignerError, Signer},
        Signature, VerifyingKey,
    },
    pkcs8::DecodePublicKey,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::hw_keystore::{Error, KeyStore, SigningKey};

// import generated Rust bindings
uniffi::include_scaffolding!("hw_keystore");

// implementation of KeyStoreError from UDL
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("Key error: {reason:?}")]
    KeyError { reason: String },
    #[error("Bridging error: {reason:?}")]
    BridgingError { reason: String },
}

// this is required to catch UnexpectedUniFFICallbackError
impl From<uniffi::UnexpectedUniFFICallbackError> for KeyStoreError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError {
            reason: value.reason,
        }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
trait KeyStoreBridge: Send + Sync + Debug {
    fn get_or_create_key(
        &self,
        identifier: String,
    ) -> Result<Box<dyn SigningKeyBridge>, KeyStoreError>;
}

trait SigningKeyBridge: Send + Sync + Debug {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

// HardwareKeyStore implements KeyStore by wrapping KeyStoreBridge from native code
pub struct HardwareKeyStore {
    bridge: Box<dyn KeyStoreBridge>,
    signing_keys: HashMap<String, HardwareSigningKey>,
}

impl HardwareKeyStore {
    fn new(bridge: Box<dyn KeyStoreBridge>) -> Self {
        HardwareKeyStore {
            bridge,
            signing_keys: HashMap::new(),
        }
    }

    pub fn key_store() -> Arc<RwLock<Self>> {
        // crash if KEY_STORE is not yet set
        Arc::clone(
            KEY_STORE
                .get()
                .expect("KEY_STORE used before init_hw_keystore() was called"),
        )
    }
}

impl KeyStore for HardwareKeyStore {
    type SigningKeyType = HardwareSigningKey;

    fn create_key(&mut self, identifier: &str) -> Result<&mut HardwareSigningKey, Error> {
        let key = match self.signing_keys.entry(identifier.to_string()) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let bridge = self.bridge.get_or_create_key(identifier.to_string())?;

                entry.insert(HardwareSigningKey::new(bridge))
            }
        };

        Ok(key)
    }

    fn get_key(&self, identifier: &str) -> Option<&HardwareSigningKey> {
        self.signing_keys.get(identifier)
    }
}

// HardwareSigningKey similary wraps SigningKeyBridge from native
#[derive(Clone)]
pub struct HardwareSigningKey {
    bridge: Arc<dyn SigningKeyBridge>,
    verifying_key: OnceCell<VerifyingKey>,
}

impl HardwareSigningKey {
    fn new(bridge: Box<dyn SigningKeyBridge>) -> Self {
        HardwareSigningKey {
            bridge: bridge.into(),
            verifying_key: OnceCell::new(),
        }
    }
}

impl SigningKey for HardwareSigningKey {
    fn verifying_key(&self) -> Result<&VerifyingKey, Error> {
        let verifying_key = self.verifying_key.get_or_try_init(|| {
            let public_key_bytes = self.bridge.public_key()?;
            // decode the DER encoded public key received from native
            let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

            Ok::<_, Error>(public_key)
        })?;

        Ok(verifying_key)
    }
}

impl From<KeyStoreError> for SignerError {
    // wrap KeyStoreError in p256::ecdsa::signature::error,
    // as try_sign() has the latter as error type
    fn from(value: KeyStoreError) -> Self {
        SignerError::from_source(value)
    }
}

impl Signer<Signature> for HardwareSigningKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, SignerError> {
        let signature_bytes = self.bridge.sign(msg.to_vec())?;

        // decode the DER encoded signature
        Signature::from_der(&signature_bytes)
    }
}

static KEY_STORE: OnceCell<Arc<RwLock<HardwareKeyStore>>> = OnceCell::new();

fn init_hw_keystore(bridge: Box<dyn KeyStoreBridge>) {
    let key_store = Arc::new(RwLock::new(HardwareKeyStore::new(bridge)));
    // crash if KEY_STORE was already set
    assert!(
        KEY_STORE.set(key_store).is_ok(),
        "Cannot call init_hw_keystore() more than once"
    )
}
