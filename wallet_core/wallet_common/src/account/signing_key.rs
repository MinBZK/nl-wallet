use std::error::Error;

use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};

pub trait EcdsaKey: Signer<Signature> {
    type Error: Error + Send + Sync + 'static;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;
}

pub trait EphemeralEcdsaKey: EcdsaKey {}

pub trait SecureEcdsaKey: EcdsaKey {}

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
