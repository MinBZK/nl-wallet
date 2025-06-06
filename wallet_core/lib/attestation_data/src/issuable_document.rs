use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_valid::Validate;

use http_utils::urls::HttpsUri;
use utils::vec_at_least::VecNonEmpty;

use crate::attributes::Attribute;
use crate::credential_payload::PreviewableCredentialPayload;
use crate::qualification::AttestationQualification;

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
    attributes: IndexMap<String, Attribute>,
}

impl IssuableDocument {
    pub fn try_new(
        attestation_type: String,
        attributes: IndexMap<String, Attribute>,
    ) -> Result<Self, serde_valid::validation::Error> {
        let document = Self {
            attestation_type,
            attributes,
        };
        document.validate()?;
        Ok(document)
    }

    fn validate(&self) -> Result<(), serde_valid::validation::Error> {
        self.attributes
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
}

pub type IssuableDocuments = VecNonEmpty<IssuableDocument>;

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    use crate::attributes::AttributeValue;

    impl IssuableDocument {
        pub fn new_mock() -> Self {
            IssuableDocument::try_new(
                "com.example.degree".to_string(),
                IndexMap::from([
                    (
                        "university".to_string(),
                        Attribute::Single(AttributeValue::Text("Example university".to_string())),
                    ),
                    (
                        "education".to_string(),
                        Attribute::Single(AttributeValue::Text("Example education".to_string())),
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
                ]),
            )
            .unwrap()
        }
    }
}
