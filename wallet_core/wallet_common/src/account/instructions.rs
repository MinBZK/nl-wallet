use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::account::{
    signing_key::{EphemeralEcdsaKey, SecureEcdsaKey},
    {serialization::DerVerifyingKey, signed::SignedDouble, WalletCertificate},
};

#[allow(unused)]
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
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        challenge: &[u8],
    ) -> Result<SignedDouble<Registration>> {
        let pin_pubkey = pin_privkey.verifying_key()?;
        let hw_pubkey = hw_privkey.verifying_key()?;

        SignedDouble::sign(
            Registration {
                pin_pubkey: pin_pubkey.into(),
                hw_pubkey: hw_pubkey.into(),
            },
            challenge,
            0,
            hw_privkey,
            pin_privkey,
        )
    }
}

#[cfg(test)]
mod tests {
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

    use super::*;

    #[test]
    fn registration() -> Result<()> {
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        // wallet provider generates a challenge
        let challenge = b"challenge";

        // wallet calculates wallet provider registration message
        let msg = Registration::new_signed(&hw_privkey, &pin_privkey, challenge)?;
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
