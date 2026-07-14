use std::marker::PhantomData;

use ciborium::value::Value;
use coset::CoseMac0;
use coset::CoseSign1;
use coset::Header;
use coset::Label;
use coset::ProtectedHeader;
use coset::iana;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Verifier;
use ring::hmac;
use serde::de::DeserializeOwned;

use crate::error::CoseError;
use crate::serialization::cbor_deserialize;

/// Trait implemented by the supported COSE message types.
pub trait Cose {
    type Key;
    const ALGORITHM: iana::Algorithm;

    fn payload(&self) -> Option<&[u8]>;
    fn protected(&self) -> &ProtectedHeader;
    fn unprotected(&self) -> &Header;
    fn verify(&self, key: &Self::Key) -> Result<(), CoseError>;
}

impl Cose for CoseSign1 {
    type Key = VerifyingKey;
    const ALGORITHM: iana::Algorithm = iana::Algorithm::ES256;

    fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }

    fn protected(&self) -> &ProtectedHeader {
        &self.protected
    }

    fn unprotected(&self) -> &Header {
        &self.unprotected
    }

    fn verify(&self, key: &VerifyingKey) -> Result<(), CoseError> {
        self.verify_signature(b"", |signature, data| {
            if self.payload.is_none() {
                return Err(CoseError::MissingPayload);
            }

            let signature = &Signature::try_from(signature).map_err(CoseError::EcdsaSignatureParsingFailed)?;
            key.verify(data, signature)
                .map_err(CoseError::EcdsaSignatureVerificationFailed)?;
            Ok(())
        })
    }
}

impl Cose for CoseMac0 {
    type Key = hmac::Key;
    const ALGORITHM: iana::Algorithm = iana::Algorithm::HMAC_256_256;

    fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }

    fn protected(&self) -> &ProtectedHeader {
        &self.protected
    }

    fn unprotected(&self) -> &Header {
        &self.unprotected
    }

    fn verify(&self, key: &hmac::Key) -> Result<(), CoseError> {
        if self.payload.is_none() {
            return Err(CoseError::MissingPayload);
        }

        self.verify_tag(b"", |tag, data| {
            hmac::verify(key, data, tag).map_err(|_| CoseError::MacVerificationFailed)
        })?;
        Ok(())
    }
}

/// A COSE message coupled to the Rust type of its CBOR payload.
#[derive(Debug, PartialEq, Eq)]
pub struct TypedCose<C, T>(C, PhantomData<T>);

impl<C, T> TypedCose<C, T> {
    pub fn as_inner(&self) -> &C {
        &self.0
    }

    pub fn as_inner_mut(&mut self) -> &mut C {
        &mut self.0
    }

    pub fn into_inner(self) -> C {
        self.0
    }
}

impl<C: Clone, T> Clone for TypedCose<C, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<C, T> TypedCose<C, T>
where
    C: Cose,
{
    /// Verify the COSE message using the specified key.
    pub fn verify(&self, key: &C::Key) -> Result<(), CoseError> {
        self.validate_algorithm()?;
        self.as_inner().verify(key)
    }

    pub fn unprotected_header_item(&self, label: &Label) -> Result<&Value, CoseError> {
        self.as_inner()
            .unprotected()
            .rest
            .iter()
            .find(|(candidate, _)| candidate == label)
            .map(|(_, value)| value)
            .ok_or_else(|| CoseError::MissingLabel(label.clone()))
    }

    pub fn protected_header(&self) -> &Header {
        &self.as_inner().protected().header
    }

    pub(crate) fn validate_algorithm(&self) -> Result<(), CoseError> {
        match self.protected_header().alg.as_ref() {
            Some(coset::Algorithm::Assigned(algorithm)) if algorithm == &C::ALGORITHM => Ok(()),
            Some(algorithm) => Err(CoseError::UnsupportedAlgorithm(algorithm.clone())),
            None => Err(CoseError::MissingAlgorithm),
        }
    }
}

impl<C, T> TypedCose<C, T>
where
    C: Cose,
    T: DeserializeOwned,
{
    /// Parse the payload without verifying the COSE signature or MAC.
    ///
    /// The authenticity of the returned payload is not established. Prefer [`TypedCose::verify_and_parse`].
    pub fn dangerous_parse_unverified(&self) -> Result<T, CoseError> {
        cbor_deserialize(self.as_inner().payload().ok_or(CoseError::MissingPayload)?).map_err(CoseError::Cbor)
    }

    /// Verify the COSE message and deserialize its authenticated CBOR payload.
    pub fn verify_and_parse(&self, key: &C::Key) -> Result<T, CoseError> {
        self.verify(key)?;
        self.dangerous_parse_unverified()
    }
}

impl<C, T> From<C> for TypedCose<C, T> {
    fn from(cose: C) -> Self {
        Self(cose, PhantomData)
    }
}
