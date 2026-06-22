use attestation_data::attributes::Attributes;
use attestation_data::attributes::AttributesError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_types::credential_format::Format;
use attestation_types::qualification::AttestationQualification;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use derive_more::Display;
use http_utils::urls::HttpsUri;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use serde::Deserialize;
use serde::Serialize;
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Constructor, Serialize, Deserialize)]
#[display("({format}): {attestation_type}")]
pub struct CredentialKind {
    pub format: Format,
    pub attestation_type: String,
}

/// Generic data model used to pass the attributes to be issued from the issuer backend to the wallet server. This model
/// should be convertable into all documents that are actually issued to the wallet, i.e. mdoc and sd-jwt.
/// ```json
/// {
///     "id": "550e8400-e29b-41d4-a716-446655440000",
///     "format": "dc+sd-jwt",
///     "attestation_type": "com.example.pid",
///     "attributes": {
///         "name": "John",
///         "lastname": "Doe"
///     }
/// }
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "mock", derive(derive_more::Into))]
pub struct IssuableDocument {
    pub id: Uuid,
    #[serde(flatten)]
    pub credential_kind: CredentialKind,
    #[validate(custom = IssuableDocument::validate_attributes)]
    attributes: Attributes,
}

impl IssuableDocument {
    pub fn try_new(
        id: Uuid,
        credential_kind: CredentialKind,
        attributes: Attributes,
    ) -> Result<Self, serde_valid::validation::Error> {
        Self::validate_attributes(&attributes)?;
        let document = Self {
            id,
            credential_kind,
            attributes,
        };
        Ok(document)
    }

    pub fn try_new_with_random_id(
        credential_kind: CredentialKind,
        attributes: Attributes,
    ) -> Result<Self, serde_valid::validation::Error> {
        Self::try_new(Uuid::new_v4(), credential_kind, attributes)
    }

    pub fn validate_attributes(attributes: &Attributes) -> Result<(), serde_valid::validation::Error> {
        attributes
            .as_ref()
            .len()
            .ge(&1)
            .then_some(())
            .ok_or_else(|| serde_valid::validation::Error::Custom("must contain at least one attribute".to_string()))?;

        Ok(())
    }

    pub fn into_id_and_previewable_credential_payload(
        self,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
        issuer_uri: HttpsUri,
        attestation_qualification: AttestationQualification,
    ) -> (Uuid, PreviewableCredentialPayload) {
        let payload = PreviewableCredentialPayload {
            attestation_type: self.credential_kind.attestation_type,
            issuer: issuer_uri,
            expires: Some(valid_until.into()),
            not_before: Some(valid_from.into()),
            attestation_qualification,
            attributes: self.attributes,
        };
        (self.id, payload)
    }

    pub fn validate_with_metadata(&self, type_metadata: &NormalizedTypeMetadata) -> Result<(), AttributesError> {
        self.attributes.validate(type_metadata)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use indexmap::IndexMap;

    use super::*;

    impl IssuableDocument {
        pub fn new_mock_degree(education: String) -> Self {
            IssuableDocument::try_new_with_random_id(
                CredentialKind::new(Format::SdJwt, "com.example.degree".to_string()),
                IndexMap::from([
                    (
                        "university".to_string(),
                        Attribute::Single(AttributeValue::Text("Example university".to_string())),
                    ),
                    (
                        "education".to_string(),
                        Attribute::Single(AttributeValue::Text(education)),
                    ),
                    (
                        "graduation_date".to_string(),
                        Attribute::Single(AttributeValue::Text("1970-01-01".to_string())),
                    ),
                    (
                        "grade".to_string(),
                        Attribute::Single(AttributeValue::Text("A".to_string())),
                    ),
                    ("cum_laude".to_string(), Attribute::Single(AttributeValue::Bool(true))),
                ])
                .into(),
            )
            .unwrap()
        }

        pub fn new_mock_museum_maandkaart() -> Self {
            IssuableDocument::try_new_with_random_id(
                CredentialKind::new(Format::MsoMdoc, "com.example.museum_maandkaart".to_string()),
                IndexMap::from([
                    (
                        "name".to_string(),
                        Attribute::Single(AttributeValue::Text("Jan de Vries".to_string())),
                    ),
                    (
                        "member_number".to_string(),
                        Attribute::Single(AttributeValue::Text("1234567890".to_string())),
                    ),
                    (
                        "valid_year".to_string(),
                        Attribute::Single(AttributeValue::Text("2026".to_string())),
                    ),
                ])
                .into(),
            )
            .unwrap()
        }
    }
}
