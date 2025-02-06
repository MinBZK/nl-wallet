use std::error::Error;

use derive_more::AsRef;
use derive_more::Deref;
use derive_more::From;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::TryFromInto;
use sha2::Digest;
use sha2::Sha256;

use crate::app_identifier::AppIdentifier;
use crate::auth_data::TruncatedAuthenticatorDataWithSource;

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
    #[error("could not get client data challenge: {0}")]
    ClientDataChallenge(Box<dyn Error + Send + Sync>),
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
    fn challenge(&self) -> Result<impl AsRef<[u8]>, Self::Error>;
}

// TODO: Use the type of the same name from `wallet_common` instead.
#[derive(Debug, Clone, AsRef)]
pub struct DerSignature(Signature);

impl TryFrom<Vec<u8>> for DerSignature {
    type Error = p256::ecdsa::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let signature = Signature::from_der(&value)?;

        Ok(DerSignature(signature))
    }
}

#[cfg(feature = "serialize")]
impl From<DerSignature> for Vec<u8> {
    fn from(value: DerSignature) -> Self {
        let DerSignature(signature) = value;

        signature.to_der().as_bytes().to_vec()
    }
}

#[derive(Debug, Clone, Copy, Default, From, Deref)]
pub struct AssertionCounter(u32);

#[derive(Debug, Clone, AsRef)]
pub struct VerifiedAssertion(Assertion);

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct Assertion {
    #[serde_as(as = "TryFromInto<Vec<u8>>")]
    pub signature: DerSignature,
    #[serde_as(as = "TryFromInto<Vec<u8>>")]
    pub authenticator_data: TruncatedAuthenticatorDataWithSource,
}

impl Assertion {
    pub fn parse(bytes: &[u8]) -> Result<Self, AssertionError> {
        let assertion = ciborium::from_reader(bytes).map_err(AssertionDecodingError::Cbor)?;

        Ok(assertion)
    }
}

impl VerifiedAssertion {
    pub fn parse_and_verify(
        bytes: &[u8],
        client_data: &impl ClientData,
        public_key: &VerifyingKey,
        app_identifier: &AppIdentifier,
        previous_counter: AssertionCounter,
        expected_challenge: &[u8],
    ) -> Result<(Self, AssertionCounter), AssertionError> {
        let assertion = Assertion::parse(bytes)?;

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
        if counter <= *previous_counter {
            return Err(AssertionValidationError::CounterTooLow {
                previous: *previous_counter,
                received: counter,
            })?;
        }

        // 6. Verify that the embedded challenge in the client data matches the earlier challenge to the client.

        if client_data
            .challenge()
            .map_err(|error| AssertionDecodingError::ClientDataChallenge(Box::new(error)))?
            .as_ref()
            != expected_challenge
        {
            return Err(AssertionValidationError::ChallengeMismatch)?;
        }

        Ok((VerifiedAssertion(assertion), AssertionCounter(counter)))
    }
}

#[cfg(feature = "mock")]
mod mock {
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::SigningKey;
    use passkey_types::ctap2::AuthenticatorData;
    use sha2::Digest;
    use sha2::Sha256;

    use crate::app_identifier::AppIdentifier;
    use crate::auth_data::AuthenticatorDataWithSource;

    use super::Assertion;
    use super::AssertionCounter;

    impl Assertion {
        /// Generate a mock [`Assertion`] based on a private key and other parameters.
        pub fn new_mock(
            private_key: &SigningKey,
            app_identifier: &AppIdentifier,
            counter: AssertionCounter,
            client_data: &[u8],
        ) -> Self {
            let authenticator_data =
                AuthenticatorDataWithSource::from(AuthenticatorData::new(app_identifier.as_ref(), Some(*counter)));

            let nonce = Sha256::new()
                .chain_update(authenticator_data.source())
                .chain_update(Sha256::digest(client_data))
                .finalize();
            let signature = super::DerSignature(private_key.try_sign(&nonce).unwrap());

            Self {
                signature,
                authenticator_data,
            }
        }

        pub fn new_mock_bytes(
            private_key: &SigningKey,
            app_identifier: &AppIdentifier,
            counter: AssertionCounter,
            client_data: &[u8],
        ) -> Vec<u8> {
            let assertion = Self::new_mock(private_key, app_identifier, counter, client_data);

            let mut bytes = Vec::new();
            ciborium::into_writer(&assertion, &mut bytes).unwrap();

            bytes
        }
    }
}
