use std::sync::LazyLock;

use assert_matches::assert_matches;
use ciborium::Value;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use passkey_types::ctap2::AuthenticatorData;
use rand_core::OsRng;
use rstest::{fixture, rstest};
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

/// The parameters used to generate an assertion.
struct AssertionParameters {
    private_key: SigningKey,
    client_data: MockClientData,
    app_identifier: AppIdentifier,
    counter: u32,
    challenge: Vec<u8>,
}

static DEFAULT_PRIVATE_KEY: LazyLock<SigningKey> = LazyLock::new(|| SigningKey::random(&mut OsRng));

/// Set some default parameters, with a static randomly generated private key.
impl Default for AssertionParameters {
    fn default() -> Self {
        let private_key = DEFAULT_PRIVATE_KEY.clone();
        let challenge = b"this is the challenge.".to_vec();
        let client_data = MockClientData {
            message: "This is a message.".to_string(),
            challenge: challenge.clone(),
        };
        let app_identifier = AppIdentifier::new("1234567890", "com.example.app");
        let counter = 1337;

        Self {
            private_key,
            client_data,
            app_identifier,
            counter,
            challenge,
        }
    }
}

impl AssertionParameters {
    fn verifying_key(&self) -> &VerifyingKey {
        self.private_key.verifying_key()
    }
}

/// Unfortunately Apple does not provide an example assertion so we have to make one ourselves. This is based on the
/// default parameters.
#[fixture]
#[once]
fn assertion_data() -> Vec<u8> {
    let parameters = AssertionParameters::default();
    let authenticator_data =
        AuthenticatorData::new(parameters.app_identifier.as_ref(), Some(parameters.counter)).to_vec();

    let nonce = Sha256::new()
        .chain_update(&authenticator_data)
        .chain_update(Sha256::digest(parameters.client_data.hash_data().unwrap().as_ref()))
        .finalize();
    let signature: Signature = parameters.private_key.try_sign(&nonce).unwrap();

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

// Vary the default parameters for different error scenarios.

fn different_private_key_parameters() -> AssertionParameters {
    AssertionParameters {
        private_key: SigningKey::random(&mut OsRng),
        ..Default::default()
    }
}

fn different_app_id_parameters() -> AssertionParameters {
    AssertionParameters {
        app_identifier: AppIdentifier::new("1234567890", "com.example.other_app"),
        ..Default::default()
    }
}

fn lower_counter_parameters() -> AssertionParameters {
    let default_parameters = AssertionParameters::default();

    AssertionParameters {
        counter: default_parameters.counter + 1,
        ..default_parameters
    }
}

fn different_challenge_parameters() -> AssertionParameters {
    AssertionParameters {
        challenge: b"this is as different challenge.".to_vec(),
        ..Default::default()
    }
}

/// Perform the tests against the generated assertion, using different parameters each time.
#[rstest]
#[case::success(AssertionParameters::default(), true, |_| {})]
#[case::validation_error_signature(
    different_private_key_parameters(),
    false,
    |error| assert_matches!(error, AssertionError::Validation(AssertionValidationError::Signature(_)))
)]
#[case::validation_error_rp_id(
    different_app_id_parameters(),
    false,
    |error| assert_matches!(error, AssertionError::Validation(AssertionValidationError::RpIdMismatch))
)]
#[case::validation_error_counter(
    lower_counter_parameters(),
    false,
    |error| assert_matches!(error, AssertionError::Validation(AssertionValidationError::CounterTooLow {
        previous,
        received
    }) if previous == 1337 && received == 1337)
)]
#[case::validation_error_challenge(
    different_challenge_parameters(),
    false,
    |error| assert_matches!(error, AssertionError::Validation(AssertionValidationError::ChallengeMismatch))
)]
fn test_assertion<F>(
    assertion_data: &[u8],
    #[case] parameters: AssertionParameters,
    #[case] should_succeed: bool,
    #[case] error_matcher: F,
) where
    F: FnOnce(AssertionError),
{
    let result = Assertion::parse_and_verify(
        assertion_data,
        &parameters.client_data,
        parameters.verifying_key(),
        &parameters.app_identifier,
        parameters.counter - 1,
        &parameters.challenge,
    );

    if should_succeed {
        let (_, counter) = result.expect("assertion should be valid");

        assert_eq!(counter, parameters.counter);
    } else {
        let error = result.expect_err("assertion should not be valid");

        error_matcher(error);
    }
}
