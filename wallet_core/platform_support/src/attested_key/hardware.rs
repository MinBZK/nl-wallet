use derive_more::Debug;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;

use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;

use crate::bridge::attested_key::get_attested_key_bridge;
use crate::bridge::attested_key::AttestationData;
use crate::bridge::attested_key::AttestedKeyBridge;
use crate::bridge::attested_key::AttestedKeyType;

pub use crate::bridge::attested_key::AttestedKeyError;

use super::AppleAssertion;
use super::AppleAttestedKey;
use super::AttestationError;
use super::AttestedKey;
use super::AttestedKeyHolder;
use super::GoogleAttestedKey;
use super::KeyWithAttestation;

use key::HardwareAttestedKey;
use key::UniqueCreatedResult;

mod key {
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use derive_more::Debug;
    use parking_lot::Mutex;

    use crate::bridge::attested_key::AttestedKeyBridge;
    use crate::bridge::attested_key::AttestedKeyError;

    static UNIQUE_IDENTIFIERS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

    #[derive(Debug)]
    pub(super) struct HardwareAttestedKey {
        #[debug(skip)]
        bridge: &'static dyn AttestedKeyBridge,
        identifier: String,
    }

    pub(super) enum UniqueCreatedResult<T> {
        Created(T),
        Duplicate(String),
    }

    /// A generic attested key that encapsulates the following behaviour:
    /// * Only one instance of the type for each identifier can be constructed.
    /// * Provide access to (blocking) methods on the bridge that require an identifier.
    ///
    /// Note that this is contained within its own submodule so that the
    /// `new()` constructor is the only way of instantiating the type.
    impl HardwareAttestedKey {
        pub fn new(bridge: &'static dyn AttestedKeyBridge, identifier: String) -> UniqueCreatedResult<Self> {
            let mut identifiers = UNIQUE_IDENTIFIERS.lock();

            match identifiers.contains(&identifier) {
                true => UniqueCreatedResult::Duplicate(identifier),
                false => {
                    identifiers.insert(identifier.clone());

                    UniqueCreatedResult::Created(Self { bridge, identifier })
                }
            }
        }

        pub async fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, AttestedKeyError> {
            self.bridge.sign(self.identifier.clone(), payload).await
        }

        pub async fn public_key(&self) -> Result<Vec<u8>, AttestedKeyError> {
            self.bridge.public_key(self.identifier.clone()).await
        }

        pub async fn delete(self) -> Result<(), AttestedKeyError> {
            self.bridge.delete(self.identifier.clone()).await

            // Note that upon returning, the `Drop` implementation will be called and
            // the identifier will be removed from the static `UNIQUE_IDENTIFIERS`.
        }
    }

