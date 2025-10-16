mod attribute;

use std::collections::HashSet;

use chrono::NaiveDate;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::AttributesError;
use attestation_data::auth::Organization;
use error_category::ErrorCategory;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::DisplayMetadata;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::PidAttributesConfiguration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum AttestationError {
    #[error("some attributes not processed by claim: {0:?}")]
    #[category(pd)]
    AttributesNotProcessedByClaim(HashSet<Vec<String>>),

    #[error("unable to convert into attestation attribute value at {}: {}", .0.as_ref().join("."), .1)]
    #[category(pd)]
    AttributeError(VecNonEmpty<String>, #[source] AttributeError),

    #[error("error converting to attributes: {0}")]
    #[category(pd)]
    Attributes(#[from] AttributesError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum AttributeError {
    #[error("JSON schema type does not match value: {0:?} vs {1:?}")]
    #[category(pd)]
    AttributeConversion(AttributeValue, Option<JsonSchemaPropertyType>),

    #[error("unable to parse attribute value into date: {0:?}")]
    #[category(pd)]
    AttributeDateValue(#[from] chrono::ParseError),
}

pub trait AttestationPresentationConfig {
    fn filtered_attribute(&self, attestation_type: &str) -> Option<&[String]>;
}

impl AttestationPresentationConfig for PidAttributesConfiguration {
    fn filtered_attribute(&self, attribute: &str) -> Option<&[String]> {
        self.sd_jwt
            .get(attribute)
            .map(|pid_paths| pid_paths.recovery_code.as_ref())
    }
}

// TODO: Separate various concerns: PVW-4675
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationPresentation {
    pub identity: AttestationIdentity,
    pub attestation_type: String,
    pub display_metadata: VecNonEmpty<DisplayMetadata>,
    pub issuer: Organization,
    pub attributes: Vec<AttestationAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttestationIdentity {
    Ephemeral,
    Fixed { id: Uuid },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationAttribute {
    pub key: VecNonEmpty<String>,
    pub metadata: Vec<ClaimDisplayMetadata>,
    pub value: AttestationAttributeValue,
    pub svg_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum AttestationAttributeValue {
    Basic(AttributeValue),
    Date(NaiveDate),
}

#[cfg(test)]
pub mod mock {
    use attestation_data::auth::Organization;
    use utils::vec_nonempty;

    use super::AttestationIdentity;
    use super::AttestationPresentation;
    use super::AttestationPresentationConfig;
    use super::DisplayMetadata;

    pub struct EmptyPresentationConfig;

    impl AttestationPresentationConfig for EmptyPresentationConfig {
        fn filtered_attribute(&self, _attestation_type: &str) -> Option<&[String]> {
            None
        }
    }

    impl AttestationPresentation {
        /// Create a nearly empty [`AttestationPresentation`] for tests that absolutely need this type.
        pub fn new_mock() -> Self {
            Self {
                identity: AttestationIdentity::Ephemeral,
                attestation_type: "mock".to_string(),
                display_metadata: vec_nonempty![DisplayMetadata {
                    lang: "nl".to_string(),
                    name: "mock".to_string(),
                    description: None,
                    summary: None,
                    rendering: None,
                }],
                issuer: Organization::new_mock(),
                attributes: vec![],
            }
        }
    }
}
