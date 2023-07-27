//! Cose objects, keys, parsing, and verification.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use ciborium::value::Value;
use coset::{iana, CoseMac0, CoseMac0Builder, CoseSign1, CoseSign1Builder, Header, HeaderBuilder, Label};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use ring::hmac;
use serde::{de::DeserializeOwned, Serialize};
use webpki::TrustAnchor;

use crate::{
    utils::serialization::{cbor_deserialize, cbor_serialize, CborError},
    utils::signer::SecureEcdsaKey,
    Result,
};

use super::{
    x509::{Certificate, CertificateError, CertificateUsage},
    Generator,
};

/// Trait for supported Cose variations ([`CoseSign1`] or [`CoseMac0`]).
pub trait Cose {
    type Key;
    fn payload(&self) -> &Option<Vec<u8>>;
    fn unprotected(&self) -> &Header;
    fn verify(&self, key: &Self::Key) -> Result<()>;
}

#[derive(thiserror::Error, Debug)]
pub enum CoseError {
    #[error("missing payload")]
    MissingPayload,
    #[error("missing label {0:?}")]
    MissingLabel(Label),
    #[error("ECDSA signature parsing failed: {0}")]
    EcdsaSignatureParsingFailed(p256::ecdsa::Error),
    #[error("ECDSA signature verification failed: {0}")]
    EcdsaSignatureVerificationFailed(p256::ecdsa::Error),
    #[error("MAC verification failed")]
    MacVerificationFailed,
    #[error(transparent)]
    Cbor(#[from] CborError),
    #[error("signing certificate header did not contain bytes")]
    CertificateUnexpectedHeaderType,
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
}

impl Cose for CoseSign1 {
    type Key = VerifyingKey;
    fn payload(&self) -> &Option<Vec<u8>> {
        &self.payload
    }
    fn unprotected(&self) -> &Header {
        &self.unprotected
    }
    fn verify(&self, key: &VerifyingKey) -> Result<()> {
        self.verify_signature(b"", |sig, data| {
            let sig = &Signature::try_from(sig).map_err(CoseError::EcdsaSignatureParsingFailed)?;
            key.verify(data, sig)
                .map_err(CoseError::EcdsaSignatureVerificationFailed)?;
            Ok(())
        })
    }
}

impl Cose for CoseMac0 {
    type Key = hmac::Key;
    fn payload(&self) -> &Option<Vec<u8>> {
        &self.payload
    }
    fn unprotected(&self) -> &Header {
        &self.unprotected
    }
    fn verify(&self, key: &hmac::Key) -> Result<()> {
        self.verify_tag(b"", |tag, data| {
            hmac::verify(key, data, tag).map_err(|_| CoseError::MacVerificationFailed)
        })?;
        Ok(())
    }
}

/// Wrapper around [`Cose`] implementors adding typesafe verification and CBOR parsing functions.
#[derive(Debug, Clone)]
pub struct MdocCose<C, T>(pub C, PhantomData<T>);

impl<C, T> MdocCose<C, T>
where
    C: Cose,
    T: DeserializeOwned,
{
    /// Parse and return the payload without verifying the Cose signature.
    /// DANGEROUS: this ignores the Cose signature/mac entirely, so the authenticity of the Cose and
    /// its payload is in no way guaranteed. Use [`MdocCose::verify_and_parse()`] instead if possible.
    fn dangerous_parse_unverified(&self) -> Result<T> {
        let payload = cbor_deserialize(
            self.0
                .payload()
                .as_ref()
                .ok_or_else(|| CoseError::MissingPayload)?
                .as_slice(),
        )
        .map_err(CoseError::Cbor)?;
        Ok(payload)
    }

    /// Verify the Cose using the specified key.
    pub fn verify(&self, key: &C::Key) -> Result<()> {
        self.0.verify(key)
    }

    /// Verify the Cose using the specified key, and if the Cose is valid,
    /// CBOR-deserialize and return its payload.
    pub fn verify_and_parse(&self, key: &C::Key) -> Result<T> {
        self.verify(key)?;
        self.dangerous_parse_unverified()
    }

    pub fn unprotected_header_item(&self, label: &Label) -> Result<&Value> {
        let header_item = &self
            .0
            .unprotected()
            .rest
            .iter()
            .find(|(l, _)| l == label)
            .ok_or_else(|| CoseError::MissingLabel(label.clone()))?
            .1;
        Ok(header_item)
    }
}

impl<C, T> From<C> for MdocCose<C, T> {
    fn from(cose: C) -> Self {
        MdocCose(cose, PhantomData::default())
    }
}

/// COSE header label for `x5chain`, defined in [RFC 9360](https://datatracker.ietf.org/doc/rfc9360/).
pub const COSE_X5CHAIN_HEADER_LABEL: i64 = 33;

impl<T> MdocCose<CoseSign1, T> {
    pub fn sign(
        obj: &T,
        unprotected_header: Header,
        private_key: &impl SecureEcdsaKey,
    ) -> Result<MdocCose<CoseSign1, T>>
    where
        T: Clone + Serialize,
    {
        let cose = CoseSign1Builder::new()
            .payload(cbor_serialize(obj).map_err(CoseError::Cbor)?)
            .protected(HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build())
            .unprotected(unprotected_header)
            .create_signature(&[], |data| private_key.sign(data).to_vec())
            .build()
            .into();
        Ok(cose)
    }

