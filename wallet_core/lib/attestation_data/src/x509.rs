use derive_more::Debug;

use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;

use crate::auth::issuer_auth::IssuerRegistration;
use crate::auth::reader_auth::ReaderRegistration;

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

#[cfg(any(test, feature = "generate"))]
pub mod generate {
    use rcgen::CustomExtension;

    use crypto::x509::BorrowingCertificateExtension;
    use crypto::x509::CertificateError;
    use crypto::x509::CertificateUsage;

    use crate::x509::CertificateType;

    impl TryFrom<CertificateType> for Vec<CustomExtension> {
        type Error = CertificateError;

        fn try_from(source: CertificateType) -> Result<Vec<CustomExtension>, CertificateError> {
            let usage = CertificateUsage::from(&source);
            let mut extensions = vec![usage.into()];

            match source {
                CertificateType::ReaderAuth(Some(reader_registration)) => {
                    let ext_reader_auth = reader_registration.to_custom_ext()?;
                    extensions.push(ext_reader_auth);
                }
                CertificateType::Mdl(Some(issuer_registration)) => {
                    let ext_issuer_auth = issuer_registration.to_custom_ext()?;
                    extensions.push(ext_issuer_auth);
                }
                _ => {}
            };
            Ok(extensions)
        }
    }

    #[cfg(any(test, feature = "mock"))]
    pub mod mock {
        use crypto::server_keys::KeyPair;
        use crypto::server_keys::generate::Ca;
        use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
        use crypto::server_keys::generate::mock::RP_CERT_CN;

        use crate::auth::issuer_auth::IssuerRegistration;
        use crate::auth::reader_auth::ReaderRegistration;

        use super::*;

        pub fn generate_issuer_mock(
            ca: &Ca,
            issuer_registration: Option<IssuerRegistration>,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                ISSUANCE_CERT_CN,
                CertificateType::Mdl(issuer_registration.map(Box::new)),
                Default::default(),
            )
        }

        pub fn generate_reader_mock(
            ca: &Ca,
            reader_registration: Option<ReaderRegistration>,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                RP_CERT_CN,
                CertificateType::ReaderAuth(reader_registration.map(Box::new)),
                Default::default(),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use time::OffsetDateTime;
    use time::macros::datetime;
    use x509_parser::certificate::X509Certificate;

    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateConfiguration;
    use utils::generator::TimeGenerator;

    use crate::auth::issuer_auth::IssuerRegistration;
    use crate::auth::reader_auth::ReaderRegistration;
    use crate::x509::CertificateType;

    use super::CertificateUsage;

    #[test]
    fn generate_and_verify_issuer_cert() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let mdl = CertificateType::from(IssuerRegistration::new_mock());

        let issuer_key_pair = ca.generate_key_pair("mycert", mdl.clone(), Default::default()).unwrap();

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

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let mdl = CertificateType::from(IssuerRegistration::new_mock());

        let issuer_key_pair = ca.generate_key_pair("mycert", mdl.clone(), config).unwrap();

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
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca
            .generate_key_pair("mycert", reader_auth.clone(), Default::default())
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

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let reader_auth: CertificateType = ReaderRegistration::new_mock().into();

        let reader_key_pair = ca.generate_key_pair("mycert", reader_auth.clone(), config).unwrap();

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
}
