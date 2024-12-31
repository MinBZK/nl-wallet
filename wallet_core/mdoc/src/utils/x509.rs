use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use derive_more::Debug;
use indexmap::IndexMap;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::pkcs8::DecodePublicKey;
use p256::pkcs8::der::asn1::Utf8StringRef;
use p256::pkcs8::der::Decode;
use p256::pkcs8::der::SliceReader;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use rustls_pki_types::UnixTime;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use webpki::ring::ECDSA_P256_SHA256;
use webpki::EndEntityCert;
use x509_parser::der_parser::Oid;
use x509_parser::extensions::GeneralName;
use x509_parser::nom::AsBytes;
use x509_parser::nom::{self};
use x509_parser::oid_registry::asn1_rs::oid;
use x509_parser::prelude::ExtendedKeyUsage;
use x509_parser::prelude::FromDer;
use x509_parser::prelude::PEMError;
use x509_parser::prelude::X509Certificate;
use x509_parser::prelude::X509Error;
use x509_parser::x509::X509Name;
use yoke::Yoke;
use yoke::Yokeable;

use error_category::ErrorCategory;
use wallet_common::generator::Generator;

use super::issuer_auth::IssuerRegistration;
use super::reader_auth::ReaderRegistration;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum CertificateError {
    #[error("certificate verification failed: {0}")]
    Verification(#[source] webpki::Error),
    #[error("certificate parsing for validation failed: {0}")]
    EndEntityCertificateParsing(#[from] webpki::Error),
    #[error("certificate content parsing failed: {0}")]
    X509CertificateParsing(#[from] x509_parser::nom::Err<X509Error>),
    #[error("pem parsing failed: {0}")]
    PemParsing(#[from] rustls_pki_types::pem::Error),
    #[cfg(any(test, feature = "generate"))]
    #[error("certificate private key generation failed: {0}")]
    #[category(unexpected)]
    GeneratingPrivateKey(#[source] p256::pkcs8::Error),
    #[cfg(any(test, feature = "generate"))]
    #[error("certificate creation failed: {0}")]
    #[category(unexpected)]
    GeneratingFailed(#[from] rcgen::Error),
    #[cfg(any(test, feature = "generate"))]
    #[error("parsed X.509 certificate is not a root CA")]
    #[category(unexpected)]
    NotRootCa,
    #[error("failed to parse certificate public key: {0}")]
    PublicKeyParsing(p256::pkcs8::spki::Error),
    #[error("EKU count incorrect ({0})")]
    #[category(critical)]
    IncorrectEkuCount(usize),
    #[error("EKU incorrect")]
    #[category(critical)]
    IncorrectEku(String),
    #[error("PEM decoding error: {0}")]
    Pem(#[from] nom::Err<PEMError>),
    #[error("unexpected PEM header: found {found}, expected {expected}")]
    #[category(critical)]
    UnexpectedPemHeader { found: String, expected: String },
    #[error("DER coding error: {0}")]
    DerEncodingError(#[from] p256::pkcs8::der::Error),
    #[error("JSON coding error: {0}")]
    JsonEncodingError(#[from] serde_json::Error),
    #[error("X509 coding error: {0}")]
    X509Error(#[from] X509Error),
    #[error("private key does not belong to public key from certificate")]
    KeyMismatch,
    #[error("failed to get public key from private key: {0}")]
    PublicKeyFromPrivate(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// An x509 certificate, unifying functionality from the following crates:
///
/// - parsing data: `x509_parser`
/// - verification of certificate chains: `webpki`
/// - signing and generating: `rcgen`
/// - verification of ecdsa signatures: `ecdsa`
#[derive(Yokeable, Debug)]
struct ParsedCertificate<'a> {
    #[debug(skip)]
    end_entity_cert: EndEntityCert<'a>,
    x509_cert: X509Certificate<'a>,
    public_key: VerifyingKey,
}

type YokedCertificate = Yoke<ParsedCertificate<'static>, Arc<CertificateDer<'static>>>;

/// The main struct for working with certificates. It represents the following types:
///
/// - webpki::end_entity::EndEntityCert
/// - x509_parser::certificate::X509Certificate
/// - p256::ecdsa::VerifyingKey
///
/// It can be constructed using the `from_der`, `from_pem` or `from_certificate_der` methods. The various types are
/// parsed on construction as borrowed types.
#[derive(Debug)]
pub struct BorrowingCertificate(YokedCertificate);

impl BorrowingCertificate {
    pub fn from_der(der_bytes: impl Into<Vec<u8>>) -> Result<Self, CertificateError> {
        let certificate_der = CertificateDer::from(der_bytes.into());
        Self::from_certificate_der(certificate_der)
    }

    pub fn from_pem(pem: impl AsRef<[u8]>) -> Result<Self, CertificateError> {
        let certificate_der = CertificateDer::from_pem_slice(pem.as_ref()).map_err(CertificateError::PemParsing)?;
        Self::from_certificate_der(certificate_der)
    }

    pub fn from_certificate_der(certificate_der: CertificateDer<'_>) -> Result<Self, CertificateError> {
        Self::from_certificate_der_arc(Arc::from(certificate_der.into_owned()))
    }

    fn from_certificate_der_arc(certificate_der: Arc<CertificateDer<'static>>) -> Result<Self, CertificateError> {
        let yoke = Yoke::try_attach_to_cart(certificate_der, |cert| {
            let end_entity_cert = cert.try_into().map_err(CertificateError::EndEntityCertificateParsing)?;
            let (_, x509_cert) =
                X509Certificate::from_der(cert.as_bytes()).map_err(CertificateError::X509CertificateParsing)?;
            let public_key = VerifyingKey::from_public_key_der(x509_cert.public_key().raw)
                .map_err(CertificateError::PublicKeyParsing)?;

            Ok::<_, CertificateError>(ParsedCertificate {
                end_entity_cert,
                x509_cert,
                public_key,
            })
        })?;

        Ok(BorrowingCertificate(yoke))
    }

    /// Verify the certificate against the specified trust anchors.
    pub fn verify(
        &self,
        usage: CertificateUsage,
        intermediate_certs: &[CertificateDer],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(), CertificateError> {
        self.end_entity_certificate()
            .verify_for_usage(
                &[ECDSA_P256_SHA256],
                trust_anchors,
                intermediate_certs,
                // unwrap is safe here because we assume the time that is generated lies after the epoch
                UnixTime::since_unix_epoch(Duration::from_secs(time.generate().timestamp().try_into().unwrap())),
                webpki::KeyUsage::required(usage.eku()),
                None,
                None,
            )
            .map(|_| ())
            .map_err(CertificateError::Verification)
    }

    pub fn end_entity_certificate(&self) -> &EndEntityCert {
        &self.0.get().end_entity_cert
    }

    pub fn x509_certificate(&self) -> &X509Certificate {
        &self.0.get().x509_cert
    }

    pub fn public_key(&self) -> &VerifyingKey {
        &self.0.get().public_key
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    pub fn subject(&self) -> Result<IndexMap<String, &str>, CertificateError> {
        self.x509_certificate()
            .subject
            .iter_attributes()
            .map(|attr| {
                Ok((
                    x509_parser::objects::oid2abbrev(attr.attr_type(), x509_parser::objects::oid_registry())
                        .map_or(attr.attr_type().to_id_string(), String::from),
                    attr.as_str()?,
                ))
            })
            .collect::<Result<_, _>>()
    }

    pub fn issuer_common_names(&self) -> Result<Vec<&str>, CertificateError> {
        x509_common_names(&self.x509_certificate().issuer)
    }

    pub fn common_names(&self) -> Result<Vec<&str>, CertificateError> {
        x509_common_names(&self.x509_certificate().subject)
    }

    /// Returns the first DNS SAN, if any, from the certificate.
    pub fn san_dns_name(&self) -> Result<Option<&str>, CertificateError> {
        let san = self.x509_certificate().subject_alternative_name()?.and_then(|ext| {
            ext.value.general_names.iter().find_map(|name| match name {
                GeneralName::DNSName(name) => Some(*name),
                _ => None,
            })
        });
        Ok(san)
    }

    pub(crate) fn parse_and_extract_custom_ext<'a, T: Deserialize<'a>>(
        &'a self,
        oid: &Oid,
    ) -> Result<Option<T>, CertificateError> {
        let x509_cert = self.x509_certificate();
        let ext = x509_cert.iter_extensions().find(|ext| ext.oid == *oid);
        ext.map(|ext| {
            let mut reader = SliceReader::new(ext.value)?;
            let json = Utf8StringRef::decode(&mut reader)?;
            let registration = serde_json::from_str(json.as_str())?;
            Ok::<_, CertificateError>(registration)
        })
        .transpose()
    }
}

impl Clone for BorrowingCertificate {
    fn clone(&self) -> Self {
        // Unwrap is safe here since the der bytes have been parsed before
        BorrowingCertificate::from_certificate_der_arc(Arc::clone(self.0.backing_cart())).unwrap()
    }
}

impl AsRef<[u8]> for BorrowingCertificate {
    fn as_ref(&self) -> &[u8] {
        self.0.backing_cart().as_ref()
    }
}

impl TryFrom<Vec<u8>> for BorrowingCertificate {
    type Error = CertificateError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        BorrowingCertificate::from_der(value.as_slice())
    }
}

impl From<BorrowingCertificate> for Vec<u8> {
    fn from(value: BorrowingCertificate) -> Self {
        value.to_vec()
    }
}

impl PartialEq for BorrowingCertificate {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for BorrowingCertificate {}

fn x509_common_names<'a>(x509name: &'a X509Name) -> Result<Vec<&'a str>, CertificateError> {
    x509name
        .iter_common_name()
        .map(|cn| cn.as_str().map_err(CertificateError::X509Error))
        .collect()
}

/// Usage of a [`Certificate`], representing its Extended Key Usage (EKU).
/// [`Certificate::verify()`] receives this as parameter and enforces that it is present in the certificate
/// being verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateUsage {
    Mdl,
    ReaderAuth,
}

pub const EXTENDED_KEY_USAGE_MDL: &Oid = &oid!(1.0.18013 .5 .1 .2);
pub const EXTENDED_KEY_USAGE_READER_AUTH: &Oid = &oid!(1.0.18013 .5 .1 .6);

impl CertificateUsage {
    fn from_certificate(cert: &X509Certificate) -> Result<Self, CertificateError> {
        let usage = cert
            .extended_key_usage()?
            .map(|eku| Self::from_key_usage(eku.value))
            .transpose()?
            .ok_or_else(|| CertificateError::IncorrectEkuCount(0))?;

        Ok(usage)
    }

    fn from_key_usage(ext_key_usage: &ExtendedKeyUsage) -> Result<Self, CertificateError> {
        if ext_key_usage.other.len() != 1 {
            return Err(CertificateError::IncorrectEkuCount(ext_key_usage.other.len()));
        }

        let key_usage_oid = ext_key_usage.other.first().unwrap();

        // Unfortunately we cannot use a match statement here.
        if key_usage_oid == EXTENDED_KEY_USAGE_MDL {
            return Ok(Self::Mdl);
        } else if key_usage_oid == EXTENDED_KEY_USAGE_READER_AUTH {
            return Ok(Self::ReaderAuth);
        }

        Err(CertificateError::IncorrectEku(key_usage_oid.to_id_string()))
    }

    pub(crate) fn eku(self) -> &'static [u8] {
        match self {
            CertificateUsage::Mdl => EXTENDED_KEY_USAGE_MDL,
            CertificateUsage::ReaderAuth => EXTENDED_KEY_USAGE_READER_AUTH,
        }
        .as_bytes()
    }
}

/// Acts as configuration for the [Certificate::new] function.
#[derive(Debug, Clone, PartialEq)]
pub enum CertificateType {
    Mdl(Option<Box<IssuerRegistration>>),
    ReaderAuth(Option<Box<ReaderRegistration>>),
}

impl CertificateType {
    pub fn from_certificate(cert: &BorrowingCertificate) -> Result<Self, CertificateError> {
        let usage = CertificateUsage::from_certificate(cert.x509_certificate())?;
        let result = match usage {
            CertificateUsage::Mdl => {
                let registration: Option<IssuerRegistration> = IssuerRegistration::from_certificate(cert)?;
                CertificateType::Mdl(registration.map(Box::new))
            }
            CertificateUsage::ReaderAuth => {
                let registration: Option<ReaderRegistration> = ReaderRegistration::from_certificate(cert)?;
                CertificateType::ReaderAuth(registration.map(Box::new))
            }
        };

        Ok(result)
    }
}

impl From<&CertificateType> for CertificateUsage {
    fn from(source: &CertificateType) -> Self {
        use CertificateType::*;
        match source {
            Mdl(_) => Self::Mdl,
            ReaderAuth(_) => Self::ReaderAuth,
        }
    }
}

pub trait MdocCertificateExtension
where
    Self: Serialize + DeserializeOwned + Sized,
{
    const OID: Oid<'static>;

    fn from_certificate(source: &BorrowingCertificate) -> Result<Option<Self>, CertificateError> {
        source.parse_and_extract_custom_ext(&Self::OID)
    }

    #[cfg(any(test, feature = "generate"))]
    fn to_custom_ext(&self) -> Result<rcgen::CustomExtension, CertificateError> {
        use p256::pkcs8::der::Encode;

        let json_string = serde_json::to_string(self)?;
        let string = Utf8StringRef::new(&json_string)?;

        let sub_identifiers = Self::OID
            .iter()
            .ok_or(CertificateError::IncorrectEku(Self::OID.to_id_string()))?
            .collect::<Vec<_>>();
        let ext = rcgen::CustomExtension::from_oid_content(sub_identifiers.as_slice(), string.to_der()?);
        Ok(ext)
    }
}

#[cfg(any(test, feature = "generate"))]
#[derive(Debug, Clone, Default)]
pub struct CertificateConfiguration {
    pub not_before: Option<DateTime<Utc>>,
    pub not_after: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use p256::pkcs8::ObjectIdentifier;
    use time::macros::datetime;
    use time::OffsetDateTime;
    use x509_parser::certificate::X509Certificate;

    use wallet_common::generator::TimeGenerator;

    use crate::server_keys::generate::SelfSignedCa;
    use crate::utils::issuer_auth::IssuerRegistration;
    use crate::utils::reader_auth::ReaderRegistration;
    use crate::utils::x509::CertificateType;

    use super::BorrowingCertificate;
    use super::CertificateConfiguration;
    use super::CertificateError;
    use super::CertificateUsage;

    #[test]
    fn mdoc_eku_encoding_works() {
        CertificateUsage::Mdl.eku();
        CertificateUsage::ReaderAuth.eku();
    }

    #[test]
    fn parse_oid() {
        let mdl_kp: ObjectIdentifier = "1.0.18013.5.1.2".parse().unwrap();
        let mdl_kp: &'static [u8] = Box::leak(mdl_kp.into()).as_bytes();
        assert_eq!(mdl_kp, CertificateUsage::Mdl.eku());
    }

    #[test]
    fn generate_ca() {
        let ca = SelfSignedCa::generate("myca", Default::default()).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.as_certificate_der().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        let x509_cert = certificate.x509_certificate();
        assert_certificate_common_name(x509_cert, &["myca"]);
        assert_certificate_default_validity(x509_cert);
    }

    #[test]
    fn generate_ca_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
        };
        let ca = SelfSignedCa::generate("myca", config).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.as_certificate_der().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        let x509_cert = certificate.x509_certificate();
        assert_certificate_common_name(x509_cert, &["myca"]);
        assert_certificate_validity(x509_cert, now, later);
    }

    fn generate_and_verify_issuer_for_validity(
        not_before: Option<DateTime<Utc>>,
        not_after: Option<DateTime<Utc>>,
    ) -> CertificateError {
        let ca = generate_ca_for_validity_test();

        let config = CertificateConfiguration { not_before, not_after };
        let mdl = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate_key_pair("mycert", &mdl, config).unwrap();
        issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca.to_trust_anchor()])
            .expect_err("Expected verify to fail")
    }

    #[test]
    fn generate_and_verify_not_yet_valid_issuer_cert() {
        let now = Utc::now();
        let start = Some(now + Duration::days(1));
        let end = Some(now + Duration::days(2));

        let error = generate_and_verify_issuer_for_validity(start, end);
        assert_matches!(error, CertificateError::Verification(webpki::Error::CertNotValidYet));
    }

    #[test]
    fn generate_and_verify_expired_issuer_cert() {
        let now = Utc::now();
        let start = Some(now - Duration::days(2));
        let end = Some(now - Duration::days(1));

        let error = generate_and_verify_issuer_for_validity(start, end);
        assert_matches!(error, CertificateError::Verification(webpki::Error::CertExpired));
    }

    #[test]
    fn generate_and_verify_issuer_cert() {
        let ca = SelfSignedCa::generate("myca", Default::default()).unwrap();
        let mdl = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate_key_pair("mycert", &mdl, Default::default()).unwrap();

        issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca.to_trust_anchor()])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original Mdl usage
        let cert_usage = CertificateType::from_certificate(issuer_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, mdl);

        let x509_cert = issuer_key_pair.certificate().x509_certificate();
        assert_certificate_common_name(x509_cert, &["mycert"]);
        assert_certificate_default_validity(x509_cert);
    }

    #[test]
    fn generate_and_verify_issuer_cert_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
        };

        let ca = SelfSignedCa::generate("myca", Default::default()).unwrap();
        let mdl = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate_key_pair("mycert", &mdl, config).unwrap();

        issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca.to_trust_anchor()])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original Mdl usage
        let cert_usage = CertificateType::from_certificate(issuer_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, mdl);

        let x509_cert = issuer_key_pair.certificate().x509_certificate();
        assert_certificate_common_name(x509_cert, &["mycert"]);
        assert_certificate_validity(x509_cert, now, later);
    }

    #[test]
    fn generate_and_verify_reader_cert() {
        let ca = SelfSignedCa::generate("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca
            .generate_key_pair("mycert", &reader_auth, Default::default())
            .unwrap();

        reader_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth,
                &[],
                &TimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap();

        // Verify whether the parsed CertificateType equals the original ReaderAuth usage
        let cert_usage = CertificateType::from_certificate(reader_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, reader_auth);

        let x509_cert = reader_key_pair.certificate().x509_certificate();
        assert_certificate_common_name(x509_cert, &["mycert"]);
        assert_certificate_default_validity(x509_cert);
    }

    #[test]
    fn generate_and_verify_reader_cert_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
        };

        let ca = SelfSignedCa::generate("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca.generate_key_pair("mycert", &reader_auth, config).unwrap();

        reader_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth,
                &[],
                &TimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap();

        // Verify whether the parsed CertificateType equals the original ReaderAuth usage
        let cert_usage = CertificateType::from_certificate(reader_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, reader_auth);

        let x509_cert = reader_key_pair.certificate().x509_certificate();
        assert_certificate_common_name(x509_cert, &["mycert"]);
        assert_certificate_validity(x509_cert, now, later);
    }

    fn assert_certificate_default_validity(certificate: &X509Certificate) {
        let not_before = certificate.validity().not_before.to_datetime();
        let not_after = certificate.validity().not_after.to_datetime();

        assert_eq!(not_before, datetime!(1975-01-01 0:00 UTC));
        assert_eq!(not_after, datetime!(4096-01-01 0:00 UTC));
    }

    fn assert_certificate_validity(
        certificate: &X509Certificate,
        expected_not_before: DateTime<Utc>,
        expected_not_after: DateTime<Utc>,
    ) {
        let expected_not_before = OffsetDateTime::from_unix_timestamp(expected_not_before.timestamp()).unwrap();
        let expected_not_after = OffsetDateTime::from_unix_timestamp(expected_not_after.timestamp()).unwrap();

        let not_before = certificate.validity().not_before.to_datetime();
        let not_after = certificate.validity().not_after.to_datetime();

        assert_eq!(not_before, expected_not_before);
        assert_eq!(not_after, expected_not_after);
    }

    fn assert_certificate_common_name(certificate: &X509Certificate, expected_common_name: &[&str]) {
        let actual_common_name = certificate
            .subject
            .iter_common_name()
            .map(|cn| cn.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual_common_name, expected_common_name);
    }

    fn generate_ca_for_validity_test() -> SelfSignedCa {
        let now = Utc::now();
        let start = now - Duration::weeks(52);
        let end = now + Duration::weeks(52);

        let config = CertificateConfiguration {
            not_before: Some(start),
            not_after: Some(end),
        };

        SelfSignedCa::generate("myca", config).unwrap()
    }
}
