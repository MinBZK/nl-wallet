use chrono::{DateTime, Local};
use p256::{ecdsa::VerifyingKey, pkcs8::EncodePublicKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use wallet_common::{
    account::{
        jwt::{EcdsaDecodingKey, Jwt, JwtClaims},
        messages::{
            auth::{Registration, WalletCertificate, WalletCertificateClaims},
            errors::{IncorrectPinData, PinTimeoutData},
            instructions::{
                CheckPin, Instruction, InstructionChallengeRequestMessage, InstructionResult, InstructionResultClaims,
            },
        },
        serialization::Base64Bytes,
        signed::{ChallengeResponsePayload, SequenceNumberComparison, SignedDouble},
    },
    keys::EcdsaKey,
    utils::{random_bytes, random_string, sha256},
};
use wallet_provider_domain::{
    generator::Generator,
    model::{
        pin_policy::{PinPolicyEvaluation, PinPolicyEvaluator},
        wallet_user::{WalletUser, WalletUserCreate},
    },
    repository::{Committable, PersistenceError, TransactionStarter, WalletUserRepository},
    wallet_provider_signing_key::WalletProviderEcdsaKey,
};

#[derive(Debug, thiserror::Error)]
pub enum AccountServerInitError {
    // Do not format original error to prevent potentially leaking key material
    #[error("server private key decoding error")]
    PrivateKeyDecoding(#[from] p256::pkcs8::Error),
    #[error("server public key decoding error")]
    PublicKeyDecoding(#[from] p256::ecdsa::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ChallengeError {
    #[error("challenge signing error: {0}")]
    ChallengeSigning(#[source] wallet_common::errors::Error),
    #[error("could not store challenge: {0}")]
    Storage(#[from] PersistenceError),
    #[error("challenge message validation error: {0}")]
    Validation(#[from] wallet_common::errors::Error),
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
    Validation(#[from] wallet_common::errors::Error),
    #[error("no registered wallet user found")]
    UserNotRegistered,
    #[error("could not retrieve registered wallet user: {0}")]
    Persistence(#[from] PersistenceError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("registration challenge UTF-8 decoding error: {0}")]
    ChallengeDecoding(#[source] std::string::FromUtf8Error),
    #[error("registration challenge validation error: {0}")]
    ChallengeValidation(#[source] wallet_common::errors::Error),
    #[error("registration message parsing error: {0}")]
    MessageParsing(#[source] wallet_common::errors::Error),
    #[error("registration message validation error: {0}")]
    MessageValidation(#[source] wallet_common::errors::Error),
    #[error("incorrect registration serial number (expected: {expected:?}, received: {received:?})")]
    SerialNumberMismatch { expected: u64, received: u64 },
    #[error("registration JWT signing error: {0}")]
    JwtSigning(#[source] wallet_common::errors::Error),
    #[error("could not store certificate: {0}")]
    CertificateStorage(#[from] PersistenceError),
    #[error("registration HW public key decoding error: {0}")]
    HwPubKeyDecoding(#[source] p256::pkcs8::spki::Error),
    #[error("registration PIN public key decoding error: {0}")]
    PinPubKeyDecoding(#[source] p256::pkcs8::spki::Error),
    #[error("registration PIN public key DER encoding error: {0}")]
    PinPubKeyEncoding(#[source] der::Error),
    #[error("wallet certificate validation error: {0}")]
    WalletCertificate(#[from] WalletCertificateError),
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
    Signing(#[source] wallet_common::errors::Error),
    #[error("persistence error: {0}")]
    Storage(#[from] PersistenceError),
}

#[derive(Debug, thiserror::Error)]
pub enum InstructionValidationError {
    #[error("instruction sequence number mismatch")]
    SequenceNumberMismatch,
    #[error("instruction challenge mismatch")]
    ChallengeMismatch,
    #[error("instruction verification failed: {0}")]
    VerificationFailed(#[source] wallet_common::errors::Error),
}

impl From<PinPolicyEvaluation> for InstructionError {
    fn from(value: PinPolicyEvaluation) -> Self {
        match value {
            PinPolicyEvaluation::Failed {
                attempts_left,
                is_final_attempt,
            } => InstructionError::IncorrectPin(IncorrectPinData {
                attempts_left,
                is_final_attempt,
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

const WALLET_CERTIFICATE_VERSION: u32 = 0;

/// Used as the challenge in the challenge-response protocol during wallet registration.
#[derive(Serialize, Deserialize, Debug)]
struct RegistrationChallengeClaims {
    wallet_id: String,
    exp: u64,

    /// Random bytes to serve as the actual challenge for the wallet to sign.
    random: Base64Bytes,
}

impl JwtClaims for RegistrationChallengeClaims {
    const SUB: &'static str = "registration_challenge";
}

pub trait HandleInstruction {
    type Result: Serialize + DeserializeOwned;

    fn handle(&self) -> Result<Self::Result, InstructionError>;
}

impl HandleInstruction for CheckPin {
    type Result = ();

    fn handle(&self) -> Result<(), InstructionError> {
        Ok(())
    }
}

pub struct AccountServer {
    certificate_signing_key: WalletProviderEcdsaKey,
    instruction_result_signing_key: WalletProviderEcdsaKey,

    pin_hash_salt: Vec<u8>,

    pub name: String,
    pub certificate_pubkey: EcdsaDecodingKey,
    pub instruction_result_pubkey: EcdsaDecodingKey,
}

impl AccountServer {
    pub fn new(
        certificate_signing_key: WalletProviderEcdsaKey,
        instruction_result_signing_key: WalletProviderEcdsaKey,
        pin_hash_salt: Vec<u8>,
        name: String,
    ) -> Result<AccountServer, AccountServerInitError> {
        let certificate_pubkey = certificate_signing_key.verifying_key()?.into();
        let instruction_result_pubkey = instruction_result_signing_key.verifying_key()?.into();

        Ok(AccountServer {
            certificate_signing_key,
            instruction_result_signing_key,
            pin_hash_salt,
            name,
            certificate_pubkey,
            instruction_result_pubkey,
        })
    }

    // Only used for registration. When a registered user sends an instruction, we should store
    // the challenge per user, instead globally.
    pub fn registration_challenge(&self) -> Result<Vec<u8>, ChallengeError> {
        let challenge = Jwt::sign(
            &RegistrationChallengeClaims {
                wallet_id: random_string(32),
                random: random_bytes(32).into(),
                exp: jsonwebtoken::get_current_timestamp() + 60,
            },
            &self.certificate_signing_key,
        )
        .map_err(ChallengeError::ChallengeSigning)?
        .0
        .as_bytes()
        .to_vec();
        Ok(challenge)
    }

    pub async fn instruction_challenge<T>(
        &self,
        challenge_request: InstructionChallengeRequestMessage,
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>),
    ) -> Result<Vec<u8>, ChallengeError>
    where
        T: Committable,
    {
        debug!("Starting database transaction");

        let tx = repositories.begin_transaction().await?;

        debug!("Verifying certificate and retrieving wallet user");

        let user = self
            .verify_wallet_certificate(&challenge_request.certificate, repositories)
            .await?;

        debug!("Parsing and verifying challenge request for user {}", user.id);

        let parsed = challenge_request.message.parse_and_verify(&user.hw_pubkey.into())?;

        debug!(
            "Verifying sequence number - provided: {}, known: {}",
            parsed.sequence_number, user.instruction_sequence_number
        );

        if parsed.sequence_number <= user.instruction_sequence_number {
            tx.commit().await?;
            return Err(ChallengeError::SequenceNumberValidation);
        }

        debug!("Sequence number valid, persisting generated challenge and incremented sequence number");

        let challenge = random_bytes(32);

        repositories
            .update_instruction_challenge_and_sequence_number(
                &tx,
                &user.wallet_id,
                Some(challenge.clone()),
                parsed.sequence_number,
            )
            .await?;
        tx.commit().await?;

        debug!("Responding with generated challenge");

        Ok(challenge)
    }

    pub async fn handle_instruction<T, I, R>(
        &self,
        instruction: Instruction<I>,
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>),
        pin_policy: &impl PinPolicyEvaluator,
        time_generator: &impl Generator<DateTime<Local>>,
    ) -> Result<InstructionResult<R>, InstructionError>
    where
        T: Committable,
        I: HandleInstruction<Result = R> + Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned,
    {
        debug!("Verifying certificate and retrieving wallet user");

        let wallet_user = self
            .verify_wallet_certificate(&instruction.certificate, repositories)
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
            time_generator.generate(),
        );

        if matches!(
            pin_eval,
            PinPolicyEvaluation::InTimeout { timeout: _ } | PinPolicyEvaluation::BlockedPermanently
        ) {
            tx.commit().await?;
            return Err(pin_eval.into());
        }

        debug!("Verifying instruction");

        match self.verify_instruction(instruction, &wallet_user) {
            Ok(payload) => {
                debug!("Instruction successfully verified, resetting pin retries");

                repositories
                    .reset_unsuccessful_pin_entries(&tx, &wallet_user.wallet_id)
                    .await?;

                debug!("Updating instruction sequence number to {}", payload.sequence_number);

                repositories
                    .update_instruction_sequence_number(&tx, &wallet_user.wallet_id, payload.sequence_number)
                    .await?;

                tx.commit().await?;

                self.sign_instruction_result(payload.payload.handle()?)
            }
            Err(validation_error) => {
                let error = if matches!(validation_error, InstructionValidationError::VerificationFailed(_)) {
                    debug!("Instruction validation failed, registering unsuccessful pin entry");

                    repositories
                        .register_unsuccessful_pin_entry(
                            &tx,
                            &wallet_user.wallet_id,
                            matches!(pin_eval, PinPolicyEvaluation::BlockedPermanently),
                            time_generator.generate(),
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

    pub async fn register<T>(
        &self,
        uuid_generator: &impl Generator<Uuid>,
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>),
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, RegistrationError>
    where
        T: Committable,
    {
        debug!("Parsing message to lookup public keys");

        // We don't have the public keys yet against which to verify the message, as those are contained within the
        // message (like in X509 certificate requests). So first parse it to grab the public keys from it.
        let unverified = registration_message
            .dangerous_parse_unverified()
            .map_err(RegistrationError::MessageParsing)?;

        debug!("Extracting challenge, wallet id, hw pubkey and pin pubkey");

        let challenge = &unverified.challenge.0;
        let wallet_id = self.verify_registration_challenge(challenge)?.wallet_id;

        let hw_pubkey = unverified.payload.hw_pubkey.0;
        let pin_pubkey = unverified.payload.pin_pubkey.0;

        debug!("Checking if challenge is signed with the provided hw and pin keys");

        registration_message
            .parse_and_verify(challenge, SequenceNumberComparison::EqualTo(0), &hw_pubkey, &pin_pubkey)
            .map_err(RegistrationError::MessageValidation)?;

        debug!("Starting database transaction");

        let tx = repositories.begin_transaction().await?;

        debug!("Creating new wallet user");

        let uuid = uuid_generator.generate();
        repositories
            .create_wallet_user(
                &tx,
                WalletUserCreate {
                    id: uuid,
                    wallet_id: wallet_id.clone(),
                    hw_pubkey_der: hw_pubkey
                        .to_public_key_der()
                        .map_err(RegistrationError::HwPubKeyDecoding)?
                        .to_vec(),
                    pin_pubkey_der: pin_pubkey
                        .to_public_key_der()
                        .map_err(RegistrationError::PinPubKeyDecoding)?
                        .to_vec(),
                },
            )
            .await?;

        debug!("Generating new wallet certificate for user {}", uuid);

        let cert_result = self.new_wallet_certificate(wallet_id, hw_pubkey, pin_pubkey)?;

        tx.commit().await?;

        Ok(cert_result)
    }

    fn new_wallet_certificate(
        &self,
        wallet_id: String,
        wallet_hw_pubkey: VerifyingKey,
        wallet_pin_pubkey: VerifyingKey,
    ) -> Result<WalletCertificate, RegistrationError> {
        let cert = WalletCertificateClaims {
            wallet_id,
            hw_pubkey: wallet_hw_pubkey.into(),
            pin_pubkey_hash: pubkey_to_hash(self.pin_hash_salt.clone(), wallet_pin_pubkey)?,
            version: WALLET_CERTIFICATE_VERSION,

            iss: self.name.clone(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign(&cert, &self.certificate_signing_key).map_err(RegistrationError::JwtSigning)
    }

    fn verify_registration_challenge(
        &self,
        challenge: &[u8],
    ) -> Result<RegistrationChallengeClaims, RegistrationError> {
        Jwt::parse_and_verify(
            &String::from_utf8(challenge.to_owned())
                .map_err(RegistrationError::ChallengeDecoding)?
                .into(),
            &self.certificate_pubkey,
        )
        .map_err(RegistrationError::ChallengeValidation)
    }

    async fn verify_wallet_certificate<T>(
        &self,
        certificate: &WalletCertificate,
        wallet_user_repository: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>),
    ) -> Result<WalletUser, WalletCertificateError>
    where
        T: Committable,
    {
        debug!("Parsing and verifying the provided certificate");

        let cert_data = certificate.parse_and_verify(&self.certificate_pubkey)?;

        debug!("Starting database transaction");

        let tx = wallet_user_repository.begin_transaction().await?;

        debug!("Fetching the user associated to the provided certificate");

        let user = wallet_user_repository
            .find_wallet_user_by_wallet_id(&tx, &cert_data.wallet_id)
            .await?;
        tx.commit().await?;

        debug!("Generating pin public key hash");

        let hash = pubkey_to_hash(self.pin_hash_salt.clone(), user.pin_pubkey.0)?;

        debug!("Verifying user matches the provided certificate");

        if hash != cert_data.pin_pubkey_hash {
            Err(WalletCertificateError::PinPubKeyMismatch)
        } else if user.hw_pubkey != cert_data.hw_pubkey {
            Err(WalletCertificateError::HwPubKeyMismatch)
        } else {
            Ok(user)
        }
    }

    fn verify_instruction<I, R>(
        &self,
        instruction: Instruction<I>,
        wallet_user: &WalletUser,
    ) -> Result<ChallengeResponsePayload<I>, InstructionValidationError>
    where
        I: HandleInstruction<Result = R> + Serialize + DeserializeOwned,
    {
        let challenge = wallet_user
            .instruction_challenge
            .as_ref()
            .ok_or(InstructionValidationError::ChallengeMismatch)?;

        let parsed = instruction
            .instruction
            .parse_and_verify(
                challenge,
                SequenceNumberComparison::LargerThan(wallet_user.instruction_sequence_number),
                &wallet_user.hw_pubkey.0,
                &wallet_user.pin_pubkey.0,
            )
            .map_err(InstructionValidationError::VerificationFailed)?;

        Ok(parsed)
    }

    fn sign_instruction_result<R>(&self, result: R) -> Result<InstructionResult<R>, InstructionError>
    where
        R: Serialize + DeserializeOwned,
    {
        let claims = InstructionResultClaims {
            result,
            iss: self.name.to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign(&claims, &self.instruction_result_signing_key).map_err(InstructionError::Signing)
    }
}

fn der_encode(payload: impl der::Encode) -> Result<Vec<u8>, der::Error> {
    let mut buf = Vec::<u8>::with_capacity(payload.encoded_len()?.try_into()?);
    payload.encode_to_vec(&mut buf)?;
    Ok(buf)
}

fn pubkey_to_hash(pin_hash_salt: Vec<u8>, pubkey: VerifyingKey) -> Result<Base64Bytes, WalletCertificateError> {
    let pin_pubkey_bts = pubkey
        .to_public_key_der()
        .map_err(WalletCertificateError::PinPubKeyDecoding)?
        .to_vec();
    let pin_pubkey_tohash =
        der_encode(vec![pin_hash_salt, pin_pubkey_bts]).map_err(WalletCertificateError::PinPubKeyEncoding)?;
    Ok(sha256(&pin_pubkey_tohash).into())
}

#[cfg(any(test, feature = "stub"))]
pub mod stub {
    use async_trait::async_trait;
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;
    use wallet_common::account::serialization::DerSigningKey;

    use wallet_provider_domain::{
        generator::stub::FixedGenerator,
        model::wallet_user::WalletUser,
        repository::{TransactionStarterStub, WalletUserRepositoryStub},
    };

    use super::*;

    pub fn account_server() -> AccountServer {
        let account_server_privkey: DerSigningKey = SigningKey::random(&mut OsRng).into();
        let instruction_result_privkey: DerSigningKey = SigningKey::random(&mut OsRng).into();

        AccountServer::new(
            account_server_privkey.into(),
            instruction_result_privkey.into(),
            random_bytes(32),
            "stub_account_server".into(),
        )
        .unwrap()
    }

    pub struct TestDeps;

    impl Generator<uuid::Uuid> for TestDeps {
        fn generate(&self) -> Uuid {
            FixedGenerator.generate()
        }
    }

    #[async_trait]
    impl TransactionStarter for TestDeps {
        type TransactionType = <TransactionStarterStub as TransactionStarter>::TransactionType;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            TransactionStarterStub.begin_transaction().await
        }
    }

    #[async_trait]
    impl WalletUserRepository for TestDeps {
        type TransactionType = <WalletUserRepositoryStub as WalletUserRepository>::TransactionType;

        async fn create_wallet_user(
            &self,
            transaction: &Self::TransactionType,
            user: WalletUserCreate,
        ) -> Result<(), PersistenceError> {
            WalletUserRepositoryStub.create_wallet_user(transaction, user).await
        }

        async fn find_wallet_user_by_wallet_id(
            &self,
            transaction: &Self::TransactionType,
            wallet_id: &str,
        ) -> Result<WalletUser, PersistenceError> {
            WalletUserRepositoryStub
                .find_wallet_user_by_wallet_id(transaction, wallet_id)
                .await
        }

        async fn clear_instruction_challenge(
            &self,
            transaction: &Self::TransactionType,
            wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            WalletUserRepositoryStub
                .clear_instruction_challenge(transaction, wallet_id)
                .await
        }

        async fn update_instruction_sequence_number(
            &self,
            transaction: &Self::TransactionType,
            wallet_id: &str,
            instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            WalletUserRepositoryStub
                .update_instruction_sequence_number(transaction, wallet_id, instruction_sequence_number)
                .await
        }

        async fn update_instruction_challenge_and_sequence_number(
            &self,
            transaction: &Self::TransactionType,
            wallet_id: &str,
            challenge: Option<Vec<u8>>,
            instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            WalletUserRepositoryStub
                .update_instruction_challenge_and_sequence_number(
                    transaction,
                    wallet_id,
                    challenge,
                    instruction_sequence_number,
                )
                .await
        }

        async fn register_unsuccessful_pin_entry(
            &self,
            transaction: &Self::TransactionType,
            wallet_id: &str,
            is_blocked: bool,
            datetime: DateTime<Local>,
        ) -> Result<(), PersistenceError> {
            WalletUserRepositoryStub
                .register_unsuccessful_pin_entry(transaction, wallet_id, is_blocked, datetime)
                .await
        }

        async fn reset_unsuccessful_pin_entries(
            &self,
            transaction: &Self::TransactionType,
            wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            WalletUserRepositoryStub
                .reset_unsuccessful_pin_entries(transaction, wallet_id)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use async_trait::async_trait;
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;
    use uuid::uuid;

    use wallet_common::account::{messages::instructions::InstructionChallengeRequest, serialization::DerVerifyingKey};
    use wallet_provider_domain::{
        model::{FailingPinPolicy, TimeoutPinPolicy},
        repository::{TransactionStarterStub, WalletUserRepositoryStub},
        EpochGenerator,
    };

    use super::{stub::TestDeps, *};

    async fn do_registration(
        account_server: &AccountServer,
        hw_privkey: &SigningKey,
        pin_privkey: &SigningKey,
    ) -> WalletCertificate {
        let challenge = account_server
            .registration_challenge()
            .expect("Could not get registration challenge");

        let registration_message =
            Registration::new_signed(hw_privkey, pin_privkey, &challenge).expect("Could not sign new registration");

        account_server
            .register(&TestDeps, &TestDeps, registration_message)
            .await
            .expect("Could not process registration message at account server")
    }

    #[tokio::test]
    async fn test_register() {
        let account_server = stub::account_server();
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let cert = do_registration(&account_server, &hw_privkey, &pin_privkey).await;

        // Verify the certificate
        let cert_data = cert
            .parse_and_verify(&account_server.certificate_pubkey)
            .expect("Could not parse and verify wallet certificate");
        assert_eq!(cert_data.iss, account_server.name);
        assert_eq!(cert_data.hw_pubkey.0, *hw_privkey.verifying_key());
    }

    struct WalletUserTestRepo {
        hw: VerifyingKey,
        pin: VerifyingKey,
        challenge: Option<Vec<u8>>,
        instruction_sequence_number: u64,
    }

    #[async_trait]
    impl WalletUserRepository for WalletUserTestRepo {
        type TransactionType = <WalletUserRepositoryStub as WalletUserRepository>::TransactionType;

        async fn create_wallet_user(
            &self,
            transaction: &Self::TransactionType,
            user: WalletUserCreate,
        ) -> Result<(), PersistenceError> {
            TestDeps.create_wallet_user(transaction, user).await
        }
        async fn find_wallet_user_by_wallet_id(
            &self,
            _transaction: &Self::TransactionType,
            wallet_id: &str,
        ) -> Result<WalletUser, PersistenceError> {
            Ok(WalletUser {
                id: uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                wallet_id: wallet_id.to_string(),
                hw_pubkey: DerVerifyingKey(self.hw),
                pin_pubkey: DerVerifyingKey(self.pin),
                unsuccessful_pin_entries: 0,
                last_unsuccessful_pin_entry: None,
                instruction_challenge: self.challenge.clone(),
                instruction_sequence_number: self.instruction_sequence_number,
            })
        }
        async fn register_unsuccessful_pin_entry(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _is_blocked: bool,
            _datetime: DateTime<Local>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
        async fn reset_unsuccessful_pin_entries(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
        async fn clear_instruction_challenge(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
        async fn update_instruction_challenge_and_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _challenge: Option<Vec<u8>>,
            _instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
        async fn update_instruction_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    #[async_trait]
    impl TransactionStarter for WalletUserTestRepo {
        type TransactionType = <TransactionStarterStub as TransactionStarter>::TransactionType;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            TransactionStarterStub.begin_transaction().await
        }
    }

    #[tokio::test]
    async fn test_check_pin() {
        let account_server = stub::account_server();
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(&account_server, &hw_privkey, &pin_privkey).await;

        let deps = WalletUserTestRepo {
            hw: hw_pubkey,
            pin: pin_pubkey,
            challenge: None,
            instruction_sequence_number: 42,
        };

        assert_matches!(
            account_server
                .instruction_challenge(
                    InstructionChallengeRequestMessage {
                        message: InstructionChallengeRequest::new_signed(9, "wallet", &hw_privkey).unwrap(),
                        certificate: cert.clone(),
                    },
                    &deps,
                )
                .await
                .expect_err("should return instruction sequence number mismatch error"),
            ChallengeError::SequenceNumberValidation
        );

        let challenge = account_server
            .instruction_challenge(
                InstructionChallengeRequestMessage {
                    message: InstructionChallengeRequest::new_signed(43, "wallet", &hw_privkey).unwrap(),
                    certificate: cert.clone(),
                },
                &deps,
            )
            .await
            .unwrap();

        assert_matches!(
            account_server
                .handle_instruction(
                    Instruction {
                        instruction: CheckPin::new_signed(43, &hw_privkey, &pin_privkey, &challenge.clone()).unwrap(),
                        certificate: cert.clone(),
                    },
                    &WalletUserTestRepo {
                        hw: hw_pubkey,
                        pin: pin_pubkey,
                        challenge: Some(challenge.clone()),
                        instruction_sequence_number: 43,
                    },
                    &FailingPinPolicy,
                    &EpochGenerator,
                )
                .await
                .expect_err("sequence number mismatch error should result in IncorrectPin error"),
            InstructionError::IncorrectPin(IncorrectPinData {
                attempts_left: _,
                is_final_attempt: _
            })
        );

        account_server
            .handle_instruction(
                Instruction {
                    instruction: CheckPin::new_signed(44, &hw_privkey, &pin_privkey, &challenge).unwrap(),
                    certificate: cert.clone(),
                },
                &WalletUserTestRepo {
                    hw: hw_pubkey,
                    pin: pin_pubkey,
                    challenge: Some(challenge),
                    instruction_sequence_number: 2,
                },
                &TimeoutPinPolicy,
                &EpochGenerator,
            )
            .await
            .expect("should return instruction result");
    }

    #[tokio::test]
    async fn valid_wallet_certificate_should_verify() {
        let account_server = stub::account_server();
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(&account_server, &hw_privkey, &pin_privkey).await;

        let challenge_request = InstructionChallengeRequestMessage {
            message: InstructionChallengeRequest::new_signed(1, "wallet", &hw_privkey).unwrap(),
            certificate: cert.clone(),
        };

        let challenge = account_server
            .instruction_challenge(
                challenge_request,
                &WalletUserTestRepo {
                    hw: hw_pubkey,
                    pin: pin_pubkey,
                    challenge: None,
                    instruction_sequence_number: 0,
                },
            )
            .await
            .unwrap();

        account_server
            .verify_wallet_certificate(
                &cert,
                &WalletUserTestRepo {
                    hw: hw_pubkey,
                    pin: pin_pubkey,
                    challenge: Some(challenge),
                    instruction_sequence_number: 0,
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn wrong_hw_key_should_not_validate() {
        let account_server = stub::account_server();
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(&account_server, &hw_privkey, &pin_privkey).await;

        account_server
            .verify_wallet_certificate(
                &cert,
                &WalletUserTestRepo {
                    hw: *SigningKey::random(&mut OsRng).verifying_key(),
                    pin: pin_pubkey,
                    challenge: None,
                    instruction_sequence_number: 0,
                },
            )
            .await
            .expect_err("Should not validate");
    }

    #[tokio::test]
    async fn wrong_pin_key_should_not_validate() {
        let account_server = stub::account_server();
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let cert = do_registration(&account_server, &hw_privkey, &pin_privkey).await;

        account_server
            .verify_wallet_certificate(
                &cert,
                &WalletUserTestRepo {
                    hw: hw_pubkey,
                    pin: *SigningKey::random(&mut OsRng).verifying_key(),
                    challenge: None,
                    instruction_sequence_number: 0,
                },
            )
            .await
            .expect_err("Should not validate");
    }
}
