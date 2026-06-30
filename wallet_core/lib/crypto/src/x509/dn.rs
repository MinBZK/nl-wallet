use std::convert::TryFrom;

use derive_more::Constructor;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use itertools::Itertools;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::oid;
use x509_parser::error::X509Error;
use x509_parser::x509::AttributeTypeAndValue;
use x509_parser::x509::X509Name;

pub const DN_TYPE_ORGANIZATION_IDENTIFIER_OID: &Oid = &oid!(1.3.6.1.1.15);
pub const DN_TYPE_SERIAL_NUMBER_OID: &Oid = &oid!(2.5.4.5);
pub const DN_TYPE_SURNAME_OID: &Oid = &oid!(2.5.4.4);
pub const DN_TYPE_GIVEN_NAME_OID: &Oid = &oid!(2.5.4.42);

/// Distinguished name of X509 certificate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistinguishedName {
    pub common_name: String,
    pub country_name: String,
    pub organization_name: Option<String>,
    pub organization_identifier: Option<String>,
    pub serial_number: Option<String>,
    pub surname: Option<String>,
    pub given_name: Option<String>,
}

impl DistinguishedName {
    fn parse_optional_attribute_type_and_value<'a>(
        name: &'static str,
        attributes: impl Iterator<Item = &'a AttributeTypeAndValue<'a>>,
    ) -> Result<Option<String>, DistinguishedNameError> {
        let attribute_values = attributes
            .map(|attr| attr.as_str().map_err(DistinguishedNameError::X509Error))
            .collect::<Result<Vec<_>, _>>()?;

        match attribute_values.as_slice() {
            [value] => Ok(Some((*value).to_string())),
            [] => Ok(None),
            _ => Err(DistinguishedNameError::MultipleX509Names {
                name,
                values: attribute_values.into_iter().map(ToString::to_string).collect(),
            })?,
        }
    }

    fn parse_required_attribute_type_and_value<'a>(
        name: &'static str,
        attributes: impl Iterator<Item = &'a AttributeTypeAndValue<'a>>,
    ) -> Result<String, DistinguishedNameError> {
        match Self::parse_optional_attribute_type_and_value(name, attributes)? {
            Some(value) => Ok(value),
            None => Err(DistinguishedNameError::MissingX509Name { name }),
        }
    }
}

#[cfg(any(test, feature = "generate", feature = "mock"))]
impl DistinguishedName {
    pub fn new(common_name: String, country_name: String) -> Self {
        Self {
            common_name,
            country_name,
            organization_name: None,
            organization_identifier: None,
            serial_number: None,
            surname: None,
            given_name: None,
        }
    }

    pub fn new_legal_person(
        common_name: String,
        country_name: String,
        organization_name: String,
        organization_identifier: String,
    ) -> Self {
        Self {
            common_name,
            country_name,
            organization_name: Some(organization_name),
            organization_identifier: Some(organization_identifier),
            serial_number: None,
            surname: None,
            given_name: None,
        }
    }

