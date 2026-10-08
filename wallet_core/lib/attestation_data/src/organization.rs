use crypto::x509::BorrowingCertificate;
use crypto::x509::DistinguishedNameError;
use derive_more::Debug;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use crate::registration_certificate::MultiLanguageString;
use crate::registration_certificate::MultiLanguageStringSet;
use crate::registration_certificate::ParsedRegistrationCertificate;
use crate::registration_certificate::StatusValidatedRegistrationCertificate;
use crate::registration_certificate::Subject;
use crate::registration_certificate::SubjectType;
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
    pub purpose: Option<MultiLanguageStringSet>,
    // TODO (PVW-6052): Require support_uri and person_type once proximity disclosure uses WRPRC organization data.
    pub support_uri: Option<String>,
    pub public_body: Option<bool>,
    pub person_type: Option<SubjectType>,
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
        let (legal_name, person_type) = match subject {
            Subject::LegalPerson { legal_name } => (legal_name.clone(), SubjectType::LegalPerson),
            Subject::NaturalPerson {
                given_name,
                family_name,
            } => (format!("{given_name} {family_name}"), SubjectType::NaturalPerson),
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
            purpose: payload.purpose.clone(),
            support_uri: Some(payload.support_uri.clone()),
            public_body: payload.public_body,
            person_type: Some(person_type),
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
                purpose: Some(utils::vec_nonempty![MultiLanguageString {
                    lang: "en".to_owned(),
                    value: "Verify your identity".to_owned(),
                }]),
                support_uri: None,
                public_body: None,
                person_type: Some(SubjectType::LegalPerson),
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
    use crate::registration_certificate::SubjectType;
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

    #[test]
    fn deserialize_organization_without_registration_fields() {
        let organization: Organization = serde_json::from_value(json!({
            "displayName": "Example",
            "legalName": "Example B.V.",
            "description": [],
            "identifier": "example",
            "countryCode": "NL",
        }))
        .unwrap();

        assert_eq!(organization.support_uri, None);
        assert_eq!(organization.public_body, None);
        assert_eq!(organization.person_type, None);
    }

    #[rstest]
    #[case::legal_person(
        DistinguishedName::create_legal_person_mock("Example"),
        "Example B.V.",
        SubjectType::LegalPerson
    )]
    #[case::natural_person(
        DistinguishedName::create_natural_person_mock("Jane", "Doe"),
        "Jane Doe",
        SubjectType::NaturalPerson
    )]
    #[tokio::test]
    async fn maps_registration_certificate_to_organization(
        #[case] subject: DistinguishedName,
        #[case] legal_name: &str,
        #[case] person_type: SubjectType,
        #[values(false, true)] has_optional_fields: bool,
    ) {
        let access_key = Ca::generate_wrpac_mock_ca()
            .unwrap()
            .generate_key_pair(subject, Default::default())
            .unwrap();
        let purpose = has_optional_fields.then(|| {
            utils::vec_nonempty![super::MultiLanguageString {
                lang: "en".to_owned(),
                value: "Verify your identity".to_owned(),
            }]
        });
        let mut payload = issuer_registration_certificate_payload(access_key.certificate(), []);
        payload.0["purpose"] = json!(purpose);
        payload.0["name"] = json!(has_optional_fields.then_some("Issuer service"));
        payload.0["info_uri"] = json!(has_optional_fields.then_some("https://example.com/info"));
        payload.0["privacy_policy"] = json!(has_optional_fields.then_some("https://example.com/privacy"));
        payload.0["support_uri"] = json!("https://example.com/support");
        payload.0["public_body"] = json!(has_optional_fields.then_some(true));
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
                purpose,
                support_uri: Some("https://example.com/support".to_owned()),
                public_body: has_optional_fields.then_some(true),
                person_type: Some(person_type),
            }
        );
    }
}
