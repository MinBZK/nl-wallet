use p256::ecdsa::{signature::Signer, Signature};

use super::pin_key::PinKey;

pub trait EphemeralSigner: Signer<Signature> {}

impl<'a> EphemeralSigner for PinKey<'a> {}

// make sure we can substitute a SigningKey instead in all tests
#[cfg(test)]
impl EphemeralSigner for p256::ecdsa::SigningKey {}
