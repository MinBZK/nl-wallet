//! Cose objects, keys, parsing, and verification.

use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use ciborium::value::Value;
use coset::CoseMac0;
use coset::CoseMac0Builder;
use coset::CoseSign1;
use coset::CoseSign1Builder;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::ProtectedHeader;
use coset::SignatureContext;
use coset::iana;
use coset::sig_structure_data;
use crypto::keys::CredentialEcdsaKey;
use crypto::keys::EcdsaKey;
use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::trust_anchor::TrustAnchors;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use error_category::ErrorCategory;
use itertools::Itertools;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Verifier;
use ring::hmac;
use serde::Serialize;
use serde::de::DeserializeOwned;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::utils::serialization::CborError;
use crate::utils::serialization::cbor_deserialize;
use crate::utils::serialization::cbor_serialize;

/// Trait for supported Cose variations ([`CoseSign1`] or [`CoseMac0`]).
pub trait Cose {
    type Key;
    fn payload(&self) -> Option<&[u8]>;
    fn protected(&self) -> &ProtectedHeader;
    fn unprotected(&self) -> &Header;
    fn verify(&self, key: &Self::Key) -> Result<(), CoseError>;
}

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum CoseError {
    #[error("missing payload")]
    #[category(critical)]
    MissingPayload,
    #[error("missing label {0:?}")]
    #[category(critical)]
    MissingLabel(Label),
    #[error("ECDSA signature parsing failed: {0}")]
    #[category(pd)]
    EcdsaSignatureParsingFailed(p256::ecdsa::Error),
    #[error("ECDSA signature verification failed: {0}")]
    #[category(pd)]
    EcdsaSignatureVerificationFailed(p256::ecdsa::Error),
    #[error("MAC verification failed")]
    #[category(critical)]
    MacVerificationFailed,
    #[error(transparent)]
    Cbor(#[from] CborError),
    #[error("signing certificate header did not contain bytes")]
    #[category(critical)]
    CertificateUnexpectedHeaderType,
    #[error("x5chain certificate chain is empty")]
    #[category(critical)]
    EmptyCertificateChain,
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
    #[error("signing failed: {0}")]
    #[category(pd)]
    Signing(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("no signature received")]
    #[category(critical)]
    SignatureMissing,
}

impl Cose for CoseSign1 {
    type Key = VerifyingKey;

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
        self.verify_signature(b"", |sig, data| {
            if self.payload.is_none() {
                return Err(CoseError::MissingPayload);
            }

            let sig = &Signature::try_from(sig).map_err(CoseError::EcdsaSignatureParsingFailed)?;
            key.verify(data, sig)
                .map_err(CoseError::EcdsaSignatureVerificationFailed)?;
            Ok(())
        })
    }
}

