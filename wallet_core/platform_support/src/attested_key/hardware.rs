use p256::{
    ecdsa::{Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};
use wallet_common::keys::{EcdsaKey, SecureEcdsaKey};

use crate::bridge::attested_key::{
    get_attested_key_bridge, AttestationData, AttestedKeyBridge, AttestedKeyError, AttestedKeyType,
};

use super::{
    AppleAttestedKey, AttestationError, AttestedKey, AttestedKeyHolder, GoogleAttestedKey, KeyWithAttestation,
};

use key::HardwareAttestedKey;

mod key {
    use std::{collections::HashSet, sync::LazyLock};

    use parking_lot::Mutex;

    use crate::bridge::attested_key::{AttestedKeyBridge, AttestedKeyError};

    static UNIQUE_IDENTIFIERS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

    pub(super) struct HardwareAttestedKey {
        bridge: &'static dyn AttestedKeyBridge,
        identifier: String,
    }

    /// A generic attested key that encapsulates the following behaviour:
    /// * Only one instance of the type for each identifier can be constructed.
    /// * Provide access to (blocking) methods on the bridge that require an identifier.
    ///
    /// Note that this is contained within its own submodule so that the
    /// `new()` constructor is the only way of instantiating the type.
    impl HardwareAttestedKey {
        pub fn new(bridge: &'static dyn AttestedKeyBridge, identifier: String) -> Option<Self> {
            let mut identifiers = UNIQUE_IDENTIFIERS.lock();

            (!identifiers.contains(&identifier)).then(|| {
                identifiers.insert(identifier.clone());

                Self { bridge, identifier }
            })
        }

        pub async fn sign<E>(&self, payload: Vec<u8>) -> Result<Vec<u8>, E>
        where
            E: From<AttestedKeyError>,
        {
            let signature = self.bridge.sign(self.identifier.clone(), payload).await?;

            Ok(signature)
        }

        pub async fn public_key<E>(&self) -> Result<Vec<u8>, E>
        where
            E: From<AttestedKeyError>,
        {
            let public_key = self.bridge.public_key(self.identifier.clone()).await?;

            Ok(public_key)
        }

        pub async fn delete<E>(self) -> Result<(), E>
        where
            E: From<AttestedKeyError>,
        {
            self.bridge.delete(self.identifier.clone()).await?;

            Ok(())

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
    Platform(AttestedKeyError),
}

impl From<AttestedKeyError> for HardwareAttestedKeyError {
    fn from(value: AttestedKeyError) -> Self {
        Self::Platform(value)
    }
}

impl From<AttestedKeyError> for AttestationError<HardwareAttestedKeyError> {
    fn from(value: AttestedKeyError) -> Self {
        let retain_identifier = matches!(value, AttestedKeyError::ServerUnreachable { .. });

        Self {
            error: value.into(),
            retain_identifier,
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
pub struct HardwareAttestedKeyHolder {
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

    async fn generate_identifier(&self) -> Result<String, Self::Error> {
        let identifier = self.bridge.generate_identifier().await?;

        Ok(identifier)
    }

    async fn attest(
        &self,
        key_identifier: String,
        challenge: Vec<u8>,
    ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>> {
        // Claim the identifier before performing attestation, if it does not exist already within the process.
        // If an error occurs within this method, the `Drop` implementation on this type will relinquish it again.
        let inner_key =
            HardwareAttestedKey::new(self.bridge, key_identifier.clone()).ok_or_else(|| AttestationError {
                error: HardwareAttestedKeyError::IdentifierInUse(key_identifier.clone()),
                retain_identifier: false,
            })?;

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
        let inner_key = HardwareAttestedKey::new(self.bridge, key_identifier.clone())
            .ok_or_else(|| HardwareAttestedKeyError::IdentifierInUse(key_identifier.clone()))?;

        // In order to have a single source of truth, ask the native implementation what the
        // platform type is, instead of having the Rust compiler determine it.
        let key = match self.bridge.key_type() {
            AttestedKeyType::Apple => AttestedKey::Apple(AppleHardwareAttestedKey(inner_key)),
            AttestedKeyType::Google => AttestedKey::Google(GoogleHardwareAttestedKey(inner_key)),
        };

        Ok(key)
    }
}

pub struct AppleHardwareAttestedKey(HardwareAttestedKey);

impl AppleAttestedKey for AppleHardwareAttestedKey {
    type Error = HardwareAttestedKeyError;

    async fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, Self::Error> {
        self.0.sign(payload).await
    }
}

pub struct GoogleHardwareAttestedKey(HardwareAttestedKey);

impl GoogleAttestedKey for GoogleHardwareAttestedKey {
    async fn delete(self) -> Result<(), Self::Error> {
        self.0.delete().await
    }
}

impl SecureEcdsaKey for GoogleHardwareAttestedKey {}

impl EcdsaKey for GoogleHardwareAttestedKey {
    type Error = HardwareAttestedKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let public_key_bytes = self.0.public_key::<Self::Error>().await?;
        let public_key = VerifyingKey::from_public_key_der(&public_key_bytes)?;

        Ok(public_key)
    }

    async fn try_sign(&self, payload: &[u8]) -> Result<Signature, Self::Error> {
        let signature_bytes = self.0.sign::<Self::Error>(payload.to_vec()).await?;
        // Only for Android is the returned signature in DER format.
        let signature = Signature::from_der(&signature_bytes)?;

        Ok(signature)
    }
}
