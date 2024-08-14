use std::borrow::Cow;

use base64::prelude::*;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use p256::{
    ecdsa::VerifyingKey,
    elliptic_curve::pkcs8::DecodePublicKey,
    pkcs8::der::{asn1::Utf8StringRef, Decode, SliceReader},
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use webpki::{EndEntityCert, Time, TrustAnchor, ECDSA_P256_SHA256};
use x509_parser::{
    der_parser::Oid,
    extensions::GeneralName,
    nom::{self, AsBytes},
    pem,
    prelude::{ExtendedKeyUsage, FromDer, PEMError, X509Certificate, X509Error},
};

use error_category::ErrorCategory;
use wallet_common::{generator::Generator, trust_anchor::DerTrustAnchor};

use super::{issuer_auth::IssuerRegistration, reader_auth::ReaderRegistration};

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum CertificateError {
    #[error("certificate verification failed: {0}")]
    Verification(#[source] webpki::Error),
    #[error("certificate parsing for validation failed: {0}")]
    ValidationParsing(#[from] webpki::Error),
    #[error("certificate content parsing failed: {0}")]
    ContentParsing(#[from] x509_parser::nom::Err<X509Error>),
    #[cfg(any(test, feature = "generate"))]
    #[error("certificate private key generation failed: {0}")]
    #[category(unexpected)]
    GeneratingPrivateKey(p256::pkcs8::Error),
    #[cfg(any(test, feature = "generate"))]
    #[error("certificate creation failed: {0}")]
    #[category(unexpected)]
    GeneratingFailed(#[from] rcgen::RcgenError),
    #[error("failed to parse certificate public key: {0}")]
    KeyParsingFailed(p256::pkcs8::spki::Error),
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
}

pub const OID_EXT_KEY_USAGE: &[u64] = &[2, 5, 29, 37];

/// An x509 certificate, unifying functionality from the following crates:
///
/// - parsing data: `x509_parser`
/// - verification of certificate chains: `webpki`
/// - signing and generating: `rcgen`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate(ByteBuf);

// Use base64 when we (de)serialize to JSON
impl Serialize for Certificate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            BASE64_STANDARD.encode(&self.0).serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for Certificate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            Ok(Certificate(ByteBuf::from(
                BASE64_STANDARD
                    .decode(String::deserialize(deserializer).map_err(serde::de::Error::custom)?)
                    .map_err(serde::de::Error::custom)?,
            )))
        } else {
            Ok(Certificate(ByteBuf::deserialize(deserializer)?))
        }
    }
}

impl<'a> TryInto<TrustAnchor<'a>> for &'a Certificate {
    type Error = CertificateError;
    fn try_into(self) -> Result<TrustAnchor<'a>, Self::Error> {
        Ok(TrustAnchor::try_from_cert_der(self.as_bytes())?)
    }
}

impl<'a> TryInto<EndEntityCert<'a>> for &'a Certificate {
    type Error = CertificateError;
    fn try_into(self) -> Result<EndEntityCert<'a>, Self::Error> {
        Ok(self.as_bytes().try_into()?)
    }
}

impl<'a> TryInto<X509Certificate<'a>> for &'a Certificate {
    type Error = CertificateError;
    fn try_into(self) -> Result<X509Certificate<'a>, Self::Error> {
        let (_, parsed) = X509Certificate::from_der(self.as_bytes())?;
        Ok(parsed)
    }
}

impl From<Certificate> for Vec<u8> {
    fn from(source: Certificate) -> Vec<u8> {
        source.0.to_vec()
    }
}

impl<'a> TryInto<DerTrustAnchor> for &'a Certificate {
    type Error = CertificateError;

    fn try_into(self) -> Result<DerTrustAnchor, Self::Error> {
        Ok(DerTrustAnchor::from_der(self.0.to_vec())?)
    }
}

impl<T: AsRef<[u8]>> From<T> for Certificate {
    fn from(value: T) -> Self {
        Certificate(ByteBuf::from(value.as_ref()))
    }
}

const PEM_CERTIFICATE_HEADER: &str = "CERTIFICATE";

