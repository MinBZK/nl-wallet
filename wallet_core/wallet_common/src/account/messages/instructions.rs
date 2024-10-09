use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    account::{
        errors::Result,
        serialization::{DerSignature, DerVerifyingKey},
        signed::{ChallengeRequest, ChallengeResponse},
    },
    jwt::{Jwt, JwtCredentialClaims, JwtSubject},
    keys::{poa::Poa, EphemeralEcdsaKey, SecureEcdsaKey},
};

use super::auth::WalletCertificate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction<T> {
    pub instruction: ChallengeResponse<T>,
    pub certificate: WalletCertificate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckPin;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePinStart {
    pub pin_pubkey: DerVerifyingKey,
    pub pop_pin_pubkey: DerSignature,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWte {
    pub key_identifier: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWteResult {
    pub wte: Jwt<JwtCredentialClaims>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructPoa {
    pub key_identifiers: Vec<String>,
    pub aud: String,
    pub nonce: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructPoaResult {
    pub poa: Poa,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionChallengeRequest {
    pub request: ChallengeRequest,
    pub certificate: WalletCertificate,
}

pub trait InstructionAndResult: Serialize + DeserializeOwned {
    const NAME: &'static str;

    type Result: Serialize + DeserializeOwned;
}

impl<R> JwtSubject for InstructionResultClaims<R> {
    const SUB: &'static str = "instruction_result";
}

impl InstructionAndResult for CheckPin {
    const NAME: &'static str = "check_pin";

    type Result = ();
}

impl InstructionAndResult for ChangePinStart {
    const NAME: &'static str = "change_pin_start";

    type Result = WalletCertificate;
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

impl InstructionAndResult for IssueWte {
    const NAME: &'static str = "issue_wte";

    type Result = IssueWteResult;
}

impl InstructionAndResult for ConstructPoa {
    const NAME: &'static str = "construct_poa";

    type Result = ConstructPoaResult;
}

impl<T> Instruction<T>
where
    T: Serialize + DeserializeOwned,
{
    pub async fn new_signed(
        instruction: T,
        challenge: Vec<u8>,
        instruction_sequence_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        certificate: WalletCertificate,
    ) -> Result<Self> {
        let signed = ChallengeResponse::sign(
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

impl InstructionChallengeRequest {
    pub async fn new_signed<I>(
        instruction_sequence_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        certificate: WalletCertificate,
    ) -> Result<Self>
    where
        I: InstructionAndResult,
    {
        let signed = ChallengeRequest::sign(instruction_sequence_number, I::NAME.to_string(), hw_privkey).await?;

        Ok(Self {
            request: signed,
            certificate,
        })
    }
}
