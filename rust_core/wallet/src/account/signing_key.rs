use p256::ecdsa::{signature::Signer, Signature, VerifyingKey};
use platform_support::hw_keystore::{HardwareKeyStoreError, PlatformSigningKey};
use std::error::Error;

use crate::account::pin_key::{PinKey, PinKeyError};

pub trait SigningKey: Signer<Signature> {
    type Error: Error + Send + Sync + 'static;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;
}

pub trait EphemeralSigningKey: SigningKey {}

pub trait SecureSigningKey: SigningKey {}

impl<'a> SigningKey for PinKey<'a> {
    type Error = PinKeyError;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.verifying_key()
    }
}

impl<'a> EphemeralSigningKey for PinKey<'a> {}

impl<K: PlatformSigningKey> SigningKey for K {
    type Error = HardwareKeyStoreError;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.verifying_key()
    }
}

impl<K: PlatformSigningKey> SecureSigningKey for K {}

// make sure we can substitute a SigningKey instead in all tests
// the other traits are already implemented in "platform_support" using the "software" feature
#[cfg(test)]
impl EphemeralSigningKey for p256::ecdsa::SigningKey {}
