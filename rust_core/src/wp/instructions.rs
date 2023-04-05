use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    serialization::DerVerifyingKey,
    wallet::{pin_key::PinKey, signed::SignedDouble, HWBoundSigningKey},
    wp::WalletCertificate,
};

struct Instruction<T: IsInstruction> {
    instruction: SignedDouble<T>,
    certificate: WalletCertificate,
}

trait IsInstruction {}

struct CheckPin;

impl IsInstruction for CheckPin {}

/// Message that the wallet sends (signed) to the wallet provider during registration.
/// This does not implement IsInstruction because it not get sent as an [`Instruction<Registration>`]. because there is
/// no certificate yet at this point.
#[derive(Serialize, Deserialize, Debug)]
pub struct Registration {
    pub pin_pubkey: DerVerifyingKey,
    pub hw_pubkey: DerVerifyingKey,
}
impl Registration {

    pub fn new_signed(
        hw_key_handle: &impl HWBoundSigningKey,
        salt: &[u8],
        pin: &str,
        challenge: &[u8],
    ) -> Result<SignedDouble<Registration>> {
        let pin_pubkey = PinKey { salt, pin }.verifying_key();
        SignedDouble::sign(
            Registration {
                pin_pubkey: pin_pubkey.into(),
                hw_pubkey: (*hw_key_handle.verifying_key()).into(),
            },
            challenge,
            0,
            hw_key_handle,
            pin,
            salt,
        )
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

    use crate::wallet::pin_key;

    use super::Registration;

    #[test]
    fn registration() -> Result<()> {
        let salt = pin_key::new_pin_salt();
        let pin = "123456";
        let hw_privkey = SigningKey::random(&mut OsRng);

        // wallet provider generates a challenge
        let challenge = b"challenge";

        // wallet calculates wallet provider registration message
        let msg = Registration::new_signed(&hw_privkey, &salt, pin, challenge)?;
        println!("{}", &msg.0);

        let unverified = msg.dangerous_parse_unverified()?;

        // wallet provider takes the public keys from the message, and verifies the signatures
        dbg!(msg.parse_and_verify(
            challenge,
            &unverified.payload.hw_pubkey.0,
            &unverified.payload.pin_pubkey.0,
        )?);

        Ok(())
    }
}
