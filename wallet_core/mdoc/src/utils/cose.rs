//! Cose objects, keys, parsing, and verification.

use std::marker::PhantomData;
use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use ciborium::value::Value;
use coset::iana;
use coset::sig_structure_data;
use coset::CoseMac0;
use coset::CoseMac0Builder;
use coset::CoseSign1;
use coset::CoseSign1Builder;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::ProtectedHeader;
use coset::SignatureContext;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use ring::hmac;
use serde::de::DeserializeOwned;
use serde::Serialize;
use webpki::types::TrustAnchor;

use error_category::ErrorCategory;
use wallet_common::generator::Generator;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::keys::EcdsaKey;

use crate::utils::serialization::cbor_deserialize;
use crate::utils::serialization::cbor_serialize;
use crate::utils::serialization::CborError;

use super::x509::Certificate;
use super::x509::CertificateError;
use super::x509::CertificateUsage;

/// Trait for supported Cose variations ([`CoseSign1`] or [`CoseMac0`]).
pub trait Cose {
    type Key;
    fn payload(&self) -> Option<&[u8]>;
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
    pub(crate) fn dangerous_parse_unverified(&self) -> Result<T, CoseError> {
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
}

impl<C, T> From<C> for MdocCose<C, T> {
    fn from(cose: C) -> Self {
        MdocCose(cose, PhantomData)
    }
}

/// COSE header label for `x5chain`, defined in [RFC 9360](https://datatracker.ietf.org/doc/rfc9360/).
pub const COSE_X5CHAIN_HEADER_LABEL: i64 = 33;

pub fn new_certificate_header(cert: &Certificate) -> Header {
    HeaderBuilder::new()
        .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(cert.as_bytes().to_vec()))
        .build()
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

    pub async fn generate_keys_and_sign<K: CredentialEcdsaKey>(
        obj: &T,
        unprotected_header: Header,
        number_of_keys: u64,
        key_factory: &impl KeyFactory<Key = K>,
        include_payload: bool,
    ) -> crate::Result<Vec<(K, MdocCose<CoseSign1, T>)>>
    where
        T: Clone + Serialize,
    {
        let payload = cbor_serialize(obj).map_err(CoseError::Cbor)?;
        let coses = generate_keys_and_sign_cose(
            &payload,
            unprotected_header,
            number_of_keys,
            key_factory,
            include_payload,
        )
        .await?;

        Ok(coses.into_iter().map(|(key, cose)| (key, cose.into())).collect())
    }

    /// Get the [`Certificate`] containing the public key with which the MSO is signed from the unsigned COSE header.
    pub fn signing_cert(&self) -> Result<Certificate, CoseError>
    where
        T: DeserializeOwned,
    {
        let cert_bts = self
            .unprotected_header_item(&Label::Int(COSE_X5CHAIN_HEADER_LABEL))?
            .as_bytes()
            .ok_or(CoseError::CertificateUnexpectedHeaderType)?;

        // The standard defining the above COSE header label (https://datatracker.ietf.org/doc/draft-ietf-cose-x509/)
        // allows multiple certificates being present in the header, but ISO 18013-5 doesn't.
        // So we can parse the bytes as a certificate.
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
    ) -> Result<T, CoseError>
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

pub async fn sign_coses<K: CredentialEcdsaKey>(
    keys_and_challenges: Vec<(K, &[u8])>,
    key_factory: &impl KeyFactory<Key = K>,
    unprotected_header: Header,
    include_payload: bool,
) -> Result<Vec<CoseSign1>, CoseError> {
    let (keys, challenges): (Vec<_>, Vec<_>) = keys_and_challenges.into_iter().unzip();

    let (sigs_data, protected_header) = signatures_data_and_header(&challenges);

    let keys_and_signature_data = keys
        .iter()
        .zip(sigs_data)
        .map(|(key, sig_data)| (sig_data, vec![key]))
        .collect::<Vec<_>>();

    let signatures = key_factory
        .sign_multiple_with_existing_keys(keys_and_signature_data)
        .await
        .map_err(|error| CoseError::Signing(error.into()))?;

    let signed = signatures
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

    Ok(signed)
}

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum KeysError {
    #[error("key generation error: {0}")]
    KeyGeneration(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub async fn generate_keys_and_sign_cose<K: CredentialEcdsaKey>(
    payload: &[u8],
    unprotected_header: Header,
    number_of_keys: u64,
    key_factory: &impl KeyFactory<Key = K>,
    include_payload: bool,
) -> crate::Result<Vec<(K, CoseSign1)>> {
    let (sig_data, protected_header) = signature_data_and_header(payload);

    let signatures = key_factory
        .sign_with_new_keys(sig_data, number_of_keys)
        .await
        .map_err(|err| KeysError::KeyGeneration(err.into()))?;

    let coses = signatures
        .into_iter()
        .zip(itertools::repeat_n(
            (protected_header, unprotected_header),
            number_of_keys.try_into().unwrap(),
        ))
        .map(|((key, signature), (protected_header, unprotected_header))| {
            (
                key,
                CoseSign1 {
                    signature: signature.to_vec(),
                    payload: include_payload.then(|| payload.to_vec()),
                    protected: protected_header,
                    unprotected: unprotected_header,
                },
            )
        })
        .collect();

    Ok(coses)
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
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde::Deserialize;
    use serde::Serialize;

    use wallet_common::generator::TimeGenerator;

    use crate::server_keys::KeyPair;
    use crate::utils::cose::CoseError;
    use crate::utils::cose::{self};
    use crate::utils::issuer_auth::IssuerRegistration;
    use crate::utils::x509::CertificateUsage;

    use super::ClonePayload;
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
        let ca = KeyPair::generate_ca("ca.example.com", Default::default()).unwrap();
        let issuer_key_pair = ca
            .generate(
                "cert.example.com",
                &IssuerRegistration::new_mock().into(),
                Default::default(),
            )
            .unwrap();

        let payload = ToyMessage::default();
        let header = cose::new_certificate_header(issuer_key_pair.certificate());
        let cose = MdocCose::sign(&payload, header, issuer_key_pair.private_key(), true)
            .await
            .unwrap();

        // Certificate should be present in the unprotected headers
        let header_cert = cose.signing_cert().unwrap();
        assert_eq!(issuer_key_pair.certificate().as_bytes(), header_cert.as_bytes());

        let trust_anchor = ca.certificate().try_into().unwrap();
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
