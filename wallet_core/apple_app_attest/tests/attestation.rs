use assert_matches::assert_matches;
use chrono::{DateTime, Utc};
use const_decoder::{Decoder, Pem};
use rstest::{fixture, rstest};
use webpki::TrustAnchor;

use apple_app_attest::{
    AppIdentifier, Attestation, AttestationEnvironment, AttestationError, AttestationValidationError,
};

// Source: https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem
const APPLE_ROOT_CA: [u8; 549] = Pem::decode(include_bytes!("../assets/Apple_App_Attestation_Root_CA.pem"));

// Source: https://developer.apple.com/documentation/devicecheck/attestation-object-validation-guide
const TEST_ATTESTATION: [u8; 5637] = Decoder::Base64.decode(include_bytes!("../assets/test_attestation_object.b64"));

/// The parameters used to validate an attestation.
struct AttestationParameters {
    trust_anchor: TrustAnchor<'static>,
    time: DateTime<Utc>,
    challenge: Vec<u8>,
    app_identifier: AppIdentifier,
    environment: AttestationEnvironment,
}

/// The default [`AttestationParameters`] can be used to validate the sample attestation provided by Apple.
impl Default for AttestationParameters {
    fn default() -> Self {
        let trust_anchor = TrustAnchor::try_from_cert_der(&APPLE_ROOT_CA).unwrap();
        let time = DateTime::parse_from_rfc3339("2024-04-18T12:00:00Z").unwrap().to_utc();
        let challenge = b"test_server_challenge".to_vec();
        let app_identifier = AppIdentifier::new("0352187391", "com.apple.example_app_attest");
        let environment = AttestationEnvironment::Production;

        Self {
            trust_anchor,
            time,
            challenge,
            app_identifier,
            environment,
        }
    }
}

#[fixture]
fn attestation_data() -> &'static [u8] {
    &TEST_ATTESTATION
}

// Vary the default parameters for different error scenarios.

fn different_time_parameters() -> AttestationParameters {
    AttestationParameters {
        time: DateTime::parse_from_rfc3339("2024-04-21T12:00:00Z").unwrap().to_utc(),
        ..Default::default()
    }
}

fn different_challenge_parameters() -> AttestationParameters {
    AttestationParameters {
        challenge: b"invalid_server_challenge".to_vec(),
        ..Default::default()
    }
}

fn different_app_id_parameters() -> AttestationParameters {
    AttestationParameters {
        app_identifier: AppIdentifier::new("0352187391", "com.apple.different_app_attest"),
        ..Default::default()
    }
}

fn different_environment_parameters() -> AttestationParameters {
    AttestationParameters {
        environment: AttestationEnvironment::Development,
        ..Default::default()
    }
}

/// Perform the tests against the provided attestation, using different parameters each time. Note that
/// `AttestationValidationError` and `KeyIdentifierMismatch` currently cannot be tested, as this requires modifying
/// `auth_data`, which invalidates the calculated nonce.
#[rstest]
#[case::success(AttestationParameters::default(), true, |_| {})]
#[case::validation_error_certificate_chain(
    different_time_parameters(),
    false,
    |error| assert_matches!(error, AttestationError::Validation(AttestationValidationError::CertificateChain(_)))
)]
#[case::validation_error_nonce(
    different_challenge_parameters(),
    false,
    |error| assert_matches!(error, AttestationError::Validation(AttestationValidationError::NonceMismatch))
)]
#[case::validation_error_rp_id(
    different_app_id_parameters(),
    false,
    |error| assert_matches!(error, AttestationError::Validation(AttestationValidationError::RpIdMismatch))
)]
#[case::validation_error_environment(
    different_environment_parameters(),
    false,
    |error| assert_matches!(error, AttestationError::Validation(AttestationValidationError::EnvironmentMismatch {
        expected,
        received
    }) if expected == AttestationEnvironment::Development.aaguid()
        && received == AttestationEnvironment::Production.aaguid())
)]
fn test_attestation<F>(
    attestation_data: &[u8],
    #[case] parameters: AttestationParameters,
    #[case] should_succeed: bool,
    #[case] error_matcher: F,
) where
    F: FnOnce(AttestationError),
{
    let result = Attestation::parse_and_verify(
        attestation_data,
        &[parameters.trust_anchor],
        parameters.time,
        &parameters.challenge,
        &parameters.app_identifier,
        parameters.environment,
    );

    if should_succeed {
        let (_, _, counter) = result.expect("attestation object should be valid");

        assert_eq!(counter, 0);
    } else {
        let error = result.expect_err("attestation object should not be valid");

        error_matcher(error);
    }
}
