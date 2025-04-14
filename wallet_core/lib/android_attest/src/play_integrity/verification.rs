use std::collections::HashSet;

use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use derive_more::AsRef;

use super::integrity_verdict::AppLicensingVerdict;
use super::integrity_verdict::AppRecognitionVerdict;
use super::integrity_verdict::DeviceRecognitionVerdict;
use super::integrity_verdict::IntegrityVerdict;

// The oldest an integrity verdict request can be is 15 minutes.
const MAX_REQUEST_AGE: TimeDelta = TimeDelta::minutes(15);

// To prevent clocks skew issues to some degree, have some margin
// when determining that a request timestamp is in the future.
const FUTURE_SKEW_MARGIN: TimeDelta = TimeDelta::minutes(5);

#[derive(Debug, thiserror::Error)]
pub enum IntegrityVerdictVerificationError {
    #[error("request package name does not match, received: {0}")]
    RequestPackageNameMismatch(String),
    #[error("request hash does not match")]
    RequestHashMismatch,
    #[error("integrity verdict was requested too long ago or in the future: {0}")]
    RequestTimestampInvalid(DateTime<Utc>),
    #[error("the app's Google Play recognition verdict is not strict enough, received: {0}")]
    NotPlayRecognized(AppRecognitionVerdict),
    #[error("integrity verdict package name does not match, received: {}", .0.as_deref().unwrap_or("<NONE>"))]
    PlayStorePackageNameMismatch(Option<String>),
    #[error("the set of play store certificate hashes in the integrity verdict do not match those provided")]
    PlayStoreCertificateMismatch,
    #[error("the device does not pass system integrity checks or does not meet Android compatibility requirements")]
    DeviceIntegrityNotMet,
    #[error("the app's Google Play licensing verdict is not strict enough, received: {0}")]
    NoAppEntitlement(AppLicensingVerdict),
}

#[derive(Debug, Clone, Copy, Default)]
pub enum InstallationMethod {
    SideloadOrPlayStore,
    #[default]
    PlayStore,
}

#[derive(Debug, Clone, AsRef)]
pub struct VerifiedIntegrityVerdict(IntegrityVerdict);

/// Wraps a verified instance of [`IntegrityVerdict`]. The verification is done according to recommendations in the
/// [Google documentation](https://developer.android.com/google/play/integrity/verdicts). It does not consider opt-in
/// fields.
impl VerifiedIntegrityVerdict {
    pub fn verify(
        integrity_verdict: IntegrityVerdict,
        expected_package_name: &str,
        expected_request_hash: &str,
        expected_certificate_hashes: &HashSet<Vec<u8>>,
        allowed_installation_method: InstallationMethod,
    ) -> Result<Self, IntegrityVerdictVerificationError> {
        Self::verify_with_time(
            integrity_verdict,
            expected_package_name,
            expected_request_hash,
            expected_certificate_hashes,
            allowed_installation_method,
            Utc::now(),
        )
    }

