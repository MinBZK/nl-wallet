use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_valid::Validate;
use uuid::Uuid;

use attestation_types::qualification::AttestationQualification;
use http_utils::urls::HttpsUri;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use crate::attributes::Attributes;
use crate::attributes::AttributesError;
use crate::credential_payload::PreviewableCredentialPayload;

/// Generic data model used to pass the attributes to be issued from the issuer backend to the wallet server. This model
/// should be convertable into all documents that are actually issued to the wallet, i.e. mdoc and sd-jwt.
/// ```json
/// {
///     "id": "550e8400-e29b-41d4-a716-446655440000",
///     "attestation_type": "com.example.pid",
///     "attributes": {
///         "name": "John",
///         "lastname": "Doe"
///     }
/// }
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "mock", derive(derive_more::Into))]
pub struct IssuableDocument {
    attestation_type: String,
    #[validate(custom = IssuableDocument::validate_attributes)]
    attributes: Attributes,
    id: Uuid,
}

impl IssuableDocument {
    pub fn try_new(
        attestation_type: String,
        attributes: Attributes,
        id: Uuid,
    ) -> Result<Self, serde_valid::validation::Error> {
        Self::validate_attributes(&attributes)?;
        let document = Self {
            attestation_type,
            attributes,
            id,
        };
        Ok(document)
    }

    pub fn try_new_with_random_id(
        attestation_type: String,
        attributes: Attributes,
    ) -> Result<Self, serde_valid::validation::Error> {
        Self::try_new(attestation_type, attributes, Uuid::new_v4())
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
            attestation_type: self.attestation_type,
            issuer: issuer_uri,
            expires: Some(valid_until.into()),
            not_before: Some(valid_from.into()),
            attestation_qualification,
            attributes: self.attributes,
        };
        (self.id, payload)
    }

    pub fn attestation_type(&self) -> &str {
        &self.attestation_type
    }

    pub fn validate_with_metadata(&self, type_metadata: &NormalizedTypeMetadata) -> Result<(), AttributesError> {
        self.attributes.validate(type_metadata)
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    use indexmap::IndexMap;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;

    impl IssuableDocument {
        pub fn new_mock_degree(education: String) -> Self {
            IssuableDocument::try_new_with_random_id(
                "com.example.degree".to_string(),
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
    }
}
