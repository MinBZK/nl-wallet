mod attribute;

use std::collections::HashSet;

use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::Utc;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::AttributesError;
use attestation_data::auth::Organization;
use attestation_data::validity::ValidityWindow;
use error_category::ErrorCategory;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::DisplayMetadata;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use token_status_list::verification::verifier::RevocationStatus;
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
    pub issuer: Box<Organization>,
    pub validity: AttestationValidity,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationValidity {
    pub revocation_status: Option<RevocationStatus>,
    pub validity_window: ValidityWindow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidityStatus {
    NotYetValid,
    Valid,
    Expired,
    ExpiresSoon,
}

impl ValidityStatus {
    pub fn from_window(window: &ValidityWindow, now: DateTime<Utc>) -> Self {
        const EXPIRES_SOON_THRESHOLD: Duration = Duration::days(7);

        // 1. Check if the start date is in the future
        if window.valid_from.is_some_and(|from| now < from) {
            return ValidityStatus::NotYetValid;
        }

        // 2. Check if the end date has passed
        if window.valid_until.is_some_and(|until| now > until) {
            return ValidityStatus::Expired;
        }

        // 3. Check if the end date is within the "soon" threshold
        if window
            .valid_until
            .is_some_and(|until| now > until - EXPIRES_SOON_THRESHOLD)
        {
            return ValidityStatus::ExpiresSoon;
        }

        // 4. Otherwise, it's currently valid
        ValidityStatus::Valid
    }
}

#[cfg(test)]
pub mod mock {
    use attestation_data::auth::Organization;
    use attestation_data::validity::ValidityWindow;
    use utils::vec_nonempty;

    use super::AttestationIdentity;
    use super::AttestationPresentation;
    use super::AttestationPresentationConfig;
    use super::AttestationValidity;
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
                validity: AttestationValidity {
                    revocation_status: None,
                    validity_window: ValidityWindow::new_valid_mock(),
                },
                attributes: vec![],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use chrono::Utc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::not_yet_valid(Some(1), Some(10), ValidityStatus::NotYetValid)]
    #[case::expired(Some(-10), Some(-1), ValidityStatus::Expired)]
    #[case::expires_soon(Some(-1), Some(5), ValidityStatus::ExpiresSoon)]
    #[case::valid_on_threshold(Some(-1), Some(7), ValidityStatus::Valid)]
    #[case::valid_outside_threshold(Some(-1), Some(10), ValidityStatus::Valid)]
    #[case::not_yet_valid_priority(Some(1), Some(2), ValidityStatus::NotYetValid)]
    #[case::open_ended_valid(Some(-1), None, ValidityStatus::Valid)]
    fn test_validity_status_logic(
        #[case] from_offset_days: Option<i64>,
        #[case] until_offset_days: Option<i64>,
        #[case] expected: ValidityStatus,
    ) {
        let now = Utc::now();
        let window = ValidityWindow {
            valid_from: from_offset_days.map(|d| now + Duration::days(d)),
            valid_until: until_offset_days.map(|d| now + Duration::days(d)),
        };

        assert_eq!(ValidityStatus::from_window(&window, now), expected);
    }
}
