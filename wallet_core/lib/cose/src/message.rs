use std::marker::PhantomData;

use ciborium::value::Value;
use coset::CoseMac0;
use coset::CoseSign1;
use coset::Header;
use coset::Label;
use coset::ProtectedHeader;
use coset::iana;
use crypto::keys::PublicKey;
use derive_more::AsMut;
use derive_more::AsRef;
use p256::ecdsa::signature::Verifier as _;
use ring::hmac;
use serde::de::DeserializeOwned;

use crate::error::CoseError;
use crate::serialization::cbor_deserialize;

/// Trait implemented by the supported COSE message types.
pub trait Cose {
    type Key;
    const ALGORITHMS: &[iana::Algorithm];

    fn payload(&self) -> Option<&[u8]>;
    fn protected(&self) -> &ProtectedHeader;
    fn unprotected(&self) -> &Header;
    fn verify(&self, key: &Self::Key) -> Result<(), CoseError>;

    fn validate_algorithm(&self) -> Result<iana::Algorithm, CoseError> {
        match self.protected().header.alg.as_ref() {
            Some(coset::Algorithm::Assigned(algorithm)) if Self::ALGORITHMS.contains(algorithm) => Ok(*algorithm),
            Some(algorithm) => Err(CoseError::UnsupportedAlgorithm(algorithm.clone())),
            None => Err(CoseError::MissingAlgorithm),
        }
    }
}

impl Cose for CoseSign1 {
    type Key = PublicKey;
    const ALGORITHMS: &[iana::Algorithm] = &[
        iana::Algorithm::ES256,
        iana::Algorithm::ESP256,
        iana::Algorithm::ES384,
        iana::Algorithm::ESP384,
        iana::Algorithm::RS256,
        iana::Algorithm::RS384,
        iana::Algorithm::RS512,
        iana::Algorithm::PS256,
        iana::Algorithm::PS384,
        iana::Algorithm::PS512,
    ];

    fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }

    fn protected(&self) -> &ProtectedHeader {
        &self.protected
    }

    fn unprotected(&self) -> &Header {
        &self.unprotected
    }

    fn verify(&self, key: &PublicKey) -> Result<(), CoseError> {
        if self.payload.is_none() {
            return Err(CoseError::MissingPayload);
        }

        let algorithm = self.validate_algorithm()?;
        self.verify_signature(b"", |signature, data| {
            match (key, algorithm) {
                (PublicKey::ESP256(verifying_key), iana::Algorithm::ES256 | iana::Algorithm::ESP256) => {
                    p256::ecdsa::Signature::from_slice(signature)
                        .and_then(|signature| verifying_key.verify(data, &signature))
                        .map_err(CoseError::Esp256SignatureVerificationFailed)
                }
                (PublicKey::ESP384(verifying_key), iana::Algorithm::ES384 | iana::Algorithm::ESP384) => {
                    p384::ecdsa::Signature::from_slice(signature)
                        .and_then(|signature| verifying_key.verify(data, &signature))
                        .map_err(CoseError::Esp384SignatureVerificationFailed)
                }
                (PublicKey::RSA(rsa_public_key), iana::Algorithm::RS256) => {
                    verify_rsa_pkcs1v15::<sha2::Sha256>(data, signature, rsa_public_key.as_ref().clone())
                }
                (PublicKey::RSA(rsa_public_key), iana::Algorithm::RS384) => {
                    verify_rsa_pkcs1v15::<sha2::Sha384>(data, signature, rsa_public_key.as_ref().clone())
                }
                (PublicKey::RSA(rsa_public_key), iana::Algorithm::RS512) => {
                    verify_rsa_pkcs1v15::<sha2::Sha512>(data, signature, rsa_public_key.as_ref().clone())
                }
                (PublicKey::RSA(rsa_public_key), iana::Algorithm::PS256) => {
                    verify_rsa_pss::<sha2::Sha256>(data, signature, rsa_public_key.as_ref().clone())
                }
                (PublicKey::RSA(rsa_public_key), iana::Algorithm::PS384) => {
                    verify_rsa_pss::<sha2::Sha384>(data, signature, rsa_public_key.as_ref().clone())
                }
                (PublicKey::RSA(rsa_public_key), iana::Algorithm::PS512) => {
                    verify_rsa_pss::<sha2::Sha512>(data, signature, rsa_public_key.as_ref().clone())
                }
                (_, _) => Err(CoseError::MismatchAlgorithmWithPublicKey(algorithm)),
            }?;

            Ok(())
        })
    }
}

fn verify_rsa_pkcs1v15<D>(data: &[u8], signature: &[u8], public_key: rsa::PublicKey) -> Result<(), CoseError>
where
    D: sha2::Digest + rsa::pkcs8::AssociatedOid,
{
    rsa::pkcs1v15::Signature::try_from(signature)
        .and_then(|signature| {
            let verifying_key = rsa::pkcs1v15::VerifyingKey::<D>::new(public_key);
            verifying_key.verify(data, &signature)
        })
        .map_err(CoseError::RsaPkcs1v15SignatureVerificationFailed)
}

fn verify_rsa_pss<D>(data: &[u8], signature: &[u8], public_key: rsa::PublicKey) -> Result<(), CoseError>
where
    D: sha2::digest::Digest + sha2::digest::FixedOutputReset,
{
    rsa::pss::Signature::try_from(signature)
        .and_then(|signature| {
            let verifying_key = rsa::pss::VerifyingKey::<D>::new(public_key);
            verifying_key.verify(data, &signature)
        })
        .map_err(CoseError::RsaPssSignatureVerificationFailed)
}

impl Cose for CoseMac0 {
    type Key = hmac::Key;
    const ALGORITHMS: &[iana::Algorithm] = &[iana::Algorithm::HMAC_256_256];

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
        self.validate_algorithm()?;
        self.verify_payload_tag(
            b"",
            || CoseError::MissingPayload,
            |tag, data| hmac::verify(key, data, tag).map_err(|_| CoseError::MacVerificationFailed),
        )?;
        Ok(())
    }
}

/// A COSE message coupled to the Rust type of its CBOR payload.
#[derive(derive_more::Debug, PartialEq, Eq, AsMut, AsRef)]
pub struct TypedCose<C, T>(
    #[as_mut]
    #[as_ref]
    C,
    PhantomData<T>,
);

impl<C, T> TypedCose<C, T> {
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
        self.as_ref().verify(key)
    }

    pub fn unprotected_header_item(&self, label: &Label) -> Result<&Value, CoseError> {
        self.as_ref()
            .unprotected()
            .rest
            .iter()
            .find(|(candidate, _)| candidate == label)
            .map(|(_, value)| value)
            .ok_or_else(|| CoseError::MissingLabel(label.clone()))
    }

    pub fn protected_header(&self) -> &Header {
        &self.as_ref().protected().header
    }
}

impl<C, T> TypedCose<C, T>
where
    C: Cose,
    T: DeserializeOwned,
{
    /// Returns algorithm
    pub fn algorithm(&self) -> Result<iana::Algorithm, CoseError> {
        self.as_ref().validate_algorithm()
    }

    /// Parse the payload without verifying the COSE signature or MAC.
    ///
    /// The authenticity of the returned payload is not established. Prefer [`TypedCose::verify_and_parse`].
    pub fn dangerous_parse_unverified(&self) -> Result<T, CoseError> {
        cbor_deserialize(self.as_ref().payload().ok_or(CoseError::MissingPayload)?).map_err(CoseError::Cbor)
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
