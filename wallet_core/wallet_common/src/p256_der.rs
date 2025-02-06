use derive_more::AsRef;
use derive_more::From;
use p256::ecdsa::Signature;
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
