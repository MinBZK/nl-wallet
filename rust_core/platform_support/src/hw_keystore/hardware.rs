use once_cell::sync::OnceCell;
use p256::{
    ecdsa::{
        signature::{Error as SignerError, Signer},
        Signature, VerifyingKey,
    },
    pkcs8::DecodePublicKey,
};
use std::{fmt::Debug, sync::Mutex};

use crate::hw_keystore::{Error, PlatformSigningKey};

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
        Self::BridgingError { reason: value.reason }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
trait KeyStoreBridge: Send + Sync + Debug {
    fn get_or_create_key(&self, identifier: String) -> Result<Box<dyn SigningKeyBridge>, KeyStoreError>;
}

trait SigningKeyBridge: Send + Sync + Debug {
    fn public_key(&self) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

// HardwareSigningKey wraps SigningKeyBridge from native
pub struct HardwareSigningKey {
    bridge: Box<dyn SigningKeyBridge>,
}

impl HardwareSigningKey {
    fn new(bridge: Box<dyn SigningKeyBridge>) -> Self {
        HardwareSigningKey { bridge }
    }
}

impl PlatformSigningKey for HardwareSigningKey {
    fn signing_key(identifier: &str) -> Result<Self, Error> {
        // crash if KEY_STORE is not yet set, then wait for key store mutex lock
        let key_store = KEY_STORE
            .get()
            .expect("KEY_STORE used before init_hw_keystore() was called")
            .lock()
            .expect("Could not get lock on KEY_STORE");
        let bridge = key_store.get_or_create_key(identifier.to_string())?;
        let key = HardwareSigningKey::new(bridge);

        Ok(key)
    }

    fn verifying_key(&self) -> Result<VerifyingKey, Error> {
        let public_key_bytes = self.bridge.public_key()?;
        let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

        Ok(public_key)
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

// protect key store with mutex, so creating or fetching keys is done atomically
static KEY_STORE: OnceCell<Mutex<Box<dyn KeyStoreBridge>>> = OnceCell::new();

fn init_hw_keystore(bridge: Box<dyn KeyStoreBridge>) {
    let key_store = Mutex::new(bridge);
    // crash if KEY_STORE was already set
    assert!(
        KEY_STORE.set(key_store).is_ok(),
        "Cannot call init_hw_keystore() more than once"
    )
}
