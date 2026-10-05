use crypto::x509::BorrowingCertificate;
use crypto::x509::DistinguishedNameError;
use derive_more::Debug;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use crate::registration_certificate::MultiLanguageString;
use crate::registration_certificate::ParsedRegistrationCertificate;
use crate::registration_certificate::StatusValidatedRegistrationCertificate;
use crate::registration_certificate::Subject;
use crate::registration_certificate::UncheckedRegistrationCertificate;
use crate::x509::RelyingParty;
use crate::x509::RelyingPartyError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDescription {
    pub translations: Vec<MultiLanguageString>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub display_name: String,
    pub legal_name: String,
    pub description: Vec<ServiceDescription>,
    pub web_url: Option<Url>,
    pub identifier: String,
    pub country_code: String,
    pub privacy_policy_url: Option<Url>,
}

#[derive(thiserror::Error, Debug)]
pub enum OrganizationError {
    #[error("distinguished name error: {0}")]
    DistinguishedName(#[source] DistinguishedNameError),

    #[error("relying party error: {0}")]
    RelyingParty(#[source] RelyingPartyError),
}

impl TryFrom<&BorrowingCertificate> for Organization {
    type Error = OrganizationError;

    fn try_from(certificate: &BorrowingCertificate) -> Result<Self, Self::Error> {
        let dn = certificate
            .to_distinguished_name()
            .map_err(OrganizationError::DistinguishedName)?;
        RelyingParty::try_from(dn)
            .map_err(OrganizationError::RelyingParty)
            .map(Self::from)
    }
}

impl From<&StatusValidatedRegistrationCertificate> for Organization {
    fn from(certificate: &StatusValidatedRegistrationCertificate) -> Self {
        Self::from_registration_certificate(certificate.payload(), certificate.subject())
    }
}

impl From<&ParsedRegistrationCertificate> for Organization {
    fn from(certificate: &ParsedRegistrationCertificate) -> Self {
        Self::from_registration_certificate(certificate.payload(), certificate.subject())
    }
}

impl Organization {
    fn from_registration_certificate(payload: &UncheckedRegistrationCertificate, subject: &Subject) -> Self {
        let legal_name = match subject {
            Subject::LegalPerson { legal_name } => legal_name.clone(),
            Subject::NaturalPerson {
                given_name,
                family_name,
            } => format!("{given_name} {family_name}"),
        };

        Self {
            display_name: payload.name.clone().unwrap_or_else(|| legal_name.clone()),
            legal_name,
            description: payload
                .srv_description
                .iter()
                .map(|description| ServiceDescription {
                    translations: description.iter().cloned().collect(),
                })
                .collect(),
            identifier: payload.sub.clone(),
            country_code: payload.country.clone(),
            web_url: payload.info_uri.clone(),
            privacy_policy_url: payload.privacy_policy.clone(),
        }
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use super::*;

    impl<'a, I: IntoIterator<Item = (&'a str, &'a str)>> From<I> for ServiceDescription {
        fn from(source: I) -> Self {
            Self {
                translations: source
                    .into_iter()
                    .map(|(lang, value)| MultiLanguageString {
                        lang: lang.to_owned(),
                        value: value.to_owned(),
                    })
                    .collect(),
            }
        }
    }

    impl Organization {
        pub fn new_mock() -> Self {
            Organization {
                display_name: "Mijn Organisatienaam".to_owned(),
                legal_name: "Organisatie".to_owned(),
                description: vec![
                    [
                        ("nl", "Beschrijving van Mijn Organisatie"),
                        ("en", "Description of My Organization"),
                    ]
                    .into(),
                ],
                identifier: "some-identifier".to_owned(),
                country_code: "NL".to_owned(),
                web_url: Some(Url::parse("https://organisation.example.com").unwrap()),
                privacy_policy_url: Some(Url::parse("https://organisation.example.com/privacy").unwrap()),
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use attestation_types::image::Image;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::DistinguishedName;
    use rstest::rstest;
    use serde_json::json;
    use token_status_list::verification::verifier::RevocationVerifier;
    use utils::generator::Generator;
    use utils::generator::TimeGenerator;

    use super::Organization;
    use crate::registration_certificate::RegistrationCertificateEnvelope;
    use crate::registration_certificate::mock::MockRegistrationCertificateAuthority;
    use crate::registration_certificate::mock::issuer_registration_certificate_payload;
    use crate::registration_certificate::verify_registration_certificate_envelope;
    use crate::x509::RelyingParty;

    #[rstest]
    #[case("image/svg+xml", "<svg></svg>", Image::Svg("<svg></svg>".to_owned()))]
    #[case("image/png", "yv4=", Image::Png(vec![0xca, 0xfe]))]
    #[case("image/jpeg", "q80=", Image::Jpeg(vec![0xab, 0xcd]))]
    fn image_deserialize(#[case] mime_type: &str, #[case] image_data: &str, #[case] expected: Image) {
        assert_eq!(
            serde_json::from_value::<Image>(json!({"mimeType": mime_type ,"imageData": image_data})).unwrap(),
            expected,
        )
    }

    #[rstest]
    #[case::legal_person(DistinguishedName::create_legal_person_mock("Example"), "Example B.V.")]
    #[case::natural_person(DistinguishedName::create_natural_person_mock("Jane", "Doe"), "Jane Doe")]
    #[tokio::test]
    async fn maps_registration_certificate_to_organization(
        #[case] subject: DistinguishedName,
        #[case] legal_name: &str,
        #[values(false, true)] has_optional_fields: bool,
    ) {
        let access_key = Ca::generate_wrpac_mock_ca()
            .unwrap()
            .generate_key_pair(subject, Default::default())
            .unwrap();
        let mut payload = issuer_registration_certificate_payload(access_key.certificate(), []);
        payload.0["name"] = json!(has_optional_fields.then_some("Issuer service"));
        payload.0["info_uri"] = json!(has_optional_fields.then_some("https://example.com/info"));
        payload.0["privacy_policy"] = json!(has_optional_fields.then_some("https://example.com/privacy"));
        payload.0["srv_description"] = json!([
            [
                { "lang": "en", "value": "First service" },
                { "lang": "nl", "value": "Eerste dienst" },
                { "lang": "en", "value": "Extra details" },
            ],
            [{ "lang": "en", "value": "Second service" }],
        ]);

        let authority = MockRegistrationCertificateAuthority::new();
        let envelope = RegistrationCertificateEnvelope::try_from(authority.sign_jwt(&payload).as_slice()).unwrap();
        let (payload, signing_subject) =
            verify_registration_certificate_envelope(&envelope, &authority.trust_anchors, &TimeGenerator)
                .unwrap()
                .into_parts();
        let relying_party = RelyingParty::try_from(access_key.certificate().to_distinguished_name().unwrap()).unwrap();
        let certificate = payload
            .validate_binding_and_time(&relying_party, TimeGenerator.generate())
            .unwrap()
            .validate_status(
                &RevocationVerifier::new_with_defaults(Arc::new(authority.status_list_client), TimeGenerator),
                &authority.trust_anchors,
                signing_subject,
                &TimeGenerator,
            )
            .await
            .unwrap();

        let organization = Organization::from(&certificate);
        assert_eq!(
            serde_json::from_value::<Organization>(serde_json::to_value(&organization).unwrap()).unwrap(),
            organization,
        );
        assert_eq!(
            organization,
            Organization {
                display_name: if has_optional_fields {
                    "Issuer service"
                } else {
                    legal_name
                }
                .to_owned(),
                legal_name: legal_name.to_owned(),
                description: vec![
                    [
                        ("en", "First service"),
                        ("nl", "Eerste dienst"),
                        ("en", "Extra details")
                    ]
                    .into(),
                    [("en", "Second service")].into(),
                ],
                identifier: certificate.payload().sub.clone(),
                country_code: "NL".to_owned(),
                web_url: has_optional_fields.then(|| "https://example.com/info".parse().unwrap()),
                privacy_policy_url: has_optional_fields.then(|| "https://example.com/privacy".parse().unwrap()),
            }
        );
    }
}
