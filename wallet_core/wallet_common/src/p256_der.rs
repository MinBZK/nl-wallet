use derive_more::From;
use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;

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
