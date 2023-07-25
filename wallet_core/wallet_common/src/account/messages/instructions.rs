use serde::{Deserialize, Serialize};

use crate::{
    account::{
        signed::{Signed, SignedDouble},
        signing_key::{EphemeralEcdsaKey, SecureEcdsaKey},
    },
    errors::Result,
};

use super::auth::WalletCertificate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction<T> {
    pub instruction: SignedDouble<T>,
    pub certificate: WalletCertificate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckPin;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionResult<R>(R);

impl<R> InstructionResult<R> {
    pub fn new(result: R) -> Self {
        Self(result)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstructionChallengeRequest {
    pub message: Signed<InstructionChallenge>,
    pub certificate: WalletCertificate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionChallenge {
    pub sequence_number: u64,
}

impl InstructionChallenge {
    pub fn new_signed(instruction_sequence_number: u64, hw_privkey: &impl SecureEcdsaKey) -> Result<Signed<Self>> {
        Signed::sign(
            Self {
                sequence_number: instruction_sequence_number,
            },
            "wallet".to_string(),
            hw_privkey,
        )
    }
}

impl CheckPin {
    pub fn new_signed(
        instruction_sequence_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        challenge: &[u8],
    ) -> Result<SignedDouble<CheckPin>> {
        SignedDouble::sign(Self, challenge, instruction_sequence_number, hw_privkey, pin_privkey)
    }
}
