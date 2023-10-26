use crate::errors::Error;
use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
    Engine,
};
use p256::{
    ecdsa::{Signature, SigningKey, VerifyingKey},
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey},
    SecretKey,
};
use serde::{de, ser, Deserialize, Serialize};
use serde_json::value::RawValue;
use std::fmt::{Display, Formatter};

use super::signed::{SignedDouble, SignedInner};

/// Bytes that (de)serialize to base64.
#[derive(Debug, Clone)]
pub struct Base64Bytes(pub Vec<u8>);
impl From<Vec<u8>> for Base64Bytes {
    fn from(val: Vec<u8>) -> Self {
        Base64Bytes(val)
    }
}
impl PartialEq for Base64Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl TryFrom<Base64Bytes> for SigningKey {
    type Error = Error;

    fn try_from(value: Base64Bytes) -> Result<Self, Self::Error> {
        Ok(SigningKey::from_pkcs8_der(&value.0)?)
    }
}

impl TryFrom<&SigningKey> for Base64Bytes {
    type Error = Error;

    fn try_from(value: &SigningKey) -> Result<Self, Self::Error> {
        Ok(value.to_pkcs8_der()?.as_bytes().to_vec().into())
    }
}

impl Serialize for Base64Bytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::serialize(&STANDARD_NO_PAD.encode(&self.0), serializer)
    }
}
impl<'de> Deserialize<'de> for Base64Bytes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bts = STANDARD_NO_PAD
            .decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(serde::de::Error::custom)?;
        Ok(bts.into())
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
        let sig = Signature::from_der(&Base64Bytes::deserialize(deserializer)?.0).map_err(de::Error::custom)?;
        Ok(sig.into())
    }
}

/// ECDSA secret key that deserializes from base64-encoded DER.
#[derive(Debug, Clone)]
pub struct DerSecretKey(pub SecretKey);

impl From<SecretKey> for DerSecretKey {
    fn from(value: SecretKey) -> Self {
        DerSecretKey(value)
    }
}

// Reuse (de)serializer from DerSigningKey
impl<'de> Deserialize<'de> for DerSecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DerSigningKey::deserialize(deserializer).map(|x| DerSecretKey(x.0.into()))
    }
}
impl Serialize for DerSecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DerSigningKey(self.0.clone().into()).serialize(serializer)
    }
}

/// ECDSA signing key that deserializes from base64-encoded DER.
#[derive(Debug, Clone)]
pub struct DerSigningKey(pub SigningKey);

impl From<SigningKey> for DerSigningKey {
    fn from(val: SigningKey) -> Self {
        DerSigningKey(val)
    }
}

impl From<&DerSigningKey> for DerVerifyingKey {
    fn from(value: &DerSigningKey) -> Self {
        (*value.0.verifying_key()).into()
    }
}

impl Display for DerVerifyingKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pubkey = STANDARD.encode(self.0.to_public_key_der().unwrap().as_bytes());
        write!(f, "{}", pubkey)
    }
}

impl<'de> Deserialize<'de> for DerSigningKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let key: SigningKey = Base64Bytes::deserialize(deserializer)?
            .try_into()
            .map_err(de::Error::custom)?;
        Ok(key.into())
    }
}

impl Serialize for DerSigningKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Base64Bytes::try_from(&self.0)
            .map_err(ser::Error::custom)?
            .serialize(serializer)
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

impl PartialEq for DerVerifyingKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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
        let key =
            VerifyingKey::from_public_key_der(&Base64Bytes::deserialize(deserializer)?.0).map_err(de::Error::custom)?;
        Ok(key.into())
    }
}

impl<T> Serialize for SignedDouble<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        RawValue::serialize(&RawValue::from_string(self.0.clone()).unwrap(), serializer)
    }
}
impl<'de, T> Deserialize<'de> for SignedDouble<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Ok(Box::<RawValue>::deserialize(deserializer)?.get().into())
    }
}

impl<T> Serialize for SignedInner<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        RawValue::serialize(&RawValue::from_string(self.0.clone()).unwrap(), serializer)
    }
}
impl<'de, T> Deserialize<'de> for SignedInner<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Ok(Box::<RawValue>::deserialize(deserializer)?.get().into())
    }
}
