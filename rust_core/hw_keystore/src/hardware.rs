use lazy_static::lazy_static;
use p256::ecdsa::{
    signature::{Error as SignerError, Signer},
    Signature, VerifyingKey,
};
use p256::pkcs8::DecodePublicKey;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::{Error, KeyStore, SigningKey};

// import generated Rust bindings
uniffi::include_scaffolding!("hw_keystore");

// implementation of KeyStoreError from UDL
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("Key error: {message:?}")]
    KeyError { message: String },
    #[error("Internal error: {reason:?}")]
    InternalError { reason: String },
}

// this is required to catch UnexpectedUniFFICallbackError
impl From<uniffi::UnexpectedUniFFICallbackError> for KeyStoreError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::InternalError {
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
    bridge: Arc<dyn KeyStoreBridge>,
}

impl HardwareKeyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for HardwareKeyStore {
    fn default() -> Self {
        // always wrap the same instance stored in the lazy static (see below):
        // 1. obtaining a read lock
        // 2. cloning the Arc
        // 3. unwrapping the optional, crashing if it is still None
        let bridge = KEY_STORE_BRIDGE
            .read()
            .expect("Could not acquire read lock to KEY_STORE_BRIDGE")
            .clone()
            .expect("KEY_STORE_BRIDGE used before init_hw_keystore() was called");

        HardwareKeyStore { bridge }
    }
}

impl KeyStore for HardwareKeyStore {
    type SigningKeyType = HardwareSigningKey;

    fn get_or_create_key(&mut self, identifier: &str) -> Result<HardwareSigningKey, Error> {
        let bridge = self.bridge.get_or_create_key(identifier.to_string())?;

        Ok(HardwareSigningKey::new(bridge))
    }
}

// HardwareSigningKey similary wraps SigningKeyBridge from native
pub struct HardwareSigningKey {
    bridge: Box<dyn SigningKeyBridge>,
}

impl HardwareSigningKey {
    fn new(bridge: Box<dyn SigningKeyBridge>) -> Self {
        HardwareSigningKey { bridge }
    }
}

impl SigningKey for HardwareSigningKey {
    fn verifying_key(&self) -> Result<VerifyingKey, Error> {
        let public_key_bytes = self.bridge.public_key()?;
        // decode the DER encoded public key received from native
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

// static reference to a singleton KeyStoreBridge on the native side
lazy_static! {
    static ref KEY_STORE_BRIDGE: RwLock<Option<Arc<dyn KeyStoreBridge>>> = RwLock::new(None);
}

fn init_hw_keystore(bridge: Box<dyn KeyStoreBridge>) {
    // first obtain write lock
    let mut static_bridge = KEY_STORE_BRIDGE
        .write()
        .expect("Could not acquire write lock to KEY_STORE_BRIDGE");
    // then check if the static is actually None, crash otherwise
    assert!(
        static_bridge.is_none(),
        "Cannot call init_hw_keystore() more than once"
    );

    // finally replace the value of the option in the static
    static_bridge.replace(Arc::from(bridge));
}
