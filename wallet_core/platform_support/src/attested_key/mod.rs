//! Unifies working with both Android and iOS key and app attestation.
//!
//! # Design
//!
//! The design of this functionality is divided into three groups of related types:
//!
//! 1. The bridging types defined in the private submodule `bridge::attested_keys` of this crate.
//!    These are simply the Rust versions of the types defined in the `platform_support.udl` and
//!    are designed to unify the behaviour of both Android and iOS key and app attestation.
//!    See the documenting comments in the UDL file mentioned for additional information.
//! 2. The traits and types in this crate, that provide a generic interface wrapping the functionality
//!    provided by the bridging types. This allows switching between the real hardware backed
//!    implementation and a fake "software" implementation, which is used during testing.
//!    It also makes the interface of the bridged types a bit more rusty, as the types described in
//!    UDL have certain restrictions.
//! 3. The concrete implementations of the traits, both the ones wrapping the bridging types in the
//!    [`hardware`] submodule and the mock ones in `software`. Note that the latter is not currently
//!    implemented.
//!
//! ## Traits and supporting types
//!
//! The [`AttestedKeyHolder`] trait is designed to be a singleton can can be used to create new
//! attested keys or instantiate existing ones. The latter results in a [`AttestedKey`] enum,
//! which contains either the Apple or Google key type. These implement the [`AppleAttestedKey`] and
//! [`GoogleAttestedKey`] traits respectively.
//!
//! [`AppleAttestedKey`] simply provides a `sign()` method that returns [`AppleAssertion`], which is
//! a wrapper around [`Vec<u8>`] for semantic reasons. These bytes are meant to be sent to a server
//! for verification.
//!
//! [`GoogleAttestedKey`] on the other hand has [`SecureEcdsaKey`] as its supertrait, meaning that
//! this type supports generating ECDSA signatures. Additionally, it provides a `delete()` method
//! to delete the attested key from storage.
//!
//! When [`AttestedKeyHolder`] is used to attest a new key through its `attest()` method, a
//! [`KeyWithAttestation`] enum is returned instead. Apart from containing a key type, it also
//! contains platform specific attestation data that is to be sent to a server. Any error resulting
//! from this method is wrapped in [`AttestationError`], which simply adds a `retryable` boolean.
//!
//! Note that only one unique key type per identifier can be held in memory at one time, as otherwise
//! a key could be used after deleting it. This would lead to undefined behaviour. For this reason
//! instantiating a second key for an identifier results in an error.
//!
//! ## Real device ("hardware") implementation
//!
//! The concrete implementation of the attested key traits provide a thin wrapper around the bridging
//! code, implementing the uniqueness checking mentioned above.
//!
//! * The [`hardware::HardwareAttestedKeyHolder`] type implements the [`AttestedKeyHolder`] trait and
//!   can be created through its implementation of [`Default`].
//! * The [`hardware::AppleHardwareAttestedKey`] type implements the [`AppleAttestedKey`] trait.
//! * The [`hardware::GoogleHardwareAttestedKey`] type implements the [`GoogleAttestedKey`] trait.
//! * The concrete error type for all of these implementations is the
//!   [`hardware::HardwareAttestedKeyError`] enum.
//! * The key types make use of an internal helper type `HardwareAttestedKey`, which encapsulates
//!   shared functionality between these types.
//!
//! ## Mock ("software") implementation
//!
//! TBD

pub mod hardware;

#[cfg(feature = "hardware_integration_test")]
pub mod test;

use std::error::Error;

use derive_more::derive::AsRef;
use derive_more::derive::Into;

use wallet_common::keys::SecureEcdsaKey;

/// Wrapper for errors encountered during attestation that includes a boolean to indicate
/// whether the caller should retry the attestation using the same identifier.
#[derive(Debug, thiserror::Error)]
#[error("could not perform key/app attestation (retryable: {retryable}): {error}")]
pub struct AttestationError<E>
where
    E: Error,
{
    #[source]
    pub error: E,
    pub retryable: bool,
}

/// Either a generic Apple or Google attested key.
#[derive(Debug)]
pub enum AttestedKey<A, G> {
    Apple(A),
    Google(G),
}

/// Either a generic Apple or Google attested key, including the platform specific attestation data.
#[derive(Debug)]
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

    /// On iOS this generates a key including a unique identifier, whereas on Android this
    /// just generates the identifier. This should be called before performing attestation.
    async fn generate(&self) -> Result<String, Self::Error>;

    /// Perform key and app attestation, using the identifier that is the result of [`AttestedKeyHolder::generate`].
    /// The result is a [`KeyWithAttestation`] type, which holds both the platform specific key type and attestation
    /// data. On failure, the error is wrapped in the [`AttestationError`] type, which includes the `retryable` boolean.
    ///
    /// Note that on Android this actually generates a key in the process of attestation,
    /// while on iOS it operates on a previously generated key.
    async fn attest(
        &self,
        key_identifier: String,
        challenge: Vec<u8>,
    ) -> Result<KeyWithAttestation<Self::AppleKey, Self::GoogleKey>, AttestationError<Self::Error>>;

    /// This returns an instance of a key. It is meant to be used after the key has been attested, but
    /// this is not checked on instantiation. Using a key before attestation may result in an error.
    ///
    /// Only one instance can exist within the process, an error is returned if a second one is created.
    fn attested_key(&self, key_identifier: String)
        -> Result<AttestedKey<Self::AppleKey, Self::GoogleKey>, Self::Error>;
}

/// A newtype around `Vec<u8>` that represent an assertion generated by Apple AppAttest.
/// It is to be treated as opaque bytes until received by the server.
#[derive(Debug, Clone, Into, AsRef)]
pub struct AppleAssertion(Vec<u8>);

/// Trait for an Apple attested key. Note that [`SecureEcdsaKey`] is not
/// a supertrait, since signing does not produce an ECDSA signature.
pub trait AppleAttestedKey {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Generate an Apple assertion using the attested key, which returns the [`AppleAssertion`] newtype.
    async fn sign(&self, payload: Vec<u8>) -> Result<AppleAssertion, Self::Error>;
}

/// Trait for a Google attested key, which includes all methods contained in [`SecureEcdsaKey`].
pub trait GoogleAttestedKey: SecureEcdsaKey {
    /// Delete the attested key, which consumes the type.
    async fn delete(self) -> Result<(), Self::Error>;
}
