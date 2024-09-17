use ciborium::Value;
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use passkey_types::ctap2::AuthenticatorData;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use apple_app_attest::{
    app_identifier::AppIdentifier,
    assertion::{Assertion, ClientData},
};

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
    let private_key = SigningKey::random(&mut OsRng);
    let challenge = b"this is the challenge.".to_vec();
    let client_data = MockClientData {
        message: "This is a message.".to_string(),
        challenge: challenge.clone(),
    };
    let app_identifier = AppIdentifier::new("1234567890", "com.example.app");
    let counter = 1337;

    let assertion_data = generate_assertion_data(&private_key, &client_data, &app_identifier, counter);

    let (_, parsed_counter) = Assertion::parse_and_verify(
        &assertion_data,
        &client_data,
        private_key.verifying_key(),
        &app_identifier,
        1336,
        &challenge,
    )
    .expect("assertion should be valid");

    assert_eq!(parsed_counter, counter);
}
