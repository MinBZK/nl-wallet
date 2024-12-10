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
use wallet_common::account::serialization::DerSigningKey;
use wallet_common::apple::MockAppleAttestedKey;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;

use super::AttestationError;
use super::AttestedKey;
use super::AttestedKeyHolder;
use super::GoogleAttestedKey;
use super::KeyWithAttestation;

/// The global state of all keys managed by [`MockAppleHardwareAttestedKeyError`] instances.
static KEY_STATES: LazyLock<RwLock<HashMap<String, AttestedKeyState>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
#[cfg_attr(
    feature = "persistent_mock_apple_attested_key",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "persistent_mock_apple_attested_key", serde(rename_all = "snake_case"))]
enum AttestedKeyState {
    Generated,
    Attested {
        signing_key: DerSigningKey,
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
    #[error("mock error to be used in tests")]
    Mock,
}

#[derive(Debug)]
pub enum KeyHolderErrorScenario {
    NoError,
    UnretryableAttestationError,
    RetryableAttestationError,
}

/// Implements [`AttestedKeyHolder`] and always returns [`MockAppleAttestedKey`],
/// types, based on the mock root CA included in the [`apple_app_attest`] crate.
#[derive(Debug)]
pub struct MockAppleHardwareAttestedKeyHolder {
    key_states: &'static RwLock<HashMap<String, AttestedKeyState>>,
    pub ca: MockAttestationCa,
    pub app_identifier: AppIdentifier,
    pub error_scenario: KeyHolderErrorScenario,
}

impl MockAppleHardwareAttestedKeyHolder {
    pub fn generate(app_identifier: AppIdentifier) -> Self {
        Self {
            key_states: &KEY_STATES,
            ca: MockAttestationCa::generate(),
            app_identifier,
            error_scenario: KeyHolderErrorScenario::NoError,
        }
    }

    #[cfg(feature = "mock_apple_ca")]
    pub fn new_mock(app_identifier: AppIdentifier) -> Self {
        Self {
            key_states: &KEY_STATES,
            ca: MockAttestationCa::new_mock(),
            app_identifier,
            error_scenario: KeyHolderErrorScenario::NoError,
        }
    }

    /// Populate a particular identifier within the global state with a signing key and counter.
    pub fn populate_key_identifier(key_identifier: String, signing_key: SigningKey, next_counter: u32) {
        let existing_state = KEY_STATES.write().insert(
            key_identifier,
            AttestedKeyState::Attested {
                signing_key: DerSigningKey(signing_key),
                next_counter: Arc::new(AtomicU32::from(next_counter)),
            },
        );

        if existing_state.is_some() {
            panic!("key identifier is already populated")
        }
    }

    fn state_from_key(key: &MockAppleAttestedKey) -> AttestedKeyState {
        AttestedKeyState::Attested {
            signing_key: DerSigningKey(key.signing_key.clone()),
            next_counter: Arc::clone(&key.next_counter),
        }
    }

    /// Insert a new random key into the global state, bypassing attestation.
    pub fn random_key(&self) -> (MockAppleAttestedKey, String) {
        let key_identifier = Uuid::new_v4().to_string();
        let key = MockAppleAttestedKey::new_random(self.app_identifier.clone());

        let existing_state = self
            .key_states
            .write()
            .insert(key_identifier.clone(), Self::state_from_key(&key));

        // Sanity check, this only happens on a key collision.
        assert!(existing_state.is_none());

        (key, key_identifier)
    }
}

impl AttestedKeyHolder for MockAppleHardwareAttestedKeyHolder {
    type Error = MockAppleHardwareAttestedKeyError;
    type AppleKey = MockAppleAttestedKey;
    type GoogleKey = DeadGoogleAttestedKey;

    async fn generate(&self) -> Result<String, Self::Error> {
        let key_identifier = Uuid::new_v4().to_string();

        // Reserve a key identifier without actually generating the private key.
        let existing_state = self
            .key_states
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
        match self.error_scenario {
            KeyHolderErrorScenario::UnretryableAttestationError => {
                return Err(AttestationError::new_unretryable(
                    MockAppleHardwareAttestedKeyError::Mock,
                ))
            }
            KeyHolderErrorScenario::RetryableAttestationError => {
                return Err(AttestationError::new_retryable(MockAppleHardwareAttestedKeyError::Mock))
            }
            _ => {}
        };

        let mut key_states = self.key_states.write();

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
        key_states.insert(key_identifier, Self::state_from_key(&key));

        let key_with_attestation = KeyWithAttestation::Apple { key, attestation_data };

        Ok(key_with_attestation)
    }

