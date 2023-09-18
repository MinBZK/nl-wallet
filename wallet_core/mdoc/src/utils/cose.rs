//! Cose objects, keys, parsing, and verification.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use ciborium::value::Value;
use coset::{
    iana, sig_structure_data, CoseMac0, CoseMac0Builder, CoseSign1, CoseSign1Builder, Header, HeaderBuilder, Label,
    ProtectedHeader, SignatureContext,
};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use ring::hmac;
use serde::{de::DeserializeOwned, Serialize};
use webpki::TrustAnchor;

use wallet_common::{generator::Generator, keys::SecureEcdsaKey};

use crate::{
    utils::serialization::{cbor_deserialize, cbor_serialize, CborError},
    Result,
};

use super::x509::{Certificate, CertificateError, CertificateUsage};

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
        MdocCose(cose, PhantomData)
    }
}

/// COSE header label for `x5chain`, defined in [RFC 9360](https://datatracker.ietf.org/doc/rfc9360/).
pub const COSE_X5CHAIN_HEADER_LABEL: i64 = 33;

impl<T> MdocCose<CoseSign1, T> {
    pub async fn sign(
        obj: &T,
        unprotected_header: Header,
        private_key: &(impl SecureEcdsaKey + Sync),
        include_payload: bool,
    ) -> Result<MdocCose<CoseSign1, T>>
    where
        T: Clone + Serialize,
    {
        let payload = cbor_serialize(obj).map_err(CoseError::Cbor)?;
        let cose = sign_cose(&payload, unprotected_header, private_key, include_payload).await;

        Ok(cose.into())
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

pub async fn sign_cose(
    payload: &[u8],
    unprotected_header: Header,
    private_key: &(impl SecureEcdsaKey + Sync),
    include_payload: bool,
) -> CoseSign1 {
    let protected_header = ProtectedHeader {
        original_data: None,
        header: HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build(),
    };

    let sig_data = sig_structure_data(
        SignatureContext::CoseSign1,
        protected_header.clone(),
        None,
        &[],
        payload,
    );

    let signature = private_key.sign(sig_data.as_ref()).await.to_vec();

    CoseSign1 {
        signature,
        payload: if include_payload {
            Some(Vec::from(payload))
        } else {
            None
        },
        protected: protected_header,
        unprotected: unprotected_header,
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
    use ciborium::Value;
    use coset::{Header, HeaderBuilder, Label};
    use p256::ecdsa::{signature::rand_core::OsRng, SigningKey};
    use serde::{Deserialize, Serialize};

    use wallet_common::generator::TimeGenerator;

    use crate::{
        utils::{
            cose::CoseError,
            x509::{Certificate, CertificateUsage},
        },
        Error,
    };

    use super::{ClonePayload, MdocCose, COSE_X5CHAIN_HEADER_LABEL};

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

    #[tokio::test]
    async fn it_works() {
        let key = SigningKey::random(&mut OsRng);
        let payload = ToyMessage::default();
        let cose = MdocCose::sign(&payload, Header::default(), &key, true).await.unwrap();

        cose.verify(key.verifying_key()).unwrap();

        let verified = cose.verify_and_parse(key.verifying_key()).unwrap();
        assert_eq!(payload, verified);

        let parsed_not_verified = cose.dangerous_parse_unverified().unwrap();
        assert_eq!(payload, parsed_not_verified);
    }

    #[tokio::test]
    async fn invalidate_cose() {
        let key = SigningKey::random(&mut OsRng);
        let payload = ToyMessage::default();
        let mut cose = MdocCose::sign(&payload, Header::default(), &key, true).await.unwrap();

        // Verification should fail if the signature is changed
        cose.0.signature[0] = !cose.0.signature[0]; // invert bits
        assert!(matches!(
            cose.verify(key.verifying_key()),
            Err(Error::Cose(CoseError::EcdsaSignatureVerificationFailed(_)))
        ));

        // Verification should fail if the signature length is not right
        let len = cose.0.signature.len();
        cose.0.signature.remove(len - 1);
        assert!(matches!(
            cose.verify(key.verifying_key()),
            Err(Error::Cose(CoseError::EcdsaSignatureParsingFailed(_)))
        ));
    }

    #[tokio::test]
    async fn cose_with_header() {
        let key = SigningKey::random(&mut OsRng);
        let payload = ToyMessage::default();
        let header = HeaderBuilder::new()
            .value(42, 0.into())
            .text_value("Hello".to_string(), "World".into())
            .build();
        let cose = MdocCose::sign(&payload, header, &key, true).await.unwrap();

        assert_eq!(
            cose.unprotected_header_item(&Label::Int(42))
                .unwrap()
                .as_integer()
                .unwrap(),
            0.into()
        );
        assert_eq!(
            cose.unprotected_header_item(&Label::Text("Hello".to_string()))
                .unwrap()
                .as_text()
                .unwrap(),
            "World"
        );

        assert!(matches!(
            cose.unprotected_header_item(&Label::Text("not_present".to_string())),
            Err(Error::Cose(CoseError::MissingLabel(_)))
        ))
    }

    #[tokio::test]
    async fn cose_with_certificate() {
        let (ca, ca_privkey) = Certificate::new_ca("ca.example.com").unwrap();
        let (cert, cert_privkey) =
            Certificate::new(&ca, &ca_privkey, "cert.example.com", CertificateUsage::Mdl).unwrap();

        let payload = ToyMessage::default();
        let header = HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(cert.as_bytes().to_vec()))
            .build();
        let cose = MdocCose::sign(&payload, header, &cert_privkey, true).await.unwrap();

        // Certificate should be present in the unprotected headers
        let header_cert = cose.signing_cert().unwrap();
        assert_eq!(cert.as_bytes(), header_cert.as_bytes());

        let trust_anchor = (&ca).try_into().unwrap();
        cose.verify_against_trust_anchors(CertificateUsage::Mdl, &TimeGenerator, &[trust_anchor])
            .unwrap();
    }

    #[tokio::test]
    async fn remove_add_payload() {
        let key = SigningKey::random(&mut OsRng);
        let payload = ToyMessage::default();

        let cose = MdocCose::sign(&payload, Header::default(), &key, true).await.unwrap();
        assert!(cose.0.payload.is_some());
        let payload_bts = cose.0.payload.as_ref().unwrap();

        let without_payload = cose.clone_without_payload();
        assert!(without_payload.0.payload.is_none());

        // Adding the payload should result in a cose containing our payload again
        let with_payload = without_payload.clone_with_payload(payload_bts.clone());
        let verified = with_payload.verify_and_parse(key.verifying_key()).unwrap();
        assert_eq!(payload, verified);
    }
}
