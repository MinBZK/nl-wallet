use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::base64::UrlSafe;
use serde_with::serde_as;
use serde_with::DeserializeFromStr;
use serde_with::TimestampMilliSeconds;

/// The decoded integrity verdict, as sent by the Google Play API. Note
/// that this only supports "standard" API request, not "classic" requests.
/// See: https://developer.android.com/google/play/integrity/verdicts
#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct IntegrityVerdict {
    pub request_details: RequestDetails,
    pub app_integrity: AppIntegrity,
    pub device_integrity: DeviceIntegrity,
    pub account_details: AccountDetails,
    // Opt-in field.
    pub environment_details: Option<EnvironmentDetails>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct RequestDetails {
    pub request_package_name: String,
    pub request_hash: String,
    #[serde(rename = "timestampMillis")]
    #[serde_as(as = "TimestampMilliSeconds<String>")]
    pub timestamp: DateTime<Utc>,
}

#[serde_as]
#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct AppIntegrity {
    pub app_recognition_verdict: AppRecognitionVerdict,
    // These field is not present when `app_recognition_verdict` is `AppRecognitionVerdict::Unevaluated`.
    pub package_name: Option<String>,
    // These field is not present when `app_recognition_verdict` is `AppRecognitionVerdict::Unevaluated`.
    #[serde_as(as = "Option<HashSet<Base64<UrlSafe>>>")]
    pub certificate_sha256_digest: Option<HashSet<Vec<u8>>>,
    // These field is not present when `app_recognition_verdict` is `AppRecognitionVerdict::Unevaluated`.
    pub version_code: Option<String>,
}

// Note that the order of this enum is relevant, as we derive Ord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, DeserializeFromStr, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "encode", derive(serde_with::SerializeDisplay))]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AppRecognitionVerdict {
    Unevaluated,
    UnrecognizedVersion,
    PlayRecognized,
}

#[serde_as]
#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct DeviceIntegrity {
    // This field is not present on verdicts generated for the emulator.
    pub device_recognition_verdict: Option<HashSet<DeviceRecognitionVerdict>>,
    // Opt-in field.
    pub recent_device_activity: Option<RecentDeviceActivity>,
    // Opt-in field.
    pub device_attributes: Option<DeviceAttributes>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceRecognitionVerdict {
    MeetsDeviceIntegrity,
    // Only for Google Play Games for PC.
    MeetsVirtualIntegrity,
    // Opt-in value.
    MeetsBasicIntegrity,
    // Opt-in value.
    MeetsStrongIntegrity,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct RecentDeviceActivity {
    pub device_activity_level: DeviceActivityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceActivityLevel {
    #[serde(rename = "LEVEL_1")]
    Level1,
    #[serde(rename = "LEVEL_2")]
    Level2,
    #[serde(rename = "LEVEL_3")]
    Level3,
    #[serde(rename = "LEVEL_4")]
    Level4,
    Unevaluated,
}

#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct DeviceAttributes {
    pub sdk_version: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    pub app_licensing_verdict: AppLicensingVerdict,
}

// Note that the order of this enum is relevant, as we derive Ord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, DeserializeFromStr, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "encode", derive(serde_with::SerializeDisplay))]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AppLicensingVerdict {
    Unevaluated,
    Unlicensed,
    Licensed,
}

#[cfg(feature = "encode")]
fn option_hash_set_none_or_empty<T>(set: &Option<HashSet<T>>) -> bool {
    let has_values = set.as_ref().map(|set| !set.is_empty()).unwrap_or_default();

    !has_values
}

#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDetails {
    // Opt-in field.
    pub app_access_risk_verdict: Option<AppAccessRiskVerdict>,
    // Opt-in field.
    pub play_protect_verdict: Option<PlayProtectVerdict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct AppAccessRiskVerdict {
    #[cfg_attr(feature = "encode", serde(skip_serializing_if = "option_hash_set_none_or_empty"))]
    pub apps_detected: Option<HashSet<AppsDetected>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppsDetected {
    KnownInstalled,
    UnknownInstalled,
    KnownCapturing,
    UnknownCapturing,
    KnownControlling,
    UnknownControlling,
    KnownOverlays,
    UnknownOverlays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayProtectVerdict {
    NoIssues,
    NoData,
    PossibleRisk,
    MediumRisk,
    HighRisk,
    Unevaluated,
}

#[cfg(feature = "mock")]
mod mock {
    use super::*;

    impl IntegrityVerdict {
        pub fn new_mock(package_name: String, request_hash: String, certificate_hashes: HashSet<Vec<u8>>) -> Self {
            Self {
                request_details: RequestDetails {
                    request_package_name: package_name.clone(),
                    request_hash,
                    timestamp: Utc::now(),
                },
                app_integrity: AppIntegrity {
                    app_recognition_verdict: AppRecognitionVerdict::PlayRecognized,
                    package_name: Some(package_name),
                    certificate_sha256_digest: Some(certificate_hashes),
                    version_code: Some("42".to_string()),
                },
                device_integrity: DeviceIntegrity {
                    device_recognition_verdict: Some(HashSet::from([DeviceRecognitionVerdict::MeetsDeviceIntegrity])),
                    recent_device_activity: None,
                    device_attributes: None,
                },
                account_details: AccountDetails {
                    app_licensing_verdict: AppLicensingVerdict::Licensed,
                },
                environment_details: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::EXAMPLE_VERDICT;
    use super::super::tests::EXAMPLE_VERDICT_JSON;
    use super::*;

    #[test]
    fn test_integrity_verdict_deserialize() {
        let verdict_json = serde_json::to_string(&*EXAMPLE_VERDICT_JSON).unwrap();

        let verdict =
            serde_json::from_str::<IntegrityVerdict>(&verdict_json).expect("integrity verdict should deserialize");

        assert_eq!(verdict, *EXAMPLE_VERDICT);
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_integrity_verdict_serialize() {
        let verdict_json = serde_json::to_string(&*EXAMPLE_VERDICT).expect("integrity verdict should serialize");

        let verdict =
            serde_json::from_str::<IntegrityVerdict>(&verdict_json).expect("integrity verdict should deserialize");

        assert_eq!(verdict, *EXAMPLE_VERDICT);
    }
}