    pub fn new_natural_person(
        common_name: String,
        country_name: String,
        serial_number: String,
        surname: String,
        given_name: String,
    ) -> Self {
        Self {
            common_name,
            country_name,
            organization_name: None,
            organization_identifier: None,
            serial_number: Some(serial_number),
            surname: Some(surname),
            given_name: Some(given_name),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DistinguishedNameError {
    #[error("X509 parsing error: {0}")]
    X509Error(#[source] X509Error),

    #[error("no `{name}` attributes")]
    MissingX509Name { name: &'static str },

    #[error("multiple {name} attributes: {}", values.iter().join(","))]
    MultipleX509Names { name: &'static str, values: Vec<String> },
}

impl TryFrom<&X509Name<'_>> for DistinguishedName {
    type Error = DistinguishedNameError;

    fn try_from(value: &X509Name<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            common_name: Self::parse_required_attribute_type_and_value("CN", value.iter_common_name())?,
            country_name: Self::parse_required_attribute_type_and_value("C", value.iter_country())?,
            organization_name: Self::parse_optional_attribute_type_and_value("O", value.iter_organization())?,
            organization_identifier: Self::parse_optional_attribute_type_and_value(
                "OID",
                value.iter_by_oid(DN_TYPE_ORGANIZATION_IDENTIFIER_OID),
            )?,
            serial_number: Self::parse_optional_attribute_type_and_value(
                "serialNumber",
                value.iter_by_oid(DN_TYPE_SERIAL_NUMBER_OID),
            )?,
            surname: Self::parse_optional_attribute_type_and_value("SN", value.iter_by_oid(DN_TYPE_SURNAME_OID))?,
            given_name: Self::parse_optional_attribute_type_and_value("GN", value.iter_by_oid(DN_TYPE_GIVEN_NAME_OID))?,
        })
    }
}

#[cfg(any(test, feature = "generate"))]
fn custom_dn_type(oid: &Oid) -> rcgen::DnType {
    rcgen::DnType::CustomDnType(
        oid.iter()
            // We only use known constants that do fit in u64, this is very theoretical
            .expect("oid sub identifier does not fit in u64")
            .collect(),
    )
}

#[cfg(any(test, feature = "generate"))]
impl From<DistinguishedName> for rcgen::DistinguishedName {
    fn from(dn: DistinguishedName) -> Self {
        let mut value = rcgen::DistinguishedName::new();
        value.push(rcgen::DnType::CommonName, dn.common_name);
        value.push(rcgen::DnType::CountryName, dn.country_name);
        if let Some(org_name) = dn.organization_name {
            value.push(rcgen::DnType::OrganizationName, org_name);
        }
        if let Some(oid) = dn.organization_identifier {
            value.push(custom_dn_type(DN_TYPE_ORGANIZATION_IDENTIFIER_OID), oid);
        }
        if let Some(serial_number) = dn.serial_number {
            value.push(custom_dn_type(DN_TYPE_SERIAL_NUMBER_OID), serial_number);
        }
        if let Some(surname) = dn.surname {
            value.push(custom_dn_type(DN_TYPE_SURNAME_OID), surname);
        }
        if let Some(given_name) = dn.given_name {
            value.push(custom_dn_type(DN_TYPE_GIVEN_NAME_OID), given_name);
        }
        value
    }
}

#[cfg(any(test, all(feature = "generate", feature = "mock")))]
impl DistinguishedName {
    pub fn create_mock(common_name: &str) -> Self {
        Self::new(common_name.to_string(), "NL".to_string())
    }

    pub fn create_legal_person_mock(common_name: &str) -> Self {
        let hash = crate::utils::sha256(common_name.as_bytes());
        let id = u64::from_be_bytes(hash[0..8].try_into().unwrap()) % 100_000_000;

        Self::new_legal_person(
            common_name.to_string(),
            "NL".to_string(),
            format!("{common_name} B.V."),
            format!("NTRNL-{id:08}"),
        )
    }

    pub fn create_natural_person_mock(given_name: &str, surname: &str) -> Self {
        let common_name = format!("{given_name} {surname}");
        let hash = crate::utils::sha256(common_name.as_bytes());
        let id = u64::from_be_bytes(hash[0..8].try_into().unwrap()) % 100_000_000;

        Self::new_natural_person(
            common_name,
            "NL".to_string(),
            format!("{id:08}"),
            surname.to_string(),
            given_name.to_string(),
        )
    }
}

/// A distinguished name encoded in a canonical, OID-registry-independent format.
/// This type is specifically designed for database persistence and comparison.
/// Format: "OID1=base64(DER1),OID2=base64(DER2),..."
#[derive(Debug, Clone, Eq, PartialEq, Hash, Display, Constructor, From, Into)]
#[cfg_attr(feature = "persistence", derive(sea_orm::DeriveValueType))]
pub struct CanonicalDistinguishedName(String);

impl CanonicalDistinguishedName {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for CanonicalDistinguishedName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use rstest::rstest;
    use x509_parser::asn1_rs::FromDer;
    use x509_parser::der_parser::Oid;
    use x509_parser::der_parser::oid;
    use x509_parser::x509::X509Name;

    use super::*;
    use crate::server_keys::generate::Ca;
    use crate::x509::BorrowingCertificate;
    use crate::x509::DistinguishedName;
    use crate::x509::DistinguishedNameError;
    use crate::x509::NO_SAN;

    const CN_OID: &Oid = &oid!(2.5.4.3);
    const COUNTRY_OID: &Oid = &oid!(2.5.4.6);
    const ORG_OID: &Oid = &oid!(2.5.4.10);
    const OID_OID: &Oid = DN_TYPE_ORGANIZATION_IDENTIFIER_OID;
    const SERIAL_NUMBER_OID: &Oid = DN_TYPE_SERIAL_NUMBER_OID;
    const SN_OID: &Oid = DN_TYPE_SURNAME_OID;
    const GN_OID: &Oid = DN_TYPE_GIVEN_NAME_OID;

    fn create_x509_name_der_from_tuples(oid_value_tuples: &[(&Oid, &str)]) -> Vec<u8> {
        let mut writer = Vec::with_capacity(128);
        writer.extend_from_slice(&[0x30, 0]);
        for (oid, value) in oid_value_tuples {
            let oid_bytes = oid.as_bytes();
            let value_bytes = value.as_bytes();
            writer.extend_from_slice(&[
                0x31,
                (oid_bytes.len() + value_bytes.len()) as u8 + 6,
                0x30,
                (oid_bytes.len() + value_bytes.len()) as u8 + 4,
            ]);
            writer.extend_from_slice(&[0x06, oid_bytes.len() as u8]);
            writer.extend_from_slice(oid_bytes);
            writer.extend_from_slice(&[0x0C, value_bytes.len() as u8]);
            writer.extend_from_slice(value_bytes);
        }
        assert!(writer.len() < 0x80, "arguments are too large in bytes");
        writer[1] = writer.len() as u8 - 2;
        writer
    }

    #[test]
    fn test_dn_parsing() {
        let mut oid_value_tuples = Vec::new();
        oid_value_tuples.extend([(CN_OID, "Test"), (COUNTRY_OID, "NL")]);
        let name_der = create_x509_name_der_from_tuples(&oid_value_tuples);

        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();
        let dn = DistinguishedName::try_from(&x509_name).unwrap();

        assert_eq!(dn, DistinguishedName::new("Test".to_string(), "NL".to_string()));
    }

    #[rstest]
    #[case(None, Some("NL"), "CN")]
    #[case(Some("test"), None, "C")]
    fn test_dn_parsing_error_missing_x509_name(
        #[case] common_name: Option<&str>,
        #[case] country_name: Option<&str>,
        #[case] expected_name: &str,
    ) {
        let mut oid_value_tuples = Vec::new();
        oid_value_tuples.extend(common_name.map(|a| (CN_OID, a)));
        oid_value_tuples.extend(country_name.map(|a| (COUNTRY_OID, a)));
        let name_der = create_x509_name_der_from_tuples(&oid_value_tuples);

        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();
        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::MissingX509Name{name} if expected_name == name);
    }

    #[rstest]
    #[case::common_name(CN_OID, "CN")]
    #[case::country(COUNTRY_OID, "C")]
    #[case::org(ORG_OID, "O")]
    #[case::oid(OID_OID, "OID")]
    #[case::serial_number(SERIAL_NUMBER_OID, "serialNumber")]
    #[case::surname(SN_OID, "SN")]
    #[case::given_name(GN_OID, "GN")]
    fn test_dn_parsing_error_multiple_x509_names(#[case] extra_oid: &Oid, #[case] expected_name: &str) {
        let mut oid_value_tuples = Vec::new();
        oid_value_tuples.extend([
            (CN_OID, "Tst"),
            (COUNTRY_OID, "NL"),
            (ORG_OID, "Tst Ltd"),
            (OID_OID, "NTRMA-1"),
            (SERIAL_NUMBER_OID, "1"),
            (SN_OID, "Doe"),
            (GN_OID, "John"),
        ]);
        oid_value_tuples.push((extra_oid, "EXTRA"));

        let name_der = create_x509_name_der_from_tuples(&oid_value_tuples);

        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();
        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::MultipleX509Names{name, values}
            if expected_name == name && values.get(1) == Some("EXTRA".to_string()).as_ref());
    }

    #[test]
    fn test_dn_from_ca() {
        let dn = DistinguishedName::create_mock("myca");
        let ca = Ca::generate(dn.clone(), Default::default()).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.certificate().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        assert_eq!(dn, certificate.to_distinguished_name().unwrap());
        assert_eq!(
            "2.5.4.3=DARteWNh,2.5.4.6=DAJOTA",
            certificate.to_canonical_distinguished_name().unwrap().as_ref()
        );

        let x509_cert = certificate.x509_certificate();
        for x509_name in [x509_cert.issuer(), x509_cert.subject()] {
            assert_x509_common_name(x509_name, &dn.common_name);
            assert_x509_country_name(x509_name, &dn.country_name);
            for oid in [ORG_OID, OID_OID, SERIAL_NUMBER_OID, SN_OID, GN_OID] {
                assert_x509_oid(x509_name, oid, None);
            }
        }
    }

    #[test]
    fn test_legal_person_dn_from_cert() {
        let ca_dn = DistinguishedName::create_mock("myca");
        let ca = Ca::generate(ca_dn.clone(), Default::default()).unwrap();
        let dn = DistinguishedName::create_legal_person_mock("mycert");
        let key_pair = ca.generate_key_pair(dn.clone(), Default::default(), NO_SAN).unwrap();
        let certificate = key_pair.certificate();

        assert_eq!(dn, certificate.to_distinguished_name().unwrap());
        assert_eq!(
            "2.5.4.3=DAZteWNlcnQ,2.5.4.6=DAJOTA,2.5.4.10=DAtteWNlcnQgQi5WLg,1.3.6.1.1.15=DA5OVFJOTC0xMjYwNjE3OA",
            certificate.to_canonical_distinguished_name().unwrap().as_ref()
        );

        let x509_cert = certificate.x509_certificate();

        assert_x509_common_name(x509_cert.issuer(), &ca_dn.common_name);
        assert_x509_country_name(x509_cert.issuer(), &ca_dn.country_name);

        assert_x509_common_name(x509_cert.subject(), &dn.common_name);
        assert_x509_country_name(x509_cert.subject(), &dn.country_name);
        assert_x509_oid(x509_cert.subject(), ORG_OID, dn.organization_name.as_ref());
        assert_x509_oid(x509_cert.subject(), OID_OID, dn.organization_identifier.as_ref());
        assert_x509_oid(x509_cert.subject(), SERIAL_NUMBER_OID, None);
        assert_x509_oid(x509_cert.subject(), SN_OID, None);
        assert_x509_oid(x509_cert.subject(), GN_OID, None);
    }

    #[test]
    fn test_natural_person_dn_from_cert() {
        let ca_dn = DistinguishedName::create_mock("myca");
        let ca = Ca::generate(ca_dn.clone(), Default::default()).unwrap();
        let dn = DistinguishedName::create_natural_person_mock("John", "Doe");
        let key_pair = ca.generate_key_pair(dn.clone(), Default::default(), NO_SAN).unwrap();
        let certificate = key_pair.certificate();

        assert_eq!(dn, certificate.to_distinguished_name().unwrap());
        assert_eq!(
            "2.5.4.3=DAhKb2huIERvZQ,2.5.4.6=DAJOTA,2.5.4.5=DAg5OTk4OTgwMg,2.5.4.4=DANEb2U,2.5.4.42=DARKb2hu",
            certificate.to_canonical_distinguished_name().unwrap().as_ref()
        );

        let x509_cert = certificate.x509_certificate();

        assert_x509_common_name(x509_cert.issuer(), &ca_dn.common_name);
        assert_x509_country_name(x509_cert.issuer(), &ca_dn.country_name);

        assert_x509_common_name(x509_cert.subject(), &dn.common_name);
        assert_x509_country_name(x509_cert.subject(), &dn.country_name);
        assert_x509_oid(x509_cert.subject(), ORG_OID, None);
        assert_x509_oid(x509_cert.subject(), OID_OID, None);
        assert_x509_oid(x509_cert.subject(), SERIAL_NUMBER_OID, dn.serial_number.as_ref());
        assert_x509_oid(x509_cert.subject(), SN_OID, dn.surname.as_ref());
        assert_x509_oid(x509_cert.subject(), GN_OID, dn.given_name.as_ref());
    }

    fn assert_x509_common_name(x509name: &X509Name, common_name: &str) {
        itertools::assert_equal(x509name.iter_common_name().map(|a| a.as_str().unwrap()), [common_name]);
    }

    fn assert_x509_country_name(x509name: &X509Name, country_name: &str) {
        itertools::assert_equal(x509name.iter_country().map(|a| a.as_str().unwrap()), [country_name]);
    }

    fn assert_x509_oid(x509name: &X509Name, oid: &Oid, value: Option<&String>) {
        itertools::assert_equal(x509name.iter_by_oid(oid).map(|a| a.as_str().unwrap()), value);
    }
}