    fn attested_key(
        &self,
        key_identifier: String,
    ) -> Result<AttestedKey<Self::AppleKey, Self::GoogleKey>, Self::Error> {
        let key_states = self.key_states.read();

        // The key's current state should be `AttestedKeyState::Attested`.
        match key_states
            .get(&key_identifier)
            .ok_or(MockAppleHardwareAttestedKeyError::UnknownIdentifier)?
        {
            AttestedKeyState::Generated => Err(MockAppleHardwareAttestedKeyError::KeyNotAttested),
            AttestedKeyState::Attested {
                signing_key: DerSigningKey(signing_key),
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
                    has_error: false,
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

#[cfg(feature = "persistent_mock_apple_attested_key")]
pub use persistent::*;

#[cfg(feature = "persistent_mock_apple_attested_key")]
mod persistent {
    use std::future::Future;
    use std::path::Path;
    use std::path::PathBuf;

    use futures::TryFutureExt;
    use tokio::fs;
    use tokio::sync::Mutex;

    use wallet_common::apple::AppleAssertion;
    use wallet_common::apple::AppleAttestedKey;

    use crate::utils::PlatformUtilities;

    use super::*;

    /// The global state of all keys managed by [`PersistentMockAppleHardwareAttestedKeyHolder`] instances.
    /// Note that this is distinct from the keys managed by [`MockAppleHardwareAttestedKeyHolder`].
    static PERSISTENT_KEY_STATES: LazyLock<RwLock<HashMap<String, AttestedKeyState>>> =
        LazyLock::new(|| RwLock::new(HashMap::new()));
    /// Async mutex around the filesystem backing store that holds [`PERSISTENT_KEY_STATES`].
    static KEY_STATES_FILE: Mutex<Option<PathBuf>> = Mutex::const_new(None);

    /// A wrapper around [`MockAppleHardwareAttestedKeyHolder`] that synchronizes the global key state
    /// with a JSON file on the filesystem. As the iOS simulator does not support attested keys, this
    /// type can be used in place of of `HardwareAttestedKeyHolder` in order to emulate generation and
    /// attestation of and signing by attested keys that survive termination and relaunch of the
    /// application. As it is specifically meant for use of the iOS simulator, the storage path is
    /// determined using [`PlatformUtilities`].
    pub struct PersistentMockAppleHardwareAttestedKeyHolder(MockAppleHardwareAttestedKeyHolder);

    impl PersistentMockAppleHardwareAttestedKeyHolder {
        const FILE_NAME: &str = "mock_apple_attested_keys.json";

        /// Initialization function that should be called exactly once within the lifetime of the application.
        /// It reads the JSON file (if present) and loads the global key state from it.
        pub async fn init<U>()
        where
            U: PlatformUtilities,
        {
            let mut key_states_file = KEY_STATES_FILE.lock().await;

            if key_states_file.is_some() {
                panic!("PersistentMockAppleHardwareAttestedKeyHolder::init() called more than once");
            }

            let storage_path = U::storage_path().await.expect("could not get application storage path");
            let file_path = storage_path.join(Path::new(Self::FILE_NAME));
            let path = file_path.as_path();

            let json = fs::try_exists(path)
                .and_then(|file_exists| async move {
                    match file_exists {
                        true => Some(fs::read(path).await).transpose(),
                        false => Ok(None),
                    }
                })
                .await
                .expect("could not read mock Apple attested keys JSON data file");

            let key_states = json
                .map(|json| {
                    serde_json::from_slice::<HashMap<String, AttestedKeyState>>(&json)
                        .expect("could not decode mock Apple attested keys JSON data file")
                })
                .unwrap_or_default();

            *PERSISTENT_KEY_STATES.write() = key_states;
            *key_states_file = Some(file_path);
        }

        pub fn new_mock(app_identifier: AppIdentifier) -> Self {
            let holder = MockAppleHardwareAttestedKeyHolder {
                key_states: &PERSISTENT_KEY_STATES,
                ca: MockAttestationCa::new_mock(),
                app_identifier,
                error_scenario: KeyHolderErrorScenario::NoError,
            };

            Self(holder)
        }

        /// Helper function that wraps an async operation. Before the operation a lock on the
        /// JSON key state file is obtained, which is used after the operation to write this
        /// state to the file.
        async fn with_key_state_write<F, T, E>(future: F) -> Result<T, E>
        where
            F: Future<Output = Result<T, E>>,
        {
            let key_states_file = KEY_STATES_FILE.lock().await;

            let Some(file_path) = key_states_file.as_deref() else {
                panic!("PersistentMockAppleHardwareAttestedKeyHolder::init() should be called first")
            };

            future
                .and_then(|key_identifier| async {
                    let json = serde_json::to_string_pretty(&*PERSISTENT_KEY_STATES.read())
                        .expect("could not encode mock Apple attested keys JSON file");
                    fs::write(file_path, json)
                        .await
                        .expect("could not write mock Apple attested keys JSON file");

                    Ok(key_identifier)
                })
                .await
        }
    }

    impl AttestedKeyHolder for PersistentMockAppleHardwareAttestedKeyHolder {
        type Error = MockAppleHardwareAttestedKeyError;
        type AppleKey = PersistentMockAppleAttestedKey;
        type GoogleKey = DeadGoogleAttestedKey;

        async fn generate(&self) -> Result<String, Self::Error> {
            Self::with_key_state_write(self.0.generate()).await
        }

        async fn attest(
            &self,
            key_identifier: String,
            challenge: Vec<u8>,
        ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>> {
            // Map the output from `KeyWithAttestation<MockAppleAttestedKey, DeadGoogleAttestedKey>`
            // to `KeyWithAttestation<PersistentMockAppleAttestedKey, DeadGoogleAttestedKey>`.
            Self::with_key_state_write(self.0.attest(key_identifier, challenge))
                .await
                .map(|key_with_attestation| match key_with_attestation {
                    KeyWithAttestation::Apple { key, attestation_data } => KeyWithAttestation::Apple {
                        key: PersistentMockAppleAttestedKey(key),
                        attestation_data,
                    },
                    KeyWithAttestation::Google {
                        key,
                        certificate_chain,
                        app_attestation_token,
                    } => KeyWithAttestation::Google {
                        key,
                        certificate_chain,
                        app_attestation_token,
                    },
                })
        }

        fn attested_key(
            &self,
            key_identifier: String,
        ) -> Result<AttestedKey<Self::AppleKey, Self::GoogleKey>, Self::Error> {
            // Map the output from `AttestedKey<MockAppleAttestedKey, DeadGoogleAttestedKey>`
            // to `AttestedKey<PersistentMockAppleAttestedKey, DeadGoogleAttestedKey>`.
            //
            // Note that we do not write the key state to disk here,
            // as `attested_key()` does not modify the global key state.
            self.0.attested_key(key_identifier).map(|key| match key {
                AttestedKey::Apple(key) => AttestedKey::Apple(PersistentMockAppleAttestedKey(key)),
                AttestedKey::Google(key) => AttestedKey::Google(key),
            })
        }
    }

    /// Wrapper type around `MockAppleAttestedKey` that writes the global key state to
    /// a JSON file whenever a key is used to sign a payload. This is necessary in
    /// order to update the counter of the key used for signing.
    #[derive(Debug)]
    pub struct PersistentMockAppleAttestedKey(MockAppleAttestedKey);

    impl AppleAttestedKey for PersistentMockAppleAttestedKey {
        type Error = <MockAppleAttestedKey as AppleAttestedKey>::Error;

        async fn sign(&self, payload: Vec<u8>) -> Result<AppleAssertion, Self::Error> {
            PersistentMockAppleHardwareAttestedKeyHolder::with_key_state_write(self.0.sign(payload)).await
        }
    }

    #[cfg(all(test, feature = "mock_utils"))]
    mod tests {
        use apple_app_attest::AppIdentifier;
        use apple_app_attest::MOCK_APPLE_TRUST_ANCHORS;

        use crate::attested_key::test;
        use crate::attested_key::test::AppleTestData;
        use crate::utils::mock::MockHardwareUtilities;

        use super::PersistentMockAppleHardwareAttestedKeyHolder;
        use super::KEY_STATES_FILE;

        #[tokio::test]
        async fn test_persistent_mock_apple_hardware_attested_key_holder() {
            PersistentMockAppleHardwareAttestedKeyHolder::init::<MockHardwareUtilities>().await;

            println!(
                "Mock Apple attested keys JSON file: {}",
                KEY_STATES_FILE.lock().await.as_deref().unwrap().to_string_lossy()
            );

            let app_identifier = AppIdentifier::new_mock();
            let mock_holder = PersistentMockAppleHardwareAttestedKeyHolder::new_mock(app_identifier);
            let challenge = b"this_is_a_challenge_string";
            let payload = b"This is a message that will be signed by the persistent mock key.";

            let apple_test_data = AppleTestData {
                app_identifier: &mock_holder.0.app_identifier,
                trust_anchors: &MOCK_APPLE_TRUST_ANCHORS,
            };

            test::create_and_verify_attested_key(
                &mock_holder,
                Some(apple_test_data),
                challenge.to_vec(),
                payload.to_vec(),
            )
            .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use apple_app_attest::AppIdentifier;

    use crate::attested_key::test;
    use crate::attested_key::test::AppleTestData;

    use super::MockAppleHardwareAttestedKeyHolder;

    #[tokio::test]
    async fn test_mock_apple_hardware_attested_key_holder() {
        let app_identifier = AppIdentifier::new_mock();
        let mock_holder = MockAppleHardwareAttestedKeyHolder::generate(app_identifier);
        let challenge = b"this_is_a_challenge_string";
        let payload = b"This is a message that will be signed by the mock key.";

        let apple_test_data = AppleTestData {
            app_identifier: &mock_holder.app_identifier,
            trust_anchors: &[mock_holder.ca.trust_anchor()],
        };

        test::create_and_verify_attested_key(
            &mock_holder,
            Some(apple_test_data),
            challenge.to_vec(),
            payload.to_vec(),
        )
        .await;
    }
}
