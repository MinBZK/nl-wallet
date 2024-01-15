use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use x509_parser::der_parser::Oid;

use crate::utils::x509::{Certificate, CertificateError};

use super::Organization;

/// oid: 2.1.123.2
/// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
/// suffix: 2, unofficial id for Issuer Authentication
const OID_EXT_ISSUER_AUTH: &[u64] = &[2, 1, 123, 2];

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuerRegistration {
    pub organization: Organization,
}

impl IssuerRegistration {
    pub fn from_certificate(source: &Certificate) -> Result<Option<Self>, CertificateError> {
        // unwrap() is safe here, because we process a fixed value
        let oid = Oid::from(OID_EXT_ISSUER_AUTH).unwrap();
        source.extract_custom_ext(oid)
    }
}

#[cfg(any(test, feature = "generate"))]
mod generate {
    use p256::pkcs8::der::{asn1::Utf8StringRef, Encode};
    use rcgen::CustomExtension;

    use crate::utils::x509::CertificateError;

    use super::{IssuerRegistration, OID_EXT_ISSUER_AUTH};

    impl IssuerRegistration {
        pub fn to_custom_ext(&self) -> Result<CustomExtension, CertificateError> {
            let json_string = serde_json::to_string(self)?;
            let string = Utf8StringRef::new(&json_string)?;
            let ext = CustomExtension::from_oid_content(OID_EXT_ISSUER_AUTH, string.to_der()?);
            Ok(ext)
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub use mock::*;

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use url::Url;

    use super::*;

    pub fn issuer_registration_mock() -> IssuerRegistration {
        let my_organization = Organization {
            display_name: vec![("nl", "Mijn Uitgever"), ("en", "My Issuer")].into(),
            legal_name: vec![("nl", "Uitgever"), ("en", "Issuer")].into(),
            description: vec![
                ("nl", "Beschrijving van Mijn Uitgever"),
                ("en", "Description of My Issuer"),
            ]
            .into(),
            category: vec![("nl", "Categorie"), ("en", "Category")].into(),
            kvk: Some("some-kvk".to_owned()),
            city: Some(vec![("nl", "Den Haag"), ("en", "The Hague")].into()),
            department: Some(vec![("nl", "Afdeling"), ("en", "Department")].into()),
            country_code: Some("nl".to_owned()),
            web_url: Some(Url::parse("https://www.ons-dorp.nl").unwrap()),
            privacy_policy_url: Some(Url::parse("https://www.ons-dorp.nl/privacy").unwrap()),
            logo: None,
        };
        IssuerRegistration {
            organization: my_organization,
        }
    }
}
