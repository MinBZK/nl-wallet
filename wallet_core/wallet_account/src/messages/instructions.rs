use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use wallet_common::apple::AppleAttestedKey;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtCredentialClaims;
use wallet_common::jwt::JwtSubject;
use wallet_common::keys::poa::Poa;
use wallet_common::keys::EphemeralEcdsaKey;
use wallet_common::keys::SecureEcdsaKey;
use wallet_common::p256_der::DerSignature;
use wallet_common::p256_der::DerVerifyingKey;
use wallet_common::vec_at_least::VecAtLeastTwoUnique;
use wallet_common::wte::WteClaims;

use crate::errors::Result;
use crate::signed::ChallengeRequest;
use crate::signed::ChallengeResponse;

use super::auth::WalletCertificate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction<T> {
    pub instruction: ChallengeResponse<T>,
    pub certificate: WalletCertificate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckPin;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePinStart {
    #[serde_as(as = "Base64")]
    pub pin_pubkey: DerVerifyingKey,
    #[serde_as(as = "Base64")]
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

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateKeyResult {
    #[serde_as(as = "Vec<(_, Base64)>")]
    pub public_keys: Vec<(String, DerVerifyingKey)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sign {
    pub messages_with_identifiers: Vec<(Vec<u8>, Vec<String>)>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SignResult {
    #[serde_as(as = "Vec<Vec<Base64>>")]
    pub signatures: Vec<Vec<DerSignature>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWte {
    pub key_identifier: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWteResult {
    pub wte: Jwt<JwtCredentialClaims<WteClaims>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructPoa {
    pub key_identifiers: VecAtLeastTwoUnique<String>,
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
    fn new(instruction: ChallengeResponse<T>, certificate: WalletCertificate) -> Self {
        Self {
            instruction,
            certificate,
        }
    }

    pub async fn new_apple(
        instruction: T,
        challenge: Vec<u8>,
        instruction_sequence_number: u64,
        attested_key: &impl AppleAttestedKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        certificate: WalletCertificate,
    ) -> Result<Self> {
        let challenge_response = ChallengeResponse::sign_apple(
            instruction,
            challenge,
            instruction_sequence_number,
            attested_key,
            pin_privkey,
        )
        .await?;

        Ok(Self::new(challenge_response, certificate))
    }

    pub async fn new_google(
        instruction: T,
        challenge: Vec<u8>,
        instruction_sequence_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        certificate: WalletCertificate,
    ) -> Result<Self> {
        let challenge_response = ChallengeResponse::sign_google(
            instruction,
            challenge,
            instruction_sequence_number,
            hw_privkey,
            pin_privkey,
        )
        .await?;

        Ok(Self::new(challenge_response, certificate))
    }
}

impl InstructionChallengeRequest {
    fn new(request: ChallengeRequest, certificate: WalletCertificate) -> Self {
        Self { request, certificate }
    }

    pub async fn new_apple<I>(
        wallet_id: String,
        instruction_sequence_number: u64,
        attested_key: &impl AppleAttestedKey,
        certificate: WalletCertificate,
    ) -> Result<Self>
    where
        I: InstructionAndResult,
    {
        let challenge_request = ChallengeRequest::sign_apple(
            wallet_id,
            instruction_sequence_number,
            I::NAME.to_string(),
            attested_key,
        )
        .await?;

        Ok(Self::new(challenge_request, certificate))
    }

    pub async fn new_google<I>(
        wallet_id: String,
        instruction_sequence_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        certificate: WalletCertificate,
    ) -> Result<Self>
    where
        I: InstructionAndResult,
    {
        let challenge_request =
            ChallengeRequest::sign_google(wallet_id, instruction_sequence_number, I::NAME.to_string(), hw_privkey)
                .await?;

        Ok(Self::new(challenge_request, certificate))
    }
}