    impl Drop for HardwareAttestedKey {
        fn drop(&mut self) {
            UNIQUE_IDENTIFIERS.lock().remove(&self.identifier);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HardwareAttestedKeyError {
    #[error("identifier is already in use in this process: {0}")]
    IdentifierInUse(String),
    #[error("could not decode DER signature: {0}")]
    Signature(#[from] p256::ecdsa::Error),
    #[error("could not decode DER public key: {0}")]
    PublicKey(#[from] p256::pkcs8::spki::Error),
    #[error("could not perform attested key operation in platform code: {0}")]
    Platform(#[source] AttestedKeyError),
}

impl From<AttestedKeyError> for HardwareAttestedKeyError {
    fn from(value: AttestedKeyError) -> Self {
        Self::Platform(value)
    }
}

impl From<AttestedKeyError> for AttestationError<HardwareAttestedKeyError> {
    fn from(value: AttestedKeyError) -> Self {
        match value {
            AttestedKeyError::ServerUnreachable { .. } => Self::new_retryable(value.into()),
            _ => Self::new_unretryable(value.into()),
        }
    }
}

impl KeyWithAttestation<AppleHardwareAttestedKey, GoogleHardwareAttestedKey> {
    fn new(inner_key: HardwareAttestedKey, attestation_data: AttestationData) -> Self {
        match attestation_data {
            AttestationData::Apple { attestation_data } => Self::Apple {
                key: AppleHardwareAttestedKey(inner_key),
                attestation_data,
            },
            AttestationData::Google {
                certificate_chain,
                app_attestation_token,
            } => Self::Google {
                key: GoogleHardwareAttestedKey(inner_key),
                certificate_chain,
                app_attestation_token,
            },
        }
    }
}

/// The main type to implement [`AttestedKeyHolder`].
#[derive(Debug)]
pub struct HardwareAttestedKeyHolder {
    #[debug(skip)]
    bridge: &'static dyn AttestedKeyBridge,
}

impl Default for HardwareAttestedKeyHolder {
    fn default() -> Self {
        Self {
            bridge: get_attested_key_bridge(),
        }
    }
}

impl AttestedKeyHolder for HardwareAttestedKeyHolder {
    type Error = HardwareAttestedKeyError;
    type AppleKey = AppleHardwareAttestedKey;
    type GoogleKey = GoogleHardwareAttestedKey;

    async fn generate(&self) -> Result<String, Self::Error> {
        let identifier = self.bridge.generate().await?;

        Ok(identifier)
    }

    async fn attest(
        &self,
        key_identifier: String,
        challenge: Vec<u8>,
    ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>> {
        // Claim the identifier before performing attestation, if it does not exist already within the process.
        // If an error occurs within this method, the `Drop` implementation on this type will relinquish it again.
        let inner_key = match HardwareAttestedKey::new(self.bridge, key_identifier.clone()) {
            UniqueCreatedResult::Created(inner_key) => inner_key,
            UniqueCreatedResult::Duplicate(identifier) => {
                return Err(AttestationError {
                    error: HardwareAttestedKeyError::IdentifierInUse(identifier),
                    retryable: false,
                })
            }
        };

        // Perform key/app attestation and convert the resulting attestation data to the corresponding key type.
        let attestation_data = self.bridge.attest(key_identifier, challenge).await?;

        let key_with_attestation = KeyWithAttestation::new(inner_key, attestation_data);

        Ok(key_with_attestation)
    }

    fn attested_key(
        &self,
        key_identifier: String,
    ) -> Result<AttestedKey<Self::AppleKey, Self::GoogleKey>, Self::Error> {
        // Return a wrapped `HardwareAttestedKey`, if the identifier is not already in use.
        let inner_key = match HardwareAttestedKey::new(self.bridge, key_identifier) {
            UniqueCreatedResult::Created(inner_key) => inner_key,
            UniqueCreatedResult::Duplicate(identifier) => {
                return Err(HardwareAttestedKeyError::IdentifierInUse(identifier))
            }
        };

        // In order to have a single source of truth, ask the native implementation what the
        // platform type is, instead of having the Rust compiler determine it.
        let key = match self.bridge.key_type() {
            AttestedKeyType::Apple => AttestedKey::Apple(AppleHardwareAttestedKey(inner_key)),
            AttestedKeyType::Google => AttestedKey::Google(GoogleHardwareAttestedKey(inner_key)),
        };

        Ok(key)
    }
}

/// This is the concrete type that implements [`AppleAttestedKey`]. It simply wraps
/// a `HardwareAttestedKey` and forwards calls to send across to bridge to it.
#[derive(Debug)]
pub struct AppleHardwareAttestedKey(HardwareAttestedKey);

impl AppleAttestedKey for AppleHardwareAttestedKey {
    type Error = HardwareAttestedKeyError;

    async fn sign(&self, payload: Vec<u8>) -> Result<AppleAssertion, Self::Error> {
        let assertion = self.0.sign(payload).await?;

        Ok(AppleAssertion::from(assertion))
    }
}

/// This is the concrete type that implements [`GoogleAttestedKey`]. It simply wraps
/// a `HardwareAttestedKey` and forwards calls to send across to bridge to it.
#[derive(Debug)]
pub struct GoogleHardwareAttestedKey(HardwareAttestedKey);

impl GoogleAttestedKey for GoogleHardwareAttestedKey {
    async fn delete(self) -> Result<(), Self::Error> {
        self.0.delete().await?;

        Ok(())
    }
}

impl SecureEcdsaKey for GoogleHardwareAttestedKey {}

impl EcdsaKey for GoogleHardwareAttestedKey {
    type Error = HardwareAttestedKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let public_key_bytes = self.0.public_key().await?;
        let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

        Ok(public_key)
    }

    async fn try_sign(&self, payload: &[u8]) -> Result<Signature, Self::Error> {
        let signature_bytes = self.0.sign(payload.to_vec()).await?;
        // Only for Android is the returned signature in DER format.
        let signature = Signature::from_der(&signature_bytes)?;

        Ok(signature)
    }
}