impl Certificate {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn from_pem(pem: &str) -> Result<Self, CertificateError> {
        let (_, pem) = pem::parse_x509_pem(pem.as_bytes())?;
        if pem.label == PEM_CERTIFICATE_HEADER {
            Ok(pem.contents.into())
        } else {
            Err(CertificateError::UnexpectedPemHeader {
                found: pem.label,
                expected: PEM_CERTIFICATE_HEADER.to_string(),
            })
        }
    }

    /// Verify the certificate against the specified trust anchors.
    pub fn verify(
        &self,
        usage: CertificateUsage,
        intermediate_certs: &[&[u8]],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(), CertificateError> {
        self.to_webpki()?
            .verify_for_usage(
                &[&ECDSA_P256_SHA256],
                trust_anchors,
                intermediate_certs,
                Time::from_seconds_since_unix_epoch(time.generate().timestamp() as u64),
                webpki::KeyUsage::required(usage.to_eku()),
                &[],
            )
            .map_err(CertificateError::Verification)
    }

    pub fn public_key(&self) -> Result<VerifyingKey, CertificateError> {
        VerifyingKey::from_public_key_der(self.to_x509()?.public_key().raw).map_err(CertificateError::KeyParsingFailed)
    }

    /// Convert the certificate to a [`X509Certificate`] from the `x509_parser` crate, to read its contents.
    pub fn to_x509(&self) -> Result<X509Certificate, CertificateError> {
        self.try_into()
    }

    /// Convert the certificate to a [`EndEntityCert`] from the `webpki` crate, to verify it (possibly along with a
    /// certificate chain) against a set of trust roots.
    pub fn to_webpki(&self) -> Result<EndEntityCert, CertificateError> {
        self.try_into()
    }

    pub fn subject(&self) -> Result<IndexMap<String, String>, CertificateError> {
        self.to_x509()?
            .subject
            .iter_attributes()
            .map(|attr| {
                Ok((
                    x509_parser::objects::oid2abbrev(attr.attr_type(), x509_parser::objects::oid_registry())
                        .map_or(attr.attr_type().to_id_string(), |v| v.to_string()),
                    attr.as_str()?.to_string(),
                ))
            })
            .collect::<Result<_, _>>()
    }

    pub fn iter_common_name(&self) -> Result<Vec<String>, CertificateError> {
        self.to_x509()?
            .subject
            .iter_common_name()
            .map(|cn| cn.as_str().map(ToOwned::to_owned).map_err(CertificateError::X509Error))
            .collect()
    }

    pub(crate) fn extract_custom_ext<'a, T: Deserialize<'a>>(
        &'a self,
        oid: Oid,
    ) -> Result<Option<T>, CertificateError> {
        let x509_cert = self.to_x509()?;
        let ext = x509_cert.iter_extensions().find(|ext| ext.oid == oid);
        ext.map(|ext| {
            let mut reader = SliceReader::new(ext.value)?;
            let json = Utf8StringRef::decode(&mut reader)?;
            let registration = serde_json::from_str(json.as_str())?;
            Ok::<_, CertificateError>(registration)
        })
        .transpose()
    }

    /// Returns the first DNS SAN, if any, from the certificate.
    pub fn san_dns_name(&self) -> Result<Option<String>, CertificateError> {
        let san = self.to_x509()?.subject_alternative_name()?.and_then(|ext| {
            ext.value.general_names.iter().find_map(|name| match name {
                GeneralName::DNSName(name) => Some(name.to_string()),
                _ => None,
            })
        });
        Ok(san)
    }
}

/// Usage of a [`Certificate`], representing its Extended Key Usage (EKU).
/// [`Certificate::verify()`] receives this as parameter and enforces that it is present in the certificate
/// being verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateUsage {
    Mdl,
    ReaderAuth,
}

/// OID 1.0.18013.5.1.2
pub const EXTENDED_KEY_USAGE_MDL: &[u8] = &[40, 129, 140, 93, 5, 1, 2];
/// OID 1.0.18013.5.1.6
pub const EXTENDED_KEY_USAGE_READER_AUTH: &[u8] = &[40, 129, 140, 93, 5, 1, 6];

pub const EKU_MDL_OID: Oid = oid_from_bytes(EXTENDED_KEY_USAGE_MDL);
pub const EKU_READER_AUTH_OID: Oid = oid_from_bytes(EXTENDED_KEY_USAGE_READER_AUTH);

