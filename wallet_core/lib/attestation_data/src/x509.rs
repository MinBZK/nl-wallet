use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use crypto::x509::CertificateUsageError;
use derive_more::Debug;
use error_category::ErrorCategory;

use crate::auth::issuer_auth::IssuerRegistration;
use crate::auth::reader_auth::ReaderRegistration;

/// Acts as configuration for the [Certificate::new] function.
#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::large_enum_variant)] // CertificateType is only used as a temporary result
pub enum CertificateType {
    Mdl(IssuerRegistration),
    ReaderAuth(ReaderRegistration),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CertificateTypeError {
    /// TODO: PVW-5885 PVW-5895 Remove when ReaderRegistration and IssuerRegistration are removed
    #[error("certificate error: {0}")]
    #[category(defer)]
    Certificate(#[from] CertificateError),

    #[error("certificate usage error: {0}")]
    #[category(critical)]
    CertificateUsage(#[source] CertificateUsageError),

    #[error("unknown usage: {0}")]
    #[category(critical)]
    UnknownUsage(CertificateUsage),

    #[error("issuer registration not found")]
    #[category(critical)]
    IssuerRegistrationNotFound,

    #[error("reader registration not found")]
    #[category(critical)]
    ReaderRegistrationNotFound,
}

impl CertificateType {
    pub fn has_certificate_type(usage: CertificateUsage) -> bool {
        matches!(usage, CertificateUsage::Mdl | CertificateUsage::ReaderAuth)
    }

    pub fn from_certificate(cert: &BorrowingCertificate) -> Result<Self, CertificateTypeError> {
        let usage = CertificateUsage::from_certificate(cert.x509_certificate())
            .map_err(CertificateTypeError::CertificateUsage)?;
        let result = match usage {
            CertificateUsage::Mdl => {
                let Some(registration) = IssuerRegistration::from_certificate(cert)? else {
                    return Err(CertificateTypeError::IssuerRegistrationNotFound);
                };
                CertificateType::Mdl(registration)
            }
            CertificateUsage::ReaderAuth => {
                let Some(registration) = ReaderRegistration::from_certificate(cert)? else {
                    return Err(CertificateTypeError::ReaderRegistrationNotFound);
                };
                CertificateType::ReaderAuth(registration)
            }
            _ => return Err(CertificateTypeError::UnknownUsage(usage)),
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
    #[cfg(any(test, feature = "mock"))]
    pub mod mock {
        use crypto::server_keys::KeyPair;
        use crypto::server_keys::generate::Ca;
        use crypto::server_keys::generate::mock::ISSUANCE_CERT_DN;
        use crypto::server_keys::generate::mock::ISSUANCE_CERT_SAN_URI;
        use crypto::server_keys::generate::mock::PID_ISSUER_CERT_DN;
        use crypto::server_keys::generate::mock::PID_ISSUER_CERT_SAN_URI;
        use crypto::server_keys::generate::mock::RP_CERT_DN;
        use crypto::server_keys::generate::mock::RP_CERT_SAN_URI;
        use crypto::x509::CertificateError;

        use crate::auth::issuer_auth::IssuerRegistration;
        use crate::auth::reader_auth::ReaderRegistration;

        pub fn generate_issuer_mock_with_registration(
            ca: &Ca,
            issuer_registration: &IssuerRegistration,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                ISSUANCE_CERT_DN.clone(),
                issuer_registration.to_certificate_configuration()?,
                [ISSUANCE_CERT_SAN_URI.clone()],
            )
        }

        pub fn generate_pid_issuer_mock_with_registration(
            ca: &Ca,
            issuer_registration: &IssuerRegistration,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                PID_ISSUER_CERT_DN.clone(),
                issuer_registration.to_certificate_configuration()?,
                [PID_ISSUER_CERT_SAN_URI.clone()],
            )
        }

        pub fn generate_reader_mock_with_registration(
            ca: &Ca,
            reader_registration: &ReaderRegistration,
        ) -> Result<KeyPair, CertificateError> {
            ca.generate_key_pair(
                RP_CERT_DN.clone(),
                reader_registration.to_certificate_configuration()?,
                [RP_CERT_SAN_URI.clone()],
            )
        }
    }
}
