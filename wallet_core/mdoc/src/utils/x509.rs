use std::borrow::Cow;

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::pkcs8::DecodePublicKey,
    pkcs8::{
        der::{asn1::SequenceOf, Encode},
        DecodePrivateKey, EncodePrivateKey, ObjectIdentifier,
    },
};
use rcgen::{
    BasicConstraints, Certificate as RcgenCertificate, CertificateParams, CustomExtension, DnType, IsCa, RcgenError,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use webpki::{EndEntityCert, Time, TrustAnchor, ECDSA_P256_SHA256};
use x509_parser::{
    der_parser::Oid,
    nom::{self, AsBytes},
    pem,
    prelude::{ExtendedKeyUsage, FromDer, PEMError, X509Certificate, X509Error},
};

use wallet_common::generator::Generator;

use super::reader_auth::ReaderRegistration;

#[derive(thiserror::Error, Debug)]
pub enum CertificateError {
    #[error("certificate verification failed: {0}")]
    Verification(#[source] webpki::Error),
    #[error("certificate parsing for validation failed: {0}")]
    ValidationParsing(#[from] webpki::Error),
    #[error("certificate content parsing failed: {0}")]
    ContentParsing(#[from] x509_parser::nom::Err<X509Error>),
    #[error("certificate private key generation failed: {0}")]
    GeneratingPrivateKey(p256::pkcs8::Error),
    #[error("certificate creation failed: {0}")]
    GeneratingFailed(#[from] RcgenError),
    #[error("failed to parse certificate public key: {0}")]
    KeyParsingFailed(p256::pkcs8::spki::Error),
    #[error("EKU count incorrect ({0})")]
    IncorrectEkuCount(usize),
    #[error("EKU incorrect")]
    IncorrectEku(String),
    #[error("PEM decoding error: {0}")]
    Pem(#[from] nom::Err<PEMError>),
    #[error("unexpected PEM header: found {found}, expected {expected}")]
    UnexpectedPemHeader { found: String, expected: String },
    #[error("DER coding error: {0}")]
    DerEncodingError(#[from] p256::pkcs8::der::Error),
    #[error("JSON coding error: {0}")]
    JsonEncodingError(#[from] serde_json::Error),
    #[error("X509 coding error: {0}")]
    X509Error(#[from] X509Error),
}

const OID_EXT_KEY_USAGE: &[u64] = &[2, 5, 29, 37];

/// A version of [`TrustAnchor`] that can more easily be used as a field
/// in another struct, as it does not require a liftetime annotation.
///
/// Can be converted from a reference to a [`TrustAnchor`] or a byte-slice
/// reference `&[u8]` using the `From<>` trait. Conversely a [`TrustAnchor`]
/// may be created from a reference to [`OwnedTrustAnchor`].
#[derive(Debug, Clone)]
pub struct OwnedTrustAnchor {
    subject: Vec<u8>,
    spki: Vec<u8>,
    name_constraints: Option<Vec<u8>>,
}

impl From<&TrustAnchor<'_>> for OwnedTrustAnchor {
    fn from(value: &TrustAnchor) -> Self {
        OwnedTrustAnchor {
            subject: value.subject.to_vec(),
            spki: value.spki.to_vec(),
            name_constraints: value.name_constraints.map(|nc| nc.to_vec()),
        }
    }
}

impl TryFrom<&[u8]> for OwnedTrustAnchor {
    type Error = CertificateError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let anchor = TrustAnchor::try_from_cert_der(value).map(|anchor| (&anchor).into())?;

        Ok(anchor)
    }
}

impl<'a> From<&'a OwnedTrustAnchor> for TrustAnchor<'a> {
    fn from(value: &'a OwnedTrustAnchor) -> Self {
        TrustAnchor {
            subject: &value.subject,
            spki: &value.spki,
            name_constraints: value.name_constraints.as_deref(),
        }
    }
}

/// An x509 certificate, unifying functionality from the following crates:
///
/// - parsing data: `x509_parser`
/// - verification of certificate chains: `webpki`
/// - signing and generating: `rcgen`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate(ByteBuf);

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

    /// Generate a new self-signed CA certificate.
    pub fn new_ca(common_name: &str) -> Result<(Certificate, SigningKey), CertificateError> {
        let mut ca_params = CertificateParams::new(vec![]);
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        ca_params.distinguished_name.push(DnType::CommonName, common_name);
        let cert = RcgenCertificate::from_params(ca_params)?;

        let privkey = Self::rcgen_cert_privkey(&cert)?;

        Ok((cert.serialize_der()?.into(), privkey))
    }

    /// Generate a new certificate signed with the specified CA certificate.
    pub fn new(
        ca: &Certificate,
        ca_privkey: &SigningKey,
        common_name: &str,
        certificate_type: CertificateType,
    ) -> Result<(Certificate, SigningKey), CertificateError> {
        let mut cert_params = CertificateParams::new(vec![]);
        cert_params.is_ca = IsCa::NoCa;
        cert_params.distinguished_name.push(DnType::CommonName, common_name);
        cert_params.custom_extensions.extend(certificate_type.to_custom_exts()?);
        let cert_unsigned = RcgenCertificate::from_params(cert_params).map_err(CertificateError::GeneratingFailed)?;

        let ca_keypair = rcgen::KeyPair::from_der(
            &ca_privkey
                .to_pkcs8_der()
                .map_err(CertificateError::GeneratingPrivateKey)?
                .to_bytes(),
        )?;
        let ca = RcgenCertificate::from_params(rcgen::CertificateParams::from_ca_cert_der(&ca.0, ca_keypair)?)?;

        let cert_bts = cert_unsigned.serialize_der_with_signer(&ca)?;
        let cert_privkey = Self::rcgen_cert_privkey(&cert_unsigned)?;

        Ok((cert_bts.into(), cert_privkey))
    }

    pub fn subject(&self) -> Result<IndexMap<String, String>, CertificateError> {
        let subject = self
            .to_x509()?
            .subject
            .iter_attributes()
            .map(|attr| {
                (
                    x509_parser::objects::oid2abbrev(attr.attr_type(), x509_parser::objects::oid_registry())
                        .map_or(attr.attr_type().to_id_string(), |v| v.to_string()),
                    attr.as_str().unwrap().to_string(), // TODO handle non-stringable values?
                )
            })
            .collect();

        Ok(subject)
    }

    fn rcgen_cert_privkey(cert: &RcgenCertificate) -> Result<SigningKey, CertificateError> {
        SigningKey::from_pkcs8_der(cert.get_key_pair().serialized_der()).map_err(CertificateError::GeneratingPrivateKey)
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
    pub fn from_certificate(cert: &Certificate) -> Result<Option<Self>, CertificateError> {
        let usage = cert
            .to_x509()?
            .extended_key_usage()?
            .and_then(|eku| Self::from_key_usage(eku.value));
        Ok(usage)
    }

    fn from_key_usage(ext_key_usage: &ExtendedKeyUsage) -> Option<Self> {
        if ext_key_usage.other.contains(&EKU_MDL_OID) {
            Some(Self::Mdl)
        } else if ext_key_usage.other.contains(&EKU_READER_AUTH_OID) {
            Some(Self::ReaderAuth)
        } else {
            None
        }
    }

    fn to_eku(&self) -> &'static [u8] {
        match self {
            CertificateUsage::Mdl => EXTENDED_KEY_USAGE_MDL,
            CertificateUsage::ReaderAuth => EXTENDED_KEY_USAGE_READER_AUTH,
        }
    }

    fn to_custom_ext(&self) -> CustomExtension {
        // The spec requires that we add mdoc-specific OIDs to the extended key usage extension, but [`CertificateParams`]
        // only supports a whitelist of key usages that it is aware of. So we DER-serialize it manually and add it to
        // the custom extensions.
        // We unwrap in these functions because they have fixed input for which they always succeed.
        let mut seq = SequenceOf::<ObjectIdentifier, 1>::new();
        seq.add(ObjectIdentifier::from_bytes(self.to_eku()).unwrap()).unwrap();
        let mut ext = CustomExtension::from_oid_content(OID_EXT_KEY_USAGE, seq.to_der().unwrap());
        ext.set_criticality(true);
        ext
    }
}

/// Acts as configuration for the [Certificate::new] function.
#[derive(Debug, Clone, PartialEq)]
pub enum CertificateType {
    Mdl,
    ReaderAuth(Box<ReaderRegistration>),
}

impl CertificateType {
    pub fn from_certificate(cert: &Certificate) -> Result<Option<Self>, CertificateError> {
        let usage = CertificateUsage::from_certificate(cert)?;
        let result = match usage {
            Some(CertificateUsage::Mdl) => Some(CertificateType::Mdl),
            Some(CertificateUsage::ReaderAuth) => {
                let registration: Option<ReaderRegistration> = ReaderRegistration::from_certificate(cert)?;
                registration.map(|r| Self::ReaderAuth(Box::new(r)))
            }
            None => None,
        };
        Ok(result)
    }

    fn to_custom_exts(&self) -> Result<Vec<CustomExtension>, CertificateError> {
        let usage: CertificateUsage = self.into();
        let mut extensions = vec![usage.to_custom_ext()];

        if let Self::ReaderAuth(auth) = self {
            let registration: &ReaderRegistration = auth;
            let ext_reader_auth = registration.to_custom_ext()?;
            extensions.push(ext_reader_auth);
        }

        Ok(extensions)
    }
}

impl From<&CertificateType> for CertificateUsage {
    fn from(source: &CertificateType) -> Self {
        use CertificateType::*;
        match source {
            Mdl => Self::Mdl,
            ReaderAuth(_) => Self::ReaderAuth,
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::Duration;
    use p256::pkcs8::ObjectIdentifier;
    use url::Url;
    use webpki::TrustAnchor;

    use wallet_common::generator::TimeGenerator;

    use crate::utils::{reader_auth::*, x509::CertificateType};

    use super::{Certificate, CertificateUsage};

    #[test]
    fn mdoc_eku_encoding_works() {
        CertificateUsage::Mdl.to_eku();
        CertificateUsage::ReaderAuth.to_eku();
    }

    #[test]
    fn generate_and_verify_cert() {
        let (ca, ca_privkey) = Certificate::new_ca("myca").unwrap();
        let ca_trustanchor: TrustAnchor = (&ca).try_into().unwrap();

        let (cert, _) = Certificate::new(&ca, &ca_privkey, "mycert", CertificateType::Mdl).unwrap();

        cert.verify(CertificateUsage::Mdl, &[], &TimeGenerator, &[ca_trustanchor])
            .unwrap();
    }

    #[test]
    fn generate_and_verify_cert_reader_auth() {
        let (ca, ca_privkey) = Certificate::new_ca("myca").unwrap();
        let ca_trustanchor: TrustAnchor = (&ca).try_into().unwrap();

        let reader_auth = CertificateType::ReaderAuth(Box::new(get_my_reader_auth()));

        let (cert, _) = Certificate::new(&ca, &ca_privkey, "mycert", reader_auth.clone()).unwrap();

        cert.verify(CertificateUsage::ReaderAuth, &[], &TimeGenerator, &[ca_trustanchor])
            .unwrap();

        // Verify whether the parsed CertificateType equals the original ReaderAuth usage
        let cert_usage = CertificateType::from_certificate(&cert).unwrap().unwrap();
        assert_eq!(cert_usage, reader_auth);
    }

    #[test]
    fn parse_oid() {
        let mdl_kp: ObjectIdentifier = "1.0.18013.5.1.2".parse().unwrap();
        let mdl_kp: &'static [u8] = Box::leak(mdl_kp.into()).as_bytes();
        assert_eq!(mdl_kp, CertificateUsage::Mdl.to_eku());
    }

    // Test fixture
    fn get_my_reader_auth() -> ReaderRegistration {
        let my_organization = Organization {
            display_name: vec![("nl", "Mijn Organisatienaam"), ("en", "My Organization Name")].into(),
            legal_name: vec![("nl", "Organisatie"), ("en", "Organization")].into(),
            description: vec![
                ("nl", "Beschrijving van Mijn Organisatie"),
                ("en", "Description of My Organization"),
            ]
            .into(),
            kvk: Some("some-kvk".to_owned()),
            city: Some(vec![("nl", "Den Haag"), ("en", "The Hague")].into()),
            country_code: Some("nl".to_owned()),
            web_url: Some(Url::parse("https://www.ons-dorp.nl").unwrap()),
            privacy_policy_url: Some(Url::parse("https://www.ons-dorp.nl/privacy").unwrap()),
            logo: None,
        };
        ReaderRegistration {
            id: "some-service-id".to_owned(),
            name: vec![("nl", "Naam van mijn dienst"), ("en", "My Service Name")].into(),
            purpose_statement: vec![("nl", "Beschrijving van mijn dienst"), ("en", "My Service Description")].into(),
            retention_policy: RetentionPolicy {
                intent_to_retain: true,
                max_duration: Some(Duration::minutes(525600)),
            },
            sharing_policy: SharingPolicy { intent_to_share: true },
            deletion_policy: DeletionPolicy { deleteable: true },
            organization: my_organization,
            attributes: Default::default(),
        }
    }
}
