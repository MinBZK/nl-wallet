use std::num::NonZeroUsize;

use derive_more::Constructor;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crypto::p256_der::DerSignature;
use crypto::p256_der::DerVerifyingKey;
use jwt::JwtSubject;
use jwt::UnverifiedJwt;
use jwt::pop::JwtPopClaims;
use jwt::wua::WuaDisclosure;
use sd_jwt::sd_jwt::UnverifiedSdJwt;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;

use crate::signed::ChallengeRequest;
use crate::signed::ChallengeResponse;

use super::registration::WalletCertificate;

/// Request for a challenge, sent by wallet to account server before sending an instruction.
#[derive(Debug, Serialize, Deserialize, Constructor)]
pub struct InstructionChallengeRequest {
    pub request: ChallengeRequest,
    pub certificate: WalletCertificate,
}

/// Request to execute an instruction, sent by wallet to account server after receiving the challenge.
#[derive(Debug, Serialize, Deserialize, Constructor)]
pub struct Instruction<T> {
    pub instruction: ChallengeResponse<T>,
    pub certificate: WalletCertificate,
}

/// The result of an instruction, sent by account server to wallet after successfully executing the instruction.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionResultMessage<R> {
    pub result: InstructionResult<R>,
}

pub type InstructionResult<R> = UnverifiedJwt<InstructionResultClaims<R>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstructionResultClaims<R> {
    pub result: R,

    pub iss: String,
    pub iat: u64,
}

impl<R> JwtSubject for InstructionResultClaims<R> {
    const SUB: &'static str = "instruction_result";
}

/// Links an instruction with its result type and name string.
pub trait InstructionAndResult: Serialize + DeserializeOwned {
    const NAME: &'static str;

    type Result: Serialize + DeserializeOwned;
}

// CheckPin instruction.

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPin;

impl InstructionAndResult for CheckPin {
    const NAME: &'static str = "check_pin";

    type Result = ();
}

// ChangePinStart instruction.

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePinStart {
    #[serde_as(as = "Base64")]
    pub pin_pubkey: DerVerifyingKey,
    #[serde_as(as = "Base64")]
    pub pop_pin_pubkey: DerSignature,
}

impl InstructionAndResult for ChangePinStart {
    const NAME: &'static str = "change_pin_start";

    type Result = WalletCertificate;
}

// ChangePinCommit instruction.

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePinCommit {}

impl InstructionAndResult for ChangePinCommit {
    const NAME: &'static str = "change_pin_commit";

    type Result = ();
}

// ChangePinRollback instruction.

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePinRollback {}

impl InstructionAndResult for ChangePinRollback {
    const NAME: &'static str = "change_pin_rollback";

    type Result = ();
}

// PerformIssuance instruction.

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformIssuance {
    pub key_count: NonZeroUsize,
    pub aud: String,
    pub nonce: Option<String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformIssuanceResult {
    pub key_identifiers: VecNonEmpty<String>,
    pub pops: VecNonEmpty<UnverifiedJwt<JwtPopClaims>>,
    pub poa: Option<Poa>,
}

impl InstructionAndResult for PerformIssuance {
    const NAME: &'static str = "perform_issuance";

    type Result = PerformIssuanceResult;
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformIssuanceWithWua {
    #[serde(flatten)]
    pub issuance_instruction: PerformIssuance,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformIssuanceWithWuaResult {
    #[serde(flatten)]
    pub issuance_result: PerformIssuanceResult,
    pub wua_disclosure: WuaDisclosure,
}

impl InstructionAndResult for PerformIssuanceWithWua {
    const NAME: &'static str = "perform_issuance_with_wua";

    type Result = PerformIssuanceWithWuaResult;
}

// Sign instruction.

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Sign {
    #[serde_as(as = "Vec<(Base64, _)>")]
    pub messages_with_identifiers: Vec<(Vec<u8>, Vec<String>)>,
    pub poa_nonce: Option<String>,
    pub poa_aud: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SignResult {
    #[serde_as(as = "Vec<Vec<Base64>>")]
    pub signatures: Vec<Vec<DerSignature>>,
    pub poa: Option<Poa>,
}

impl InstructionAndResult for Sign {
    const NAME: &'static str = "sign";

    type Result = SignResult;
}

// DiscloseRecoveryCode instruction.

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscloseRecoveryCode {
    /// PID in SD JWT format with one disclosure: the recovery code
    pub recovery_code_disclosure: UnverifiedSdJwt,
}

impl InstructionAndResult for DiscloseRecoveryCode {
    const NAME: &'static str = "disclose_recovery_code";

    type Result = DiscloseRecoveryCodeResult;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscloseRecoveryCodeResult {
    pub transfer_available: bool,
}

// PrepareTransfer instruction.

#[derive(Debug, Serialize, Deserialize)]
pub struct PrepareTransfer {
    pub app_version: String,
}

impl InstructionAndResult for PrepareTransfer {
    const NAME: &'static str = "prepare_transfer";

    type Result = PrepareTransferResult;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrepareTransferResult {
    pub transfer_session_id: String,
}

#[cfg(feature = "client")]
mod client {
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use crypto::keys::EphemeralEcdsaKey;
    use crypto::keys::SecureEcdsaKey;
    use platform_support::attested_key::AppleAttestedKey;

    use crate::error::EncodeError;
    use crate::messages::registration::WalletCertificate;
    use crate::signed::ChallengeRequest;
    use crate::signed::ChallengeResponse;

    use super::Instruction;
    use super::InstructionAndResult;
    use super::InstructionChallengeRequest;

    // Constructors for Instruction.
    impl<T> Instruction<T>
    where
        T: Serialize + DeserializeOwned,
    {
        pub async fn new_apple(
            instruction: T,
            challenge: Vec<u8>,
            instruction_sequence_number: u64,
            attested_key: &impl AppleAttestedKey,
            pin_privkey: &impl EphemeralEcdsaKey,
            certificate: WalletCertificate,
        ) -> Result<Self, EncodeError> {
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
        ) -> Result<Self, EncodeError> {
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
        pub async fn new_apple<I>(
            wallet_id: String,
            instruction_sequence_number: u64,
            attested_key: &impl AppleAttestedKey,
            certificate: WalletCertificate,
        ) -> Result<Self, EncodeError>
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
        ) -> Result<Self, EncodeError>
        where
            I: InstructionAndResult,
        {
            let challenge_request =
                ChallengeRequest::sign_google(wallet_id, instruction_sequence_number, I::NAME.to_string(), hw_privkey)
                    .await?;

            Ok(Self::new(challenge_request, certificate))
        }
    }
}
