use std::error::Error;

use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct EcdsaKeyError(#[from] pub Box<dyn Error + Send + Sync>);

pub trait EcdsaKey: Signer<Signature> {
    fn verifying_key(&self) -> Result<VerifyingKey, EcdsaKeyError>;
}

pub trait EphemeralEcdsaKey: EcdsaKey {}

pub trait SecureEcdsaKey: EcdsaKey {}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use super::*;

    // make sure we can substitute a SigningKey instead in tests
    impl EcdsaKey for p256::ecdsa::SigningKey {
        fn verifying_key(&self) -> Result<VerifyingKey, EcdsaKeyError> {
            Ok(*self.verifying_key())
        }
    }

    impl EphemeralEcdsaKey for p256::ecdsa::SigningKey {}
    impl SecureEcdsaKey for p256::ecdsa::SigningKey {}
}
