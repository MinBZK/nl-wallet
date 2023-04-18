use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};
use std::error::Error;

pub trait EcdsaKey: Signer<Signature> {
    type Error: Error + Send + Sync + 'static;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;
}

pub trait EphemeralEcdsaKey: EcdsaKey {}

pub trait SecureEcdsaKey: EcdsaKey {}
