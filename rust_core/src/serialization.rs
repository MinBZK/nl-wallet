use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use p256::{
    ecdsa::{Signature, VerifyingKey},
    pkcs8::{DecodePublicKey, EncodePublicKey},
};
use serde::{de, ser, Deserialize, Serialize};

use crate::wallet::signed::WalletSigned;

/// Bytes that (de)serialize to base64.
#[derive(Debug, Clone)]
pub struct Base64Bytes(pub Vec<u8>);
impl From<Vec<u8>> for Base64Bytes {
    fn from(val: Vec<u8>) -> Self {
        Base64Bytes(val)
    }
}

impl Serialize for Base64Bytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::serialize(&STANDARD_NO_PAD.encode(&self.0), serializer)
    }
}
impl<'de> Deserialize<'de> for Base64Bytes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(STANDARD_NO_PAD
            .decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(serde::de::Error::custom)?
            .into())
    }
}

/// ECDSA signature that (de)serializes from/to base64-encoded DER.
#[derive(Debug, Clone)]
pub struct DerSignature(pub Signature);
impl From<Signature> for DerSignature {
    fn from(val: Signature) -> Self {
        DerSignature(val)
    }
}

impl Serialize for DerSignature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Base64Bytes::serialize(&self.0.to_der().as_bytes().to_vec().into(), serializer)
    }
}
impl<'de> Deserialize<'de> for DerSignature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(
            Signature::from_der(&Base64Bytes::deserialize(deserializer)?.0)
                .map_err(de::Error::custom)?
                .into(),
        )
    }
}

/// ECDSA public key that (de)serializes from/to base64-encoded DER.
#[derive(Debug, Clone)]
pub struct DerVerifyingKey(pub VerifyingKey);
impl From<VerifyingKey> for DerVerifyingKey {
    fn from(val: VerifyingKey) -> Self {
        DerVerifyingKey(val)
    }
}

impl Serialize for DerVerifyingKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Base64Bytes::serialize(
            &self
                .0
                .to_public_key_der()
                .map_err(ser::Error::custom)?
                .into_vec()
                .into(),
            serializer,
        )
    }
}
impl<'de> Deserialize<'de> for DerVerifyingKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(
            VerifyingKey::from_public_key_der(&Base64Bytes::deserialize(deserializer)?.0)
                .map_err(de::Error::custom)?
                .into(),
        )
    }
}

impl<T> Serialize for WalletSigned<T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        String::serialize(&self.0, serializer)
    }
}
impl<'de, T> Deserialize<'de> for WalletSigned<T> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        String::deserialize(deserializer).map(WalletSigned::from)
    }
}
