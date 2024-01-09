use p256::{
    ecdsa::{Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};

use wallet_common::{
    keys::{ConstructibleWithIdentifier, EcdsaKey, SecureEcdsaKey, SecureEncryptionKey, WithIdentifier},
    spawn,
};

use crate::bridge::hw_keystore::{get_encryption_key_bridge, get_signing_key_bridge};

use super::{HardwareKeyStoreError, KeyStoreError, PlatformEcdsaKey};

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

impl EcdsaKey for HardwareEcdsaKey {
    type Error = HardwareKeyStoreError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let identifier = self.identifier.to_owned();

        spawn::blocking(|| {
            let public_key_bytes = get_signing_key_bridge().public_key(identifier)?;
            let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

            Ok::<_, Self::Error>(public_key)
        })
        .await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let identifier = self.identifier.to_owned();
        let payload = msg.to_vec();

        let signature_bytes = spawn::blocking(|| get_signing_key_bridge().sign(identifier, payload)).await?;

        // decode the DER encoded signature
        Ok(Signature::from_der(&signature_bytes)?)
    }
}

impl SecureEcdsaKey for HardwareEcdsaKey {}

impl ConstructibleWithIdentifier for HardwareEcdsaKey {
    fn new(identifier: &str) -> Self {
        HardwareEcdsaKey {
            identifier: identifier.to_string(),
        }
    }
}

impl WithIdentifier for HardwareEcdsaKey {
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

impl ConstructibleWithIdentifier for HardwareEncryptionKey {
    fn new(identifier: &str) -> Self {
        HardwareEncryptionKey {
            identifier: identifier.to_string(),
        }
    }
}

impl WithIdentifier for HardwareEncryptionKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl SecureEncryptionKey for HardwareEncryptionKey {
    type Error = HardwareKeyStoreError;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let identifier = self.identifier.to_owned();
        let payload = msg.to_vec();
        let encrypted = spawn::blocking(|| get_encryption_key_bridge().encrypt(identifier, payload)).await?;
        Ok(encrypted)
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, HardwareKeyStoreError> {
        let identifier = self.identifier.to_owned();
        let payload = msg.to_vec();
        let decrypted = spawn::blocking(|| get_encryption_key_bridge().decrypt(identifier, payload)).await?;
        Ok(decrypted)
    }
}