    // TODO deal with possible multiple certs being present here, https://datatracker.ietf.org/doc/draft-ietf-cose-x509/
    /// Get the [`Certificate`] containing the public key with which the MSO is signed from the unsigned COSE header.
    pub fn signing_cert(&self) -> Result<Certificate>
    where
        T: DeserializeOwned,
    {
        let cert_bts = self
            .unprotected_header_item(&Label::Int(COSE_X5CHAIN_HEADER_LABEL))?
            .as_bytes()
            .ok_or_else(|| CoseError::CertificateUnexpectedHeaderType)?;

        let cert = Certificate::from(cert_bts);
        Ok(cert)
    }

    /// Verify the COSE against the specified trust anchors, using the certificate(s) in the `x5chain` COSE header
    /// as intermediate certificates.
    pub fn verify_against_trust_anchors(
        &self,
        usage: CertificateUsage,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let cert = self.signing_cert()?;

        // Verify the certificate against the trusted IACAs
        cert.verify(usage, &[], time, trust_anchors)
            .map_err(CoseError::Certificate)?;

        // Grab the certificate's public key and verify the Cose
        let issuer_pk = cert.public_key().map_err(CoseError::Certificate)?;
        self.verify_and_parse(&issuer_pk)
    }
}

pub trait ClonePayload {
    fn clone_with_payload(&self, bts: Vec<u8>) -> Self;
    fn clone_without_payload(&self) -> Self;
}

impl<C, T> ClonePayload for MdocCose<C, T>
where
    C: ClonePayload + Cose,
{
    fn clone_with_payload(&self, bts: Vec<u8>) -> Self {
        self.0.clone_with_payload(bts).into()
    }
    fn clone_without_payload(&self) -> Self {
        self.0.clone_without_payload().into()
    }
}

impl ClonePayload for CoseSign1 {
    fn clone_with_payload(&self, bts: Vec<u8>) -> CoseSign1 {
        CoseSign1Builder::new()
            .signature(self.signature.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .payload(bts)
            .build()
    }

    fn clone_without_payload(&self) -> CoseSign1 {
        CoseSign1Builder::new()
            .signature(self.signature.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .build()
    }
}

impl ClonePayload for CoseMac0 {
    fn clone_with_payload(&self, bts: Vec<u8>) -> CoseMac0 {
        CoseMac0Builder::new()
            .tag(self.tag.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .payload(bts)
            .build()
    }

    fn clone_without_payload(&self) -> CoseMac0 {
        CoseMac0Builder::new()
            .tag(self.tag.clone())
            .protected(self.protected.header.clone())
            .unprotected(self.unprotected.clone())
            .build()
    }
}

#[derive(Debug, Clone)]
pub struct CoseKey(pub coset::CoseKey);
impl From<coset::CoseKey> for CoseKey {
    fn from(key: coset::CoseKey) -> Self {
        CoseKey(key)
    }
}

impl coset::AsCborValue for CoseKey {
    fn from_cbor_value(value: Value) -> coset::Result<Self> {
        let deserialized = coset::CoseKey::from_cbor_value(value)?.into();
        Ok(deserialized)
    }
    fn to_cbor_value(self) -> coset::Result<Value> {
        self.0.to_cbor_value()
    }
}

#[cfg(test)]
mod tests {
    use coset::{CoseSign1, Header};
    use p256::ecdsa::{signature::rand_core::OsRng, SigningKey};
    use serde::{Deserialize, Serialize};

    use super::MdocCose;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct ToyMessage {
        number: u8,
        string: String,
    }
    impl Default for ToyMessage {
        fn default() -> Self {
            Self {
                number: 42,
                string: "Hello, world!".to_string(),
            }
        }
    }

    #[test]
    fn it_works() {
        let key = SigningKey::random(&mut OsRng);
        let payload = ToyMessage::default();
        let cose = MdocCose::<CoseSign1, ToyMessage>::sign(&payload, Header::default(), &key).unwrap();
        let verified = cose.verify_and_parse(key.verifying_key()).unwrap();
        assert_eq!(payload, verified);
    }
}