const fn oid_from_bytes(bytes: &'static [u8]) -> Oid {
    Oid::new(Cow::Borrowed(bytes))
}

impl CertificateUsage {
    pub fn from_certificate(cert: &Certificate) -> Result<Self, CertificateError> {
        let usage = cert
            .to_x509()?
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
        if key_usage_oid == &EKU_MDL_OID {
            return Ok(Self::Mdl);
        } else if key_usage_oid == &EKU_READER_AUTH_OID {
            return Ok(Self::ReaderAuth);
        }

        Err(CertificateError::IncorrectEku(key_usage_oid.to_id_string()))
    }

    pub(crate) fn to_eku(&self) -> &'static [u8] {
        match self {
            CertificateUsage::Mdl => EXTENDED_KEY_USAGE_MDL,
            CertificateUsage::ReaderAuth => EXTENDED_KEY_USAGE_READER_AUTH,
        }
    }
}

/// Acts as configuration for the [Certificate::new] function.
#[derive(Debug, Clone, PartialEq)]
pub enum CertificateType {
    Mdl(Option<Box<IssuerRegistration>>),
    ReaderAuth(Option<Box<ReaderRegistration>>),
}

impl CertificateType {
    pub fn from_certificate(cert: &Certificate) -> Result<Self, CertificateError> {
        let usage = CertificateUsage::from_certificate(cert)?;
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
    const OID: &'static [u64];

    fn from_certificate(source: &Certificate) -> Result<Option<Self>, CertificateError> {
        // unwrap() is safe here, because we process a fixed value
        let oid = Oid::from(Self::OID).unwrap();
        source.extract_custom_ext(oid)
    }

    #[cfg(any(test, feature = "generate"))]
    fn to_custom_ext(&self) -> Result<rcgen::CustomExtension, CertificateError> {
        use p256::pkcs8::der::Encode;

        let json_string = serde_json::to_string(self)?;
        let string = Utf8StringRef::new(&json_string)?;
        let ext = rcgen::CustomExtension::from_oid_content(Self::OID, string.to_der()?);
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
    use chrono::{DateTime, Duration, Utc};
    use p256::pkcs8::ObjectIdentifier;
    use time::{macros::datetime, OffsetDateTime};
    use webpki::TrustAnchor;

    use wallet_common::generator::TimeGenerator;
    use x509_parser::certificate::X509Certificate;

    use crate::{
        server_keys::KeyPair,
        utils::{issuer_auth::IssuerRegistration, reader_auth::ReaderRegistration, x509::CertificateType},
    };

    use super::{CertificateConfiguration, CertificateError, CertificateUsage};

    #[test]
    fn mdoc_eku_encoding_works() {
        CertificateUsage::Mdl.to_eku();
        CertificateUsage::ReaderAuth.to_eku();
    }

    #[test]
    fn parse_oid() {
        let mdl_kp: ObjectIdentifier = "1.0.18013.5.1.2".parse().unwrap();
        let mdl_kp: &'static [u8] = Box::leak(mdl_kp.into()).as_bytes();
        assert_eq!(mdl_kp, CertificateUsage::Mdl.to_eku());
    }

    #[test]
    fn generate_ca() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();

        let x509_cert = ca.certificate().to_x509().unwrap();
        assert_certificate_common_name(&x509_cert, vec!["myca"]);
        assert_certificate_default_validity(&x509_cert);
    }

    #[test]
    fn generate_ca_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
        };
        let ca = KeyPair::generate_ca("myca", config).unwrap();

