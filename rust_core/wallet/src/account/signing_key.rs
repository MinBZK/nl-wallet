use p256::ecdsa::VerifyingKey;

use wallet_shared::account::signing_key::{EphemeralSigningKey, SigningKey};

use crate::pin::key::{PinKey, PinKeyError};

impl<'a> SigningKey for PinKey<'a> {
    type Error = PinKeyError;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.verifying_key()
    }
}

impl<'a> EphemeralSigningKey for PinKey<'a> {}