impl Cose for CoseMac0 {
    type Key = hmac::Key;

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

/// Wrapper around [`Cose`] implementors adding typesafe verification and CBOR parsing functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdocCose<C, T>(pub C, PhantomData<T>);

impl<C, T> MdocCose<C, T>
where
    C: Cose,
    T: DeserializeOwned,
{
    /// Parse and return the payload without verifying the Cose signature.
    /// DANGEROUS: this ignores the Cose signature/mac entirely, so the authenticity of the Cose and
    /// its payload is in no way guaranteed. Use [`MdocCose::verify_and_parse()`] instead if possible.
    pub fn dangerous_parse_unverified(&self) -> Result<T, CoseError> {
        let payload =
            cbor_deserialize(self.0.payload().ok_or_else(|| CoseError::MissingPayload)?).map_err(CoseError::Cbor)?;
        Ok(payload)
    }

    /// Verify the Cose using the specified key.
    pub fn verify(&self, key: &C::Key) -> Result<(), CoseError> {
        self.0.verify(key)
    }

    /// Verify the Cose using the specified key, and if the Cose is valid,
    /// CBOR-deserialize and return its payload.
    pub fn verify_and_parse(&self, key: &C::Key) -> Result<T, CoseError> {
        self.verify(key)?;
        self.dangerous_parse_unverified()
    }

    pub fn unprotected_header_item(&self, label: &Label) -> Result<&Value, CoseError> {
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

    pub fn protected_header(&self) -> &Header {
        &self.0.protected().header
    }
}

impl<C, T> From<C> for MdocCose<C, T> {
    fn from(cose: C) -> Self {
        MdocCose(cose, PhantomData)
    }
}

/// COSE header label for `x5chain`, defined in [RFC 9360](https://datatracker.ietf.org/doc/rfc9360/).
pub const COSE_X5CHAIN_HEADER_LABEL: i64 = 33;

const ONE: NonZeroUsize = NonZeroUsize::new(1usize).expect("1 is non-zero");

pub fn header_with_x5chain(chain: &VecNonEmpty<&BorrowingCertificate>) -> Header {
    let cbor_encode = |c: &&BorrowingCertificate| Value::Bytes(c.to_vec());

    if chain.len() == ONE {
        let cert = chain.first();
        HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, cbor_encode(cert))
            .build()
    } else {
        let certs = chain.iter().map(cbor_encode).collect_vec();
        HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Array(certs))
            .build()
    }
}

impl<T> MdocCose<CoseSign1, T> {
    pub async fn sign(
        obj: &T,
        unprotected_header: Header,
        private_key: &impl EcdsaKey,
        include_payload: bool,
    ) -> Result<MdocCose<CoseSign1, T>, CoseError>
    where
        T: Clone + Serialize,
    {
        let payload = cbor_serialize(obj).map_err(CoseError::Cbor)?;
        let cose = sign_cose(&payload, unprotected_header, private_key, include_payload).await?;

        Ok(cose.into())
    }

    /// Get the [`BorrowingCertificate`] chain from the unsigned COSE header field `x5chain`.
    pub fn x5chain(&self) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError>
    where
        T: DeserializeOwned,
    {
        let header_item = self.unprotected_header_item(&Label::Int(COSE_X5CHAIN_HEADER_LABEL))?;

        match header_item {
            Value::Bytes(bytes) => Ok(vec_nonempty![BorrowingCertificate::from_der(bytes.clone())?]),
            Value::Array(items) => {
                let certificates: Vec<_> = items
                    .iter()
                    .map(|item| {
                        item.as_bytes()
                            .ok_or(CoseError::CertificateUnexpectedHeaderType)
                            .and_then(|bytes| Ok(BorrowingCertificate::from_der(bytes.clone())?))
                    })
                    .try_collect()?;
                certificates.try_into().map_err(|_| CoseError::EmptyCertificateChain)
            }
            _ => Err(CoseError::CertificateUnexpectedHeaderType),
        }
    }

    /// Verify the COSE against the specified trust anchors, using the certificate(s) in the `x5chain` COSE header
    /// as intermediate certificates.
    pub fn verify_against_trust_anchors(
        &self,
        usage: CertificateUsage,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[BorrowingTrustAnchor],
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let (cert, chain) = self.x5chain()?.into_nonempty_iter().next();

        let chain = chain.into_iter().collect_vec();
        let trust_anchors = TrustAnchors::try_from(trust_anchors.to_vec())?;

        // Verify the certificate against the trusted IACAs
        cert.verify(usage, &chain, time, &trust_anchors)
            .map_err(CoseError::Certificate)?;

        // Grab the certificate's public key and verify the Cose
        self.verify_and_parse(cert.public_key())
    }
}

fn signature_data_and_header(payload: &[u8]) -> (Vec<u8>, ProtectedHeader) {
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

    (sig_data, protected_header)
}

fn signatures_data_and_header(payloads: &[&[u8]]) -> (Vec<Vec<u8>>, ProtectedHeader) {
    let protected_header = ProtectedHeader {
        original_data: None,
        header: HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build(),
    };

    let sigs_data = payloads
        .iter()
        .map(|payload| {
            sig_structure_data(
                SignatureContext::CoseSign1,
                protected_header.clone(),
                None,
                &[],
                payload,
            )
        })
        .collect();

    (sigs_data, protected_header)
}

