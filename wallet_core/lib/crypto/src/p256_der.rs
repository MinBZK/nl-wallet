use std::hash::Hash;
use std::hash::Hasher;

use base64::prelude::*;
use derive_more::AsRef;
use derive_more::From;
use p256::SecretKey;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePrivateKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePrivateKey;
use p256::pkcs8::EncodePublicKey;
use serde::Deserializer;
use serde::Serializer;
use serde_with::Bytes;
use serde_with::DeserializeAs;
use serde_with::SerializeAs;
use zeroize::Zeroizing;

use crate::keys::AsPublicKeyRef;
use crate::keys::PublicKeyRef;
use crate::utils::sha256;

/// A PKCS#8 DER adapter for a [`SigningKey`].
pub type DerSigningKey = DerPrivateKey<SigningKey>;

/// A PKCS#8 DER adapter for a [`SecretKey`].
pub type DerSecretKey = DerPrivateKey<SecretKey>;

/// Serde adapter for reading and writing P-256 private keys as PKCS#8 DER bytes.
#[derive(Debug, Clone, From)]
pub struct DerPrivateKey<T>(T);

impl<T> DerPrivateKey<T> {
    pub fn as_inner(&self) -> &T {
        let Self(private_key) = self;

        private_key
    }

    pub fn into_inner(self) -> T {
        let Self(private_key) = self;

        private_key
    }
}

// This trait is a workaround for the `TryFrom<Vec<u8>>` implementation below, which would conflict with a Rust blanket
// implementation if the trait bound were `DecodePrivateKey`.
trait PrivateKey: DecodePrivateKey {}
impl PrivateKey for SigningKey {}
impl PrivateKey for SecretKey {}

impl<T> TryFrom<Vec<u8>> for DerPrivateKey<T>
where
    T: PrivateKey,
{
    type Error = p256::pkcs8::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let private_key = T::from_pkcs8_der(&Zeroizing::new(value))?;

        Ok(Self(private_key))
    }
}

impl<T: EncodePrivateKey> SerializeAs<T> for DerPrivateKey<T> {
    fn serialize_as<S: Serializer>(key: &T, serializer: S) -> Result<S::Ok, S::Error> {
        let der = key.to_pkcs8_der().map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(der.as_bytes())
    }
}

impl<'de, T: PrivateKey> DeserializeAs<'de, T> for DerPrivateKey<T> {
    fn deserialize_as<D: Deserializer<'de>>(deserializer: D) -> Result<T, D::Error> {
        let der: Vec<u8> = Bytes::deserialize_as(deserializer)?;
        Self::try_from(der)
            .map(Self::into_inner)
            .map_err(serde::de::Error::custom)
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

impl AsPublicKeyRef for DerVerifyingKey {
    fn as_public_key_ref<'a>(&'a self) -> PublicKeyRef<'a> {
        PublicKeyRef::ESP256(&self.0)
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
    BASE64_STANDARD.encode(sha256(key.to_sec1_point(false).as_bytes()))
}
