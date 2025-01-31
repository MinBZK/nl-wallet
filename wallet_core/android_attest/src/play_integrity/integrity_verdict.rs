use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::hex::Hex;
use serde_with::serde_as;
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
    #[serde_as(as = "Base64")]
    pub request_hash: Vec<u8>,
    #[serde(rename = "timestampMillis")]
    #[serde_as(as = "TimestampMilliSeconds<String>")]
    pub timestamp: DateTime<Utc>,
}

#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct AppIntegrity {
    pub app_recognition_verdict: AppRecognitionVerdict,
    // These fields are not set when `app_recognition_verdict` is `AppRecognitionVerdict::Unevaluated`.
    #[serde(flatten)]
    pub details: Option<AppIntegrityDetails>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct AppIntegrityDetails {
    pub package_name: String,
    #[serde_as(as = "HashSet<Hex>")]
    pub certificate_sha256_digest: HashSet<Vec<u8>>,
    pub version_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, strum::Display)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AppRecognitionVerdict {
    PlayRecognized,
    UnrecognizedVersion,
    Unevaluated,
}

#[cfg_attr(feature = "encode", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct DeviceIntegrity {
    pub device_recognition_verdict: HashSet<DeviceRecognitionVerdict>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, strum::Display)]
#[cfg_attr(feature = "encode", derive(serde::Serialize))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AppLicensingVerdict {
    Licensed,
    Unlicensed,
    Unevaluated,
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
    use super::super::verification::VerifyPlayStore;
    use super::*;

    impl IntegrityVerdict {
        pub fn new_mock(package_name: String, request_hash: Vec<u8>, verify_play_store: VerifyPlayStore) -> Self {
            let (app_integrity, account_details) = match verify_play_store {
                VerifyPlayStore::NoVerify => (
                    AppIntegrity {
                        app_recognition_verdict: AppRecognitionVerdict::Unevaluated,
                        details: None,
                    },
                    AccountDetails {
                        app_licensing_verdict: AppLicensingVerdict::Unlicensed,
                    },
                ),
                VerifyPlayStore::Verify {
                    play_store_certificate_hashes,
                } => (
                    AppIntegrity {
                        app_recognition_verdict: AppRecognitionVerdict::PlayRecognized,
                        details: Some(AppIntegrityDetails {
                            package_name: package_name.clone(),
                            certificate_sha256_digest: play_store_certificate_hashes,
                            version_code: "42".to_string(),
                        }),
                    },
                    AccountDetails {
                        app_licensing_verdict: AppLicensingVerdict::Licensed,
                    },
                ),
            };

            Self {
                request_details: RequestDetails {
                    request_package_name: package_name,
                    request_hash,
                    timestamp: Utc::now(),
                },
                app_integrity,
                device_integrity: DeviceIntegrity {
                    device_recognition_verdict: HashSet::from([DeviceRecognitionVerdict::MeetsDeviceIntegrity]),
                    recent_device_activity: None,
                    device_attributes: None,
                },
                account_details,
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
