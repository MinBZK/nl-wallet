pub mod client;
pub mod integrity_verdict;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use chrono::NaiveDate;
    use serde_json::json;
    use serde_json::Value;

    use super::integrity_verdict::*;

    pub static EXAMPLE_VERDICT_JSON: LazyLock<Value> = LazyLock::new(|| {
        json!({
            "requestDetails": {
                "requestPackageName": "com.package.name",
                "requestHash": "aGVsbG8gd29scmQgdGhlcmU",
                "timestampMillis": "1675655009345"
            },
            "appIntegrity": {
                "appRecognitionVerdict": "PLAY_RECOGNIZED",
                "packageName": "com.package.name",
                "certificateSha256Digest": ["6a6a1474b5cbbb2b1aa57e0bc3"],
                "versionCode": "42"
            },
            "deviceIntegrity": {
                "deviceRecognitionVerdict": ["MEETS_DEVICE_INTEGRITY"],
                "recentDeviceActivity": {
                    "deviceActivityLevel": "LEVEL_2"
                },
                "deviceAttributes": {
                    "sdkVersion": 33
                }
            },
            "accountDetails": {
                "appLicensingVerdict": "LICENSED"
            },
            "environmentDetails": {
                "appAccessRiskVerdict": {
                    "appsDetected": ["KNOWN_INSTALLED", "UNKNOWN_INSTALLED", "UNKNOWN_CAPTURING"]
                },
                "playProtectVerdict": "NO_ISSUES",
            }
        })
    });

    pub static EXAMPLE_VERDICT: LazyLock<IntegrityVerdict> = LazyLock::new(|| IntegrityVerdict {
        request_details: RequestDetails {
            request_package_name: "com.package.name".to_string(),
            request_hash: "aGVsbG8gd29scmQgdGhlcmU".to_string(),
            timestamp: NaiveDate::from_ymd_opt(2023, 2, 6)
                .unwrap()
                .and_hms_milli_opt(3, 43, 29, 345)
                .unwrap()
                .and_utc(),
        },
        app_integrity: AppIntegrity {
            app_recognition_verdict: AppRecognitionVerdict::PlayRecognized,
            details: Some(AppIntegrityDetails {
                package_name: "com.package.name".to_string(),
                certificate_sha256_digest: vec![b"\x6a\x6a\x14\x74\xb5\xcb\xbb\x2b\x1a\xa5\x7e\x0b\xc3".to_vec()],
                version_code: 42.to_string(),
            }),
        },
        device_integrity: DeviceIntegrity {
            device_recognition_verdict: HashSet::from([DeviceRecognitionVerdict::MeetsDeviceIntegrity]),
            recent_device_activity: Some(RecentDeviceActivity {
                device_activity_level: DeviceActivityLevel::Level2,
            }),
            device_attributes: Some(DeviceAttributes { sdk_version: Some(33) }),
        },
        account_details: AccountDetails {
            app_licensing_verdict: AppLicensingVerdict::Licensed,
        },
        environment_details: Some(EnvironmentDetails {
            app_access_risk_verdict: Some(AppAccessRiskVerdict {
                apps_detected: Some(HashSet::from([
                    AppsDetected::KnownInstalled,
                    AppsDetected::UnknownInstalled,
                    AppsDetected::UnknownCapturing,
                ])),
            }),
            play_protect_verdict: Some(PlayProtectVerdict::NoIssues),
        }),
    });
}
