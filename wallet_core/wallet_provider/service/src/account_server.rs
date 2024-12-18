use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::VerifyingKey;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use tracing::debug;
use uuid::Uuid;

use wallet_common::account::errors::Error as AccountError;
use wallet_common::account::messages::auth::Registration;
use wallet_common::account::messages::auth::WalletCertificate;
use wallet_common::account::messages::errors::IncorrectPinData;
use wallet_common::account::messages::errors::PinTimeoutData;
use wallet_common::account::messages::instructions::ChangePinRollback;
use wallet_common::account::messages::instructions::ChangePinStart;
use wallet_common::account::messages::instructions::Instruction;
use wallet_common::account::messages::instructions::InstructionAndResult;
use wallet_common::account::messages::instructions::InstructionChallengeRequest;
use wallet_common::account::messages::instructions::InstructionResult;
use wallet_common::account::messages::instructions::InstructionResultClaims;
use wallet_common::account::signed::ChallengeResponse;
use wallet_common::account::signed::ChallengeResponsePayload;
use wallet_common::account::signed::SequenceNumberComparison;
use wallet_common::generator::Generator;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtError;
use wallet_common::jwt::JwtSubject;
use wallet_common::keys::poa::PoaError;
use wallet_common::utils::random_bytes;
use wallet_common::utils::random_string;
use wallet_provider_domain::model::encrypted::Encrypted;
use wallet_provider_domain::model::encrypter::Decrypter;
use wallet_provider_domain::model::encrypter::Encrypter;
use wallet_provider_domain::model::hsm::Hsm;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::pin_policy::PinPolicyEvaluation;
use wallet_provider_domain::model::pin_policy::PinPolicyEvaluator;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::hsm::HsmError;
use crate::instructions::HandleInstruction;
use crate::instructions::ValidateInstruction;
use crate::keys::InstructionResultSigningKey;
use crate::keys::WalletCertificateSigningKey;
use crate::wallet_certificate::new_wallet_certificate;
use crate::wallet_certificate::parse_claims_and_retrieve_wallet_user;
use crate::wallet_certificate::verify_wallet_certificate;
use crate::wallet_certificate::verify_wallet_certificate_public_keys;
use crate::wte_issuer::WteIssuer;

