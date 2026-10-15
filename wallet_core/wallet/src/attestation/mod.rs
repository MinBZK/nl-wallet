mod attribute;
pub(crate) mod metadata;

use std::collections::HashSet;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributesError;
use attestation_data::organization::Organization;
use attestation_data::validity::ValidityWindow;
use attestation_types::credential_format::Format;
use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::Utc;
use error_category::ErrorCategory;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::base64::UrlSafe;
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use token_status_list::verification::verifier::RevocationStatus;
use utils::vec_at_least::VecNonEmpty;
use uuid::Uuid;
use wallet_configuration::wallet_config::PidAttributesConfiguration;

use crate::attestation::metadata::AttestationDisplayMetadata;
use crate::attestation::metadata::ClaimDisplay;

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
    #[error("attribute conversion failed: {0:?}")]
    #[category(pd)]
    AttributeConversion(Attribute),

    #[error("unable to parse attribute value into date: {0:?}")]
    #[category(pd)]
    AttributeDateValue(#[from] chrono::ParseError),
}

pub trait AttestationPresentationConfig {
    fn filtered_attribute(&self, format: Format, attestation_type: &str) -> Option<&[String]>;
}

impl AttestationPresentationConfig for PidAttributesConfiguration {
    fn filtered_attribute(&self, format: Format, attribute: &str) -> Option<&[String]> {
        let paths_by_attestation_type = match format {
            Format::MsoMdoc => &self.mso_mdoc,
            Format::SdJwt => &self.sd_jwt,
        };

        paths_by_attestation_type
            .get(attribute)
            .map(|pid_paths| pid_paths.recovery_code.as_ref())
    }
}

// TODO: Separate various concerns: PVW-4675
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationPresentation {
    pub identity: AttestationIdentity,
    pub format: Format,
    pub attestation_type: String,
    pub display_metadata: Vec<AttestationDisplayMetadata>,
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
    pub metadata: Vec<ClaimDisplay>,
    pub value: AttributeValue,
    pub svg_id: Option<String>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum AttributeValue {
    Null,
    Number(i64),
    Bool(bool),
    Text(String),
    Array(Vec<AttributeValue>),
    Object(IndexMap<String, AttributeValue>),

    // mdoc only
    Date(NaiveDate),
    Bytes(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] Vec<u8>),
    Image(ImageAttribute),
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum ImageAttribute {
    Jpeg(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] Vec<u8>),
    Jpeg2000(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] Vec<u8>),
}

impl From<Attribute> for AttributeValue {
    fn from(value: Attribute) -> Self {
        match value {
            Attribute::Null => Self::Null,
            Attribute::Number(number) => Self::Number(number),
            Attribute::Bool(bool) => Self::Bool(bool),
            Attribute::Text(string) => Self::Text(string),
            Attribute::Array(array) => Self::Array(array.into_iter().map(Self::from).collect()),
            Attribute::Object(map) => Self::Object(
                map.into_iter()
                    .map(|(key, attribute)| (key, attribute.into()))
                    .collect(),
            ),
            Attribute::Date(date) => Self::Date(date),
            Attribute::Bytes(bytes) => match ImageAttribute::try_from(bytes) {
                Ok(image) => Self::Image(image),
                Err(bytes) => Self::Bytes(bytes),
            },
        }
    }
}

impl TryFrom<Vec<u8>> for ImageAttribute {
    type Error = Vec<u8>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.as_slice() {
            // JPEG header and footer.
            [0xFF, 0xD8, .., 0xFF, 0xD9] => Ok(Self::Jpeg(value)),
            // JPEG 2000 header.
            [0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20, 0x0D, 0x0A, 0x87, 0x0A] => Ok(Self::Jpeg2000(value)),
            _ => Err(value),
        }
    }
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
    use attestation_data::organization::Organization;
    use attestation_data::validity::ValidityWindow;
    use attestation_types::credential_format::Format;

    use super::AttestationDisplayMetadata;
    use super::AttestationIdentity;
    use super::AttestationPresentation;
    use super::AttestationPresentationConfig;
    use super::AttestationValidity;

    pub struct EmptyPresentationConfig;

    impl AttestationPresentationConfig for EmptyPresentationConfig {
        fn filtered_attribute(&self, _format: Format, _attestation_type: &str) -> Option<&[String]> {
            None
        }
    }

    impl AttestationPresentation {
        /// Create a nearly empty [`AttestationPresentation`] for tests that absolutely need this type.
        pub fn new_mock() -> Self {
            Self {
                identity: AttestationIdentity::Ephemeral,
                format: Format::SdJwt,
                attestation_type: "mock".to_string(),
                display_metadata: vec![AttestationDisplayMetadata {
                    locale: "nl".to_string(),
                    name: "mock".to_string(),
                    description: None,
                    summary: None,
                    rendering: None,
                }],
                issuer: Organization::new_mock().into(),
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
