use std::error::Error;

use nutype::nutype;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use serde::Deserialize;
use serde_with::{serde_as, TryFromInto};
use sha2::{Digest, Sha256};

use crate::{app_identifier::AppIdentifier, auth_data::AuthenticatorDataWithSource};

#[derive(Debug, thiserror::Error)]
pub enum AssertionError {
    #[error("assertion could not be decoded: {0}")]
    Decoding(#[from] AssertionDecodingError),
    #[error("asseration did not validate: {0}")]
    Validation(#[from] AssertionValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum AssertionDecodingError {
    #[error("deserializing assertion CBOR failed: {0}")]
    Cbor(#[source] ciborium::de::Error<std::io::Error>),
    #[error("could not get client data hash: {0}")]
    ClientDataHash(Box<dyn Error + Send + Sync>),
    #[error("counter is not present in authenticator data")]
    CounterMissing,
}

#[derive(Debug, thiserror::Error)]
pub enum AssertionValidationError {
    #[error("signature does not validate: {0}")]
    Signature(#[source] p256::ecdsa::Error),
    #[error("relying party identifier does not match calculated value")]
    RpIdMismatch,
    #[error("counter does not exceed previous value {previous}, received: {received}")]
    CounterTooLow { previous: u32, received: u32 },
    #[error("challenge in client data does not match expected challenge")]
    ChallengeMismatch,
}

pub trait ClientData {
    type Error: Error + Send + Sync + 'static;

    fn hash_data(&self) -> Result<impl AsRef<[u8]>, Self::Error>;
    fn challenge(&self) -> impl AsRef<[u8]>;
}

#[nutype(derive(Debug, Clone, AsRef))]
pub struct DerSignature(Signature);

impl TryFrom<Vec<u8>> for DerSignature {
    type Error = p256::ecdsa::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let signature = Signature::from_der(&value)?;

        Ok(DerSignature::new(signature))
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assertion {
    #[serde_as(as = "TryFromInto<Vec<u8>>")]
    pub signature: DerSignature,
    #[serde_as(as = "TryFromInto<Vec<u8>>")]
    pub authenticator_data: AuthenticatorDataWithSource,
}

impl Assertion {
    pub fn parse_and_verify(
        bytes: &[u8],
        client_data: &impl ClientData,
        public_key: &VerifyingKey,
        app_identifier: &AppIdentifier,
        previous_counter: u32,
        challenge: &[u8],
    ) -> Result<(Self, u32), AssertionError> {
        let assertion: Self = ciborium::from_reader(bytes).map_err(AssertionDecodingError::Cbor)?;

        // 1. Compute clientDataHash as the SHA256 hash of clientData.

        let client_data_hash = Sha256::digest(
            client_data
                .hash_data()
                .map_err(|error| AssertionDecodingError::ClientDataHash(Box::new(error)))?,
        );

        // 2. Concatenate authenticatorData and clientDataHash, and apply a SHA256 hash over the result to form nonce.

        let nonce = Sha256::new()
            .chain_update(assertion.authenticator_data.source())
            .chain_update(client_data_hash)
            .finalize();

        // 3. Use the public key that you store from the attestation object to verify that the assertion’s signature is
        //    valid for nonce.

        public_key
            .verify(&nonce, assertion.signature.as_ref())
            .map_err(AssertionValidationError::Signature)?;

        // 4. Compute the SHA256 hash of the client’s App ID, and verify that it matches the RP ID in the authenticator
        //    data.

        if assertion.authenticator_data.as_ref().rp_id_hash() != app_identifier.sha256_hash() {
            return Err(AssertionValidationError::RpIdMismatch)?;
        }

        // 5. Verify that the authenticator data’s counter value is greater than the value from the previous assertion,
        //    or greater than 0 on the first assertion.

        let counter = assertion
            .authenticator_data
            .as_ref()
            .counter
            .ok_or(AssertionDecodingError::CounterMissing)?;
        if counter <= previous_counter {
            return Err(AssertionValidationError::CounterTooLow {
                previous: previous_counter,
                received: counter,
            })?;
        }

        // 6. Verify that the embedded challenge in the client data matches the earlier challenge to the client.

        if client_data.challenge().as_ref() != challenge {
            return Err(AssertionValidationError::ChallengeMismatch)?;
        }

        Ok((assertion, counter))
    }
}

#[cfg(test)]
mod tests {
    use ciborium::Value;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use passkey_types::ctap2::AuthenticatorData;
    use rand_core::OsRng;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::app_identifier::AppIdentifier;

    use super::{Assertion, ClientData};

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
}
