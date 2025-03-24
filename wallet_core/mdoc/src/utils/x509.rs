use crypto::x509::BorrowingCertificateExtension;
use derive_more::Debug;
use x509_parser::der_parser::Oid;
use x509_parser::oid_registry::asn1_rs::oid;
use x509_parser::prelude::ExtendedKeyUsage;
use x509_parser::prelude::X509Certificate;

use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;

use super::issuer_auth::IssuerRegistration;
use super::reader_auth::ReaderRegistration;

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

    pub fn eku(self) -> &'static [u8] {
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

    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateConfiguration;
    use wallet_common::generator::TimeGenerator;

    use crate::utils::issuer_auth::IssuerRegistration;
    use crate::utils::reader_auth::ReaderRegistration;
    use crate::utils::x509::CertificateType;

    use super::BorrowingCertificate;
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
        let ca = Ca::generate("myca", Default::default()).unwrap();
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
        let ca = Ca::generate("myca", config).unwrap();
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
        let mdl = IssuerRegistration::new_mock();

        let issuer_key_pair = ca.generate_key_pair("mycert", mdl, config).unwrap();
        issuer_key_pair
            .certificate()
            .verify(
                CertificateUsage::Mdl.eku(),
                &[],
                &TimeGenerator,
                &[ca.to_trust_anchor()],
            )
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
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let mdl = CertificateType::from(IssuerRegistration::new_mock());

        let issuer_key_pair = ca.generate_key_pair("mycert", mdl.clone(), Default::default()).unwrap();

        issuer_key_pair
            .certificate()
            .verify(
                CertificateUsage::Mdl.eku(),
                &[],
                &TimeGenerator,
                &[ca.to_trust_anchor()],
            )
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

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let mdl = CertificateType::from(IssuerRegistration::new_mock());

        let issuer_key_pair = ca.generate_key_pair("mycert", mdl.clone(), config).unwrap();

        issuer_key_pair
            .certificate()
            .verify(
                CertificateUsage::Mdl.eku(),
                &[],
                &TimeGenerator,
                &[ca.to_trust_anchor()],
            )
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
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca
            .generate_key_pair("mycert", reader_auth.clone(), Default::default())
            .unwrap();

        reader_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth.eku(),
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

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca.generate_key_pair("mycert", reader_auth.clone(), config).unwrap();

        reader_key_pair
            .certificate()
            .verify(
                CertificateUsage::ReaderAuth.eku(),
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

    fn generate_ca_for_validity_test() -> Ca {
        let now = Utc::now();
        let start = now - Duration::weeks(52);
        let end = now + Duration::weeks(52);

        let config = CertificateConfiguration {
            not_before: Some(start),
            not_after: Some(end),
        };

        Ca::generate("myca", config).unwrap()
    }
}
