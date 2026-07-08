use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use crypto::x509::CertificateUsageError;
use crypto::x509::DistinguishedName;
use crypto::x509::DistinguishedNameError;
use derive_more::Debug;
use error_category::ErrorCategory;

use crate::auth::Organization;
use crate::auth::issuer_auth::IssuerRegistration;
use crate::auth::reader_auth::ReaderRegistration;

/// Relying party of X509 certificates following ETSI EN 319 412-2 and ETSI EN 319 412-3 standard.
#[derive(Debug)]
pub enum RelyingParty {
    LegalPerson {
        common_name: String,
        country_name: String,
        organization_name: String,
        organization_identifier: String,
    },
    NaturalPerson {
        common_name: String,
        country_name: String,
        serial_number: String,
        surname: String,
        given_name: String,
    },
}

#[derive(thiserror::Error, Debug)]
#[error("cannot derive RelyingParty from DistinguishedName: {0:?}")]
pub struct RelyingPartyError(Box<DistinguishedName>);

impl TryFrom<DistinguishedName> for RelyingParty {
    type Error = RelyingPartyError;

    fn try_from(value: DistinguishedName) -> Result<Self, Self::Error> {
        match value {
            DistinguishedName {
                common_name,
                country_name,
                serial_number: Some(serial_number),
                surname: Some(surname),
                given_name: Some(given_name),
                ..
            } => Ok(RelyingParty::NaturalPerson {
                common_name,
                country_name,
                serial_number,
                surname,
                given_name,
            }),
            DistinguishedName {
                common_name,
                country_name,
                organization_name: Some(organization_name),
                organization_identifier: Some(organization_identifier),
                ..
            } => Ok(RelyingParty::LegalPerson {
                common_name,
                country_name,
                organization_name,
                organization_identifier,
            }),
            _ => Err(RelyingPartyError(value.into())),
        }
    }
}

impl RelyingParty {
    pub fn amend_to_organization(self, organization: &mut Organization) {
        match self {
            RelyingParty::LegalPerson {
                common_name,
                country_name,
                organization_name,
                organization_identifier,
            } => {
                organization.display_name = common_name;
                organization.legal_name = organization_name;
                organization.identifier = Some(organization_identifier);
                organization.country_code = country_name;
            }
            RelyingParty::NaturalPerson {
                common_name,
                country_name,
                given_name,
                surname,
                ..
            } => {
                organization.display_name = common_name;
                organization.legal_name = format!("{} {}", given_name, surname);
                organization.identifier = None;
                organization.country_code = country_name;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_legal_person_name() {
        let dn = DistinguishedName::create_legal_person_mock("Test");
        let rp = RelyingParty::try_from(dn.clone()).unwrap();
        let RelyingParty::LegalPerson {
            common_name,
            country_name,
            organization_name,
            organization_identifier,
        } = rp
        else {
            panic!("not a legal person");
        };
        assert_eq!(common_name, dn.common_name);
        assert_eq!(country_name, dn.country_name);
        assert_eq!(Some(organization_name), dn.organization_name);
        assert_eq!(Some(organization_identifier), dn.organization_identifier);
    }

    #[test]
    fn parse_natural_person_name() {
        let dn = DistinguishedName::create_natural_person_mock("John", "Doe");
        let rp = RelyingParty::try_from(dn.clone()).unwrap();
        let RelyingParty::NaturalPerson {
            common_name,
            country_name,
            serial_number,
            surname,
            given_name,
        } = rp
        else {
            panic!("not a natural person");
        };
        assert_eq!(common_name, dn.common_name);
        assert_eq!(country_name, dn.country_name);
        assert_eq!(Some(serial_number), dn.serial_number);
        assert_eq!(Some(surname), dn.surname);
        assert_eq!(Some(given_name), dn.given_name);
    }

    #[test]
    fn parse_natural_person_name_with_organization() {
        let mut dn = DistinguishedName::create_natural_person_mock("John", "Doe");
        dn.organization_name = Some("Test B.V.".into());
        dn.organization_identifier = Some("NTRNL-12345678".into());
        let rp = RelyingParty::try_from(dn.clone()).unwrap();
        assert_matches!(rp, RelyingParty::NaturalPerson { .. });
    }

    #[test]
    fn parse_no_rp() {
        let dn = DistinguishedName::create_mock("Test");
        let err = RelyingParty::try_from(dn.clone()).unwrap_err();
        assert_matches!(err, RelyingPartyError(err_dn) if *err_dn == dn);
    }
}

/// Acts as configuration for the [Certificate::new] function
///
/// TODO: PVW-5885 PVW-5870 Remove when ReaderRegistration and IssuerRegistration are removed
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "CertificateType is only used as a temporary result"
)]
pub enum CertificateType {
    Mdl(IssuerRegistration),
    ReaderAuth(ReaderRegistration),
}

/// TODO: PVW-5885 PVW-5870 Remove when ReaderRegistration and IssuerRegistration are removed
#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CertificateTypeError {
    #[error("certificate error: {0}")]
    #[category(defer)]
    Certificate(#[from] CertificateError),

    #[error("distinguished name error: {0}")]
    #[category(critical)]
    DistinguishedName(#[source] DistinguishedNameError),

    #[error("relying party error: {0}")]
    #[category(critical)]
    RelyingParty(#[source] RelyingPartyError),

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
                let Some(mut registration) = IssuerRegistration::from_certificate(cert)? else {
                    return Err(CertificateTypeError::IssuerRegistrationNotFound);
                };

                // TODO: PVW-5885 Temporarily hack to fill in access certification fields into registration organization
                let dn = cert
                    .to_distinguished_name()
                    .map_err(CertificateTypeError::DistinguishedName)?;
                let rp = RelyingParty::try_from(dn).map_err(CertificateTypeError::RelyingParty)?;
                rp.amend_to_organization(registration.organization.as_mut());

                CertificateType::Mdl(registration)
            }
            CertificateUsage::ReaderAuth => {
                let Some(mut registration) = ReaderRegistration::from_certificate(cert)? else {
                    return Err(CertificateTypeError::ReaderRegistrationNotFound);
                };

                // TODO: PVW-5895 Temporarily hack to fill in access certification fields into registration organization
                let dn = cert
                    .to_distinguished_name()
                    .map_err(CertificateTypeError::DistinguishedName)?;
                let rp = RelyingParty::try_from(dn).map_err(CertificateTypeError::RelyingParty)?;
                rp.amend_to_organization(registration.organization.as_mut());

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

/// TODO: PVW-5885 PVW-5870 Remove when ReaderRegistration and IssuerRegistration are removed
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
