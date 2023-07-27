use serde::{Deserialize, Serialize};

use crate::{
    account::{
        jwt::{Jwt, JwtClaims},
        signed::SignedDouble,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionResultClaims<R> {
    pub result: R,

    pub iss: String,
    pub iat: u64,
}

impl<R> JwtClaims for InstructionResultClaims<R> {
    const SUB: &'static str = "instruction_result";
}

pub type InstructionResult<R> = Jwt<InstructionResultClaims<R>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionResultMessage<R> {
    pub result: InstructionResult<R>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionChallengeRequestClaims {
    pub sequence_number: u64,

    pub iss: String,
    pub iat: u64,
}

impl JwtClaims for InstructionChallengeRequestClaims {
    const SUB: &'static str = "instruction_challenge_request";
}

pub type InstructionChallengeRequest = Jwt<InstructionChallengeRequestClaims>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionChallengeRequestMessage {
    pub message: InstructionChallengeRequest,
    pub certificate: WalletCertificate,
}

impl InstructionChallengeRequest {
    pub fn new_signed(
        instruction_sequence_number: u64,
        issuer: &str,
        hw_privkey: &impl SecureEcdsaKey,
    ) -> Result<Self> {
        let cert = InstructionChallengeRequestClaims {
            sequence_number: instruction_sequence_number,
            iss: issuer.to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign(&cert, hw_privkey)
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