    pub fn verify_with_time(
        integrity_verdict: IntegrityVerdict,
        expected_package_name: &str,
        expected_request_hash: &str,
        expected_certificate_hashes: &HashSet<Vec<u8>>,
        allowed_installation_method: InstallationMethod,
        time: DateTime<Utc>,
    ) -> Result<Self, IntegrityVerdictVerificationError> {
        if integrity_verdict.request_details.request_package_name != expected_package_name {
            return Err(IntegrityVerdictVerificationError::RequestPackageNameMismatch(
                integrity_verdict.request_details.request_package_name,
            ));
        }

        if integrity_verdict.request_details.request_hash != expected_request_hash {
            return Err(IntegrityVerdictVerificationError::RequestHashMismatch);
        }

        // This is sensitive to clock skews on the host machine. As this will also reject timestamps
        // that are in the future, we apply some amount of margin here in that direction.
        let request_time_delta = time.signed_duration_since(integrity_verdict.request_details.timestamp);
        if request_time_delta > MAX_REQUEST_AGE || request_time_delta < -FUTURE_SKEW_MARGIN {
            return Err(IntegrityVerdictVerificationError::RequestTimestampInvalid(
                integrity_verdict.request_details.timestamp,
            ));
        }

        let min_recognition_verdict = match allowed_installation_method {
            InstallationMethod::SideloadOrPlayStore => AppRecognitionVerdict::UnrecognizedVersion,
            InstallationMethod::PlayStore => AppRecognitionVerdict::PlayRecognized,
        };
        if integrity_verdict.app_integrity.app_recognition_verdict < min_recognition_verdict {
            return Err(IntegrityVerdictVerificationError::NotPlayRecognized(
                integrity_verdict.app_integrity.app_recognition_verdict,
            ));
        }

        if integrity_verdict.app_integrity.package_name.as_deref() != Some(expected_package_name) {
            return Err(IntegrityVerdictVerificationError::PlayStorePackageNameMismatch(
                integrity_verdict.app_integrity.package_name,
            ));
        }

        // Verification of the certificate hashes will only succeed if one of these conditions is met:
        // 1. Sideloading is allowed.
        // 2. The integrity verdict contains at least one hash AND all of the hashes in the verdict are present in the
        //    set of required hashes.
        if !matches!(allowed_installation_method, InstallationMethod::SideloadOrPlayStore)
            && !integrity_verdict
                .app_integrity
                .certificate_sha256_digest
                .as_ref()
                .map(|certificate_sha256_digest| {
                    !certificate_sha256_digest.is_empty()
                        && certificate_sha256_digest.is_subset(expected_certificate_hashes)
                })
                .unwrap_or(false)
        {
            return Err(IntegrityVerdictVerificationError::PlayStoreCertificateMismatch);
        }

        if !integrity_verdict
            .device_integrity
            .device_recognition_verdict
            .as_ref()
            .ok_or(IntegrityVerdictVerificationError::DeviceIntegrityNotMet)?
            .contains(&DeviceRecognitionVerdict::MeetsDeviceIntegrity)
        {
            return Err(IntegrityVerdictVerificationError::DeviceIntegrityNotMet);
        }

        let min_licensing_verdict = match allowed_installation_method {
            InstallationMethod::SideloadOrPlayStore => AppLicensingVerdict::Unevaluated,
            InstallationMethod::PlayStore => AppLicensingVerdict::Licensed,
        };
        if integrity_verdict.account_details.app_licensing_verdict < min_licensing_verdict {
            return Err(IntegrityVerdictVerificationError::NoAppEntitlement(
                integrity_verdict.account_details.app_licensing_verdict,
            ));
        }

        Ok(Self(integrity_verdict))
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use chrono::NaiveDate;
    use rstest::rstest;

    use super::super::tests::EXAMPLE_VERDICT;
    use super::*;

    fn verify_example_verdict_with_hashes(
        integrity_verdict: IntegrityVerdict,
        certificate_hashes: &HashSet<Vec<u8>>,
        installation_method: InstallationMethod,
    ) -> Result<(), IntegrityVerdictVerificationError> {
        VerifiedIntegrityVerdict::verify_with_time(
            integrity_verdict,
            "com.package.name",
            "aGVsbG8gd29scmQgdGhlcmU",
            certificate_hashes,
            installation_method,
            NaiveDate::from_ymd_opt(2023, 2, 6)
                .unwrap()
                .and_hms_opt(3, 45, 0)
                .unwrap()
                .and_utc(),
        )
        .map(|_| ())
    }

    fn verify_example_verdict(
        integrity_verdict: IntegrityVerdict,
        installation_method: InstallationMethod,
    ) -> Result<(), IntegrityVerdictVerificationError> {
        verify_example_verdict_with_hashes(
            integrity_verdict,
            &HashSet::from([b"\x6a\x6a\x14\x74\xb5\xcb\xbb\x2b\x1a\xa5\x7e\x0b\xc3".to_vec()]),
            installation_method,
        )
    }

    #[rstest]
    fn test_verified_integrity_verdict(
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
    ) {
        verify_example_verdict(EXAMPLE_VERDICT.clone(), installation_method)
            .expect("integrity verdict should verify successfully");
    }

    #[cfg(feature = "mock")]
    #[rstest]
    fn test_verified_integrity_verdict_mock(
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
    ) {
        let package_name = "com.package.mock";
        let request_hash = "request_hash";

        let mock_verdict = IntegrityVerdict::new_mock(
            package_name.to_string(),
            request_hash.to_string(),
            HashSet::from([b"hash_hash_hash".to_vec()]),
        );

        VerifiedIntegrityVerdict::verify(
            mock_verdict.clone(),
            package_name,
            request_hash,
            &HashSet::from([b"hash_hash_hash".to_vec()]),
            installation_method,
        )
        .expect("integrity verdict should verify successfully");
    }

    #[rstest]
    fn test_verified_integrity_verdict_request_package_name_mismatch_error(
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.request_details.request_package_name = "com.package.different".to_string();

        let error =
            verify_example_verdict(verdict, installation_method).expect_err("integrity verdict should not verify");

        assert_matches!(
            error,
            IntegrityVerdictVerificationError::RequestPackageNameMismatch(name) if name == "com.package.different"
        )
    }

    #[rstest]
    fn test_verified_integrity_verdict_request_hash_mismatch_error(
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.request_details.request_hash = "different_hash".to_string();

        let error =
            verify_example_verdict(verdict, installation_method).expect_err("integrity verdict should not verify");

        assert_matches!(error, IntegrityVerdictVerificationError::RequestHashMismatch)
    }

    #[rstest]
    // Too long ago.
    #[case(NaiveDate::from_ymd_opt(2023, 2, 6).unwrap().and_hms_opt(3, 25, 0).unwrap().and_utc(), false)]
    // Within the max timestamp age.
    #[case(NaiveDate::from_ymd_opt(2023, 2, 6).unwrap().and_hms_opt(3, 35, 0).unwrap().and_utc(), true)]
    // In the future, within the acceptable margin.
    #[case(NaiveDate::from_ymd_opt(2023, 2, 6).unwrap().and_hms_opt(3, 47, 0).unwrap().and_utc(), true)]
    // Too far into the future.
    #[case(NaiveDate::from_ymd_opt(2023, 2, 6).unwrap().and_hms_opt(3, 51, 0).unwrap().and_utc(), false)]
    fn test_verified_integrity_verdict_request_timestamp_inconsistent_error(
        #[case] verdict_timestamp: DateTime<Utc>,
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
        #[case] should_succeed: bool,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.request_details.timestamp = verdict_timestamp;

        // Note that the timestamp is checked against a "current" time of 2024-02-06T03:45:00Z.
        let result = verify_example_verdict(verdict, installation_method);

        if should_succeed {
            result.expect("integrity verdict should verify successfully");
        } else {
            assert_matches!(
                result,
                Err(IntegrityVerdictVerificationError::RequestTimestampInvalid(date)) if date == verdict_timestamp
            )
        }
    }

    #[rstest]
    #[case(AppRecognitionVerdict::UnrecognizedVersion, InstallationMethod::PlayStore, false)]
    #[case(
        AppRecognitionVerdict::UnrecognizedVersion,
        InstallationMethod::SideloadOrPlayStore,
        true
    )]
    #[case(AppRecognitionVerdict::Unevaluated, InstallationMethod::PlayStore, false)]
    #[case(AppRecognitionVerdict::Unevaluated, InstallationMethod::SideloadOrPlayStore, false)]
    fn test_verified_integrity_verdict_not_play_recognized_error(
        #[case] app_recognition_verdict: AppRecognitionVerdict,
        #[case] installation_method: InstallationMethod,
        #[case] should_succeed: bool,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.app_integrity.app_recognition_verdict = app_recognition_verdict;

        let result = verify_example_verdict(verdict, installation_method);

        if should_succeed {
            result.expect("integrity verdict should verify successfully");
        } else {
            let error = result.expect_err("integrity verdict should not verify");

            assert_matches!(
                error,
                IntegrityVerdictVerificationError::NotPlayRecognized(recognition_verdict)
                    if recognition_verdict == app_recognition_verdict
            )
        }
    }

    #[rstest]
    fn test_verified_integrity_verdict_play_store_package_name_mismatch_error(
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict
            .app_integrity
            .package_name
            .replace("com.package.different".to_string());

        let error =
            verify_example_verdict(verdict, installation_method).expect_err("integrity verdict should not verify");

        assert_matches!(
            error,
            IntegrityVerdictVerificationError::PlayStorePackageNameMismatch(name)
                if name == Some("com.package.different".to_string())
        )
    }

    #[rstest]
    // When no hashes are required, the integrity verdict should never verify.
    #[case(None, HashSet::new(), false)]
    #[case(Some(HashSet::new()), HashSet::new(), false)]
    #[case(Some(HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec()])), HashSet::new(), false)]
    // If any hashes are required, receiving a verdict without hashes should not verify.
    #[case(None, HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec(), b"hash_3".to_vec()]), false)]
    #[case(Some(HashSet::new()), HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec(), b"hash_3".to_vec()]), false)]
    // If the verdict contains hashes, they should form a subset of the required hashes.
    #[case(
        Some(HashSet::from([b"hash_2".to_vec()])),
        HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec(), b"hash_3".to_vec()]),
        true
    )]
    #[case(
        Some(HashSet::from([b"hash_3".to_vec(), b"hash_2".to_vec()])),
        HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec(), b"hash_3".to_vec()]),
        true
    )]
    #[case(
        Some(HashSet::from([b"hash_4".to_vec()])),
        HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec(), b"hash_3".to_vec()]),
        false
    )]
    #[case(
        Some(HashSet::from([b"hash_3".to_vec(), b"hash_2".to_vec()])),
        HashSet::from([b"hash_3".to_vec()]),
        false
    )]
    #[case(
        Some(HashSet::from([b"hash_3".to_vec(), b"hash_4".to_vec()])),
        HashSet::from([b"hash_1".to_vec(), b"hash_2".to_vec(), b"hash_3".to_vec()]),
        false
    )]
    fn test_verified_integrity_verdict_play_store_certificate_mismatch_error(
        #[case] verdict_hashes: Option<HashSet<Vec<u8>>>,
        #[case] required_hashes: HashSet<Vec<u8>>,
        #[case] should_succeed: bool,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.app_integrity.certificate_sha256_digest = verdict_hashes;

        let result =
            verify_example_verdict_with_hashes(verdict.clone(), &required_hashes, InstallationMethod::PlayStore);

        if should_succeed {
            result.expect("integrity verdict should verify successfully");
        } else {
            let error = result.expect_err("integrity verdict should not verify");

            assert_matches!(error, IntegrityVerdictVerificationError::PlayStoreCertificateMismatch)
        }

        // When sideloading is allowed, any hashes should be completely ignored.
        verify_example_verdict_with_hashes(verdict, &required_hashes, InstallationMethod::SideloadOrPlayStore)
            .expect("integrity verdict should verify successfully");
    }

    #[rstest]
    fn test_verified_integrity_verdict_device_integrity_not_met_error(
        #[values(InstallationMethod::SideloadOrPlayStore, InstallationMethod::PlayStore)]
        installation_method: InstallationMethod,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.device_integrity.device_recognition_verdict.take();

        let error =
            verify_example_verdict(verdict, installation_method).expect_err("integrity verdict should not verify");

        assert_matches!(error, IntegrityVerdictVerificationError::DeviceIntegrityNotMet)
    }

    #[rstest]
    #[case(AppLicensingVerdict::Unlicensed, InstallationMethod::PlayStore, false)]
    #[case(AppLicensingVerdict::Unlicensed, InstallationMethod::SideloadOrPlayStore, true)]
    #[case(AppLicensingVerdict::Unevaluated, InstallationMethod::PlayStore, false)]
    #[case(AppLicensingVerdict::Unevaluated, InstallationMethod::SideloadOrPlayStore, true)]
    fn test_verified_integrity_verdict_no_app_entitlement_error(
        #[case] app_licensing_verdict: AppLicensingVerdict,
        #[case] installation_method: InstallationMethod,
        #[case] should_succeed: bool,
    ) {
        let mut verdict = EXAMPLE_VERDICT.clone();
        verdict.account_details.app_licensing_verdict = app_licensing_verdict;

        let result = verify_example_verdict(verdict, installation_method);

        if should_succeed {
            result.expect("integrity verdict should verify successfully");
        } else {
            let error = result.expect_err("integrity verdict should not verify");

            assert_matches!(
                error,
                IntegrityVerdictVerificationError::NoAppEntitlement(license_verdict)
                    if license_verdict == app_licensing_verdict
            )
        }
    }
}
