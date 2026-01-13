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
    NotYetValid {
        valid_from: DateTime<Utc>,
    },
    AlwaysValid,
    ValidUntil {
        expires_at: DateTime<Utc>,
        notify_at: DateTime<Utc>,
    },
    ExpiresSoon {
        expires_at: DateTime<Utc>,
        notify_at: DateTime<Utc>,
    },
    Expired {
        expired_at: DateTime<Utc>,
    },
}

const EXPIRES_SOON_WINDOW: Duration = Duration::days(7);

impl ValidityStatus {
    pub fn from_window(window: &ValidityWindow, now: DateTime<Utc>) -> Self {
        // 1. Check if the start date is in the future
        if let Some(from) = window.valid_from
            && now < from
        {
            return ValidityStatus::NotYetValid { valid_from: from };
        }

        // 2. Check if the end date has passed
        if let Some(until) = window.valid_until
            && now > until
        {
            return ValidityStatus::Expired { expired_at: until };
        }

        // 3. Check if we are within the "soon" threshold
        if let Some(until) = window.valid_until {
            let notify_at = until - EXPIRES_SOON_WINDOW;
            if now > notify_at {
                return ValidityStatus::ExpiresSoon {
                    expires_at: until,
                    notify_at,
                };
            }
            return ValidityStatus::ValidUntil {
                expires_at: until,
                notify_at,
            };
        }

        // 4. Fallback for open-ended valid
        ValidityStatus::AlwaysValid
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
    use chrono::DurationRound;
    use chrono::TimeDelta;
    use chrono::Utc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::not_yet_valid(
        Some(Duration::days(1)),
        Some(Duration::days(10)),
        ValidityStatus::NotYetValid { valid_from: Utc::now() + Duration::days(1) }
    )]
    #[case::expired(
        Some(Duration::days(-10)),
        Some(Duration::days(-1)),
        ValidityStatus::Expired { expired_at: Utc::now() - Duration::days(1) }
    )]
    #[case::expires_soon(
        Some(Duration::days(-1)),
        Some(EXPIRES_SOON_WINDOW - Duration::hours(1)),
        ValidityStatus::ExpiresSoon {
                expires_at: Utc::now() + EXPIRES_SOON_WINDOW - Duration::hours(1),
                notify_at: Utc::now()
            }
    )]
    #[case::valid_until(
        Some(Duration::days(-1)),
        Some(EXPIRES_SOON_WINDOW + Duration::days(3)),
        ValidityStatus::ValidUntil {
                expires_at: Utc::now() + EXPIRES_SOON_WINDOW + Duration::days(3),
                notify_at: Utc::now()
            }
    )]
    #[case::valid_on_threshold(
        Some(Duration::days(-1)),
        Some(EXPIRES_SOON_WINDOW),
        ValidityStatus::ValidUntil {
                expires_at: Utc::now() + EXPIRES_SOON_WINDOW,
                notify_at: Utc::now()
            }
    )]
    #[case::not_yet_valid_priority(
        Some(Duration::hours(1)),
        Some(Duration::hours(2)),
        ValidityStatus::NotYetValid { valid_from: Utc::now() + Duration::hours(1) }
    )]
    #[case::always_valid(Some(Duration::days(-1)), None, ValidityStatus::AlwaysValid)]
    fn test_validity_status_logic(
        #[case] from_offset: Option<Duration>,
        #[case] until_offset: Option<Duration>,
        #[case] expected: ValidityStatus,
    ) {
        let now = Utc::now();
        let window = ValidityWindow {
            valid_from: from_offset.map(|d| now + d),
            valid_until: until_offset.map(|d| now + d),
        };

        let actual = ValidityStatus::from_window(&window, now);

        match (actual, expected) {
            (
                ValidityStatus::ValidUntil {
                    expires_at: actual_expires_at,
                    ..
                },
                ValidityStatus::ValidUntil {
                    expires_at: expected_expires_at,
                    ..
                },
            ) => assert_eq!(
                actual_expires_at.duration_round(TimeDelta::hours(1)),
                expected_expires_at.duration_round(TimeDelta::hours(1))
            ),
            (
                ValidityStatus::ExpiresSoon {
                    expires_at: actual_expires_at,
                    ..
                },
                ValidityStatus::ExpiresSoon {
                    expires_at: expected_expires_at,
                    ..
                },
            ) => assert_eq!(
                actual_expires_at.duration_round(TimeDelta::hours(1)),
                expected_expires_at.duration_round(TimeDelta::hours(1))
            ),
            (
                ValidityStatus::Expired {
                    expired_at: actual_expired_at,
                },
                ValidityStatus::Expired {
                    expired_at: expected_expired_at,
                },
            ) => assert_eq!(
                actual_expired_at.duration_round(TimeDelta::hours(1)),
                expected_expired_at.duration_round(TimeDelta::hours(1))
            ),
            (
                ValidityStatus::NotYetValid {
                    valid_from: actual_valid_from,
                },
                ValidityStatus::NotYetValid {
                    valid_from: expected_valid_from,
                },
            ) => assert_eq!(
                actual_valid_from.duration_round(TimeDelta::hours(1)),
                expected_valid_from.duration_round(TimeDelta::hours(1))
            ),
            (a, b) => assert_eq!(a, b),
        }
    }
}
