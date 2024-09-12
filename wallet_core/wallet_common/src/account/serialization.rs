use std::{
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
};

use base64::prelude::*;
use config::ValueKind;
use p256::{
    ecdsa::{Signature, SigningKey, VerifyingKey},
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey},
    SecretKey,
};
use serde::{de, ser, Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Padded,
    DeserializeAs, SerializeAs,
};

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
        Base64::<Standard, Padded>::serialize_as(&self.0.to_der().as_bytes(), serializer)
    }
}

impl<'de> Deserialize<'de> for DerSignature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes: Vec<u8> = Base64::<Standard, Padded>::deserialize_as::<D>(deserializer)?;
        let sig = Signature::from_der(&bytes).map_err(de::Error::custom)?;
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
        let pubkey = BASE64_STANDARD.encode(self.0.to_public_key_der().unwrap().as_bytes());
        write!(f, "{}", pubkey)
    }
}

impl<'de> Deserialize<'de> for DerSigningKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes: Vec<u8> = Base64::<Standard, Padded>::deserialize_as::<D>(deserializer)?;
        let key: SigningKey = SigningKey::from_pkcs8_der(&bytes).map_err(de::Error::custom)?;
        Ok(key.into())
    }
}

impl Serialize for DerSigningKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Base64::<Standard, Padded>::serialize_as(
            &self.0.to_pkcs8_der().map_err(ser::Error::custom)?.as_bytes().to_vec(),
            serializer,
        )
    }
}

/// ECDSA public key that (de)serializes from/to base64-encoded DER.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DerVerifyingKey(pub VerifyingKey);

impl From<VerifyingKey> for DerVerifyingKey {
    fn from(val: VerifyingKey) -> Self {
        DerVerifyingKey(val)
    }
}

impl Serialize for DerVerifyingKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Base64::<Standard, Padded>::serialize_as(
            &self.0.to_public_key_der().map_err(ser::Error::custom)?.into_vec(),
            serializer,
        )
    }
}

impl<'de> Deserialize<'de> for DerVerifyingKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes: Vec<u8> = Base64::<Standard, Padded>::deserialize_as::<D>(deserializer)?;
        let key = VerifyingKey::from_public_key_der(&bytes).map_err(de::Error::custom)?;
        Ok(key.into())
    }
}

impl Hash for DerVerifyingKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_sec1_bytes().hash(state);
    }
}

impl From<DerVerifyingKey> for ValueKind {
    fn from(value: DerVerifyingKey) -> Self {
        serde_json::to_value(value)
            .expect("DerVerifyingKey should be serializable to String")
            .as_str()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use crate::account::serialization::DerVerifyingKey;
    use config::Config;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_from_der_veriyfing_key_to_valuekind() {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct TestConfig {
            key1: DerVerifyingKey,
        }

        let signing_key = SigningKey::random(&mut OsRng);
        let veriyfing_key = *signing_key.verifying_key();
        let der_verifying_key: DerVerifyingKey = veriyfing_key.into();

        let test_config: TestConfig = Config::builder()
            .set_override("key1", der_verifying_key.clone())
            .unwrap()
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        assert_eq!(der_verifying_key.0, test_config.key1.0);
    }
}