pub async fn sign_cose(
    payload: &[u8],
    unprotected_header: Header,
    private_key: &impl EcdsaKey,
    include_payload: bool,
) -> Result<CoseSign1, CoseError> {
    let (sig_data, protected_header) = signature_data_and_header(payload);

    let signature = private_key
        .try_sign(sig_data.as_ref())
        .await
        .map_err(|error| CoseError::Signing(error.into()))?
        .to_vec();

    let signed = CoseSign1 {
        signature,
        payload: include_payload.then(|| payload.to_vec()),
        protected: protected_header,
        unprotected: unprotected_header,
    };

    Ok(signed)
}

pub async fn sign_coses<K: CredentialEcdsaKey, P: WscdPoa>(
    keys_and_challenges: Vec<(K, &[u8])>,
    wscd: &impl DisclosureWscd<Key = K, Poa = P>,
    unprotected_header: Header,
    poa_input: P::Input,
    include_payload: bool,
) -> Result<(Vec<CoseSign1>, Option<P>), CoseError> {
    let (keys, challenges): (Vec<_>, Vec<_>) = keys_and_challenges.into_iter().unzip();

    let (sigs_data, protected_header) = signatures_data_and_header(&challenges);

    let keys_and_signature_data = keys
        .iter()
        .zip(sigs_data)
        .map(|(key, sig_data)| (sig_data, vec![key]))
        .collect::<Vec<_>>();

    let result = wscd
        .sign(keys_and_signature_data, poa_input)
        .await
        .map_err(|error| CoseError::Signing(error.into()))?;

    let signed = result
        .signatures
        .into_iter()
        .zip(challenges)
        .map(|(signature, payload)| {
            let cose = CoseSign1 {
                signature: signature.first().ok_or(CoseError::SignatureMissing)?.to_vec(),
                payload: include_payload.then(|| payload.to_vec()),
                protected: protected_header.clone(),
                unprotected: unprotected_header.clone(),
            };
            Ok(cose)
        })
        .collect::<Result<Vec<_>, CoseError>>()?;

    Ok((signed, result.poa))
}

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum KeysError {
    #[error("key generation error: {0}")]
    KeyGeneration(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
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

#[derive(Debug, Clone, PartialEq)]
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
    use coset::Header;
    use coset::HeaderBuilder;
    use coset::Label;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateUsage;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde::Deserialize;
    use serde::Serialize;
    use utils::generator::TimeGenerator;
    use utils::vec_nonempty;

    use super::ClonePayload;
    use super::MdocCose;
    use crate::utils::cose;
    use crate::utils::cose::CoseError;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            Err(CoseError::EcdsaSignatureVerificationFailed(_))
        ));

        // Verification should fail if the signature length is not right
        let len = cose.0.signature.len();
        cose.0.signature.remove(len - 1);
        assert!(matches!(
            cose.verify(key.verifying_key()),
            Err(CoseError::EcdsaSignatureParsingFailed(_))
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
            Err(CoseError::MissingLabel(_))
        ));
    }

    #[tokio::test]
    async fn cose_with_certificate() {
        let ca = Ca::generate("ca.example.com", Default::default()).unwrap();
        let issuer_key_pair = ca
            .generate_key_pair("cert.example.com", CertificateUsage::Mdl, Default::default())
            .unwrap();

        let payload = ToyMessage::default();
        let header = cose::header_with_x5chain(&vec_nonempty![issuer_key_pair.certificate()]);
        let cose = MdocCose::sign(&payload, header, issuer_key_pair.private_key(), true)
            .await
            .unwrap();

        // Certificate should be present in the unprotected headers
        let header_cert = cose.x5chain().unwrap().into_first();
        assert_eq!(issuer_key_pair.certificate().as_ref(), header_cert.as_ref());

        cose.verify_against_trust_anchors(CertificateUsage::Mdl, &TimeGenerator, &[ca.to_borrowing_trust_anchor()])
            .unwrap();
    }

    #[tokio::test]
    async fn x5chain_single_cert() {
        let ca = Ca::generate("ca.example.com", Default::default()).unwrap();
        let issuer_key_pair = ca
            .generate_key_pair("cert.example.com", CertificateUsage::Mdl, Default::default())
            .unwrap();

        let cert = issuer_key_pair.certificate();
        let header = cose::header_with_x5chain(&vec_nonempty![cert]);
        let cose: MdocCose<_, ToyMessage> =
            MdocCose::sign(&ToyMessage::default(), header, issuer_key_pair.private_key(), true)
                .await
                .unwrap();

        let chain = cose.x5chain().unwrap();
        assert_eq!(chain.len().get(), 1);
        assert_eq!(cert.as_ref(), chain[0].as_ref());
    }

    #[tokio::test]
    async fn verify_against_trust_anchors_with_intermediate_cert() {
        // Root CA that permits one level of intermediate CAs
        let root_ca = Ca::generate_with_intermediate_count("root-ca.example.com", Default::default(), 1).unwrap();

        // Intermediate CA and cert
        let intermediate_ca = root_ca
            .generate_intermediate(
                "intermediate-ca.example.com",
                CertificateUsage::Mdl.into(),
                Default::default(),
            )
            .unwrap();
        let intermediate_certificate = intermediate_ca.as_borrowing_certificate().unwrap();

        // Leaf key pair and
        let leaf_key_pair = intermediate_ca
            .generate_key_pair("leaf.example.com", CertificateUsage::Mdl, Default::default())
            .unwrap();
        let leaf_certificate = leaf_key_pair.certificate();

        // x5chain: [leaf_cert, intermediate_cert]
        let header = cose::header_with_x5chain(&vec_nonempty![leaf_certificate, &intermediate_certificate]);

        // Sign COSE and include x5chain header
        let cose: MdocCose<_, ToyMessage> =
            MdocCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key(), true)
                .await
                .unwrap();

        //
        let chain = cose.x5chain().unwrap();
        assert_eq!(chain.len().get(), 2);
        assert_eq!(leaf_certificate.as_ref(), chain[0].as_ref());
        assert_eq!(intermediate_certificate.as_ref(), chain[1].as_ref());

        // Verify signed COSE
        let result = cose.verify_against_trust_anchors(
            CertificateUsage::Mdl,
            &TimeGenerator,
            &[root_ca.to_borrowing_trust_anchor()],
        );
        assert_eq!(result.unwrap(), ToyMessage::default());
    }

    #[tokio::test]
    async fn verify_against_trust_anchors_missing_intermediate() {
        // Root CA that permits one level of intermediate CAs
        let root_ca = Ca::generate_with_intermediate_count("root-ca.example.com", Default::default(), 1).unwrap();
        let intermediate_ca = root_ca
            .generate_intermediate(
                "intermediate-ca.example.com",
                CertificateUsage::Mdl.into(),
                Default::default(),
            )
            .unwrap();
        let leaf_key_pair = intermediate_ca
            .generate_key_pair("leaf.example.com", CertificateUsage::Mdl, Default::default())
            .unwrap();

        // x5chain only contains the leaf cert, not the intermediate
        let header = cose::header_with_x5chain(&vec_nonempty![leaf_key_pair.certificate()]);
        let cose: MdocCose<_, ToyMessage> =
            MdocCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key(), true)
                .await
                .unwrap();

        // Verification should fail because the intermediate cert is absent from the chain
        assert!(matches!(
            cose.verify_against_trust_anchors(
                CertificateUsage::Mdl,
                &TimeGenerator,
                &[root_ca.to_borrowing_trust_anchor()],
            ),
            Err(CoseError::Certificate(_))
        ));
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
