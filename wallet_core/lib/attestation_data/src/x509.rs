use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use crypto::x509::CertificateUsageError;
use crypto::x509::DistinguishedName;
use derive_more::Debug;
use error_category::ErrorCategory;

use crate::auth::Organization;
use crate::auth::OrganizationError;
use crate::auth::issuer_auth::IssuerRegistration;

/// Relying party of X509 certificates following ETSI EN 319 412-2 and ETSI EN 319 412-3 standard.
#[derive(Debug, Clone)]
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

impl From<RelyingParty> for Organization {
    fn from(rp: RelyingParty) -> Self {
        match rp {
            RelyingParty::LegalPerson {
                common_name,
                country_name,
                organization_name,
                organization_identifier,
            } => Organization {
                display_name: common_name,
                legal_name: organization_name,
                description: Default::default(),
                category: Default::default(),
                logo: None,
                web_url: None,
                identifier: organization_identifier,
                city: None,
                department: None,
                country_code: country_name,
                privacy_policy_url: None,
            },
            RelyingParty::NaturalPerson {
                common_name,
                country_name,
                serial_number,
                given_name,
                surname,
            } => Organization {
                display_name: common_name,
                legal_name: format!("{}, {}", surname, given_name),
                description: Default::default(),
                category: Default::default(),
                logo: None,
                web_url: None,
                identifier: serial_number,
                city: None,
                department: None,
                country_code: country_name,
                privacy_policy_url: None,
            },
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
/// TODO: PVW-5870 Remove when IssuerRegistration are removed
#[derive(Debug, Clone, PartialEq)]
pub enum CertificateType {
    Mdl(IssuerRegistration),
}

/// TODO: PVW-5870 Remove when IssuerRegistration are removed
#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CertificateTypeError {
    #[error("certificate error: {0}")]
    #[category(defer)]
    Certificate(#[from] CertificateError),

    #[error("organization error: {0}")]
    #[category(critical)]
    Organization(#[source] OrganizationError),

    #[error("certificate usage error: {0}")]
    #[category(critical)]
    CertificateUsage(#[source] CertificateUsageError),

    #[error("unknown usage: {0}")]
    #[category(critical)]
    UnknownUsage(CertificateUsage),

    #[error("issuer registration not found")]
    #[category(critical)]
    IssuerRegistrationNotFound,
}

impl CertificateType {
    pub fn has_certificate_type(usage: CertificateUsage) -> bool {
        matches!(usage, CertificateUsage::Mdl)
    }

    pub fn from_certificate(cert: &BorrowingCertificate) -> Result<Self, CertificateTypeError> {
        let usage = CertificateUsage::from_certificate(cert.x509_certificate())
            .map_err(CertificateTypeError::CertificateUsage)?;
        let result = match usage {
            CertificateUsage::Mdl => {
                let Some(mut registration) = IssuerRegistration::from_certificate(cert)? else {
                    return Err(CertificateTypeError::IssuerRegistrationNotFound);
                };

                // TODO: PVW-5870 PVW-6111 Temporarily hack to fill in access certification fields into organization
                let org = Organization::try_from(cert).map_err(CertificateTypeError::Organization)?;
                registration.organization.display_name = org.display_name;
                registration.organization.legal_name = org.legal_name;
                registration.organization.identifier = org.identifier;
                registration.organization.country_code = org.country_code;

                CertificateType::Mdl(registration)
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
        }
    }
}

/// TODO: PVW-5870 Remove when IssuerRegistration are removed
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
        use crypto::x509::CertificateError;

        use crate::auth::issuer_auth::IssuerRegistration;

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
    }
}
