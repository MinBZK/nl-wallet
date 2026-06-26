use std::convert::Into;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter::Iterator;
use std::iter::once;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Debug;
use error_category::ErrorCategory;
use http_utils::urls::HttpsUri;
use http_utils::urls::HttpsUriError;
use indexmap::IndexMap;
use itertools::Itertools;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::pkcs8::DecodePublicKey;
use p256::pkcs8::der::Decode;
use p256::pkcs8::der::SliceReader;
use p256::pkcs8::der::asn1::Utf8StringRef;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::UnixTime;
use rustls_pki_types::pem::PemObject;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use webpki::EndEntityCert;
use webpki::ring::ECDSA_P256_SHA256;
use x509_parser::asn1_rs::SerializeError;
use x509_parser::asn1_rs::ToDer;
use x509_parser::der_parser::Oid;
use x509_parser::extensions::GeneralName;
use x509_parser::nom::AsBytes;
use x509_parser::prelude::FromDer;
use x509_parser::prelude::PEMError;
use x509_parser::prelude::ParsedExtension;
use x509_parser::prelude::X509Certificate;
use x509_parser::prelude::X509Error;
use x509_parser::x509::X509Name;
use yoke::Yoke;
use yoke::Yokeable;

use crate::trust_anchor::TrustAnchors;

#[cfg(any(test, feature = "generate"))]
mod config;
mod dn;
mod key_identifier;
#[cfg(any(test, feature = "generate"))]
mod san;
mod usage;

