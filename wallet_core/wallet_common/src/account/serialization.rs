use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

use base64::prelude::*;
use config::ValueKind;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePublicKey;
use serde::de;
use serde::ser;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::base64::Standard;
use serde_with::formats::Padded;
use serde_with::DeserializeAs;
use serde_with::SerializeAs;

impl Display for DerVerifyingKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pubkey = BASE64_STANDARD.encode(self.0.to_public_key_der().unwrap().as_bytes());
        write!(f, "{}", pubkey)
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
    use serde::Deserialize;
    use serde::Serialize;

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