#[derive(Debug, thiserror::Error)]
pub enum AccountServerInitError {
    // Do not format original error to prevent potentially leaking key material
    #[error("server private key decoding error")]
    PrivateKeyDecoding(#[from] p256::pkcs8::Error),
    #[error("server public key decoding error")]
    PublicKeyDecoding(#[from] HsmError),
}

#[derive(Debug, thiserror::Error)]
pub enum ChallengeError {
    #[error("challenge signing error: {0}")]
    ChallengeSigning(#[from] JwtError),
    #[error("could not store challenge: {0}")]
    Storage(#[from] PersistenceError),
    #[error("challenge message validation error: {0}")]
    Validation(#[from] wallet_common::account::errors::Error),
    #[error("wallet certificate validation error: {0}")]
    WalletCertificate(#[from] WalletCertificateError),
    #[error("instruction sequence number validation failed")]
    SequenceNumberValidation,
}

#[derive(Debug, thiserror::Error)]
pub enum WalletCertificateError {
    #[error("registration PIN public key DER encoding error: {0}")]
    PinPubKeyEncoding(#[source] der::Error),
    #[error("registration PIN public key decoding error: {0}")]
    PinPubKeyDecoding(#[source] p256::pkcs8::spki::Error),
    #[error("stored hardware public key does not match provided one")]
    HwPubKeyMismatch,
    #[error("stored pin public key does not match provided one")]
    PinPubKeyMismatch,
    #[error("validation failed: {0}")]
    Validation(#[from] JwtError),
    #[error("no registered wallet user found")]
    UserNotRegistered,
    #[error("registered wallet user blocked")]
    UserBlocked,
    #[error("could not retrieve registered wallet user: {0}")]
    Persistence(#[from] PersistenceError),
    #[error("hsm error: {0}")]
    HsmError(#[from] HsmError),
    #[error("wallet certificate JWT signing error: {0}")]
    JwtSigning(#[source] JwtError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("registration challenge UTF-8 decoding error: {0}")]
    ChallengeDecoding(#[source] std::string::FromUtf8Error),
    #[error("registration challenge validation error: {0}")]
    ChallengeValidation(#[source] JwtError),
    #[error("registration message parsing error: {0}")]
    MessageParsing(#[source] wallet_common::account::errors::Error),
    #[error("registration message validation error: {0}")]
    MessageValidation(#[source] wallet_common::account::errors::Error),
    #[error("incorrect registration serial number (expected: {expected:?}, received: {received:?})")]
    SerialNumberMismatch { expected: u64, received: u64 },
    #[error("could not store certificate: {0}")]
    CertificateStorage(#[from] PersistenceError),
    #[error("registration PIN public key DER encoding error: {0}")]
    PinPubKeyEncoding(#[source] der::Error),
    #[error("wallet certificate validation error: {0}")]
    WalletCertificate(#[from] WalletCertificateError),
    #[error("hsm error: {0}")]
    HsmError(#[from] HsmError),
}

#[derive(Debug, thiserror::Error)]
pub enum InstructionError {
    #[error("wallet certificate validation error: {0}")]
    WalletCertificate(#[from] WalletCertificateError),
    #[error("instruction validation error: {0}")]
    Validation(#[from] InstructionValidationError),
    #[error("instruction validation pin error ({0:?})")]
    IncorrectPin(IncorrectPinData),
    #[error("instruction validation pin timeout ({0:?})")]
    PinTimeout(PinTimeoutData),
    #[error("account is blocked")]
    AccountBlocked,
    #[error("instruction result signing error: {0}")]
    Signing(#[source] JwtError),
    #[error("persistence error: {0}")]
    Storage(#[from] PersistenceError),
    #[error("hsm error: {0}")]
    HsmError(#[from] HsmError),
    #[error("WTE issuance: {0}")]
    WteIssuance(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("instruction referenced nonexisting key: {0}")]
    NonexistingKey(String),
    #[error("PoA construction error: {0}")]
    Poa(#[from] PoaError),
}

#[derive(Debug, thiserror::Error)]
pub enum InstructionValidationError {
    #[error("instruction sequence number mismatch")]
    SequenceNumberMismatch,
    #[error("instruction challenge mismatch")]
    ChallengeMismatch,
    #[error("instruction challenge timeout")]
    ChallengeTimeout,
    #[error("instruction verification failed: {0}")]
    VerificationFailed(#[source] AccountError),
    #[error("pin change is in progress")]
    PinChangeInProgress,
    #[error("pin change is not in progress")]
    PinChangeNotInProgress,
    #[error("hsm error: {0}")]
    HsmError(#[from] HsmError),
    #[error("WTE already issued")]
    WteAlreadyIssued,
    #[error("received instruction to sign a PoA with the Sign instruction")]
    PoaMessage,
}

impl From<PinPolicyEvaluation> for InstructionError {
    fn from(value: PinPolicyEvaluation) -> Self {
        match value {
            PinPolicyEvaluation::Failed {
                attempts_left_in_round,
                is_final_round,
            } => InstructionError::IncorrectPin(IncorrectPinData {
                attempts_left_in_round,
                is_final_round,
            }),
            PinPolicyEvaluation::Timeout { timeout } | PinPolicyEvaluation::InTimeout { timeout } => {
                InstructionError::PinTimeout(PinTimeoutData {
                    time_left_in_ms: u64::try_from(timeout.num_milliseconds())
                        .expect("number of milliseconds in timeout cannot be negative"),
                })
            }
            PinPolicyEvaluation::BlockedPermanently => InstructionError::AccountBlocked,
        }
    }
}

/// Used as the challenge in the challenge-response protocol during wallet registration.
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct RegistrationChallengeClaims {
    wallet_id: String,
    exp: u64,

    /// Random bytes to serve as the actual challenge for the wallet to sign.
    #[serde_as(as = "Base64")]
    random: Vec<u8>,
}

impl JwtSubject for RegistrationChallengeClaims {
    const SUB: &'static str = "registration_challenge";
}

pub struct AccountServer {
    instruction_challenge_timeout: Duration,

    pub name: String,

    wallet_certificate_signing_pubkey: EcdsaDecodingKey,
    encryption_key_identifier: String,
    pin_public_disclosure_protection_key_identifier: String,
}

impl AccountServer {
    pub fn new(
        instruction_challenge_timeout: Duration,
        name: String,
        wallet_certificate_signing_pubkey: EcdsaDecodingKey,
        encryption_key_identifier: String,
        pin_public_disclosure_protection_key_identifier: String,
    ) -> Result<Self, AccountServerInitError> {
        Ok(AccountServer {
            instruction_challenge_timeout,
            name,
            wallet_certificate_signing_pubkey,
            encryption_key_identifier,
            pin_public_disclosure_protection_key_identifier,
        })
    }

    // Only used for registration. When a registered user sends an instruction, we should store
    // the challenge per user, instead globally.
    pub async fn registration_challenge(
        &self,
        certificate_signing_key: &impl WalletCertificateSigningKey,
    ) -> Result<Vec<u8>, ChallengeError> {
        let challenge = Jwt::sign_with_sub(
            &RegistrationChallengeClaims {
                wallet_id: random_string(32),
                random: random_bytes(32),
                exp: jsonwebtoken::get_current_timestamp() + 60,
            },
            certificate_signing_key,
        )
        .await
        .map_err(ChallengeError::ChallengeSigning)?
        .0
        .as_bytes()
        .to_vec();
        Ok(challenge)
    }

    pub async fn register<T, R, H>(
        &self,
        certificate_signing_key: &impl WalletCertificateSigningKey,
        uuid_generator: &impl Generator<Uuid>,
        repositories: &R,
        hsm: &H,
        registration_message: ChallengeResponse<Registration>,
    ) -> Result<WalletCertificate, RegistrationError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
    {
        debug!("Parsing message to lookup public keys");

        // We don't have the public keys yet against which to verify the message, as those are contained within the
        // message (like in X509 certificate requests). So first parse it to grab the public keys from it.
        let unverified = registration_message
            .dangerous_parse_unverified()
            .map_err(RegistrationError::MessageParsing)?;

        debug!("Extracting challenge, wallet id, hw pubkey and pin pubkey");

        let challenge = &unverified.challenge;
        let wallet_id =
            Self::verify_registration_challenge(&self.wallet_certificate_signing_pubkey, challenge)?.wallet_id;

        let hw_pubkey = unverified.payload.hw_pubkey.0;
        let pin_pubkey = unverified.payload.pin_pubkey.0;

        debug!("Checking if challenge is signed with the provided hw and pin keys");

        registration_message
            .parse_and_verify_ecdsa(challenge, SequenceNumberComparison::EqualTo(0), &hw_pubkey, &pin_pubkey)
            .map_err(RegistrationError::MessageValidation)?;

        debug!("Starting database transaction");

        let encrypted_pin_pubkey = Encrypter::encrypt(hsm, &self.encryption_key_identifier, pin_pubkey).await?;

        let tx = repositories.begin_transaction().await?;

        debug!("Creating new wallet user");

        let uuid = uuid_generator.generate();
        repositories
            .create_wallet_user(
                &tx,
                WalletUserCreate {
                    id: uuid,
                    wallet_id: wallet_id.clone(),
                    hw_pubkey,
                    encrypted_pin_pubkey,
                },
            )
            .await?;

        debug!("Generating new wallet certificate for user {}", uuid);

        let wallet_certificate = new_wallet_certificate(
            self.name.clone(),
            &self.pin_public_disclosure_protection_key_identifier,
            certificate_signing_key,
            wallet_id,
            hw_pubkey,
            &pin_pubkey,
            hsm,
        )
        .await?;

        tx.commit().await?;

        Ok(wallet_certificate)
    }

    pub async fn instruction_challenge<T, R, H>(
        &self,
        challenge_request: InstructionChallengeRequest,
        repositories: &R,
        time_generator: &impl Generator<DateTime<Utc>>,
        hsm: &H,
    ) -> Result<Vec<u8>, ChallengeError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
    {
        debug!("Parse certificate and retrieving wallet user");
        let (user, claims) = parse_claims_and_retrieve_wallet_user(
            &challenge_request.certificate,
            &self.wallet_certificate_signing_pubkey,
            repositories,
        )
        .await?;

        debug!("Parsing and verifying challenge request for user {}", user.id);
        let request = challenge_request.request.parse_and_verify_ecdsa(
            &claims.wallet_id,
            SequenceNumberComparison::LargerThan(user.instruction_sequence_number),
            &user.hw_pubkey.0,
        )?;

        debug!("Verifying wallet certificate");
        verify_wallet_certificate_public_keys(
            claims,
            &self.pin_public_disclosure_protection_key_identifier,
            &self.encryption_key_identifier,
            &user.hw_pubkey,
            if request.instruction_name == ChangePinRollback::NAME {
                user.encrypted_previous_pin_pubkey.unwrap_or(user.encrypted_pin_pubkey)
            } else {
                user.encrypted_pin_pubkey
            },
            hsm,
        )
        .await?;

        debug!("Challenge request valid, persisting generated challenge and incremented sequence number");
        let challenge = InstructionChallenge {
            bytes: random_bytes(32),
            expiration_date_time: time_generator.generate() + self.instruction_challenge_timeout,
        };

        debug!("Starting database transaction");
        let tx = repositories.begin_transaction().await?;
        repositories
            .update_instruction_challenge_and_sequence_number(
                &tx,
                &user.wallet_id,
                challenge.clone(),
                request.sequence_number,
            )
            .await?;
        tx.commit().await?;

        debug!("Responding with generated challenge");
        Ok(challenge.bytes)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn handle_instruction<T, R, I, IR, G, H>(
        &self,
        instruction: Instruction<I>,
        instruction_result_signing_key: &impl InstructionResultSigningKey,
        generators: &G,
        repositories: &R,
        pin_policy: &impl PinPolicyEvaluator,
        wallet_user_hsm: &H,
        wte_issuer: &impl WteIssuer,
    ) -> Result<InstructionResult<IR>, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        I: HandleInstruction<Result = IR> + InstructionAndResult + ValidateInstruction + Serialize + DeserializeOwned,
        IR: Serialize + DeserializeOwned,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
        H: WalletUserHsm<Error = HsmError>
            + Hsm<Error = HsmError>
            + Decrypter<VerifyingKey, Error = HsmError>
            + Encrypter<VerifyingKey, Error = HsmError>,
    {
        let (wallet_user, instruction_payload) = self
            .verify_and_extract_instruction(
                instruction,
                generators,
                repositories,
                pin_policy,
                wallet_user_hsm,
                |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
            )
            .await?;

        let instruction_result = instruction_payload
            .handle(&wallet_user, generators, repositories, wallet_user_hsm, wte_issuer)
            .await?;

        self.sign_instruction_result(instruction_result_signing_key, instruction_result)
            .await
    }

    // Implements the logic behind the ChangePinStart instruction.
    //
    // The ChangePinStart instruction is handled here explicitly instead of relying on the generic instruction
    // handling mechanism. The reason is that a new wallet_certificate has to be constructed here, similar to the
    // registration functionality. Since both methods (registration and change_pin_start) mostly use the same
    // dependencies (which are different from the dependencies for handling instructions) they are kept together here.
    //
    // Changing the PIN is implemented by saving the current PIN in a separate location and replacing it by the new
    // PIN. From then on, the new PIN is used, although the pin change has to be committed first. A rollback is
    // verified against the previous PIN that is stored separately.
    pub async fn handle_change_pin_start_instruction<T, R, G, H>(
        &self,
        instruction: Instruction<ChangePinStart>,
        signing_keys: (&impl InstructionResultSigningKey, &impl WalletCertificateSigningKey),
        generators: &G,
        repositories: &R,
        pin_policy: &impl PinPolicyEvaluator,
        wallet_user_hsm: &H,
    ) -> Result<InstructionResult<WalletCertificate>, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
        H: WalletUserHsm<Error = HsmError>
            + Hsm<Error = HsmError>
            + Decrypter<VerifyingKey, Error = HsmError>
            + Encrypter<VerifyingKey, Error = HsmError>,
    {
        let (wallet_user, instruction_payload) = self
            .verify_and_extract_instruction(
                instruction,
                generators,
                repositories,
                pin_policy,
                wallet_user_hsm,
                |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
            )
            .await?;

        let pin_pubkey = instruction_payload.pin_pubkey.0;

        if let Some(challenge) = wallet_user.instruction_challenge {
            pin_pubkey
                .verify(challenge.bytes.as_slice(), &instruction_payload.pop_pin_pubkey.0)
                .map_err(|_| InstructionError::Validation(InstructionValidationError::ChallengeMismatch))?;
        } else {
            return Err(InstructionError::Validation(
                InstructionValidationError::ChallengeMismatch,
            ));
        }

        let encrypted_pin_pubkey =
            Encrypter::encrypt(wallet_user_hsm, &self.encryption_key_identifier, pin_pubkey).await?;

        let tx = repositories.begin_transaction().await?;

        repositories
            .change_pin(&tx, wallet_user.wallet_id.as_str(), encrypted_pin_pubkey)
            .await?;

        let wallet_certificate = new_wallet_certificate(
            self.name.clone(),
            &self.pin_public_disclosure_protection_key_identifier,
            signing_keys.1,
            wallet_user.wallet_id,
            wallet_user.hw_pubkey.0,
            &pin_pubkey,
            wallet_user_hsm,
        )
        .await?;

        let result = self.sign_instruction_result(signing_keys.0, wallet_certificate).await;

        tx.commit().await?;

        result
    }

    // Implements the logic behind the ChangePinRollback instruction.
    //
    // The ChangePinRollback instruction is handled here explicitly instead of relying on the generic instruction
    // handling mechanism. The reason is that the wallet_certificate included in the instruction has to be verified
    // against the temporarily saved previous pin public key of the wallet_user.
    pub async fn handle_change_pin_rollback_instruction<T, R, G, H>(
        &self,
        instruction: Instruction<ChangePinRollback>,
        instruction_result_signing_key: &impl InstructionResultSigningKey,
        generators: &G,
        repositories: &R,
        pin_policy: &impl PinPolicyEvaluator,
        wallet_user_hsm: &H,
    ) -> Result<InstructionResult<()>, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
        H: WalletUserHsm<Error = HsmError> + Hsm<Error = HsmError> + Decrypter<VerifyingKey, Error = HsmError>,
    {
        let (wallet_user, _) = self
            .verify_and_extract_instruction(
                instruction,
                generators,
                repositories,
                pin_policy,
                wallet_user_hsm,
                |wallet_user| {
                    wallet_user
                        .encrypted_previous_pin_pubkey
                        .clone()
                        .unwrap_or(wallet_user.encrypted_pin_pubkey.clone())
                },
            )
            .await?;

        debug!(
            "Starting database transaction and instruction handling process for user {}",
            &wallet_user.id
        );

        let tx = repositories.begin_transaction().await?;

        repositories
            .rollback_pin_change(&tx, wallet_user.wallet_id.as_str())
            .await?;

        tx.commit().await?;

        self.sign_instruction_result(instruction_result_signing_key, ()).await
    }

    async fn verify_and_extract_instruction<T, R, I, G, H, F>(
        &self,
        instruction: Instruction<I>,
        generators: &G,
        repositories: &R,
        pin_policy: &impl PinPolicyEvaluator,
        wallet_user_hsm: &H,
        pin_pubkey: F,
    ) -> Result<(WalletUser, I), InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        I: InstructionAndResult + ValidateInstruction,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
        H: Hsm<Error = HsmError> + Decrypter<VerifyingKey, Error = HsmError>,
        F: Fn(&WalletUser) -> Encrypted<VerifyingKey>,
    {
        debug!("Verifying certificate and retrieving wallet user");

        let wallet_user = verify_wallet_certificate(
            &instruction.certificate,
            &self.wallet_certificate_signing_pubkey,
            &self.pin_public_disclosure_protection_key_identifier,
            &self.encryption_key_identifier,
            repositories,
            wallet_user_hsm,
            pin_pubkey,
        )
        .await?;

        debug!(
            "Starting database transaction and instruction handling process for user {}",
            &wallet_user.id
        );

        let tx = repositories.begin_transaction().await?;

        debug!("Clearing instruction challenge");

        repositories
            .clear_instruction_challenge(&tx, &wallet_user.wallet_id)
            .await?;

        debug!("Evaluating pin policy state");

        let pin_eval = pin_policy.evaluate(
            wallet_user.unsuccessful_pin_entries + 1,
            wallet_user.last_unsuccessful_pin_entry,
            generators.generate(),
        );

        // An evaluation result of blocked permanently can only occur once. This fact is stored in the database
        // for the wallet_user. Subsequent calls will verify if the user is blocked against the database.
        if matches!(pin_eval, PinPolicyEvaluation::InTimeout { timeout: _ }) {
            tx.commit().await?;
            return Err(pin_eval.into());
        }

        debug!("Verifying instruction");

        let verification_result = Self::verify_instruction(
            self.encryption_key_identifier.as_str(),
            instruction,
            &wallet_user,
            generators,
            wallet_user_hsm,
        )
        .await;

        match verification_result {
            Ok(challenge_response_payload) => {
                debug!("Instruction successfully verified, validating instruction");

                challenge_response_payload.payload.validate_instruction(&wallet_user)?;

                debug!("Instruction successfully validated, resetting pin retries");

                repositories
                    .reset_unsuccessful_pin_entries(&tx, &wallet_user.wallet_id)
                    .await?;

                debug!(
                    "Updating instruction sequence number to {}",
                    challenge_response_payload.sequence_number
                );

                repositories
                    .update_instruction_sequence_number(
                        &tx,
                        &wallet_user.wallet_id,
                        challenge_response_payload.sequence_number,
                    )
                    .await?;

                tx.commit().await?;

                Ok((wallet_user, challenge_response_payload.payload))
            }
            Err(validation_error) => {
                let error = if matches!(validation_error, InstructionValidationError::VerificationFailed(_)) {
                    debug!("Instruction validation failed, registering unsuccessful pin entry");

                    repositories
                        .register_unsuccessful_pin_entry(
                            &tx,
                            &wallet_user.wallet_id,
                            matches!(pin_eval, PinPolicyEvaluation::BlockedPermanently),
                            generators.generate(),
                        )
                        .await?;
                    Err(pin_eval.into())
                } else {
                    Err(validation_error)?
                };

                tx.commit().await?;
                error
            }
        }
    }

    fn verify_registration_challenge(
        certificate_signing_pubkey: &EcdsaDecodingKey,
        challenge: &[u8],
    ) -> Result<RegistrationChallengeClaims, RegistrationError> {
        Jwt::parse_and_verify_with_sub(
            &String::from_utf8(challenge.to_owned())
                .map_err(RegistrationError::ChallengeDecoding)?
                .into(),
            certificate_signing_pubkey,
        )
        .map_err(RegistrationError::ChallengeValidation)
    }

    fn verify_instruction_challenge<'a>(
        wallet_user: &'a WalletUser,
        time_generator: &impl Generator<DateTime<Utc>>,
    ) -> Result<&'a InstructionChallenge, InstructionValidationError> {
        let challenge = wallet_user
            .instruction_challenge
            .as_ref()
            .ok_or(InstructionValidationError::ChallengeMismatch)?;

        if challenge.expiration_date_time < time_generator.generate() {
            return Err(InstructionValidationError::ChallengeTimeout);
        }

        Ok(challenge)
    }

    async fn verify_instruction<I, D>(
        encryption_key_identifier: &str,
        instruction: Instruction<I>,
        wallet_user: &WalletUser,
        time_generator: &impl Generator<DateTime<Utc>>,
        verifying_key_decrypter: &D,
    ) -> Result<ChallengeResponsePayload<I>, InstructionValidationError>
    where
        I: InstructionAndResult,
        D: Decrypter<VerifyingKey, Error = HsmError>,
    {
        let challenge = Self::verify_instruction_challenge(wallet_user, time_generator)?;

        let pin_pubkey = verifying_key_decrypter
            .decrypt(encryption_key_identifier, wallet_user.encrypted_pin_pubkey.clone())
            .await?;

        let parsed = instruction
            .instruction
            .parse_and_verify_ecdsa(
                &challenge.bytes,
                SequenceNumberComparison::LargerThan(wallet_user.instruction_sequence_number),
                &wallet_user.hw_pubkey.0,
                &pin_pubkey,
            )
            .map_err(InstructionValidationError::VerificationFailed)?;

        Ok(parsed)
    }

    async fn sign_instruction_result<R>(
        &self,
        instruction_result_signing_key: &impl InstructionResultSigningKey,
        result: R,
    ) -> Result<InstructionResult<R>, InstructionError>
    where
        R: Serialize + DeserializeOwned,
    {
        let claims = InstructionResultClaims {
            result,
            iss: self.name.to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign_with_sub(&claims, instruction_result_signing_key)
            .await
            .map_err(InstructionError::Signing)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use crate::wallet_certificate;

    use super::*;

    pub fn setup_account_server(certificate_signing_pubkey: &VerifyingKey) -> AccountServer {
        AccountServer::new(
            Duration::from_millis(15000),
            "mock_account_server".into(),
            certificate_signing_pubkey.into(),
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER.to_string(),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER.to_string(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use chrono::TimeZone;
    use hmac::digest::crypto_common::rand_core::OsRng;
    use p256::ecdsa::SigningKey;
    use tokio::sync::OnceCell;

    use wallet_common::account::messages::instructions::ChangePinCommit;
    use wallet_common::account::messages::instructions::CheckPin;
    use wallet_common::account::messages::instructions::InstructionChallengeRequest;
    use wallet_common::keys::EcdsaKey;
    use wallet_provider_domain::generator::mock::MockGenerators;
    use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;
    use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
    use wallet_provider_domain::model::FailingPinPolicy;
    use wallet_provider_domain::model::TimeoutPinPolicy;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_domain::EpochGenerator;
    use wallet_provider_domain::FixedUuidGenerator;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;
    use wallet_provider_persistence::repositories::mock::WalletUserTestRepo;

    use crate::wallet_certificate::mock::WalletCertificateSetup;
    use crate::wallet_certificate::{self};
    use crate::wte_issuer::mock::MockWteIssuer;

    use super::*;

    static HSM: OnceCell<MockPkcs11Client<HsmError>> = OnceCell::const_new();

    async fn get_global_hsm() -> &'static MockPkcs11Client<HsmError> {
        HSM.get_or_init(wallet_certificate::mock::setup_hsm).await
    }

    async fn do_registration(
        account_server: &AccountServer,
        hsm: &MockPkcs11Client<HsmError>,
        certificate_signing_key: &impl WalletCertificateSigningKey,
        hw_privkey: &SigningKey,
        pin_privkey: &SigningKey,
    ) -> WalletCertificate {
        let challenge = account_server
            .registration_challenge(certificate_signing_key)
            .await
            .expect("Could not get registration challenge");

        let registration_message = ChallengeResponse::<Registration>::new_signed(hw_privkey, pin_privkey, challenge)
            .await
            .expect("Could not sign new registration");

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo.expect_create_wallet_user().returning(|_, _| Ok(()));

        account_server
            .register(
                certificate_signing_key,
                &FixedUuidGenerator,
                &wallet_user_repo,
                hsm,
                registration_message,
            )
            .await
            .expect("Could not process registration message at account server")
    }

    async fn setup_and_do_registration() -> (
        WalletCertificateSetup,
        AccountServer,
        WalletCertificate,
        WalletUserTestRepo,
    ) {
        let setup = wallet_certificate::mock::WalletCertificateSetup::new().await;
        let account_server = mock::setup_account_server(&setup.signing_pubkey);

        let cert = do_registration(
            &account_server,
            get_global_hsm().await,
            &setup.signing_key,
            &setup.hw_privkey,
            &setup.pin_privkey,
        )
        .await;

        let repo = WalletUserTestRepo {
            hw_pubkey: setup.hw_pubkey,
            encrypted_pin_pubkey: setup.encrypted_pin_pubkey.clone(),
            previous_encrypted_pin_pubkey: None,
            challenge: None,
            instruction_sequence_number: 0,
        };

        (setup, account_server, cert, repo)
    }

    async fn do_instruction_challenge<I>(
        account_server: &AccountServer,
        repo: &WalletUserTestRepo,
        hw_privkey: &SigningKey,
        wallet_certificate: WalletCertificate,
        instruction_sequence_number: u64,
        hsm: &MockPkcs11Client<HsmError>,
    ) -> Result<Vec<u8>, ChallengeError>
    where
        I: InstructionAndResult,
    {
        account_server
            .instruction_challenge(
                InstructionChallengeRequest::new_signed::<I>(
                    wallet_certificate.dangerous_parse_unverified().unwrap().1.wallet_id,
                    instruction_sequence_number,
                    hw_privkey,
                    wallet_certificate,
                )
                .await
                .unwrap(),
                repo,
                &EpochGenerator,
                hsm,
            )
            .await
    }

    async fn do_check_pin(
        account_server: &AccountServer,
        repo: WalletUserTestRepo,
        wallet_certificate_setup: &WalletCertificateSetup,
        wallet_certificate: WalletCertificate,
        instruction_result_signing_key: &SigningKey,
    ) -> Result<InstructionResult<()>, anyhow::Error> {
        let challenge = do_instruction_challenge::<CheckPin>(
            account_server,
            &repo,
            &wallet_certificate_setup.hw_privkey,
            wallet_certificate.clone(),
            43,
            get_global_hsm().await,
        )
        .await?;

        let instruction_error = account_server
            .handle_instruction(
                Instruction::new_signed(
                    CheckPin,
                    challenge.clone(),
                    43,
                    &wallet_certificate_setup.hw_privkey,
                    &wallet_certificate_setup.pin_privkey,
                    wallet_certificate.clone(),
                )
                .await
                .unwrap(),
                instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    challenge: Some(challenge.clone()),
                    instruction_sequence_number: 43,
                    ..repo.clone()
                },
                &FailingPinPolicy,
                get_global_hsm().await,
                &MockWteIssuer,
            )
            .await
            .expect_err("sequence number mismatch error should result in IncorrectPin error");

        assert_matches!(
            instruction_error,
            InstructionError::IncorrectPin(IncorrectPinData {
                attempts_left_in_round: _,
                is_final_round: _
            })
        );

        let result = account_server
            .handle_instruction(
                Instruction::new_signed(
                    CheckPin,
                    challenge.clone(),
                    44,
                    &wallet_certificate_setup.hw_privkey,
                    &wallet_certificate_setup.pin_privkey,
                    wallet_certificate.clone(),
                )
                .await
                .unwrap(),
                instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    challenge: Some(challenge),
                    instruction_sequence_number: 2,
                    ..repo
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
                &MockWteIssuer,
            )
            .await?;

        Ok(result)
    }

    async fn do_pin_change_start(
        account_server: &AccountServer,
        repo: &WalletUserTestRepo,
        wallet_certificate_setup: &WalletCertificateSetup,
        wallet_certificate: WalletCertificate,
        instruction_result_signing_key: &SigningKey,
    ) -> (SigningKey, VerifyingKey, Encrypted<VerifyingKey>, WalletCertificate) {
        let new_pin_privkey = SigningKey::random(&mut OsRng);
        let new_pin_pubkey = *new_pin_privkey.verifying_key();

        let encrypted_new_pin_pubkey = Encrypter::<VerifyingKey>::encrypt(
            &MockPkcs11Client::<HsmError>::default(),
            crate::wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            new_pin_pubkey,
        )
        .await
        .unwrap();

        let challenge = do_instruction_challenge::<ChangePinStart>(
            account_server,
            repo,
            &wallet_certificate_setup.hw_privkey,
            wallet_certificate.clone(),
            43,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        let pop_pin_pubkey = new_pin_privkey.try_sign(challenge.as_slice()).await.unwrap();

        let new_certificate_result = account_server
            .handle_change_pin_start_instruction(
                Instruction::new_signed(
                    ChangePinStart {
                        pin_pubkey: new_pin_pubkey.into(),
                        pop_pin_pubkey: pop_pin_pubkey.into(),
                    },
                    challenge.clone(),
                    44,
                    &wallet_certificate_setup.hw_privkey,
                    &wallet_certificate_setup.pin_privkey,
                    wallet_certificate.clone(),
                )
                .await
                .unwrap(),
                (instruction_result_signing_key, &wallet_certificate_setup.signing_key),
                &MockGenerators,
                &WalletUserTestRepo {
                    hw_pubkey: wallet_certificate_setup.hw_pubkey,
                    encrypted_pin_pubkey: wallet_certificate_setup.encrypted_pin_pubkey.clone(),
                    previous_encrypted_pin_pubkey: None,
                    challenge: Some(challenge),
                    instruction_sequence_number: 2,
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
            )
            .await
            .expect("should return instruction result");

        let new_certificate = new_certificate_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result")
            .result;

        (
            new_pin_privkey,
            new_pin_pubkey,
            encrypted_new_pin_pubkey,
            new_certificate,
        )
    }

    #[tokio::test]
    async fn test_register() {
        let (setup, account_server, cert, repo) = setup_and_do_registration().await;

        let cert_data = cert
            .parse_and_verify_with_sub(&setup.signing_key.verifying_key().into())
            .expect("Could not parse and verify wallet certificate");
        assert_eq!(cert_data.iss, account_server.name);
        assert_eq!(cert_data.hw_pubkey.0, setup.hw_pubkey);

        verify_wallet_certificate(
            &cert,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            &repo,
            get_global_hsm().await,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn valid_instruction_challenge_should_verify() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;

        let challenge_request = InstructionChallengeRequest::new_signed::<CheckPin>(
            cert.dangerous_parse_unverified().unwrap().1.wallet_id,
            1,
            &setup.hw_privkey,
            cert.clone(),
        )
        .await
        .unwrap();

        let challenge = account_server
            .instruction_challenge(challenge_request, &repo, &EpochGenerator, get_global_hsm().await)
            .await
            .unwrap();

        repo.challenge = Some(challenge.clone());

        let tx = repo.begin_transaction().await.unwrap();
        let wallet_user = repo.find_wallet_user_by_wallet_id(&tx, "0").await.unwrap();
        tx.commit().await.unwrap();

        assert_matches!(
            wallet_user,
            WalletUserQueryResult::Found(user) if AccountServer::verify_instruction(
                wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
                Instruction::new_signed(CheckPin, challenge, 44, &setup.hw_privkey, &setup.pin_privkey, cert.clone())
                    .await
                    .unwrap(),
                &user,
                &EpochGenerator,
                get_global_hsm().await,
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn wrong_instruction_challenge_should_not_verify() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;

        let challenge_request = InstructionChallengeRequest::new_signed::<CheckPin>(
            cert.dangerous_parse_unverified().unwrap().1.wallet_id,
            1,
            &setup.hw_privkey,
            cert.clone(),
        )
        .await
        .unwrap();

        let challenge = account_server
            .instruction_challenge(challenge_request, &repo, &EpochGenerator, get_global_hsm().await)
            .await
            .unwrap();

        repo.challenge = Some(random_bytes(32));

        let tx = repo.begin_transaction().await.unwrap();
        let wallet_user = repo.find_wallet_user_by_wallet_id(&tx, "0").await.unwrap();
        tx.commit().await.unwrap();

        assert_matches!(
            wallet_user,
            WalletUserQueryResult::Found(user) if matches!(
                AccountServer::verify_instruction(
                    wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
                    Instruction::new_signed(
                            CheckPin,
                            challenge,
                            44,
                            &setup.hw_privkey,
                            &setup.pin_privkey,
                            cert.clone()
                    ).await.unwrap(),
                    &user,
                    &EpochGenerator,
                    get_global_hsm().await,
                ).await,
                Err(InstructionValidationError::VerificationFailed(
                    wallet_common::account::errors::Error::ChallengeMismatch
                ))
            )
        );
    }

    struct ExpiredAtEpochGeneretor;

    impl Generator<DateTime<Utc>> for ExpiredAtEpochGeneretor {
        fn generate(&self) -> DateTime<Utc> {
            Utc.timestamp_nanos(-1)
        }
    }

    #[tokio::test]
    async fn expired_instruction_challenge_should_not_verify() {
        let (setup, account_server, cert, repo) = setup_and_do_registration().await;

        let challenge_request = InstructionChallengeRequest::new_signed::<CheckPin>(
            cert.dangerous_parse_unverified().unwrap().1.wallet_id,
            1,
            &setup.hw_privkey,
            cert.clone(),
        )
        .await
        .unwrap();

        let challenge = account_server
            .instruction_challenge(challenge_request, &repo, &EpochGenerator, get_global_hsm().await)
            .await
            .unwrap();

        let tx = repo.begin_transaction().await.unwrap();
        let wallet_user = repo.find_wallet_user_by_wallet_id(&tx, "0").await.unwrap();
        assert_matches!(wallet_user, WalletUserQueryResult::Found(_));

        if let WalletUserQueryResult::Found(mut user) = wallet_user {
            user.instruction_challenge = Some(InstructionChallenge {
                bytes: challenge.clone(),
                expiration_date_time: ExpiredAtEpochGeneretor.generate(),
            });

            assert_matches!(
                AccountServer::verify_instruction(
                    wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
                    Instruction::new_signed(
                        CheckPin,
                        challenge,
                        44,
                        &setup.hw_privkey,
                        &setup.pin_privkey,
                        cert.clone()
                    )
                    .await
                    .unwrap(),
                    &user,
                    &EpochGenerator,
                    get_global_hsm().await,
                )
                .await,
                Err(InstructionValidationError::ChallengeTimeout)
            );
        }
    }

    #[tokio::test]
    async fn test_check_pin() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;
        repo.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let challenge_error = do_instruction_challenge::<CheckPin>(
            &account_server,
            &repo,
            &setup.hw_privkey,
            cert.clone(),
            9,
            get_global_hsm().await,
        )
        .await
        .expect_err("should return instruction sequence number mismatch error");

        assert_matches!(
            challenge_error,
            ChallengeError::Validation(wallet_common::account::errors::Error::SequenceNumberMismatch)
        );

        let instruction_result = do_check_pin(&account_server, repo, &setup, cert, &instruction_result_signing_key)
            .await
            .expect("should return unit instruction result");

        instruction_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result");
    }

    #[tokio::test]
    async fn test_change_pin_start_commit() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;
        repo.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let (new_pin_privkey, new_pin_pubkey, encrypted_new_pin_pubkey, new_cert) = do_pin_change_start(
            &account_server,
            &repo,
            &setup,
            cert.clone(),
            &instruction_result_signing_key,
        )
        .await;

        verify_wallet_certificate(
            &new_cert,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            &repo,
            get_global_hsm().await,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
        )
        .await
        .expect_err("verifying with the old pin_pubkey should fail");

        repo.encrypted_pin_pubkey = encrypted_new_pin_pubkey.clone();

        verify_wallet_certificate(
            &new_cert,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            &repo,
            get_global_hsm().await,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
        )
        .await
        .expect("verifying with the new pin_pubkey should succeed");

        let challenge = do_instruction_challenge::<ChangePinCommit>(
            &account_server,
            &repo,
            &setup.hw_privkey,
            new_cert.clone(),
            45,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        account_server
            .handle_instruction(
                Instruction::new_signed(
                    ChangePinCommit {},
                    challenge.clone(),
                    46,
                    &setup.hw_privkey,
                    &setup.pin_privkey,
                    cert.clone(),
                )
                .await
                .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    challenge: Some(challenge.clone()),
                    previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
                    ..repo.clone()
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
                &MockWteIssuer,
            )
            .await
            .expect_err("should fail for old pin");

        let instruction_result = account_server
            .handle_instruction(
                Instruction::new_signed(
                    ChangePinCommit {},
                    challenge.clone(),
                    46,
                    &setup.hw_privkey,
                    &new_pin_privkey,
                    new_cert.clone(),
                )
                .await
                .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    encrypted_pin_pubkey: encrypted_new_pin_pubkey.clone(),
                    previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
                    challenge: Some(challenge.clone()),
                    ..repo.clone()
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
                &MockWteIssuer,
            )
            .await
            .expect("should return instruction result");

        instruction_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result");

        account_server
            .handle_instruction(
                Instruction::new_signed(
                    ChangePinCommit {},
                    challenge.clone(),
                    46,
                    &setup.hw_privkey,
                    &new_pin_privkey,
                    new_cert.clone(),
                )
                .await
                .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    encrypted_pin_pubkey: encrypted_new_pin_pubkey.clone(),
                    previous_encrypted_pin_pubkey: None,
                    challenge: Some(challenge),
                    ..repo.clone()
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
                &MockWteIssuer,
            )
            .await
            .expect("committing double should succeed");

        do_check_pin(
            &account_server,
            repo,
            &WalletCertificateSetup {
                pin_privkey: new_pin_privkey,
                pin_pubkey: new_pin_pubkey,
                encrypted_pin_pubkey: encrypted_new_pin_pubkey,
                ..setup
            },
            new_cert,
            &instruction_result_signing_key,
        )
        .await
        .expect("should be able to send CheckPin instruction with the new certificate");
    }

    #[tokio::test]
    async fn test_change_pin_start_invalid_pop() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;
        repo.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let new_pin_privkey = SigningKey::random(&mut OsRng);
        let new_pin_pubkey = *new_pin_privkey.verifying_key();

        let challenge = do_instruction_challenge::<ChangePinStart>(
            &account_server,
            &repo,
            &setup.hw_privkey,
            cert.clone(),
            43,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        let pop_pin_pubkey = new_pin_privkey.try_sign(random_bytes(32).as_slice()).await.unwrap();

        let error = account_server
            .handle_change_pin_start_instruction(
                Instruction::new_signed(
                    ChangePinStart {
                        pin_pubkey: new_pin_pubkey.into(),
                        pop_pin_pubkey: pop_pin_pubkey.into(),
                    },
                    challenge.clone(),
                    44,
                    &setup.hw_privkey,
                    &setup.pin_privkey,
                    cert.clone(),
                )
                .await
                .unwrap(),
                (&instruction_result_signing_key, &setup.signing_key),
                &MockGenerators,
                &WalletUserTestRepo {
                    hw_pubkey: setup.hw_pubkey,
                    encrypted_pin_pubkey: setup.encrypted_pin_pubkey.clone(),
                    previous_encrypted_pin_pubkey: None,
                    challenge: Some(challenge),
                    instruction_sequence_number: 2,
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
            )
            .await
            .expect_err("should return instruction error for invalid PoP");

        assert_matches!(
            error,
            InstructionError::Validation(InstructionValidationError::ChallengeMismatch)
        );
    }

    #[tokio::test]
    async fn test_change_pin_start_rollback() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;
        repo.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let (new_pin_privkey, new_pin_pubkey, encrypted_new_pin_pubkey, new_cert) = do_pin_change_start(
            &account_server,
            &repo,
            &setup,
            cert.clone(),
            &instruction_result_signing_key,
        )
        .await;

        let challenge = do_instruction_challenge::<ChangePinRollback>(
            &account_server,
            &repo,
            &setup.hw_privkey,
            cert.clone(),
            45,
            get_global_hsm().await,
        )
        .await
        .unwrap();

        account_server
            .handle_change_pin_rollback_instruction(
                Instruction::new_signed(
                    ChangePinRollback {},
                    challenge.clone(),
                    46,
                    &setup.hw_privkey,
                    &new_pin_privkey,
                    new_cert.clone(),
                )
                .await
                .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    challenge: Some(challenge.clone()),
                    previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
                    ..repo.clone()
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
            )
            .await
            .expect_err("should fail for new pin");

        account_server
            .handle_change_pin_rollback_instruction(
                Instruction::new_signed(
                    ChangePinRollback {},
                    challenge.clone(),
                    46,
                    &setup.hw_privkey,
                    &setup.pin_privkey,
                    cert.clone(),
                )
                .await
                .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    challenge: Some(challenge.clone()),
                    previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
                    ..repo.clone()
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
            )
            .await
            .expect("should succeed for old pin");

        let instruction_result = account_server
            .handle_change_pin_rollback_instruction(
                Instruction::new_signed(
                    ChangePinRollback {},
                    challenge.clone(),
                    47,
                    &setup.hw_privkey,
                    &setup.pin_privkey,
                    cert.clone(),
                )
                .await
                .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    challenge: Some(challenge),
                    previous_encrypted_pin_pubkey: None,
                    ..repo.clone()
                },
                &TimeoutPinPolicy,
                get_global_hsm().await,
            )
            .await
            .expect("should return instruction result for old pin");

        instruction_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result");

        do_check_pin(
            &account_server,
            WalletUserTestRepo {
                encrypted_pin_pubkey: setup.encrypted_pin_pubkey.clone(),
                ..repo.clone()
            },
            &WalletCertificateSetup {
                pin_privkey: new_pin_privkey,
                pin_pubkey: new_pin_pubkey,
                encrypted_pin_pubkey: encrypted_new_pin_pubkey,
                ..setup.clone()
            },
            new_cert,
            &instruction_result_signing_key,
        )
        .await
        .expect_err("should not be able to send CheckPin instruction with new certificate");

        do_check_pin(
            &account_server,
            WalletUserTestRepo {
                encrypted_pin_pubkey: setup.encrypted_pin_pubkey.clone(),
                ..repo
            },
            &setup,
            cert,
            &instruction_result_signing_key,
        )
        .await
        .expect("should be able to send CheckPin instruction with old certificate");
    }

    #[tokio::test]
    async fn test_change_pin_no_other_instructions_allowed() {
        let (setup, account_server, cert, mut repo) = setup_and_do_registration().await;
        repo.instruction_sequence_number = 42;
        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let (_new_pin_privkey, _new_pin_pubkey, encrypted_new_pin_pubkey, _new_cert) = do_pin_change_start(
            &account_server,
            &repo,
            &setup,
            cert.clone(),
            &instruction_result_signing_key,
        )
        .await;

        repo.previous_encrypted_pin_pubkey = Some(encrypted_new_pin_pubkey);
        let error = do_check_pin(&account_server, repo, &setup, cert, &instruction_result_signing_key)
            .await
            .expect_err("other instructions than change_pin_commit and change_pin_rollback are not allowed");
        assert_eq!(
            "instruction validation error: pin change is in progress",
            error.to_string()
        );
    }
}
