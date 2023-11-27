use chrono::{DateTime, TimeZone, Utc};
use webpki::TrustAnchor;

use wallet_common::{generator::Generator, keys::software::SoftwareEcdsaKey};

use crate::{
    examples::{Example, Examples},
    holder::Mdoc,
    identifiers::AttributeIdentifier,
    server_keys::PrivateKey,
    utils::x509::{Certificate, CertificateError, CertificateType},
    DeviceResponse,
};

/// Some of the certificates in the ISO examples are valid from Oct 1, 2020 to Oct 1, 2021.
/// This generator returns a time in that window.
pub struct IsoCertTimeGenerator;
impl Generator<DateTime<Utc>> for IsoCertTimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()
    }
}

/// Out of the example data structures in the standard, assemble an mdoc.
/// The issuer-signed part of the mdoc is based on a [`DeviceResponse`] in which not all attributes of the originating
/// mdoc are disclosed. Consequentially, the issuer signed-part of the resulting mdoc misses some [`IssuerSignedItem`]
/// instances, i.e. attributes.
/// This is because the other attributes are actually nowhere present in the standard so it is impossible to construct
/// the example mdoc with all attributes present.
///
/// Using tests should not rely on all attributes being present.
pub fn mdoc_from_example_device_response(trust_anchors: &[TrustAnchor<'_>]) -> Mdoc {
    // Prepare the mdoc's private key
    let static_device_key = Examples::static_device_key();
    SoftwareEcdsaKey::insert("example_static_device_key", static_device_key);

    let issuer_signed = DeviceResponse::example().documents.as_ref().unwrap()[0]
        .issuer_signed
        .clone();

    Mdoc::new::<SoftwareEcdsaKey>(
        "example_static_device_key".to_string(),
        issuer_signed,
        &IsoCertTimeGenerator,
        trust_anchors,
    )
    .unwrap()
}

const ISSUANCE_CA_CN: &str = "ca.issuer.example.com";
const ISSUANCE_CERT_CN: &str = "cert.issuer.example.com";

pub fn generate_issuance_key_and_ca() -> Result<(PrivateKey, Certificate), CertificateError> {
    // Issuer CA certificate and normal certificate
    let (ca, ca_privkey) = Certificate::new_ca(ISSUANCE_CA_CN)?;
    let (issuer_cert, issuer_privkey) = Certificate::new(&ca, &ca_privkey, ISSUANCE_CERT_CN, CertificateType::Mdl)?;
    let issuance_key = PrivateKey::new(issuer_privkey, issuer_cert.as_bytes().into());

    Ok((issuance_key, ca))
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AttributeIdParsingError {
    #[error("Expected string with 3 parts separated by '/', got {0} parts")]
    InvalidPartsCount(usize),
}

// This implementation is solely intended for unit testing purposes to easily construct AttributeIdentifiers.
// This implementation should never end up in production code, because the use of '/' is officially allowed in the
// various parts.
impl std::str::FromStr for AttributeIdentifier {
    type Err = AttributeIdParsingError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let parts = source.split('/').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(AttributeIdParsingError::InvalidPartsCount(parts.len()));
        }
        let result = Self {
            doc_type: parts[0].to_owned(),
            namespace: parts[1].to_owned(),
            attribute: parts[2].to_owned(),
        };
        Ok(result)
    }
}
