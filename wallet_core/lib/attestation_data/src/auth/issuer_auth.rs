use crypto::x509::BorrowingCertificateExtension;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;
use x509_parser::der_parser::Oid;
use x509_parser::der_parser::oid;

use crate::auth::LocalizedStrings;
use crate::x509::CertificateType;

/// Legacy issuer certificate extension; remove in PVW-5870.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyIssuerRegistration {
    pub organization: Box<LegacyOrganization>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyOrganization {
    pub display_name: String,
    pub legal_name: String,
    pub description: LocalizedStrings,
    pub web_url: Option<Url>,
    #[serde(rename = "kvk")]
    pub identifier: String,
    pub country_code: String,
    pub privacy_policy_url: Option<Url>,
}

impl LegacyIssuerRegistration {
    #[cfg(any(test, feature = "mock"))]
    pub fn to_certificate_configuration(
        &self,
    ) -> Result<crypto::x509::CertificateConfiguration, crypto::x509::CertificateError> {
        let custom_ext = self.to_custom_ext()?;
        Ok(crypto::x509::CertificateConfiguration::with_usage_and_extension(
            crypto::x509::CertificateUsage::Mdl,
            custom_ext,
        ))
    }
}

impl BorrowingCertificateExtension for LegacyIssuerRegistration {
    /// oid: 2.1.123.2
    /// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
    /// suffix: 2, unofficial id for Issuer Authentication
    #[rustfmt::skip]
    const OID: Oid<'static> = oid!(2.1.123.2);
}

impl From<LegacyIssuerRegistration> for CertificateType {
    fn from(source: LegacyIssuerRegistration) -> Self {
        CertificateType::Mdl(source)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    impl LegacyIssuerRegistration {
        pub fn new_mock() -> Self {
            LegacyIssuerRegistration {
                organization: Box::new(LegacyOrganization {
                    display_name: "Cert issuer".to_string(),
                    legal_name: "Cert issuer B.V.".to_string(),
                    description: [
                        ("nl", "Beschrijving van Mijn Organisatie"),
                        ("en", "Description of My Organization"),
                    ]
                    .into(),
                    identifier: "NTRNL-50198052".to_string(),
                    country_code: "NL".to_string(),
                    web_url: Some("https://organisation.example.com".parse().unwrap()),
                    privacy_policy_url: Some("https://example.com/privacy".parse().unwrap()),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::LegacyIssuerRegistration;

    #[test]
    fn preserve_legacy_organization_json() {
        let json = json!({
            "organization": {
                "displayName": "Issuer",
                "legalName": "Issuer B.V.",
                "description": { "en": "Service", "nl": "Dienst" },
                "kvk": "NTRNL-12345678",
                "countryCode": "NL",
            }
        });
        let registration: LegacyIssuerRegistration = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(registration).unwrap(), json);
    }
}
