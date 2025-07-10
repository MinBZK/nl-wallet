use std::hash::Hash;
use std::hash::Hasher;

use base64::prelude::*;
use derive_more::AsRef;
use derive_more::From;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePrivateKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePublicKey;

use crate::utils::sha256;

/// Wraps a [`SigningKey`] and implements `TryFrom<Vec<u8>>` for deserialization using `serde_with`.
#[derive(Debug, Clone, From)]
pub struct DerSigningKey(SigningKey);

impl DerSigningKey {
    pub fn as_inner(&self) -> &SigningKey {
        let Self(signing_key) = self;

        signing_key
    }

    pub fn into_inner(self) -> SigningKey {
        let Self(signing_key) = self;

        signing_key
    }
}

impl TryFrom<Vec<u8>> for DerSigningKey {
    type Error = p256::pkcs8::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let signing_key = SigningKey::from_pkcs8_der(&value)?;

        Ok(Self(signing_key))
    }
}

/// Wraps a [`VerifyingKey`] and implements both `TryFrom<Vec<u8>>` and
/// `AsRef<[u8]>` in order to support (de)serialization using `serde_with`.
#[derive(Debug, Clone, AsRef)]
pub struct DerVerifyingKey(VerifyingKey, #[as_ref([u8])] Vec<u8>);

impl DerVerifyingKey {
    pub fn as_inner(&self) -> &VerifyingKey {
        let Self(verifying_key, _) = self;

        verifying_key
    }

    pub fn into_inner(self) -> VerifyingKey {
        let Self(verifying_key, _) = self;

        verifying_key
    }
}

impl From<VerifyingKey> for DerVerifyingKey {
    fn from(value: VerifyingKey) -> Self {
        let der = value
            .to_public_key_der()
            .expect("any p256 verifying key should be encodable to DER")
            .into_vec();

        Self(value, der)
    }
}

impl TryFrom<Vec<u8>> for DerVerifyingKey {
    type Error = p256::pkcs8::spki::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let verifying_key = VerifyingKey::from_public_key_der(&value)?;

        Ok(Self(verifying_key, value))
    }
}

impl PartialEq for DerVerifyingKey {
    fn eq(&self, other: &Self) -> bool {
        let Self(signature, _) = self;
        let Self(other_signature, _) = other;

        signature.eq(other_signature)
    }
}

impl Eq for DerVerifyingKey {}

impl Hash for DerVerifyingKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self(_, der) = self;

        der.hash(state)
    }
}

/// Wraps a [`Signature`] and implements both `TryFrom<Vec<u8>>` and `AsRef<[u8]>`
/// in order to support (de)serialization using `serde_with`.
#[derive(Debug, Clone, AsRef)]
pub struct DerSignature(Signature, #[as_ref([u8])] Vec<u8>);

impl DerSignature {
    pub fn as_inner(&self) -> &Signature {
        let Self(signature, _) = self;

        signature
    }

    pub fn into_inner(self) -> Signature {
        let Self(signature, _) = self;

        signature
    }
}

impl From<Signature> for DerSignature {
    fn from(value: Signature) -> Self {
        Self(value, value.to_der().to_bytes().to_vec())
    }
}

impl TryFrom<Vec<u8>> for DerSignature {
    type Error = p256::ecdsa::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let signature = Signature::from_der(&value)?;

        Ok(DerSignature(signature, value))
    }
}

pub fn verifying_key_sha256(key: &VerifyingKey) -> String {
    BASE64_STANDARD.encode(sha256(key.to_encoded_point(false).as_bytes()))
}
