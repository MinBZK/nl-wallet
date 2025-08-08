use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use cfg_if::cfg_if;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Signer;
use parking_lot::RwLock;
use rand_core::OsRng;
use uuid::Uuid;

use crypto::keys::EcdsaKey;
use crypto::keys::SecureEcdsaKey;

use super::AppleAssertion;
use super::AppleAttestedKey;
use super::AttestationError;
use super::AttestedKey;
use super::AttestedKeyHolder;
use super::GoogleAttestedKey;
use super::KeyWithAttestation;

type KeyStates = Arc<RwLock<HashMap<String, AttestedKeyState>>>;

#[derive(Debug)]
struct SharedSigningKey {
    signing_key: Arc<SigningKey>,
    #[cfg(feature = "persistent_mock_attested_key")]
    der: Vec<u8>,
}

impl SharedSigningKey {
    fn new(signing_key: Arc<SigningKey>) -> Self {
        #[cfg(feature = "persistent_mock_attested_key")]
        let der = {
            use p256::pkcs8::EncodePrivateKey;

            signing_key.to_pkcs8_der().unwrap().as_bytes().to_vec()
        };

        Self {
            signing_key,
            #[cfg(feature = "persistent_mock_attested_key")]
            der,
        }
    }
}

impl AsRef<Arc<SigningKey>> for SharedSigningKey {
    fn as_ref(&self) -> &Arc<SigningKey> {
        &self.signing_key
    }
}

#[cfg_attr(feature = "persistent_mock_attested_key", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug)]
#[cfg_attr(
    feature = "persistent_mock_attested_key",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "persistent_mock_attested_key", serde(rename_all = "snake_case"))]
