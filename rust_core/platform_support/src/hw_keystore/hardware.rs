use p256::{
    ecdsa::{
        signature::{Error as SignerError, Signer},
        Signature, VerifyingKey,
    },
    pkcs8::DecodePublicKey,
};
use wallet_shared::account::signing_key::SecureEcdsaKey;

use super::{HardwareKeyStoreError, KeyStoreError, PlatformEcdsaKey, PlatformEncryptionKey};
use crate::bridge::hw_keystore::{EncryptionKeyBridge, SigningKeyBridge, KEY_STORE};

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
        let bridge = key_store.get_or_create_signing_key(identifier.to_string())?;
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
    fn encryption_key(identifier: &str) -> Result<Self, HardwareKeyStoreError>
    where
        Self: Sized,
    {
        // crash if KEY_STORE is not yet set, then wait for key store mutex lock
        let key_store = KEY_STORE
            .get()
            .expect("KEY_STORE used before init_hw_keystore() was called")
            .lock()
            .expect("Could not get lock on KEY_STORE");
        let bridge = key_store.get_or_create_encryption_key(identifier.to_string())?;
        let key = HardwareEncryptionKey::new(bridge);

        Ok(key)
    }

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let result = self.bridge.encrypt(msg.to_vec())?;
        Ok(result)
    }

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let result = self.bridge.decrypt(msg.to_vec())?;
        Ok(result)
    }
}
