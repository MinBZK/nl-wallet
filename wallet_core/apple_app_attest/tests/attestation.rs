use assert_matches::assert_matches;
use chrono::{DateTime, Utc};
use const_decoder::{Decoder, Pem};
use webpki::TrustAnchor;

use apple_app_attest::{
    app_identifier::AppIdentifier,
    attestation::{Attestation, AttestationEnvironment, AttestationError, AttestationValidationError},
};

// Source: https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem
const APPLE_ROOT_CA: [u8; 549] = Pem::decode(include_bytes!("../assets/Apple_App_Attestation_Root_CA.pem"));

// Source: https://developer.apple.com/documentation/devicecheck/attestation-object-validation-guide
const TEST_ATTESTATION: [u8; 5637] = Decoder::Base64.decode(include_bytes!("../assets/test_attestation_object.b64"));
const TEST_CHALLENGE: &[u8] = b"test_server_challenge";
const TEST_ATTESTATION_VALID_DATE: &str = "2024-04-18T12:00:00Z";
const TEST_APP_PREFIX: &str = "0352187391";
const TEST_APP_BUNDLE_ID: &str = "com.apple.example_app_attest";

fn prepare_attestation_parameters() -> (TrustAnchor<'static>, DateTime<Utc>, AppIdentifier) {
    let trust_anchor = TrustAnchor::try_from_cert_der(&APPLE_ROOT_CA).unwrap();
    let time = DateTime::parse_from_rfc3339(TEST_ATTESTATION_VALID_DATE)
        .unwrap()
        .to_utc();
    let app_identifier = AppIdentifier::new(TEST_APP_PREFIX, TEST_APP_BUNDLE_ID);

    (trust_anchor, time, app_identifier)
}

#[test]
fn test_attestation() {
    let (trust_anchor, time, app_identifier) = prepare_attestation_parameters();

    let (_, _, counter) = Attestation::parse_and_verify(
        &TEST_ATTESTATION,
        &[trust_anchor],
        time,
        TEST_CHALLENGE,
        &app_identifier,
        AttestationEnvironment::Production,
    )
    .expect("attestation object should be valid");

    assert_eq!(counter, 0);
}

#[test]
fn test_attestation_validation_error_certificate_chain() {
    let (trust_anchor, _, app_identifier) = prepare_attestation_parameters();
    let time = DateTime::parse_from_rfc3339("2024-04-21T12:00:00Z").unwrap().to_utc();

    let error = Attestation::parse_and_verify(
        &TEST_ATTESTATION,
        &[trust_anchor],
        time,
        TEST_CHALLENGE,
        &app_identifier,
        AttestationEnvironment::Production,
    )
    .expect_err("attestation object should not be valid");

    assert_matches!(
        error,
        AttestationError::Validation(AttestationValidationError::CertificateChain(_))
    );
}

#[test]
fn test_attestation_validation_error_nonce_mismatch() {
    let (trust_anchor, time, app_identifier) = prepare_attestation_parameters();

    let error = Attestation::parse_and_verify(
        &TEST_ATTESTATION,
        &[trust_anchor],
        time,
        b"invalid_server_challenge",
        &app_identifier,
        AttestationEnvironment::Production,
    )
    .expect_err("attestation object should not be valid");

    assert_matches!(
        error,
        AttestationError::Validation(AttestationValidationError::NonceMismatch)
    );
}

#[test]
fn test_attestation_validation_error_rp_id_mismatch() {
    let (trust_anchor, time, _) = prepare_attestation_parameters();
    let app_identifier = AppIdentifier::new(TEST_APP_PREFIX, "com.apple.different_app_attest");

    let error = Attestation::parse_and_verify(
        &TEST_ATTESTATION,
        &[trust_anchor],
        time,
        TEST_CHALLENGE,
        &app_identifier,
        AttestationEnvironment::Production,
    )
    .expect_err("attestation object should not be valid");

    assert_matches!(
        error,
        AttestationError::Validation(AttestationValidationError::RpIdMismatch)
    );
}

#[test]
fn test_attestation_validation_error_environment_mismatch() {
    let (trust_anchor, time, app_identifier) = prepare_attestation_parameters();

    let error = Attestation::parse_and_verify(
        &TEST_ATTESTATION,
        &[trust_anchor],
        time,
        TEST_CHALLENGE,
        &app_identifier,
        AttestationEnvironment::Development,
    )
    .expect_err("attestation object should not be valid");

    assert_matches!(
        error,
        AttestationError::Validation(AttestationValidationError::EnvironmentMismatch {
            expected,
            received
        }) if expected == AttestationEnvironment::Development.aaguid()
            && received == AttestationEnvironment::Production.aaguid()
    );
}

// Note that `AttestationValidationError` and `KeyIdentifierMismatch` currently cannot be tested, as this requires
// modifying `auth_data`, which invalidates the calculated nonce.