enum AttestedKeyState {
    Generated,
    Attested {
        #[cfg_attr(
            feature = "persistent_mock_attested_key",
            serde_as(as = "serde_with::base64::Base64")
        )]
        signing_key: SharedSigningKey,

        // This is set to `None` for Google attested keys.
        next_counter: Option<Arc<AtomicU32>>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MockHardwareAttestedKeyError {
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
pub enum KeyHolderType {
    #[cfg(feature = "mock_attested_key_apple")]
    Apple {
        ca: apple_app_attest::MockAttestationCa,
        environment: apple_app_attest::AttestationEnvironment,
        app_identifier: apple_app_attest::AppIdentifier,
    },
    #[cfg(feature = "mock_attested_key_google")]
    Google {
        ca_chain: android_attest::mock_chain::MockCaChain,
    },
}

#[derive(Debug, Default)]
pub enum KeyHolderErrorScenario {
    #[default]
    NoError,
    GenerateError,
    UnretryableAttestationError,
    RetryableAttestationError,
    SigningError,
}

/// Implements [`AttestedKeyHolder`] and returns either [`MockAppleAttestedKey`] types, based on
/// the mock root CA included in the [`apple_app_attest`] crate, or [`MockGoogleAttestedKey`]
/// types based on the mock root CA chain included in the [`android_attest`] crate.
#[derive(Debug)]
pub struct MockHardwareAttestedKeyHolder {
    key_states: KeyStates,
    pub holder_type: KeyHolderType,
    pub error_scenario: KeyHolderErrorScenario,
}

impl MockHardwareAttestedKeyHolder {
    fn new(holder_type: KeyHolderType) -> Self {
        Self {
            key_states: Arc::new(RwLock::new(HashMap::new())),
            holder_type,
            error_scenario: KeyHolderErrorScenario::default(),
        }
    }

    pub fn is_attested(&self, key_identifier: &str) -> bool {
        self.key_states.read().contains_key(key_identifier)
    }

    /// Populate a particular identifier within the state with a signing key
    /// and, for Apple attested keys, a default initial counter.
    pub fn populate_key_identifier(&self, key_identifier: String, signing_key: SigningKey) {
        let next_counter = match self.holder_type {
            #[cfg(feature = "mock_attested_key_apple")]
            KeyHolderType::Apple { .. } => Some(Arc::new(AtomicU32::from(1))),
            #[cfg(feature = "mock_attested_key_google")]
            KeyHolderType::Google { .. } => None,
        };
        let existing_state = self.key_states.write().insert(
            key_identifier,
            AttestedKeyState::Attested {
                signing_key: SharedSigningKey::new(Arc::new(signing_key)),
                next_counter,
            },
        );

        if existing_state.is_some() {
            panic!("key identifier is already populated")
        }
    }

    /// Insert a new random key into the global state, bypassing attestation.
    pub fn random_key(&self) -> (AttestedKey<MockAppleAttestedKey, MockGoogleAttestedKey>, String) {
        let key_identifier = Uuid::new_v4().to_string();

        let has_error = matches!(self.error_scenario, KeyHolderErrorScenario::SigningError);
        let (key, state) = match &self.holder_type {
            #[cfg(feature = "mock_attested_key_apple")]
            KeyHolderType::Apple { app_identifier, .. } => {
                let mut key = MockAppleAttestedKey::new_random(app_identifier.clone());
                key.has_error = has_error;

                let state = Self::state_from_apple_key(&key);

                (AttestedKey::Apple(key), state)
            }
            #[cfg(feature = "mock_attested_key_google")]
            KeyHolderType::Google { .. } => {
                let mut key = MockGoogleAttestedKey::new_random(Arc::clone(&self.key_states), key_identifier.clone());
                key.has_error = has_error;

                let state = Self::state_from_google_key(&key);

                (AttestedKey::Google(key), state)
            }
        };

        let existing_state = self.key_states.write().insert(key_identifier.clone(), state);

        // Sanity check, this only happens on a key collision.
        assert!(existing_state.is_none());

        (key, key_identifier)
    }
}

impl AttestedKeyHolder for MockHardwareAttestedKeyHolder {
    type Error = MockHardwareAttestedKeyError;
    type AppleKey = MockAppleAttestedKey;
    type GoogleKey = MockGoogleAttestedKey;

    async fn generate(&self) -> Result<String, Self::Error> {
        if let KeyHolderErrorScenario::GenerateError = self.error_scenario {
            return Err(MockHardwareAttestedKeyError::Mock);
        }

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
        _google_cloud_project_number: u64,
    ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>> {
        match self.error_scenario {
            KeyHolderErrorScenario::UnretryableAttestationError => {
                return Err(AttestationError::new_unretryable(MockHardwareAttestedKeyError::Mock));
            }
            KeyHolderErrorScenario::RetryableAttestationError => {
                return Err(AttestationError::new_retryable(MockHardwareAttestedKeyError::Mock));
            }
            _ => {}
        };

        let mut key_states = self.key_states.write();

        // The key's current state should be `AttestedKeyState::Generated`,
        // return the relevant error if this is not the case.
        let AttestedKeyState::Generated = key_states
            .get(&key_identifier)
            .ok_or(AttestationError::new_unretryable(
                MockHardwareAttestedKeyError::UnknownIdentifier,
            ))?
        else {
            return Err(AttestationError::new_unretryable(
                MockHardwareAttestedKeyError::KeyAttested,
            ));
        };

        let has_error = matches!(self.error_scenario, KeyHolderErrorScenario::SigningError);
        let key_with_attestation = match &self.holder_type {
            #[cfg(feature = "mock_attested_key_apple")]
            KeyHolderType::Apple {
                ca,
                environment,
                app_identifier,
            } => {
                // Generate a new Apple key and mock attestation data.
                let (mut key, attestation_data) =
                    MockAppleAttestedKey::new_with_attestation(ca, &challenge, *environment, app_identifier.clone());
                key.has_error = has_error;

                // Update the global key state with both the key's private key and counter.
                key_states.insert(key_identifier, Self::state_from_apple_key(&key));

                KeyWithAttestation::Apple { key, attestation_data }
            }
            #[cfg(feature = "mock_attested_key_google")]
            KeyHolderType::Google { ca_chain } => {
                let (mut key, certificate_chain, app_attestation_token) =
                    self.mock_google_attest(key_identifier.clone(), ca_chain, challenge);

                key.has_error = has_error;

                key_states.insert(key_identifier, Self::state_from_google_key(&key));

                KeyWithAttestation::Google {
                    key,
                    certificate_chain,
                    app_attestation_token,
                }
            }
        };

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
            .ok_or(MockHardwareAttestedKeyError::UnknownIdentifier)?
        {
            AttestedKeyState::Generated => Err(MockHardwareAttestedKeyError::KeyNotAttested),
            AttestedKeyState::Attested {
                signing_key: SharedSigningKey { signing_key, .. },
                next_counter,
            } => {
                // Use the Arc's reference counter as a proxy to determine if a key already
                // exists within memory, as this would own the second reference to it.
                if Arc::strong_count(signing_key) > 1 {
                    return Err(MockHardwareAttestedKeyError::IdentifierInUse);
                }

                // Construct the correct type of key based on the private key
                // retrieved from the state and, for Apple keys, the counter.
                let has_error = matches!(self.error_scenario, KeyHolderErrorScenario::SigningError);
                let signing_key = Arc::clone(signing_key);
                let key = match (&self.holder_type, next_counter) {
                    #[cfg(feature = "mock_attested_key_apple")]
                    (KeyHolderType::Apple { app_identifier, .. }, Some(next_counter)) => {
                        let key = MockAppleAttestedKey {
                            app_identifier: app_identifier.clone(),
                            signing_key,
                            next_counter: Arc::clone(next_counter),
                            has_error,
                        };

                        AttestedKey::Apple(key)
                    }
                    #[cfg(feature = "mock_attested_key_google")]
                    (KeyHolderType::Google { .. }, None) => {
                        let key = MockGoogleAttestedKey {
                            key_states: Arc::clone(&self.key_states),
                            key_identifier,
                            signing_key,
                            has_error,
                        };

                        AttestedKey::Google(key)
                    }
                    // The `next_counter` field should always be used for Apple keys and never for Google keys.
                    _ => unreachable!(),
                };

                Ok(key)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("mock error to be used in tests")]
pub struct MockAttestedKeyError {}

#[derive(Debug)]
pub struct MockAppleAttestedKey {
    #[cfg(feature = "mock_attested_key_apple")]
    pub app_identifier: apple_app_attest::AppIdentifier,
    pub signing_key: Arc<SigningKey>,
    pub next_counter: Arc<AtomicU32>,
    pub has_error: bool,
}

impl AppleAttestedKey for MockAppleAttestedKey {
    type Error = MockAttestedKeyError;

    #[cfg_attr(not(feature = "mock_attested_key_apple"), allow(unused_variables))]
    async fn sign(&self, payload: Vec<u8>) -> Result<AppleAssertion, Self::Error> {
        if self.has_error {
            return Err(MockAttestedKeyError {});
        }

        cfg_if! {
            if #[cfg(feature = "mock_attested_key_apple")] {
                Ok(self.sign_mock(&payload))
            } else {
                Err(MockAttestedKeyError {})
            }
        }
    }
}

/// Mock Google attested key that mostly wraps a `SigningKey`. It also contains its own key identifier
/// and a referenced copy of the key state, so that it may delete itself from the state.
#[derive(Debug)]
pub struct MockGoogleAttestedKey {
    key_states: KeyStates,
    key_identifier: String,
    pub signing_key: Arc<SigningKey>,
    pub has_error: bool,
}

impl GoogleAttestedKey for MockGoogleAttestedKey {
    async fn delete(self) -> Result<(), Self::Error> {
        self.key_states.write().remove(&self.key_identifier);

        Ok(())
    }
}

impl SecureEcdsaKey for MockGoogleAttestedKey {}

impl EcdsaKey for MockGoogleAttestedKey {
    type Error = MockAttestedKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let verifying_key = *self.signing_key.verifying_key();

        Ok(verifying_key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        if self.has_error {
            return Err(MockAttestedKeyError {});
        }

        let signature = Signer::try_sign(self.signing_key.as_ref(), msg).unwrap();

        Ok(signature)
    }
}

#[cfg(feature = "mock_attested_key_apple")]
mod apple {
    use std::sync::atomic::Ordering;

    use apple_app_attest::AppIdentifier;
    use apple_app_attest::Assertion;
    use apple_app_attest::AssertionCounter;
    use apple_app_attest::Attestation;
    use apple_app_attest::AttestationEnvironment;
    use apple_app_attest::MockAttestationCa;

    use super::*;

    impl MockHardwareAttestedKeyHolder {
        /// Create a key holder that produces mock Apple attested keys by generating a self-signed Apple CA.
        pub fn generate_apple(environment: AttestationEnvironment, app_identifier: AppIdentifier) -> Self {
            Self::new(KeyHolderType::Apple {
                ca: MockAttestationCa::generate(),
                environment,
                app_identifier,
            })
        }

        pub(super) fn state_from_apple_key(key: &MockAppleAttestedKey) -> AttestedKeyState {
            AttestedKeyState::Attested {
                signing_key: SharedSigningKey::new(Arc::clone(&key.signing_key)),
                next_counter: Some(Arc::clone(&key.next_counter)),
            }
        }
    }

    impl MockAppleAttestedKey {
        fn new(app_identifier: AppIdentifier, signing_key: SigningKey) -> Self {
            Self {
                app_identifier,
                signing_key: Arc::new(signing_key),
                next_counter: Arc::new(AtomicU32::new(1)),
                has_error: false,
            }
        }

        pub fn new_random(app_identifier: AppIdentifier) -> Self {
            Self::new(app_identifier, SigningKey::random(&mut OsRng))
        }

        pub fn new_with_attestation(
            mock_ca: &MockAttestationCa,
            challenge: &[u8],
            environment: AttestationEnvironment,
            app_identifier: AppIdentifier,
        ) -> (Self, Vec<u8>) {
            let (attestation, signing_key) =
                Attestation::new_mock_bytes(mock_ca, challenge, environment, &app_identifier);
            let attested_key = Self::new(app_identifier, signing_key);

            (attested_key, attestation)
        }

        pub fn verifying_key(&self) -> &VerifyingKey {
            self.signing_key.verifying_key()
        }

        pub fn next_counter(&self) -> AssertionCounter {
            AssertionCounter::from(self.next_counter.load(Ordering::Relaxed))
        }
    }

    impl MockAppleAttestedKey {
        pub(super) fn sign_mock(&self, payload: &[u8]) -> AppleAssertion {
            let assertion_bytes = Assertion::new_mock_bytes(
                &self.signing_key,
                &self.app_identifier,
                AssertionCounter::from(self.next_counter.fetch_add(1, Ordering::Relaxed)),
                payload,
            );

            assertion_bytes.into()
        }
    }
}

#[cfg(feature = "mock_attested_key_google")]
mod google {
    use base64::prelude::*;

    use android_attest::attestation_extension::key_description::KeyDescription;
    use android_attest::mock_chain::MockCaChain;

    use super::*;

    impl MockHardwareAttestedKeyHolder {
        #[cfg(feature = "mock_attested_key_google")]
        /// Create a key holder that produces mock Google attested keys by generating a self-signed Google CA chain.
        pub fn generate_google() -> Self {
            Self::new(KeyHolderType::Google {
                ca_chain: MockCaChain::generate(1),
            })
        }

        #[cfg(feature = "mock_attested_key_google")]
        pub(super) fn state_from_google_key(key: &MockGoogleAttestedKey) -> AttestedKeyState {
            AttestedKeyState::Attested {
                signing_key: SharedSigningKey::new(Arc::clone(&key.signing_key)),
                next_counter: None,
            }
        }

        pub(super) fn mock_google_attest(
            &self,
            key_identifier: String,
            ca_chain: &MockCaChain,
            challenge: Vec<u8>,
        ) -> (MockGoogleAttestedKey, Vec<Vec<u8>>, String) {
            // The token is simply the Base64 encoded challenge hash, which can then be used by
            // a mock Play Integrity implementation in order to generate an integrity verdict.
            let app_attestation_token = BASE64_STANDARD.encode(&challenge);

            let key_description = KeyDescription::new_valid_mock(challenge);

            // Generate a new Google key and mock certificate chain.
            let (certificate_chain, signing_key) = ca_chain.generate_attested_leaf_certificate(&key_description);
            let key = MockGoogleAttestedKey::new(Arc::clone(&self.key_states), key_identifier, signing_key);

            (key, certificate_chain, app_attestation_token)
        }
    }

    impl MockGoogleAttestedKey {
        fn new(key_states: KeyStates, key_identifier: String, signing_key: SigningKey) -> Self {
            Self {
                key_states,
                key_identifier,
                signing_key: Arc::new(signing_key),
                has_error: false,
            }
        }

        pub(super) fn new_random(key_states: KeyStates, key_identifier: String) -> Self {
            Self::new(key_states, key_identifier, SigningKey::random(&mut OsRng))
        }

        pub fn verifying_key(&self) -> &VerifyingKey {
            self.signing_key.verifying_key()
        }
    }
}

#[cfg(feature = "mock_attested_key_apple_ca")]
mod apple_ca {
    use super::*;

    use apple_app_attest::AppIdentifier;
    use apple_app_attest::AttestationEnvironment;
    use apple_app_attest::MockAttestationCa;

    impl MockHardwareAttestedKeyHolder {
        /// Create a key holder that produces mock Apple attested keys using the
        /// self-signed static Apple CA contained in the "mock_ca" files.
        pub fn new_apple_mock(environment: AttestationEnvironment, app_identifier: AppIdentifier) -> Self {
            Self::new(KeyHolderType::Apple {
                ca: MockAttestationCa::new_mock(),
                environment,
                app_identifier,
            })
        }
    }
}

#[cfg(feature = "persistent_mock_attested_key")]
pub use persistent::*;

#[cfg(feature = "persistent_mock_attested_key")]
mod persistent {
    use std::future::Future;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    use futures::TryFutureExt;
    use p256::pkcs8::DecodePrivateKey;
    use tokio::fs;
    use tokio::sync::Mutex;

    use apple_app_attest::AppIdentifier;
    use apple_app_attest::AttestationEnvironment;
    use apple_app_attest::MockAttestationCa;

    use crate::utils::PlatformUtilities;

    use super::*;

    /// The global state of all keys managed by [`PersistentMockAttestedKeyHolder`] instances.
    static KEY_STATES: LazyLock<KeyStates> = LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));
    /// Async mutex around the filesystem backing store that holds [`KEY_STATES`].
    static KEY_STATES_FILE: Mutex<Option<PathBuf>> = Mutex::const_new(None);

    // Have `SharedSigningKey` be serializable as Base64 through `serde_with`.
    impl AsRef<[u8]> for SharedSigningKey {
        fn as_ref(&self) -> &[u8] {
            &self.der
        }
    }

    // Have `SharedSigningKey` be deserializable as Base64 through `serde_with`.
    impl TryFrom<Vec<u8>> for SharedSigningKey {
        type Error = p256::pkcs8::Error;

        fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
            let signing_key = SigningKey::from_pkcs8_der(&value)?;

            Ok(Self::new(Arc::new(signing_key)))
        }
    }

    /// A wrapper around [`MockHardwareAttestedKeyHolder`] that synchronizes the global key state
    /// with a JSON file on the filesystem. As the iOS simulator does not support attested keys, this
    /// type can be used in place of of `HardwareAttestedKeyHolder` in order to emulate generation and
    /// attestation of and signing by attested keys that survive termination and relaunch of the
    /// application. As it is specifically meant for use of the iOS simulator, the storage path is
    /// determined using [`PlatformUtilities`] and it will only ever produce mock Apple attested keys.
    ///
    /// Note that this only ever produces mock Apple keys.
    #[derive(Debug)]
    pub struct PersistentMockAttestedKeyHolder(MockHardwareAttestedKeyHolder);

    impl PersistentMockAttestedKeyHolder {
        const FILE_NAME: &str = "mock_apple_attested_keys.json";

        /// Initialization function that should be called exactly once within the lifetime of the application.
        /// It reads the JSON file (if present) and loads the global key state from it.
        pub async fn init<U>()
        where
            U: PlatformUtilities,
        {
            let mut key_states_file = KEY_STATES_FILE.lock().await;

            if key_states_file.is_some() {
                panic!("PersistentMockAttestedKeyHolder::init() called more than once");
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

            *KEY_STATES.write() = key_states;
            *key_states_file = Some(file_path);
        }

        pub fn new_mock(environment: AttestationEnvironment, app_identifier: AppIdentifier) -> Self {
            let holder = MockHardwareAttestedKeyHolder {
                key_states: Arc::clone(&KEY_STATES),
                holder_type: KeyHolderType::Apple {
                    ca: MockAttestationCa::new_mock(),
                    environment,
                    app_identifier,
                },
                error_scenario: KeyHolderErrorScenario::default(),
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
                panic!("PersistentMockAttestedKeyHolder::init() should be called first")
            };

            future
                .and_then(|key_identifier| async {
                    let json = serde_json::to_string_pretty(&*KEY_STATES.read())
                        .expect("could not encode mock Apple attested keys JSON file");
                    fs::write(file_path, json)
                        .await
                        .expect("could not write mock Apple attested keys JSON file");

                    Ok(key_identifier)
                })
                .await
        }

        #[cfg(feature = "xcode_env")]
        pub fn new_mock_xcode(environment: AttestationEnvironment) -> Self {
            Self::new_mock(environment, AppIdentifier::default())
        }
    }

    impl AttestedKeyHolder for PersistentMockAttestedKeyHolder {
        type Error = MockHardwareAttestedKeyError;
        type AppleKey = PersistentMockAppleAttestedKey;
        type GoogleKey = MockGoogleAttestedKey;

        async fn generate(&self) -> Result<String, Self::Error> {
            Self::with_key_state_write(self.0.generate()).await
        }

        async fn attest(
            &self,
            key_identifier: String,
            challenge: Vec<u8>,
            google_cloud_project_number: u64,
        ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>> {
            // Map the output from `KeyWithAttestation<MockAppleAttestedKey, MockGoogleAttestedKey>`
            // to `KeyWithAttestation<PersistentMockAppleAttestedKey, MockGoogleAttestedKey>`.
            Self::with_key_state_write(self.0.attest(key_identifier, challenge, google_cloud_project_number))
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
            PersistentMockAttestedKeyHolder::with_key_state_write(self.0.sign(payload)).await
        }
    }

    #[cfg(all(test, feature = "mock_utils"))]
    mod tests {
        use apple_app_attest::AppIdentifier;
        use apple_app_attest::AttestationEnvironment;

        use crate::attested_key::test;
        use crate::utils::mock::MockHardwareUtilities;

        use super::KEY_STATES_FILE;
        use super::PersistentMockAttestedKeyHolder;

        #[tokio::test]
        async fn test_persistent_mock_attested_key_holder() {
            PersistentMockAttestedKeyHolder::init::<MockHardwareUtilities>().await;

            println!(
                "Mock Apple attested keys JSON file: {}",
                KEY_STATES_FILE.lock().await.as_deref().unwrap().to_string_lossy()
            );

            let app_identifier = AppIdentifier::new_mock();
            let mock_holder =
                PersistentMockAttestedKeyHolder::new_mock(AttestationEnvironment::Development, app_identifier);
            let PersistentMockAttestedKeyHolder(mock_holder_inner) = &mock_holder;

            let challenge = b"this_is_a_challenge_string";
            let payload = b"This is a message that will be signed by the persistent mock key.";

            test::create_and_verify_attested_key(
                &mock_holder,
                mock_holder_inner.to_test_data(),
                challenge.to_vec(),
                payload.to_vec(),
            )
            .await;
        }
    }
}

// Only run these tests when both Apple and Google mock attested keys are enabled.
#[cfg(all(feature = "mock_attested_key", test))]
mod tests {
    use android_attest::root_public_key::RootPublicKey;
    use apple_app_attest::AppIdentifier;
    use apple_app_attest::AttestationEnvironment;

    use crate::attested_key::test;
    use crate::attested_key::test::AndroidTestData;
    use crate::attested_key::test::AppleTestData;
    use crate::attested_key::test::TestData;

    use super::KeyHolderType;
    use super::MockHardwareAttestedKeyHolder;

    impl MockHardwareAttestedKeyHolder {
        pub fn to_test_data(&self) -> TestData<'_> {
            match &self.holder_type {
                KeyHolderType::Apple { ca, app_identifier, .. } => TestData::Apple(AppleTestData {
                    app_identifier,
                    trust_anchors: vec![ca.trust_anchor()],
                }),
                KeyHolderType::Google { ca_chain } => TestData::Android(AndroidTestData {
                    root_public_keys: vec![RootPublicKey::Rsa(ca_chain.root_public_key.clone())],
                }),
            }
        }
    }

    async fn test_mock_hardware_attested_key_holder(mock_holder: MockHardwareAttestedKeyHolder) {
        let challenge = b"this_is_a_challenge_string";
        let payload = b"This is a message that will be signed by the mock key.";

        test::create_and_verify_attested_key(
            &mock_holder,
            mock_holder.to_test_data(),
            challenge.to_vec(),
            payload.to_vec(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_mock_apple_hardware_attested_key_holder() {
        test_mock_hardware_attested_key_holder(MockHardwareAttestedKeyHolder::generate_apple(
            AttestationEnvironment::Development,
            AppIdentifier::new_mock(),
        ))
        .await
    }

    #[tokio::test]
    async fn test_mock_google_hardware_attested_key_holder() {
        test_mock_hardware_attested_key_holder(MockHardwareAttestedKeyHolder::generate_google()).await
    }
}
