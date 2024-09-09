use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    account::{
        errors::Result,
        serialization::{DerSignature, DerVerifyingKey},
        signed::{SignedChallengeRequest, SignedChallengeResponse},
    },
    jwt::{Jwt, JwtSubject},
    keys::{EphemeralEcdsaKey, SecureEcdsaKey},
};

use super::auth::WalletCertificate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction<T> {
    pub instruction: SignedChallengeResponse<T>,
    pub certificate: WalletCertificate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckPin;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionChallengeRequest {
    pub request: SignedChallengeRequest,
    pub certificate: WalletCertificate,
}

pub trait InstructionEndpoint: Serialize + DeserializeOwned {
    const ENDPOINT: &'static str;

    type Result: Serialize + DeserializeOwned;
}

impl<R> JwtSubject for InstructionResultClaims<R> {
    const SUB: &'static str = "instruction_result";
}

impl InstructionEndpoint for CheckPin {
    const ENDPOINT: &'static str = "check_pin";

    type Result = ();
}

impl InstructionEndpoint for GenerateKey {
    const ENDPOINT: &'static str = "generate_key";

    type Result = GenerateKeyResult;
}

impl InstructionEndpoint for Sign {
    const ENDPOINT: &'static str = "sign";

    type Result = SignResult;
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
        let signed = SignedChallengeResponse::sign(
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
