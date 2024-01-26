use p256::{
    ecdsa::{Signature, SigningKey},
    pkcs8::DecodePrivateKey,
};

use wallet_common::keys::{EcdsaKey, SecureEcdsaKey};

use crate::{utils::x509::Certificate, Result};

pub struct KeyPair {
    private_key: SigningKey,
    cert_bts: Certificate,
}

#[derive(thiserror::Error, Debug)]
pub enum KeysError {
    #[error("failed to parse DER-encoded private key: {0}")]
    DerParsing(#[from] p256::pkcs8::Error),
    #[error("key generation error: {0}")]
    KeyGeneration(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl KeyPair {
    pub fn new(private_key: SigningKey, cert_bts: Certificate) -> KeyPair {
        KeyPair { private_key, cert_bts }
    }

    pub fn from_der(private_key: &[u8], cert: &[u8]) -> Result<KeyPair> {
        let key = Self::new(
            SigningKey::from_pkcs8_der(private_key).map_err(KeysError::DerParsing)?,
            Certificate::from(cert),
        );
        Ok(key)
    }

    pub fn private_key(&self) -> &SigningKey {
        &self.private_key
    }

    pub fn certificate(&self) -> &Certificate {
        &self.cert_bts
    }
}

impl From<KeyPair> for Certificate {
    fn from(source: KeyPair) -> Certificate {
        source.cert_bts
    }
}

impl EcdsaKey for KeyPair {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> std::result::Result<p256::ecdsa::VerifyingKey, Self::Error> {
        Ok(*self.private_key.verifying_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, Self::Error> {
        p256::ecdsa::signature::Signer::try_sign(&self.private_key, msg)
    }
}
impl SecureEcdsaKey for KeyPair {}

pub trait KeyRing {
    fn private_key(&self, id: &str) -> Option<&KeyPair>;
    fn contains_key(&self, id: &str) -> bool {
        self.private_key(id).is_some()
    }
}

/// An implementation of [`KeyRing`] containing a single key.
pub struct SingleKeyRing(pub KeyPair);

impl KeyRing for SingleKeyRing {
    fn private_key(&self, _: &str) -> Option<&KeyPair> {
        Some(&self.0)
    }
}
