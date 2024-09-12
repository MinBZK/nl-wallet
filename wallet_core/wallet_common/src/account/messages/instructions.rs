use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    account::{
        errors::Result,
        serialization::{DerSignature, DerVerifyingKey},
        signed::SignedDouble,
    },
    jwt::{Jwt, JwtSubject},
    keys::{EphemeralEcdsaKey, SecureEcdsaKey},
};

use super::auth::{Certificate, WalletCertificate};

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction<T> {
    pub instruction: SignedDouble<T>,
    pub certificate: WalletCertificate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckPin;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePinStart {
    pub pin_pubkey: DerVerifyingKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePinCommit {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePinRollback {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateKey {
    pub identifiers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateKeyResult {
    pub public_keys: Vec<(String, DerVerifyingKey)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sign {
    pub messages_with_identifiers: Vec<(Vec<u8>, Vec<String>)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignResult {
    pub signatures: Vec<Vec<DerSignature>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionResultClaims<R> {
    pub result: R,

    pub iss: String,
    pub iat: u64,
}

pub type InstructionResult<R> = Jwt<InstructionResultClaims<R>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionResultMessage<R> {
    pub result: InstructionResult<R>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionChallengeRequestClaims {
    pub sequence_number: u64,
    pub instruction_name: String,

    pub iss: String,
    pub iat: u64,
}

pub type InstructionChallengeRequest = Jwt<InstructionChallengeRequestClaims>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionChallengeRequestMessage {
    pub message: InstructionChallengeRequest,
    pub certificate: WalletCertificate,
}

pub trait InstructionAndResult: Serialize + DeserializeOwned {
    const NAME: &'static str;

    type Result: Serialize + DeserializeOwned;
}

impl<R> JwtSubject for InstructionResultClaims<R> {
    const SUB: &'static str = "instruction_result";
}

impl JwtSubject for InstructionChallengeRequestClaims {
    const SUB: &'static str = "instruction_challenge_request";
}

impl InstructionChallengeRequest {
    pub async fn new_signed<I>(
        instruction_sequence_number: u64,
        issuer: String,
        hw_privkey: &impl SecureEcdsaKey,
    ) -> Result<Self>
    where
        I: InstructionAndResult,
    {
        let cert = InstructionChallengeRequestClaims {
            sequence_number: instruction_sequence_number,
            instruction_name: I::NAME.to_string(),
            iss: issuer,
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Ok(Jwt::sign_with_sub(&cert, hw_privkey).await?)
    }
}

impl InstructionAndResult for CheckPin {
    const NAME: &'static str = "check_pin";

    type Result = ();
}

impl InstructionAndResult for ChangePinStart {
    const NAME: &'static str = "change_pin_start";

    type Result = Certificate;
}

impl InstructionAndResult for ChangePinCommit {
    const NAME: &'static str = "change_pin_commit";

    type Result = ();
}

impl InstructionAndResult for ChangePinRollback {
    const NAME: &'static str = "change_pin_rollback";

    type Result = ();
}

impl InstructionAndResult for GenerateKey {
    const NAME: &'static str = "generate_key";

    type Result = GenerateKeyResult;
}

impl InstructionAndResult for Sign {
    const NAME: &'static str = "sign";

    type Result = SignResult;
}

impl<T> Instruction<T>
where
    T: Serialize + DeserializeOwned,
{
    pub async fn new_signed(
        instruction: T,
        instruction_sequence_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        challenge: &[u8],
        certificate: WalletCertificate,
    ) -> Result<Self> {
        let signed = SignedDouble::sign(
            instruction,
            challenge,
            instruction_sequence_number,
            hw_privkey,
            pin_privkey,
        )
        .await?;

        Ok(Self {
            instruction: signed,
            certificate,
        })
    }
}
