use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_valid::Validate;

use attestation_types::qualification::AttestationQualification;
use http_utils::urls::HttpsUri;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

use crate::attributes::Attributes;
use crate::attributes::AttributesError;
use crate::credential_payload::PreviewableCredentialPayload;

/// Generic data model used to pass the attributes to be issued from the issuer backend to the wallet server. This model
/// should be convertable into all documents that are actually issued to the wallet, i.e. mdoc and sd-jwt.
/// ```json
/// {
///     "attestation_type": "com.example.pid",
///     "attributes": {
///         "name": "John",
///         "lastname": "Doe"
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[validate(custom = IssuableDocument::validate)]
pub struct IssuableDocument {
    attestation_type: String,
    attributes: Attributes,
}

impl IssuableDocument {
    pub fn try_new(attestation_type: String, attributes: Attributes) -> Result<Self, serde_valid::validation::Error> {
        let document = Self {
            attestation_type,
            attributes,
        };
        document.validate()?;
        Ok(document)
    }

    fn validate(&self) -> Result<(), serde_valid::validation::Error> {
        self.attributes
            .as_ref()
            .len()
            .ge(&1)
            .then_some(())
            .ok_or_else(|| serde_valid::validation::Error::Custom("must contain at least one attribute".to_string()))?;

        Ok(())
    }

    pub fn into_previewable_credential_payload(
        self,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
        issuer_uri: HttpsUri,
        attestation_qualification: AttestationQualification,
    ) -> PreviewableCredentialPayload {
        PreviewableCredentialPayload {
            attestation_type: self.attestation_type,
            issuer: issuer_uri,
            expires: Some(valid_until.into()),
            not_before: Some(valid_from.into()),
            attestation_qualification,
            attributes: self.attributes,
        }
    }

    pub fn attestation_type(&self) -> &str {
        &self.attestation_type
    }

    pub fn validate_with_metadata(&self, type_metadata: &NormalizedTypeMetadata) -> Result<(), AttributesError> {
        self.attributes.validate(type_metadata)
    }
}

pub type IssuableDocuments = VecNonEmpty<IssuableDocument>;

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    use indexmap::IndexMap;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;

    impl IssuableDocument {
        pub fn new_mock_degree(education: String) -> Self {
            IssuableDocument::try_new(
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
