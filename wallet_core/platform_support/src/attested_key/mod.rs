pub mod hardware;

use std::error::Error;

use wallet_common::keys::SecureEcdsaKey;

/// Wrapper for errors encountered during attestation that includes a boolean to indicate
/// that the caller should retain the identifier for future retries of the attestation.
#[derive(Debug, thiserror::Error)]
#[error("could not perform key/app attestation (retain_identifier: {retain_identifier}): {error}")]
pub struct AttestationError<E>
where
    E: Error,
{
    #[source]
    pub error: E,
    pub retain_identifier: bool,
}

/// Either a generic Apple or Google attested key.
pub enum AttestedKey<A, G> {
    Apple(A),
    Google(G),
}

/// Either a generic Apple or Google attested key, including the platform specific attestation data.
pub enum KeyWithAttestation<A, G> {
    Apple {
        key: A,
        attestation_data: Vec<u8>,
    },
    Google {
        key: G,
        certificate_chain: Vec<Vec<u8>>,
        app_attestation_token: Vec<u8>,
    },
}

/// Trait for a type that can be used perform key/app attestation or retrieve already attested keys.
/// It produces one of two different key types, constrained by the [`AppleAttestedKey`] and
/// [`GoogleAttestedKey`] traits.
pub trait AttestedKeyHolder {
    type Error: std::error::Error + Send + Sync + 'static;
    type AppleKey: AppleAttestedKey;
    type GoogleKey: GoogleAttestedKey;

    async fn generate_identifier(&self) -> Result<String, Self::Error>;
    async fn attest(
        &self,
        key_identifier: String,
        challenge: Vec<u8>,
    ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>>;
    async fn attested_key(
        &self,
        key_identifier: String,
    ) -> Result<AttestedKey<Self::AppleKey, Self::GoogleKey>, Self::Error>;
}

/// Trait for an Apple attested key. Note that [`SecureEcdsaKey`] is not
/// a supertrait, since signing does not produce an ECDSA signature.
pub trait AppleAttestedKey {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn sign(&self, payload: Vec<u8>) -> Result<Vec<u8>, Self::Error>;
}

/// Trait for a Google attested key, which includes all methods contained in [`SecureEcdsaKey`].
pub trait GoogleAttestedKey: SecureEcdsaKey {
    async fn delete(self) -> Result<(), Self::Error>;
}
