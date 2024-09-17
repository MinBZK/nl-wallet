use chrono::DateTime;
use const_decoder::{Decoder, Pem};
use webpki::TrustAnchor;

use apple_app_attest::{
    app_identifier::AppIdentifier,
    attestation::{Attestation, AttestationEnvironment},
};

// Source: https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem
const APPLE_ROOT_CA: [u8; 549] = Pem::decode(include_bytes!("../assets/Apple_App_Attestation_Root_CA.pem"));

// Source: https://developer.apple.com/documentation/devicecheck/attestation-object-validation-guide
const TEST_ATTESTATION: [u8; 5637] = Decoder::Base64.decode(include_bytes!("../assets/test_attestation_object.b64"));
const TEST_CHALLENGE: &[u8] = b"test_server_challenge";
const TEST_ATTESTATION_VALID_DATE: &str = "2024-04-18T12:00:00Z";
const TEST_APP_PREFIX: &str = "0352187391";
const TEST_APP_BUNDLE_ID: &str = "com.apple.example_app_attest";

#[test]
fn test_attestation() {
    let trust_anchor = TrustAnchor::try_from_cert_der(&APPLE_ROOT_CA).unwrap();
    let time = DateTime::parse_from_rfc3339(TEST_ATTESTATION_VALID_DATE)
        .unwrap()
        .to_utc();
    let app_id = AppIdentifier::new(TEST_APP_PREFIX, TEST_APP_BUNDLE_ID);

    let (_, _, counter) = Attestation::parse_and_verify(
        &TEST_ATTESTATION,
        &[trust_anchor],
        time,
        TEST_CHALLENGE,
        &app_id,
        AttestationEnvironment::Production,
    )
    .expect("attestation object should be valid");

    assert_eq!(counter, 0);
}
