use assert_matches::assert_matches;
use ciborium::Value;
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use passkey_types::ctap2::AuthenticatorData;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use apple_app_attest::{AppIdentifier, Assertion, AssertionError, AssertionValidationError, ClientData};

#[derive(Debug, Serialize, Deserialize)]
struct MockClientData {
    message: String,
    challenge: Vec<u8>,
}

impl ClientData for MockClientData {
    type Error = serde_json::Error;

    fn hash_data(&self) -> Result<impl AsRef<[u8]>, Self::Error> {
        let json = serde_json::to_vec(self)?;

        Ok(json)
    }

    fn challenge(&self) -> impl AsRef<[u8]> {
        &self.challenge
    }
}

fn prepare_assertion_parameters() -> (SigningKey, MockClientData, AppIdentifier, Vec<u8>) {
    let private_key = SigningKey::random(&mut OsRng);
    let challenge = b"this is the challenge.".to_vec();
    let client_data = MockClientData {
        message: "This is a message.".to_string(),
        challenge: challenge.clone(),
    };
    let app_identifier = AppIdentifier::new("1234567890", "com.example.app");

    (private_key, client_data, app_identifier, challenge)
}

// Unfortunately Apple does not provide an example assertion so we have to make one ourselves.
fn generate_assertion_data(
    private_key: &SigningKey,
    client_data: &impl ClientData,
    app_identifier: &AppIdentifier,
    counter: u32,
) -> Vec<u8> {
    let authenticator_data = AuthenticatorData::new(app_identifier.as_ref(), Some(counter)).to_vec();

    let nonce = Sha256::new()
        .chain_update(&authenticator_data)
        .chain_update(Sha256::digest(client_data.hash_data().unwrap().as_ref()))
        .finalize();
    let signature: Signature = private_key.try_sign(&nonce).unwrap();

    let map = Value::Map(
        [
            (
                Value::Text("signature".to_string()),
                Value::Bytes(signature.to_der().as_bytes().to_vec()),
            ),
            (
                Value::Text("authenticatorData".to_string()),
                Value::Bytes(authenticator_data),
            ),
        ]
        .to_vec(),
    );

    let mut bytes = Vec::<u8>::new();
    ciborium::into_writer(&map, &mut bytes).unwrap();

    bytes
}

#[test]
fn test_assertion() {
    let (private_key, client_data, app_identifier, challenge) = prepare_assertion_parameters();

    let assertion_data = generate_assertion_data(&private_key, &client_data, &app_identifier, 1337);

    let (_, parsed_counter) = Assertion::parse_and_verify(
        &assertion_data,
        &client_data,
        private_key.verifying_key(),
        &app_identifier,
        1336,
        &challenge,
    )
    .expect("assertion should be valid");

    assert_eq!(parsed_counter, 1337);
}

#[test]
fn test_assertion_validation_error_signature() {
    let (private_key, client_data, app_identifier, challenge) = prepare_assertion_parameters();

    let assertion_data = generate_assertion_data(&private_key, &client_data, &app_identifier, 1337);
    let other_private_key = SigningKey::random(&mut OsRng);

    let error = Assertion::parse_and_verify(
        &assertion_data,
        &client_data,
        other_private_key.verifying_key(),
        &app_identifier,
        1336,
        &challenge,
    )
    .expect_err("assertion should not be valid");

    assert_matches!(
        error,
        AssertionError::Validation(AssertionValidationError::Signature(_))
    );
}

#[test]
fn test_assertion_validation_error_rp_id_mismatch() {
    let (private_key, client_data, app_identifier, challenge) = prepare_assertion_parameters();

    let assertion_data = generate_assertion_data(&private_key, &client_data, &app_identifier, 1337);
    let other_app_identifier = AppIdentifier::new("1234567890", "com.example.other_app");

    let error = Assertion::parse_and_verify(
        &assertion_data,
        &client_data,
        private_key.verifying_key(),
        &other_app_identifier,
        1336,
        &challenge,
    )
    .expect_err("assertion should not be valid");

    assert_matches!(
        error,
        AssertionError::Validation(AssertionValidationError::RpIdMismatch)
    );
}

#[test]
fn test_assertion_validation_error_counter_too_low() {
    let (private_key, client_data, app_identifier, challenge) = prepare_assertion_parameters();

    let assertion_data = generate_assertion_data(&private_key, &client_data, &app_identifier, 1336);

    let error = Assertion::parse_and_verify(
        &assertion_data,
        &client_data,
        private_key.verifying_key(),
        &app_identifier,
        1336,
        &challenge,
    )
    .expect_err("assertion should not be valid");

    assert_matches!(
        error,
        AssertionError::Validation(AssertionValidationError::CounterTooLow {
            previous,
            received
        }) if previous == 1336 && received == 1336
    );
}

#[test]
fn test_assertion_validation_error_challenge_mismatch() {
    let (private_key, client_data, app_identifier, _) = prepare_assertion_parameters();

    let assertion_data = generate_assertion_data(&private_key, &client_data, &app_identifier, 1337);

    let error = Assertion::parse_and_verify(
        &assertion_data,
        &client_data,
        private_key.verifying_key(),
        &app_identifier,
        1336,
        b"this is as different challenge.",
    )
    .expect_err("assertion should not be valid");

    assert_matches!(
        error,
        AssertionError::Validation(AssertionValidationError::ChallengeMismatch)
    );
}
