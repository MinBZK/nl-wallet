use once_cell::sync::OnceCell;
use p256::{
    ecdsa::{
        signature::{Error as SignerError, Signer},
        Signature, VerifyingKey,
    },
    pkcs8::DecodePublicKey,
};
use std::{fmt::Debug, sync::Mutex};
use wallet_shared::account::signing_key::SecureEcdsaKey;

use super::{HardwareKeyStoreError, KeyStoreError, PlatformEcdsaKey};

// import generated Rust bindings
uniffi::include_scaffolding!("hw_keystore");

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
pub struct HardwareEcdsaKey {
    bridge: Box<dyn SigningKeyBridge>,
}

impl HardwareEcdsaKey {
    fn new(bridge: Box<dyn SigningKeyBridge>) -> Self {
        HardwareEcdsaKey { bridge }
    }
}

impl wallet_shared::account::signing_key::EcdsaKey for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let public_key_bytes = self.bridge.public_key()?;
        let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;
        Ok(public_key)
    }
}
impl SecureEcdsaKey for HardwareEcdsaKey {}

impl PlatformEcdsaKey for HardwareEcdsaKey {
    fn signing_key(identifier: &str) -> Result<Self, HardwareKeyStoreError> {
        // crash if KEY_STORE is not yet set, then wait for key store mutex lock
        let key_store = KEY_STORE
            .get()
            .expect("KEY_STORE used before init_hw_keystore() was called")
            .lock()
            .expect("Could not get lock on KEY_STORE");
        let bridge = key_store.get_or_create_key(identifier.to_string())?;
        let key = HardwareEcdsaKey::new(bridge);

        Ok(key)
    }
}

impl From<KeyStoreError> for SignerError {
    // wrap KeyStoreError in p256::ecdsa::signature::error,
    // as try_sign() has the latter as error type
    fn from(value: KeyStoreError) -> Self {
        SignerError::from_source(value)
    }
}

impl Signer<Signature> for HardwareEcdsaKey {
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
