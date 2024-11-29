use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::LazyLock;

use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use parking_lot::RwLock;
use uuid::Uuid;

use apple_app_attest::AppIdentifier;
use apple_app_attest::MockAttestationCa;
use wallet_common::apple::MockAppleAttestedKey;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;

use super::AttestationError;
use super::AttestedKey;
use super::AttestedKeyHolder;
use super::GoogleAttestedKey;
use super::KeyWithAttestation;

static KEY_STATES: LazyLock<RwLock<HashMap<String, AttestedKeyState>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

enum AttestedKeyState {
    Generated,
    Attested {
        signing_key: SigningKey,
        next_counter: Arc<AtomicU32>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MockAppleHardwareAttestedKeyError {
    #[error("identifier was not generated or attested")]
    UnknownIdentifier,
    #[error("key with identifier was already attested")]
    KeyAttested,
    #[error("key with identifier was not attested")]
    KeyNotAttested,
    #[error("identifier is already in use in this process")]
    IdentifierInUse,
}

/// Implements [`AttestedKeyHolder`] and always returns [`MockAppleAttestedKey`],
/// types, based on the mock root CA included in the [`apple_app_attest`] crate.
pub struct MockAppleHardwareAttestedKeyHolder {
    pub ca: MockAttestationCa,
    pub app_identifier: AppIdentifier,
}

impl MockAppleHardwareAttestedKeyHolder {
    pub fn new_mock() -> Self {
        let ca = MockAttestationCa::new_mock();
        let app_identifier = AppIdentifier::new_mock();

        Self { ca, app_identifier }
    }
}

impl AttestedKeyHolder for MockAppleHardwareAttestedKeyHolder {
    type Error = MockAppleHardwareAttestedKeyError;
    type AppleKey = MockAppleAttestedKey;
    type GoogleKey = DeadGoogleAttestedKey;

    async fn generate(&self) -> Result<String, Self::Error> {
        let key_identifier = Uuid::new_v4().to_string();

        // Reserve a key identifier without actually generating the private key.
        let existing_state = KEY_STATES
            .write()
            .insert(key_identifier.clone(), AttestedKeyState::Generated);

        // Sanity check, this only happens on a key collision.
        assert!(existing_state.is_none());

        Ok(key_identifier)
    }

    async fn attest(
        &self,
        key_identifier: String,
        challenge: Vec<u8>,
    ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>> {
        let mut key_states = KEY_STATES.write();

        // The key's current state should be `AttestedKeyState::Generated`,
        // return the relevant error if this is not the case.
        if let AttestedKeyState::Attested { .. } =
            key_states
                .get(&key_identifier)
                .ok_or(AttestationError::new_unretryable(
                    MockAppleHardwareAttestedKeyError::UnknownIdentifier,
                ))?
        {
            return Err(AttestationError::new_unretryable(
                MockAppleHardwareAttestedKeyError::KeyAttested,
            ));
        }

        // Generate a new key and mock attestation data.
        let (key, attestation_data) =
            MockAppleAttestedKey::new_with_attestation(&self.ca, self.app_identifier.clone(), &challenge);

        // Update the global key state with both the key's private key and counter.
        key_states.insert(
            key_identifier,
            AttestedKeyState::Attested {
                signing_key: key.signing_key.clone(),
                next_counter: Arc::clone(&key.next_counter),
            },
        );

        let key_with_attestation = KeyWithAttestation::Apple { key, attestation_data };

        Ok(key_with_attestation)
    }

    fn attested_key(
        &self,
        key_identifier: String,
    ) -> Result<AttestedKey<Self::AppleKey, Self::GoogleKey>, Self::Error> {
        let key_states = KEY_STATES.read();

        // The key's current state should be `AttestedKeyState::Attested`.
        match key_states
            .get(&key_identifier)
            .ok_or(MockAppleHardwareAttestedKeyError::UnknownIdentifier)?
        {
            AttestedKeyState::Generated => Err(MockAppleHardwareAttestedKeyError::KeyNotAttested),
            AttestedKeyState::Attested {
                signing_key,
                next_counter,
            } => {
                // Use the Arc's reference counter as a proxy to determine if a `MockAppleAttestedKey`
                // already exists within memory, as this would own the second reference to it.
                if Arc::strong_count(next_counter) > 1 {
                    return Err(MockAppleHardwareAttestedKeyError::IdentifierInUse);
                }

                // Construct a `MockAppleAttestedKey` based on the private key and counter.
                let key = MockAppleAttestedKey {
                    app_identifier: self.app_identifier.clone(),
                    signing_key: signing_key.clone(),
                    next_counter: Arc::clone(next_counter),
                };

                Ok(AttestedKey::Apple(key))
            }
        }
    }
}

/// A faux type that does nothing, which we have to have because of [`AttestedKeyHolder::GoogleKey`].
#[derive(Debug)]
pub struct DeadGoogleAttestedKey;

impl GoogleAttestedKey for DeadGoogleAttestedKey {
    async fn delete(self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl SecureEcdsaKey for DeadGoogleAttestedKey {}

impl EcdsaKey for DeadGoogleAttestedKey {
    type Error = Infallible;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        unimplemented!()
    }

    async fn try_sign(&self, _msg: &[u8]) -> Result<Signature, Self::Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::super::test;
    use super::MockAppleHardwareAttestedKeyHolder;

    #[tokio::test]
    async fn test_mock_apple_hardware_attested_key_holder() {
        let mock_holder = MockAppleHardwareAttestedKeyHolder::new_mock();
        let challenge = b"this_is_a_challenge_string";
        let payload = b"This is a message that will be signed by the mock key.";

        test::create_and_verify_attested_key(
            &mock_holder,
            Some(&mock_holder.app_identifier),
            challenge.to_vec(),
            payload.to_vec(),
        )
        .await;
    }
}
