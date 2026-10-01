use crypto::x509::DistinguishedName;
use derive_more::Debug;

use crate::auth::Organization;

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
                web_url: None,
                identifier: organization_identifier,
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
                web_url: None,
                identifier: serial_number,
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
