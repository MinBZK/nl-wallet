use std::collections::HashMap;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

use wallet_common::generator::Generator;
use wallet_common::update_policy::UpdatePolicyResponse;
use wallet_common::update_policy::VersionReq;
use wallet_common::update_policy::VersionState;

const WARN_THRESHOLD: Duration = Duration::from_secs(7 * 24 * 60 * 60);

// Convenience type s.t. configuring DateTimes is optional
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum::Display,
    strum::EnumString,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[strum(serialize_all = "PascalCase")]
pub enum VersionPolicy {
    Notify,
    Recommend,
    Block,
}

impl From<VersionPolicy> for VersionPolicyWithDate {
    fn from(config: VersionPolicy) -> Self {
        match config {
            VersionPolicy::Notify => Self::Notify(DateTime::<Utc>::MIN_UTC),
            VersionPolicy::Recommend => Self::Recommend(DateTime::<Utc>::MIN_UTC),
            VersionPolicy::Block => Self::Block(DateTime::<Utc>::MIN_UTC),
        }
    }
}

// Possible configured state for a version, in order of severity. TOML deserialization of config does a strange thing
// where the tags are read in lowercase even though they are capitalized in the file, so the aliases are there to cover
// this.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VersionPolicyWithDate {
    #[serde(alias = "notify")]
    Notify(DateTime<Utc>),
    #[serde(alias = "recommend")]
    Recommend(DateTime<Utc>),
    #[serde(alias = "block")]
    Block(DateTime<Utc>),
}

impl VersionPolicyWithDate {
    fn to_state(&self, time: &impl Generator<DateTime<Utc>>) -> VersionState {
        let now = time.generate();
        match self {
            Self::Block(block) if block < &now => VersionState::Block,
            // unwrap is safe because block >= now
            Self::Block(block) if (*block - now).to_std().unwrap() < WARN_THRESHOLD => {
                VersionState::Warn((*block - now).to_std().unwrap())
            }
            Self::Recommend(recommend) if recommend < &now => VersionState::Recommend,
            Self::Notify(notify) if notify < &now => VersionState::Notify,
            _ => VersionState::Ok,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum VersionPolicyConfig {
    WithoutDate(VersionPolicy),
    WithDate(VersionPolicyWithDate),
}

// Configuration type of the Update Policy, this type lives on the server only
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePolicyConfig(HashMap<VersionReq, VersionPolicyConfig>);

impl UpdatePolicyConfig {
    pub fn into_response(self, time: &impl Generator<DateTime<Utc>>) -> UpdatePolicyResponse {
        UpdatePolicyResponse(
            self.0
                .into_iter()
                .map(|(range, policy)| {
                    let policy = match policy {
                        VersionPolicyConfig::WithoutDate(policy) => policy.into(),
                        VersionPolicyConfig::WithDate(policy) => policy,
                    };

                    (range, policy.to_state(time))
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod test {
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case(json!("Notify"), VersionPolicyWithDate::Notify(DateTime::<Utc>::MIN_UTC))]
    #[case(json!("Recommend"), VersionPolicyWithDate::Recommend(DateTime::<Utc>::MIN_UTC))]
    #[case(json!("Block"), VersionPolicyWithDate::Block(DateTime::<Utc>::MIN_UTC))]
    #[case(json!({ "Notify": "1970-01-01T00:00:00Z" }), VersionPolicyWithDate::Notify(DateTime::UNIX_EPOCH))]
    #[case(json!({ "Recommend": "1970-01-01T00:00:00Z" }), VersionPolicyWithDate::Recommend(DateTime::UNIX_EPOCH))]
    #[case(json!({ "Block": "1970-01-01T00:00:00Z" }), VersionPolicyWithDate::Block(DateTime::UNIX_EPOCH))]
    fn test_deserialize_version_policy_config(#[case] v: serde_json::Value, #[case] expected: VersionPolicyWithDate) {
        let config = serde_json::from_value::<VersionPolicyConfig>(v).unwrap();
        let result = match config {
            VersionPolicyConfig::WithDate(policy) => policy,
            VersionPolicyConfig::WithoutDate(policy) => policy.into(),
        };
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(json!({}), Default::default())]
    #[case(json!({
            "=1.0.0": { "Notify": "1969-01-01T00:00:00Z" }
    }), UpdatePolicyResponse(
            IndexMap::from(
                [
                    (VersionReq::parse("=1.0.0").unwrap(), VersionState::Notify),
                ]
            )
    ))]
    #[case(json!({
        ">=0.1.0, <1.0.0": { "Block": "1970-01-05T00:00:00Z" },
        "=0.5.0": { "Recommend": "1969-01-01T00:00:00Z" },
        "=1.0.0": { "Notify": "1969-01-01T00:00:00Z" },
        "=1.0.0": { "Recommend": "1970-01-01T00:00:00Z" },
        "=2.0.0": { "Block": "1971-01-01T00:00:00Z" },
        "=1.1.0": "Notify",
        "=1.2.0": "Block",
    }), UpdatePolicyResponse(
            IndexMap::from(
                [
                    (
                        VersionReq::parse(">=0.1.0, <1.0.0").unwrap(),
                        VersionState::Warn(Duration::from_secs(4 * 24 * 60 * 60))
                    ),
                    (VersionReq::parse("=0.5.0").unwrap(), VersionState::Recommend),
                    (VersionReq::parse("=1.1.0").unwrap(), VersionState::Notify),
                    (VersionReq::parse("=1.0.0").unwrap(), VersionState::Ok),
                    (VersionReq::parse("=2.0.0").unwrap(), VersionState::Ok),
                    (VersionReq::parse("=1.1.0").unwrap(), VersionState::Notify),
                    (VersionReq::parse("=1.2.0").unwrap(), VersionState::Block),
                ]
            )
    ))]
    fn test_update_policy(#[case] v: serde_json::Value, #[case] expected: UpdatePolicyResponse) {
        use wallet_common::generator::mock::MockTimeGenerator;

        let update_policy = serde_json::from_value::<UpdatePolicyConfig>(v).unwrap();
        assert_eq!(update_policy.into_response(&MockTimeGenerator::epoch()), expected);
    }
}
