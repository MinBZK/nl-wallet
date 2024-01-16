use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::x509::MdocCertificateExtension;

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

impl MdocCertificateExtension for IssuerRegistration {
    const OID: &'static [u64] = OID_EXT_ISSUER_AUTH;
}

#[cfg(feature = "mock")]
pub use mock::*;

#[cfg(feature = "mock")]
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
