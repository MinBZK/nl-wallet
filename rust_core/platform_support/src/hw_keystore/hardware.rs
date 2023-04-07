use p256::{
    ecdsa::{
        signature::{Error as SignerError, Signer},
        Signature, VerifyingKey,
    },
    pkcs8::DecodePublicKey,
};

use super::{
    error::{HardwareKeyStoreError, KeyStoreError},
    PlatformSigningKey,
    PlatformEncryptionKey
};
use crate::bridge::hw_keystore::{SigningKeyBridge, KEY_STORE, EncryptionKeyBridge};

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
    fn signing_key(identifier: &str) -> Result<Self, HardwareKeyStoreError> {
        // crash if KEY_STORE is not yet set, then wait for key store mutex lock
        let key_store = KEY_STORE
            .get()
            .expect("KEY_STORE used before init_hw_keystore() was called")
            .lock()
            .expect("Could not get lock on KEY_STORE");
        let bridge = key_store.get_or_create_signing_key(identifier.to_string())?;
        let key = HardwareSigningKey::new(bridge);

        Ok(key)
    }

    fn verifying_key(&self) -> Result<VerifyingKey, HardwareKeyStoreError> {
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

// HardwareSigningKey wraps SigningKeyBridge from native
pub struct HardwareEncryptionKey {
    bridge: Box<dyn EncryptionKeyBridge>,
}

impl HardwareEncryptionKey {
    fn new(bridge: Box<dyn EncryptionKeyBridge>) -> Self {
        HardwareEncryptionKey { bridge }
    }
}

impl PlatformEncryptionKey for HardwareEncryptionKey {
    fn encryption_key(identifier: &str) -> Result<Self, HardwareKeyStoreError> where Self: Sized {
        todo!()
    }

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        todo!()
    }

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        todo!()
    }
}
