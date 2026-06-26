use std::convert::TryFrom;

use derive_more::Constructor;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use itertools::Itertools;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::oid;
use x509_parser::error::X509Error;
use x509_parser::x509::X509Name;

pub const DN_TYPE_ORGANIZATION_IDENTIFIER_OID: &Oid = &oid!(1.3.6.1.1.15);

/// Distinguished name of X509 certificates following ETSI EN 319 412-2 and ETSI EN 319 412-3 standard.
///
/// ETSI EN 319 412-2 and ETSI EN 319 412-3 specify that `common_name`, `country_name` are mandatory.
/// For legal persons, additionally `organization_name` and `organization_identifier` are mandatory.
/// For natural persons that is only optional. TODO: PVW-6025
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistinguishedName {
    pub common_name: String,
    pub country_name: String,
    pub organization_name: String,
    pub organization_identifier: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DistinguishedNameError {
    #[error("X509 parsing error: {0}")]
    X509Error(#[source] X509Error),

    #[error("no common name")]
    NoCommonName(),

    #[error("multiple common names: {}", _0.iter().join(","))]
    MultipleCommonNames(Vec<String>),

    #[error("no country name")]
    NoCountryName(),

    #[error("multiple country names: {}", _0.iter().join(","))]
    MultipleCountryNames(Vec<String>),

    #[error("no organization name")]
    NoOrganizationName(),

    #[error("multiple organization names: {}", _0.iter().join(","))]
    MultipleOrganizationNames(Vec<String>),

    #[error("no organization identifier")]
    NoOrganizationIdentifier(),

    #[error("multiple organization identifiers: {}", _0.iter().join(","))]
    MultipleOrganizationIdentifiers(Vec<String>),
}

impl TryFrom<&X509Name<'_>> for DistinguishedName {
    type Error = DistinguishedNameError;

    fn try_from(value: &X509Name<'_>) -> Result<Self, Self::Error> {
        let common_names = value
            .iter_common_name()
            .map(|attr| attr.as_str().map_err(DistinguishedNameError::X509Error))
            .collect::<Result<Vec<_>, _>>()?;
        let common_name = match common_names.as_slice() {
            [name] => name.to_string(),
            [] => Err(DistinguishedNameError::NoCommonName())?,
            _ => Err(DistinguishedNameError::MultipleCommonNames(
                common_names.into_iter().map(ToString::to_string).collect(),
            ))?,
        };

        let country_names = value
            .iter_country()
            .map(|attr| attr.as_str().map_err(DistinguishedNameError::X509Error))
            .collect::<Result<Vec<_>, _>>()?;
        let country_name = match country_names.as_slice() {
            [name] => name.to_string(),
            [] => Err(DistinguishedNameError::NoCountryName())?,
            _ => Err(DistinguishedNameError::MultipleCountryNames(
                country_names.into_iter().map(ToString::to_string).collect(),
            ))?,
        };

        let organization_names = value
            .iter_organization()
            .map(|attr| attr.as_str().map_err(DistinguishedNameError::X509Error))
            .collect::<Result<Vec<_>, _>>()?;
        let organization_name = match organization_names.as_slice() {
            [name] => name.to_string(),
            [] => Err(DistinguishedNameError::NoOrganizationName())?,
            _ => Err(DistinguishedNameError::MultipleOrganizationNames(
                organization_names.into_iter().map(ToString::to_string).collect(),
            ))?,
        };

        let organization_identifiers = value
            .iter_by_oid(DN_TYPE_ORGANIZATION_IDENTIFIER_OID)
            .map(|attr| attr.as_str().map_err(DistinguishedNameError::X509Error))
            .collect::<Result<Vec<_>, _>>()?;
        let organization_identifier = match organization_identifiers.as_slice() {
            [name] => name.to_string(),
            [] => Err(DistinguishedNameError::NoOrganizationIdentifier())?,
            _ => Err(DistinguishedNameError::MultipleOrganizationIdentifiers(
                organization_identifiers.into_iter().map(ToString::to_string).collect(),
            ))?,
        };

        Ok(Self {
            common_name,
            country_name,
            organization_name,
            organization_identifier,
        })
    }
}

#[cfg(any(test, feature = "generate"))]
impl From<DistinguishedName> for rcgen::DistinguishedName {
    fn from(dn: DistinguishedName) -> Self {
        let dn_type_oid = rcgen::DnType::CustomDnType(
            DN_TYPE_ORGANIZATION_IDENTIFIER_OID
                .iter()
                .expect("oid does not fit in u64")
                .collect(),
        );
        let mut value = rcgen::DistinguishedName::new();
        value.push(rcgen::DnType::CommonName, dn.common_name);
        value.push(rcgen::DnType::CountryName, dn.country_name);
        value.push(rcgen::DnType::OrganizationName, dn.organization_name);
        value.push(dn_type_oid, dn.organization_identifier);
        value
    }
}

#[cfg(any(test, all(feature = "generate", feature = "mock")))]
impl DistinguishedName {
    pub fn create_mock(common_name: &str) -> Self {
        let hash = crate::utils::sha256(common_name.as_bytes());
        let id = u64::from_be_bytes(hash[0..8].try_into().unwrap()) % 100_000_000;

        Self {
            common_name: common_name.to_string(),
            country_name: "NL".to_string(),
            organization_name: format!("{common_name} B.V."),
            organization_identifier: format!("NTRNL-{id:08}"),
        }
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
    const C_OID: &Oid = &oid!(2.5.4.6);
    const O_OID: &Oid = &oid!(2.5.4.10);
    const OID_OID: &Oid = DN_TYPE_ORGANIZATION_IDENTIFIER_OID;

    fn create_x509_name_der_from_tuples(oid_value_tuples: &[(&Oid, &str)]) -> Vec<u8> {
        let mut writer = Vec::with_capacity(256);
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
        writer[1] = writer.len() as u8 - 2;
        writer
    }

    fn create_x509_name_der(
        common_names: &[&str],
        country_names: &[&str],
        organization_names: &[&str],
        organization_ids: &[&str],
    ) -> Vec<u8> {
        let mut oid_value_tuples = Vec::new();
        oid_value_tuples.extend(common_names.iter().map(|a| (CN_OID, *a)));
        oid_value_tuples.extend(country_names.iter().map(|a| (C_OID, *a)));
        oid_value_tuples.extend(organization_names.iter().map(|a| (O_OID, *a)));
        oid_value_tuples.extend(organization_ids.iter().map(|a| (OID_OID, *a)));
        create_x509_name_der_from_tuples(&oid_value_tuples)
    }

    #[test]
    fn test_dn_parsing() {
        let name_der = create_x509_name_der_from_tuples(&[
            (CN_OID, "test"),
            (C_OID, "NL"),
            (O_OID, "ICTU"),
            (OID_OID, "NTRNL-27381312"),
        ]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();
        let dn = DistinguishedName::try_from(&x509_name).unwrap();

        assert_eq!(dn.common_name, "test");
        assert_eq!(dn.country_name, "NL");
        assert_eq!(dn.organization_name, "ICTU");
        assert_eq!(dn.organization_identifier, "NTRNL-27381312");
    }

    #[test]
    fn test_dn_parsing_error_no_common_name() {
        let name_der = create_x509_name_der(&[], &["NL"], &["ICTU"], &["NTRNL-27381312"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::NoCommonName());
    }

    #[test]
    fn test_dn_parsing_error_multiple_common_names() {
        let name_der = create_x509_name_der(&["a", "b"], &["NL"], &["ICTU"], &["NTRNL-27381312"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::MultipleCommonNames(names) if names == ["a", "b"]);
    }

    #[test]
    fn test_dn_parsing_error_no_country_name() {
        let name_der = create_x509_name_der(&["test"], &[], &["ICTU"], &["NTRNL-27381312"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::NoCountryName());
    }

    #[test]
    fn test_dn_parsing_error_multiple_country_names() {
        let name_der = create_x509_name_der(&["test"], &["NL", "DE"], &["ICTU"], &["NTRNL-27381312"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::MultipleCountryNames(names) if names == ["NL", "DE"]);
    }

    #[test]
    fn test_dn_parsing_error_no_organization_name() {
        let name_der = create_x509_name_der(&["test"], &["NL"], &[], &["NTRNL-27381312"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::NoOrganizationName());
    }

    #[test]
    fn test_dn_parsing_error_multiple_organization_names() {
        let name_der = create_x509_name_der(&["test"], &["NL"], &["ICTU A", "ICTU B"], &["NTRNL-27381312"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::MultipleOrganizationNames(names) if names == ["ICTU A", "ICTU B"]);
    }

    #[test]
    fn test_dn_parsing_error_no_organization_identifier() {
        let name_der = create_x509_name_der(&["test"], &["NL"], &["ICTU"], &[]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::NoOrganizationIdentifier());
    }

    #[test]
    fn test_dn_parsing_error_multiple_organization_identifiers() {
        let name_der = create_x509_name_der(&["test"], &["NL"], &["ICTU"], &["B01", "B02"]);
        let (_, x509_name) = X509Name::from_der(&name_der).unwrap();

        let err = DistinguishedName::try_from(&x509_name).expect_err("expected error");
        assert_matches!(err, DistinguishedNameError::MultipleOrganizationIdentifiers(ids) if ids == ["B01", "B02"]);
    }

    #[test]
    fn test_dn_from_ca() {
        let dn = DistinguishedName::create_mock("myca");
        let ca = Ca::generate(dn.clone(), Default::default()).unwrap();
        let certificate = BorrowingCertificate::from_certificate_der(ca.certificate().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        assert_eq!(dn, certificate.to_distinguished_name().unwrap());
        assert_eq!(
            "2.5.4.3=DARteWNh,2.5.4.6=DAJOTA,2.5.4.10=DAlteWNhIEIuVi4,1.3.6.1.1.15=DA5OVFJOTC0xMTQzMDQxNA",
            certificate.to_canonical_distinguished_name().unwrap().as_ref()
        );

        let x509_cert = certificate.x509_certificate();
        for x509_name in [x509_cert.issuer(), x509_cert.subject()] {
            assert_x509_common_name(x509_name, &dn.common_name);
            assert_x509_country_name(x509_name, &dn.country_name);
            assert_x509_organization_name(x509_name, &dn.organization_name);
            assert_x509_organization_identifier(x509_name, &dn.organization_identifier);
        }
    }

    #[test]
    fn test_dn_from_cert() {
        let ca_dn = DistinguishedName::create_mock("myca");
        let ca = Ca::generate(ca_dn.clone(), Default::default()).unwrap();
        let dn = DistinguishedName::create_mock("mycert");
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
        assert_x509_organization_name(x509_cert.issuer(), &ca_dn.organization_name);
        assert_x509_organization_identifier(x509_cert.issuer(), &ca_dn.organization_identifier);

        assert_x509_common_name(x509_cert.subject(), &dn.common_name);
        assert_x509_country_name(x509_cert.subject(), &dn.country_name);
        assert_x509_organization_name(x509_cert.subject(), &dn.organization_name);
        assert_x509_organization_identifier(x509_cert.subject(), &dn.organization_identifier);
    }

    fn assert_x509_common_name(x509name: &X509Name, common_name: &str) {
        itertools::assert_equal(x509name.iter_common_name().map(|a| a.as_str().unwrap()), [common_name]);
    }

    fn assert_x509_country_name(x509name: &X509Name, country_name: &str) {
        itertools::assert_equal(x509name.iter_country().map(|a| a.as_str().unwrap()), [country_name]);
    }

    fn assert_x509_organization_name(x509name: &X509Name, organization_name: &str) {
        itertools::assert_equal(
            x509name.iter_organization().map(|a| a.as_str().unwrap()),
            [organization_name],
        );
    }

    fn assert_x509_organization_identifier(x509name: &X509Name, organization_identifier: &str) {
        itertools::assert_equal(
            x509name
                .iter_by_oid(DN_TYPE_ORGANIZATION_IDENTIFIER_OID)
                .map(|a| a.as_str().unwrap()),
            [organization_identifier],
        );
    }
}