#[cfg(any(test, feature = "generate"))]
pub use config::CertificateConfiguration;
pub use dn::CanonicalDistinguishedName;
pub use dn::DistinguishedName;
pub use dn::DistinguishedNameError;
pub use key_identifier::KeyIdentifier;
#[cfg(any(test, feature = "generate"))]
pub use san::NO_SAN;
#[cfg(any(test, feature = "generate"))]
pub use san::SubjectAltNameUri;
pub use usage::CertificateUsage;
pub use usage::CertificateUsageError;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(pd)]
pub enum CertificateError {
    #[error("certificate verification failed: {0}")]
    Verification(#[source] Box<webpki::Error>),
    #[error("certificate parsing failed: {0}")]
    CertificateParsing(#[source] Box<webpki::Error>),
    #[error("certificate parsing for validation failed: {0}")]
    EndEntityCertificateParsing(#[source] Box<webpki::Error>),
    #[error("certificate content parsing failed: {0}")]
    X509CertificateParsing(#[from] x509_parser::nom::Err<X509Error>),
    #[error("pem parsing failed: {0}")]
    PemParsing(#[from] rustls_pki_types::pem::Error),
    #[cfg(any(test, feature = "generate"))]
    #[error("certificate private key generation failed: {0}")]
    #[category(unexpected)]
    GeneratingPrivateKey(#[source] Box<p256::pkcs8::Error>),
    #[cfg(any(test, feature = "generate"))]
    #[error("certificate creation failed: {0}")]
    #[category(unexpected)]
    GeneratingFailed(#[from] rcgen::Error),
    #[cfg(any(test, feature = "generate"))]
    #[error("parsed X.509 certificate is not a root CA")]
    #[category(unexpected)]
    NotRootCa,
    #[cfg(any(test, feature = "generate"))]
    #[error("the basic constraint of this CA does not allow generating an intermediate CA")]
    #[category(unexpected)]
    BasicConstraintViolation,
    #[error("failed to parse certificate public key: {0}")]
    PublicKeyParsing(#[source] Box<p256::pkcs8::spki::Error>),
    #[error("PEM decoding error: {0}")]
    Pem(#[from] x509_parser::nom::Err<PEMError>),
    #[error("DER coding error: {0}")]
    DerEncodingError(#[source] Box<p256::pkcs8::der::Error>),
    #[error("JSON coding error: {0}")]
    JsonEncodingError(#[from] serde_json::Error),
    #[error("X509 coding error: {0}")]
    X509Error(#[from] X509Error),
    #[error("private key does not belong to public key from certificate")]
    KeyMismatch,
    #[error("failed to get public key from private key: {0}")]
    PublicKeyFromPrivate(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("missing SAN extension")]
    MissingSan,
    #[error("missing SAN DNS name or URI")]
    MissingSanDnsNameOrUri,
    #[error("SAN DNS name is not a URI: {0}")]
    SanDnsNameOrUriIsNotAnHttpsUri(HttpsUriError),
    #[error("could not serialize to DER: {0}")]
    DerSerialization(#[from] SerializeError),
    #[error("certificate chain must not contain the trust anchor")]
    #[category(critical)]
    TrustAnchorInChain,
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
    #[debug("subject: {}, issuer: {}, serial: {}", x509_cert.subject(), x509_cert.issuer(), x509_cert.tbs_certificate.raw_serial_as_string())]
    x509_cert: X509Certificate<'a>,
    #[debug(skip)]
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
pub struct BorrowingCertificate(#[debug("{:?}", _0.get())] YokedCertificate);

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
            let end_entity_cert = cert
                .try_into()
                .map_err(|error| CertificateError::EndEntityCertificateParsing(Box::new(error)))?;
            let (_, x509_cert) =
                X509Certificate::from_der(cert.as_bytes()).map_err(CertificateError::X509CertificateParsing)?;
            let public_key = VerifyingKey::from_public_key_der(x509_cert.public_key().raw)
                .map_err(|error| CertificateError::PublicKeyParsing(Box::new(error)))?;

            Ok::<_, CertificateError>(ParsedCertificate {
                end_entity_cert,
                x509_cert,
                public_key,
            })
        })?;

        Ok(BorrowingCertificate(yoke))
    }

    /// Verify the certificate against the specified trust anchors.
    ///
    /// Additionally, this method verifies that the trust anchors are not contained in the intermediate
    /// certificates as mandated by HAIP 1.0.
    pub fn verify(
        &self,
        usage: CertificateUsage,
        intermediate_certs: &[BorrowingCertificate],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &TrustAnchors,
    ) -> Result<(), CertificateError> {
        let chain = once(self).chain(intermediate_certs).collect_vec();

        // HAIP 1.0 requires that the x5c header does not contain the trust anchor.
        if chain.iter().any(|cert| trust_anchors.contains(cert)) {
            return Err(CertificateError::TrustAnchorInChain);
        }

        let intermediate_certs = intermediate_certs
            .iter()
            .map(BorrowingCertificate::as_der)
            .cloned()
            .collect_vec();

        self.end_entity_certificate()
            .verify_for_usage(
                &[ECDSA_P256_SHA256],
                trust_anchors.as_trust_anchor_slice(),
                intermediate_certs.as_slice(),
                // unwrap is safe here because we assume the time that is generated lies after the epoch
                UnixTime::since_unix_epoch(Duration::from_secs(time.generate().timestamp().try_into().unwrap())),
                webpki::KeyUsage::required(usage.as_oid_bytes()),
                None,
                None,
            )
            .map(|_| ())
            .map_err(|error| CertificateError::Verification(Box::new(error)))
    }

    pub fn end_entity_certificate(&self) -> &EndEntityCert<'_> {
        &self.0.get().end_entity_cert
    }

    pub fn x509_certificate(&self) -> &X509Certificate<'_> {
        &self.0.get().x509_cert
    }

    pub fn public_key(&self) -> &VerifyingKey {
        &self.0.get().public_key
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    pub fn as_der<'a>(&'a self) -> &'a CertificateDer<'a> {
        self.0.backing_cart()
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

    /// Returns the first CN, if any, from the certificate.
    pub fn common_name(&self) -> Result<Option<&str>, CertificateError> {
        Ok(self.common_names()?.into_iter().next())
    }

    pub fn to_distinguished_name(&self) -> Result<DistinguishedName, DistinguishedNameError> {
        let subject = &self.0.get().x509_cert.subject;
        subject.try_into()
    }

    // Returns a human-readable string representation of the distinguished name attributes with the raw OID values and
    // base64-encoded DER values. Can be used for persistence and comparison, since this representation is independent
    // of an OID registry.
    pub fn to_canonical_distinguished_name(&self) -> Result<CanonicalDistinguishedName, CertificateError> {
        let encoded_attrs: Vec<_> = self
            .x509_certificate()
            .subject()
            .iter_attributes()
            .map(|attr| {
                let r#type = attr.attr_type().to_id_string();
                let value = BASE64_STANDARD_NO_PAD.encode(&attr.attr_value().to_der_vec()?);
                Ok::<_, CertificateError>(format!("{}={}", r#type, value))
            })
            .try_collect()?;

        Ok(CanonicalDistinguishedName::new(encoded_attrs.join(",")))
    }

    /// Returns the SAN DNS names and URIs from the certificate, as an HTTPS URI.
    pub fn san_dns_name_or_uris(&self) -> Result<VecNonEmpty<HttpsUri>, CertificateError> {
        let san_ext = self
            .x509_certificate()
            .subject_alternative_name()?
            .ok_or(CertificateError::MissingSan)?;

        let san_dns_name_or_uri = san_ext.value.general_names.iter().filter_map(|name| match name {
            GeneralName::DNSName(name) => Some(format!("https://{name}")),
            GeneralName::URI(uri) => Some(uri.to_string()),
            _ => None,
        });

        let san_https_uris = san_dns_name_or_uri
            .map(|san| san.parse().map_err(CertificateError::SanDnsNameOrUriIsNotAnHttpsUri))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| CertificateError::MissingSanDnsNameOrUri)?;

        Ok(san_https_uris)
    }

    /// From the AuthorityKeyIdentifier in the certificate, if present, return the key identifier field:
    /// the hash over the public key that signed this certificate.
    pub fn authority_key_id(&self) -> Option<KeyIdentifier> {
        self.x509_certificate().extensions().iter().find_map(|ext| {
            let ParsedExtension::AuthorityKeyIdentifier(aki) = ext.parsed_extension() else {
                return None;
            };
            aki.key_identifier.as_ref().map(|ki| ki.0.to_vec().into())
        })
    }

    pub(crate) fn parse_and_extract_custom_ext<'a, T: Deserialize<'a>>(
        &'a self,
        oid: &Oid,
    ) -> Result<Option<T>, CertificateError> {
        let x509_cert = self.x509_certificate();
        let ext = x509_cert.iter_extensions().find(|ext| ext.oid == *oid);
        ext.map(|ext| {
            let mut reader =
                SliceReader::new(ext.value).map_err(|error| CertificateError::DerEncodingError(Box::new(error)))?;
            let json = Utf8StringRef::decode(&mut reader)
                .map_err(|error| CertificateError::DerEncodingError(Box::new(error)))?;
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

impl Hash for BorrowingCertificate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

fn x509_common_names<'a>(x509name: &'a X509Name) -> Result<Vec<&'a str>, CertificateError> {
    x509name
        .iter_common_name()
        .map(|cn| cn.as_str().map_err(CertificateError::X509Error))
        .collect()
}

pub trait BorrowingCertificateExtension
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
        let string =
            Utf8StringRef::new(&json_string).map_err(|error| CertificateError::DerEncodingError(Box::new(error)))?;

        let sub_identifiers = Self::OID
            .iter()
            .expect("oid sub identifier does not fit in u64")
            .collect::<Vec<_>>();
        let ext = rcgen::CustomExtension::from_oid_content(
            sub_identifiers.as_slice(),
            string
                .to_der()
                .map_err(|error| CertificateError::DerEncodingError(Box::new(error)))?,
        );
        Ok(ext)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::slice::from_ref;

    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use time::OffsetDateTime;
    use time::macros::datetime;
    use utils::generator::TimeGenerator;
    use x509_parser::certificate::X509Certificate;

    use super::*;
    use crate::server_keys::generate::Ca;
    use crate::trust_anchor::TrustAnchors;

    #[test]
    fn generate_ca() {
        let dn = DistinguishedName::create_mock("myca");
        let ca = Ca::generate(dn.clone(), Default::default()).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.certificate().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        let x509_cert = certificate.x509_certificate();
        let basic_constraint = x509_cert.basic_constraints().unwrap().unwrap();
        assert!(basic_constraint.critical);
        assert!(basic_constraint.value.ca);
        assert_eq!(basic_constraint.value.path_len_constraint, Some(0));
        assert_certificate_default_validity(x509_cert);
    }

    #[test]
    fn generate_ca_with_configuration() {
        let now = Utc::now();
        let later = now + Duration::days(42);

        let config = CertificateConfiguration {
            not_before: Some(now),
            not_after: Some(later),
            ..Default::default()
        };
        let dn = DistinguishedName {
            common_name: "myca".to_string(),
            country_name: "NL".to_string(),
            organization_name: "My CA B.V.".to_string(),
            organization_identifier: "VATNL-123456789B01".to_string(),
        };
        let ca = Ca::generate(dn.clone(), config).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.certificate().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        assert_eq!(dn, certificate.to_distinguished_name().unwrap());
        assert_eq!(
            "2.5.4.3=DARteWNh,2.5.4.6=DAJOTA,2.5.4.10=DApNeSBDQSBCLlYu,1.3.6.1.1.15=DBJWQVROTC0xMjM0NTY3ODlCMDE",
            certificate.to_canonical_distinguished_name().unwrap().as_ref()
        );

        let x509_cert = certificate.x509_certificate();
        assert_certificate_validity(x509_cert, now, later);
    }

    fn generate_and_verify_issuer_for_validity(
        not_before: Option<DateTime<Utc>>,
        not_after: Option<DateTime<Utc>>,
    ) -> CertificateError {
        let ca = generate_ca_for_validity_test();

        let config = CertificateConfiguration {
            not_before,
            not_after,
            usage: Some(CertificateUsage::Mdl),
            ..Default::default()
        };

        let issuer_key_pair = ca
            .generate_key_pair(DistinguishedName::create_mock("mycert"), config, NO_SAN)
            .unwrap();
        issuer_key_pair
            .certificate()
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, &TrustAnchors::from(&ca))
            .expect_err("Expected verify to fail")
    }

    #[test]
    fn generate_and_verify_not_yet_valid_issuer_cert() {
        let now = Utc::now();
        let start = Some(now + Duration::days(1));
        let end = Some(now + Duration::days(2));

        let error = generate_and_verify_issuer_for_validity(start, end);
        assert_matches!(
            error,
            CertificateError::Verification(error) if matches!(*error, webpki::Error::CertNotValidYet { .. })
        );
    }

    #[test]
    fn generate_and_verify_expired_issuer_cert() {
        let now = Utc::now();
        let start = Some(now - Duration::days(2));
        let end = Some(now - Duration::days(1));

        let error = generate_and_verify_issuer_for_validity(start, end);
        assert_matches!(
            error,
            CertificateError::Verification(error) if matches!(*error, webpki::Error::CertExpired { .. })
        );
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

    fn generate_ca_for_validity_test() -> Ca {
        let now = Utc::now();
        let start = now - Duration::weeks(52);
        let end = now + Duration::weeks(52);

        let config = CertificateConfiguration {
            not_before: Some(start),
            not_after: Some(end),
            ..Default::default()
        };

        Ca::generate(DistinguishedName::create_mock("myca"), config).unwrap()
    }

    // Cross-signing scenario:
    //
    // An old CA signs the new CA's key (cross-cert), which acts as an intermediate. The new CA
    // later self-signs its own root cert. During the transition:
    //   - Certificates issued under the new key still verify against the old CA trust anchor (using the cross-cert as
    //     intermediate).
    //   - The cross-cert is NOT mistakenly flagged as a trust anchor even when the self-signed new CA IS a trust
    //     anchor, because their issuers differ (the issuer check).
    //   - New certificates issued under the new key verify directly against the new CA trust anchor.
    //   - The self-signed new CA cert IS correctly detected as a trust anchor when included in the chain (its issuer
    //     equals its own subject).
    #[test]
    fn cross_sign_scenario() {
        let time = TimeGenerator;

        // Old CA must allow one intermediate so it can sign the cross-cert.
        let old_ca =
            Ca::generate_with_intermediate_count(DistinguishedName::create_mock("old_ca"), Default::default(), 1)
                .unwrap();
        // new_ca uses the same key pair for both the self-signed root and the cross-cert (signed
        // by old_ca). This is the core of a cross-signing setup.
        let cert_config = CertificateConfiguration::with_usage(CertificateUsage::Mdl);
        let (new_ca, cross_cert_der) = old_ca
            .generate_root_and_cross_cert(DistinguishedName::create_mock("new_ca"), cert_config.clone())
            .unwrap();
        let cross_cert = BorrowingCertificate::from_certificate_der(cross_cert_der).unwrap();

        // Both leaves are signed by the new CA key (same key in cross-cert and self-signed cert).
        let leaf = new_ca
            .generate_key_pair(DistinguishedName::create_mock("leaf"), cert_config, NO_SAN)
            .unwrap();

        // Phase 1: leaf verified against old CA with the cross-cert as intermediate.
        // Path: leaf ← cross-cert (new key, signed by old CA) ← old CA trust anchor.
        leaf.certificate()
            .verify(
                CertificateUsage::Mdl,
                from_ref(&cross_cert),
                &time,
                &TrustAnchors::from(&old_ca),
            )
            .expect("leaf should verify against old CA via cross-cert intermediate");

        // Phase 2: same verification but with the self-signed new CA cert added to the
        // intermediates as well. It must not trigger TrustAnchorInChain because the trust anchor
        // here is old CA (different subject/SPKI from the new CA cert).
        leaf.certificate()
            .verify(
                CertificateUsage::Mdl,
                &[cross_cert, new_ca.as_borrowing_certificate().unwrap()],
                &time,
                &TrustAnchors::from(&old_ca),
            )
            .expect("leaf should still verify when self-signed new CA cert is added to intermediates");

        // Phase 3: leaf verified directly against the new CA trust anchor (no intermediates).
        leaf.certificate()
            .verify(CertificateUsage::Mdl, &[], &time, &TrustAnchors::from(&new_ca))
            .expect("leaf should verify against self-signed new CA");
    }

    #[test]
    fn chain_with_intermediate_scenario() {
        let time = TimeGenerator;
        let cert_config = CertificateConfiguration::with_usage(CertificateUsage::ReaderAuth);

        // CA
        let ca = Ca::generate_with_intermediate_count(
            DistinguishedName::create_mock("ca"),
            CertificateConfiguration::default(),
            2,
        )
        .unwrap();
        let ca_ta = ca.to_borrowing_trust_anchor();
        let ca_cert = ca.as_borrowing_certificate().unwrap();

        // Intermediate
        let intermediate_ca = ca
            .generate_intermediate(DistinguishedName::create_mock("intermediate"), cert_config.clone())
            .unwrap();
        let intermediate_ta = intermediate_ca.to_borrowing_trust_anchor();
        let intermediate_cert = intermediate_ca.as_borrowing_certificate().unwrap();

        // Leaf
        let leaf_key_pair = intermediate_ca
            .generate_key_pair(DistinguishedName::create_mock("leaf"), cert_config, NO_SAN)
            .unwrap();

        // Verify whole chain with leaf, intermediate and ca trust anchor
        leaf_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth,
                from_ref(&intermediate_cert),
                &time,
                &TrustAnchors::from(&ca),
            )
            .expect("should verify");

        // Verify leaf with only intermediate as trust anchor
        leaf_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth,
                &[],
                &time,
                &TrustAnchors::from(&intermediate_ca),
            )
            .expect("should verify");

        // Verify whole chain with intermediate both in intermediates and trust anchors
        let error = leaf_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth,
                from_ref(&intermediate_cert),
                &time,
                &TrustAnchors::try_from(vec![intermediate_ta, ca_ta.clone()]).unwrap(),
            )
            .expect_err("should detect TrustAnchorInChain");
        assert_matches!(error, CertificateError::TrustAnchorInChain);

        // Verify whole chain with ca in both intermediates and trust anchors
        let error = leaf_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth,
                &[intermediate_cert, ca_cert],
                &time,
                &TrustAnchors::from(&ca),
            )
            .expect_err("should detect TrustAnchorInChain");
        assert_matches!(error, CertificateError::TrustAnchorInChain);
    }
}
