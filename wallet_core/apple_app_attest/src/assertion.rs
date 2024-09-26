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
    #[error("assertion did not validate: {0}")]
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

/// Represents the server request that is to be signed with the attested key. The `hash_data()` method produces a
/// serialized representation of that request, which must include the challenge provided by the server. This is part of
/// the contract of this trait. The `challenge()` method provides the server challenge itself.
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

        // The steps below are listed at:
        // https://developer.apple.com/documentation/devicecheck/validating-apps-that-connect-to-your-server#Verify-the-assertion

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