        let x509_cert = ca.certificate().to_x509().unwrap();
        assert_certificate_common_name(&x509_cert, vec!["myca"]);
        assert_certificate_validity(&x509_cert, now, later);
    }

    #[test]
    fn generate_and_verify_not_yet_valid_issuer_cert() {
        let ca = generate_ca_for_validity_test();

        let now = Utc::now();
        let start = now + Duration::days(1);
        let end = now + Duration::days(2);

        let config = CertificateConfiguration {
            not_before: Some(start),
            not_after: Some(end),
        };

        let mdl = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate("mycert", mdl, config).unwrap();

        let ca_trustanchor: TrustAnchor = ca.certificate().try_into().unwrap();
        let error = issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca_trustanchor])
            .expect_err("Expected verify to fail");
        assert_matches!(error, CertificateError::Verification(webpki::Error::CertNotValidYet));
    }

    #[test]
    fn generate_and_verify_expired_issuer_cert() {
        let ca = generate_ca_for_validity_test();

        let now = Utc::now();
        let start = now - Duration::days(2);
        let end = now - Duration::days(1);

        let config = CertificateConfiguration {
            not_before: Some(start),
            not_after: Some(end),
        };

        let mdl = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate("mycert", mdl, config).unwrap();

        let ca_trustanchor: TrustAnchor = ca.certificate().try_into().unwrap();
        let error = issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca_trustanchor])
            .expect_err("Expected verify to fail");
        assert_matches!(error, CertificateError::Verification(webpki::Error::CertExpired));
    }

    #[test]
    fn generate_and_verify_issuer_cert() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let mdl: CertificateType = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate("mycert", mdl.clone(), Default::default()).unwrap();

        let ca_trustanchor: TrustAnchor = ca.certificate().try_into().unwrap();
        issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca_trustanchor])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original Mdl usage
        let cert_usage = CertificateType::from_certificate(issuer_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, mdl);

        let x509_cert = issuer_key_pair.certificate().to_x509().unwrap();
        assert_certificate_common_name(&x509_cert, vec!["mycert"]);
        assert_certificate_default_validity(&x509_cert);
    }

    #[test]
    fn generate_and_verify_issuer_cert_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
        };

        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let mdl: CertificateType = IssuerRegistration::new_mock().into();

        let issuer_key_pair = ca.generate("mycert", mdl.clone(), config).unwrap();

        let ca_trustanchor: TrustAnchor = ca.certificate().try_into().unwrap();
        issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca_trustanchor])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original Mdl usage
        let cert_usage = CertificateType::from_certificate(issuer_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, mdl);

        let x509_cert = issuer_key_pair.certificate().to_x509().unwrap();
        assert_certificate_common_name(&x509_cert, vec!["mycert"]);
        assert_certificate_validity(&x509_cert, now, later);
    }

    #[test]
    fn generate_and_verify_reader_cert() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca.generate("mycert", reader_auth.clone(), Default::default()).unwrap();

        let ca_trustanchor: TrustAnchor = ca.certificate().try_into().unwrap();
        reader_key_pair
            .certificate()
            .verify(CertificateUsage::ReaderAuth, &[], &TimeGenerator, &[ca_trustanchor])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original ReaderAuth usage
        let cert_usage = CertificateType::from_certificate(reader_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, reader_auth);

        let x509_cert = reader_key_pair.certificate().to_x509().unwrap();
        assert_certificate_common_name(&x509_cert, vec!["mycert"]);
        assert_certificate_default_validity(&x509_cert);
    }

    #[test]
    fn generate_and_verify_reader_cert_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
        };

        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca.generate("mycert", reader_auth.clone(), config).unwrap();

        let ca_trustanchor: TrustAnchor = ca.certificate().try_into().unwrap();
        reader_key_pair
            .certificate()
            .verify(CertificateUsage::ReaderAuth, &[], &TimeGenerator, &[ca_trustanchor])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original ReaderAuth usage
        let cert_usage = CertificateType::from_certificate(reader_key_pair.certificate()).unwrap();
        assert_eq!(cert_usage, reader_auth);

        let x509_cert = reader_key_pair.certificate().to_x509().unwrap();
        assert_certificate_common_name(&x509_cert, vec!["mycert"]);
        assert_certificate_validity(&x509_cert, now, later);
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

    fn assert_certificate_common_name(certificate: &X509Certificate, expected_common_name: Vec<&str>) {
        let actual_common_name = certificate
            .subject
            .iter_common_name()
            .map(|cn| cn.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual_common_name, expected_common_name);
    }

    fn generate_ca_for_validity_test() -> KeyPair {
        let now = Utc::now();
        let start = now - Duration::weeks(52);
        let end = now + Duration::weeks(52);

        let config = CertificateConfiguration {
            not_before: Some(start),
            not_after: Some(end),
        };
        KeyPair::generate_ca("myca", config).unwrap()
    }
}
