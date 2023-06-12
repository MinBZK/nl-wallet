use p256::{
    ecdsa::{signature::Signer, Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};

use wallet_common::account::signing_key::{EcdsaKey, SecureEcdsaKey};

use super::{
    ConstructableWithIdentifier, HardwareKeyStoreError, KeyStoreError, PlatformEcdsaKey, PlatformEncryptionKey,
};
use crate::bridge::hw_keystore::{get_encryption_key_bridge, get_signing_key_bridge};

impl From<KeyStoreError> for p256::ecdsa::Error {
    // wrap KeyStoreError in p256::ecdsa::signature::error,
    // as try_sign() has the latter as error type
    fn from(value: KeyStoreError) -> Self {
        p256::ecdsa::Error::from_source(value)
    }
}

// HardwareSigningKey wraps SigningKeyBridge from native
#[derive(Clone)]
pub struct HardwareEcdsaKey {
    identifier: String,
}

impl Signer<Signature> for HardwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        let signature_bytes = get_signing_key_bridge().sign(self.identifier.to_owned(), msg.to_vec())?;

        // decode the DER encoded signature
        Signature::from_der(&signature_bytes)
    }
}
impl EcdsaKey for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let public_key_bytes = get_signing_key_bridge().public_key(self.identifier.to_owned())?;
        let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

        Ok(public_key)
    }
}
impl SecureEcdsaKey for HardwareEcdsaKey {}

impl ConstructableWithIdentifier for HardwareEcdsaKey {
    fn new(identifier: &str) -> Self {
        HardwareEcdsaKey {
            identifier: identifier.to_string(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}
impl PlatformEcdsaKey for HardwareEcdsaKey {}

// HardwareEncryptionKey wraps EncryptionKeyBridge from native
#[derive(Clone)]
pub struct HardwareEncryptionKey {
    identifier: String,
}

impl ConstructableWithIdentifier for HardwareEncryptionKey {
    fn new(identifier: &str) -> Self {
        HardwareEncryptionKey {
            identifier: identifier.to_string(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}
impl PlatformEncryptionKey for HardwareEncryptionKey {
    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let result = get_encryption_key_bridge().encrypt(self.identifier.to_owned(), msg.to_vec())?;

        Ok(result)
    }

    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let result = get_encryption_key_bridge().decrypt(self.identifier.to_owned(), msg.to_vec())?;

        Ok(result)
    }
}
