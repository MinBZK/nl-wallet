#[cfg(feature = "integration-test")]
pub mod integration_test;
#[cfg(feature = "software-keys")]
pub mod software;

use std::error::Error;

use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};

pub trait EcdsaKey: Signer<Signature> {
    type Error: Error + Send + Sync + 'static;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;
}

/// Contract for ECDSA private keys which are short-lived and deterministically derived from a PIN.
pub trait EphemeralEcdsaKey: EcdsaKey {}

/// Contract for ECDSA private keys that are stored in some form of secure hardware from which they cannot be extracted,
/// e.g., a HSM, Android's TEE/StrongBox, or Apple's SE.
pub trait SecureEcdsaKey: EcdsaKey {}

/// The contract of this trait includes that a constructed type with the same
/// identifier behaves exactly the same, i.e. has the same key material backing it.
pub trait ConstructableWithIdentifier {
    fn new(identifier: &str) -> Self
    where
        Self: Sized;

    fn identifier(&self) -> &str;
}

/// Contract for encryption keys suitable for use in the wallet, e.g. for securely storing the database key.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`ConstructableWithIdentifier::new()`].
pub trait SecureEncryptionKey: ConstructableWithIdentifier {
    // from ConstructableWithIdentifier: new(), identifier()
    type Error: Error + Send + Sync + 'static;

    fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
    fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use super::*;

    // make sure we can substitute a SigningKey instead in tests
    impl EcdsaKey for p256::ecdsa::SigningKey {
        type Error = p256::ecdsa::Error;

        fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
            Ok(*self.verifying_key())
        }
    }

    impl EphemeralEcdsaKey for p256::ecdsa::SigningKey {}
    impl SecureEcdsaKey for p256::ecdsa::SigningKey {}
}