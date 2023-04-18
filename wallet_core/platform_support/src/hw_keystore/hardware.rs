use p256::{
    ecdsa::{signature::Signer, Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};
use wallet_shared::account::signing_key::{EcdsaKey, SecureEcdsaKey};

use super::{HardwareKeyStoreError, KeyStoreError, PlatformEcdsaKey, PlatformEncryptionKey};
use crate::bridge::hw_keystore::{get_key_store, EncryptionKeyBridge, SigningKeyBridge};

impl From<KeyStoreError> for p256::ecdsa::Error {
    // wrap KeyStoreError in p256::ecdsa::signature::error,
    // as try_sign() has the latter as error type
    fn from(value: KeyStoreError) -> Self {
        p256::ecdsa::Error::from_source(value)
    }
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

impl Signer<Signature> for HardwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        let signature_bytes = self.bridge.sign(msg.to_vec())?;

        // decode the DER encoded signature
        Signature::from_der(&signature_bytes)
    }
}
impl EcdsaKey for HardwareEcdsaKey {
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
        let bridge = get_key_store().get_or_create_signing_key(identifier.to_string())?;
        let key = HardwareEcdsaKey::new(bridge);

        Ok(key)
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
        let bridge = get_key_store().get_or_create_encryption_key(identifier.to_string())?;
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
