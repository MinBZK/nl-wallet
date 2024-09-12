use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use p256::{ecdsa::VerifyingKey, pkcs8::EncodePublicKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use tracing::debug;
use uuid::Uuid;

use wallet_common::{
    account::{
        errors::Error as AccountError,
        messages::{
            auth::{Registration, WalletCertificate, WalletCertificateClaims},
            errors::{IncorrectPinData, PinTimeoutData},
            instructions::{Instruction, InstructionChallengeRequest, InstructionResult, InstructionResultClaims},
        },
        signed::{ChallengeResponse, SequenceNumberComparison, SignedChallengeResponse},
    },
    generator::Generator,
    jwt::{EcdsaDecodingKey, Jwt, JwtError, JwtSubject},
    utils::{random_bytes, random_string},
};
use wallet_provider_domain::{
    model::{
        encrypter::{Decrypter, Encrypter},
        hsm::{Hsm, WalletUserHsm},
        pin_policy::{PinPolicyEvaluation, PinPolicyEvaluator},
        wallet_user::{InstructionChallenge, WalletUser, WalletUserCreate, WalletUserQueryResult},
    },
    repository::{Committable, PersistenceError, TransactionStarter, WalletUserRepository},
};

use crate::{
    hsm::HsmError,
    instructions::HandleInstruction,
    keys::{CertificateSigningKey, InstructionResultSigningKey},
};

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
    #[error("registration JWT signing error: {0}")]
    JwtSigning(#[source] JwtError),
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
    #[error("hsm error: {0}")]
    HsmError(#[from] HsmError),
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

const WALLET_CERTIFICATE_VERSION: u32 = 0;

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

    certificate_signing_pubkey: EcdsaDecodingKey,
    encryption_key_identifier: String,
    pin_public_disclosure_protection_key_identifier: String,
}

impl AccountServer {
    pub async fn new(
        instruction_challenge_timeout: Duration,
        name: String,
        certificate_signing_pubkey: EcdsaDecodingKey,
        encryption_key_identifier: String,
        pin_public_disclosure_protection_key_identifier: String,
    ) -> Result<Self, AccountServerInitError> {
        Ok(AccountServer {
            instruction_challenge_timeout,
            name,
            certificate_signing_pubkey,
            encryption_key_identifier,
            pin_public_disclosure_protection_key_identifier,
        })
    }

    // Only used for registration. When a registered user sends an instruction, we should store
    // the challenge per user, instead globally.
    pub async fn registration_challenge(
        &self,
        certificate_signing_key: &impl CertificateSigningKey,
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
        debug!("Verifying certificate and retrieving wallet user");

        let user = self
            .verify_wallet_certificate(&challenge_request.certificate, repositories, hsm)
            .await?;

        debug!("Parsing and verifying challenge request for user {}", user.id);

        let request = challenge_request.request.parse_and_verify(
            SequenceNumberComparison::LargerThan(user.instruction_sequence_number),
            &user.hw_pubkey.0,
        )?;

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

    pub async fn handle_instruction<T, R, I, IR, G, H>(
        &self,
        instruction: Instruction<I>,
        instruction_result_signing_key: &impl InstructionResultSigningKey,
        generators: &G,
        repositories: &R,
        pin_policy: &impl PinPolicyEvaluator,
        wallet_user_hsm: &H,
    ) -> Result<InstructionResult<IR>, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        I: HandleInstruction<Result = IR> + Serialize + DeserializeOwned,
        IR: Serialize + DeserializeOwned,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
        H: WalletUserHsm<Error = HsmError> + Hsm<Error = HsmError> + Decrypter<VerifyingKey, Error = HsmError>,
    {
        debug!("Verifying certificate and retrieving wallet user");

        let wallet_user = self
            .verify_wallet_certificate(&instruction.certificate, repositories, wallet_user_hsm)
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

        match self
            .verify_instruction(instruction, &wallet_user, generators, wallet_user_hsm)
            .await
        {
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

                let instruction_result = payload
                    .payload
                    .handle(&wallet_user, generators, repositories, wallet_user_hsm)
                    .await?;
                self.sign_instruction_result(instruction_result_signing_key, instruction_result)
                    .await
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

    pub async fn register<T, R, H>(
        &self,
        certificate_signing_key: &impl CertificateSigningKey,
        uuid_generator: &impl Generator<Uuid>,
        repositories: &R,
        hsm: &H,
        registration_message: SignedChallengeResponse<Registration>,
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
        let wallet_id = self
            .verify_registration_challenge(&self.certificate_signing_pubkey, challenge)?
            .wallet_id;

        let hw_pubkey = unverified.payload.hw_pubkey.0;
        let pin_pubkey = unverified.payload.pin_pubkey.0;

        debug!("Checking if challenge is signed with the provided hw and pin keys");

        registration_message
            .parse_and_verify(challenge, SequenceNumberComparison::EqualTo(0), &hw_pubkey, &pin_pubkey)
            .map_err(RegistrationError::MessageValidation)?;

        debug!("Starting database transaction");

        let encrypted_pin_pubkey = Encrypter::encrypt(hsm, &self.encryption_key_identifier, pin_pubkey).await?;

        debug!("Creating new wallet user");

        let uuid = uuid_generator.generate();
        let tx = repositories.begin_transaction().await?;
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
        tx.commit().await?;

        debug!("Generating new wallet certificate for user {}", uuid);

        let cert_result = self
            .new_wallet_certificate(certificate_signing_key, wallet_id, hw_pubkey, pin_pubkey, hsm)
            .await?;

        Ok(cert_result)
    }

    async fn new_wallet_certificate<H>(
        &self,
        certificate_signing_key: &impl CertificateSigningKey,
        wallet_id: String,
        wallet_hw_pubkey: VerifyingKey,
        wallet_pin_pubkey: VerifyingKey,
        hsm: &H,
    ) -> Result<WalletCertificate, RegistrationError>
    where
        H: Hsm<Error = HsmError>,
    {
        let pin_pubkey_hash = sign_pin_pubkey(
            wallet_pin_pubkey,
            &self.pin_public_disclosure_protection_key_identifier,
            hsm,
        )
        .await?;

        let cert = WalletCertificateClaims {
            wallet_id,
            hw_pubkey: wallet_hw_pubkey.into(),
            pin_pubkey_hash,
            version: WALLET_CERTIFICATE_VERSION,

            iss: self.name.clone(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign_with_sub(&cert, certificate_signing_key)
            .await
            .map_err(RegistrationError::JwtSigning)
    }

    fn verify_registration_challenge(
        &self,
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

    async fn verify_wallet_certificate<T, R, H>(
        &self,
        certificate: &WalletCertificate,
        wallet_user_repository: &R,
        hsm: &H,
    ) -> Result<WalletUser, WalletCertificateError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
    {
        debug!("Parsing and verifying the provided certificate");

        let cert_data = certificate.parse_and_verify_with_sub(&self.certificate_signing_pubkey)?;

        debug!("Starting database transaction");

        let tx = wallet_user_repository.begin_transaction().await?;

        debug!("Fetching the user associated to the provided certificate");

        let user_result = wallet_user_repository
            .find_wallet_user_by_wallet_id(&tx, &cert_data.wallet_id)
            .await?;
        tx.commit().await?;

        match user_result {
            WalletUserQueryResult::NotFound => {
                debug!("No user found for the provided certificate: {}", &cert_data.wallet_id);
                Err(WalletCertificateError::UserNotRegistered)
            }
            WalletUserQueryResult::Blocked => {
                debug!("User found for the provided certificate is blocked");
                Err(WalletCertificateError::UserBlocked)
            }
            WalletUserQueryResult::Found(user_boxed) => {
                debug!("Generating pin public key hash");

                let user = *user_boxed;

                let pin_pubkey =
                    Decrypter::decrypt(hsm, &self.encryption_key_identifier, user.encrypted_pin_pubkey.clone()).await?;

                let pin_hash_verification = verify_pin_pubkey(
                    pin_pubkey,
                    cert_data.pin_pubkey_hash,
                    &self.pin_public_disclosure_protection_key_identifier,
                    hsm,
                )
                .await;

                debug!("Verifying user matches the provided certificate");

                if pin_hash_verification.is_err() {
                    Err(WalletCertificateError::PinPubKeyMismatch)
                } else if user.hw_pubkey != cert_data.hw_pubkey {
                    Err(WalletCertificateError::HwPubKeyMismatch)
                } else {
                    Ok(user)
                }
            }
        }
    }

    async fn verify_instruction<I, R, D>(
        &self,
        instruction: Instruction<I>,
        wallet_user: &WalletUser,
        time_generator: &impl Generator<DateTime<Utc>>,
        verifying_key_decrypter: &D,
    ) -> Result<ChallengeResponse<I>, InstructionValidationError>
    where
        I: HandleInstruction<Result = R> + Serialize + DeserializeOwned,
        D: Decrypter<VerifyingKey, Error = HsmError>,
    {
        let challenge = wallet_user
            .instruction_challenge
            .as_ref()
            .ok_or(InstructionValidationError::ChallengeMismatch)?;

        if challenge.expiration_date_time < time_generator.generate() {
            return Err(InstructionValidationError::ChallengeTimeout);
        }

        let pin_pubkey = verifying_key_decrypter
            .decrypt(
                &self.encryption_key_identifier,
                wallet_user.encrypted_pin_pubkey.clone(),
            )
            .await?;

        let parsed = instruction
            .instruction
            .parse_and_verify(
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

async fn sign_pin_pubkey<H>(
    pubkey: VerifyingKey,
    key_identifier: &str,
    hsm: &H,
) -> Result<Vec<u8>, WalletCertificateError>
where
    H: Hsm<Error = HsmError>,
{
    let pin_pubkey_bts = pubkey
        .to_public_key_der()
        .map_err(WalletCertificateError::PinPubKeyDecoding)?
        .to_vec();

    let signature = hsm.sign_hmac(key_identifier, Arc::new(pin_pubkey_bts)).await?;

    Ok(signature)
}

async fn verify_pin_pubkey<H>(
    pubkey: VerifyingKey,
    pin_pubkey_hash: Vec<u8>,
    key_identifier: &str,
    hsm: &H,
) -> Result<(), WalletCertificateError>
where
    H: Hsm<Error = HsmError>,
{
    let pin_pubkey_bts = pubkey
        .to_public_key_der()
        .map_err(WalletCertificateError::PinPubKeyDecoding)?
        .to_vec();

    hsm.verify_hmac(key_identifier, Arc::new(pin_pubkey_bts), pin_pubkey_hash)
        .await?;

    Ok(())
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;

    use super::*;

    pub async fn account_server_and_hsm(
        certificate_signing_pubkey: EcdsaDecodingKey,
    ) -> (AccountServer, MockPkcs11Client<HsmError>) {
        let account_server = AccountServer::new(
            Duration::from_millis(15000),
            "mock_account_server".into(),
            certificate_signing_pubkey,
            "encryption_key_1".into(),
            "signing_key_2".into(),
        )
        .await
        .unwrap();

        let hsm = MockPkcs11Client::default();
        hsm.generate_generic_secret_key("signing_key_2").await.unwrap();

        (account_server, hsm)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use chrono::TimeZone;
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;
    use uuid::uuid;

    use wallet_common::{
        account::{
            messages::instructions::{CheckPin, InstructionChallengeRequest},
            serialization::DerVerifyingKey,
            signed::SignedChallengeRequest,
        },
        keys::{software::SoftwareEcdsaKey, EcdsaKey},
    };
    use wallet_provider_domain::{
        generator::mock::MockGenerators,
        model::{
            hsm::mock::MockPkcs11Client, wallet_user::WalletUserKeys, wrapped_key::WrappedKey, FailingPinPolicy,
            TimeoutPinPolicy,
        },
        repository::{MockTransaction, MockTransactionStarter},
        EpochGenerator, FixedUuidGenerator,
    };
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use super::*;

    async fn do_registration(
        account_server: &AccountServer,
        hsm: &MockPkcs11Client<HsmError>,
        certificate_signing_key: &impl CertificateSigningKey,
        hw_privkey: &SigningKey,
        pin_privkey: &SigningKey,
    ) -> WalletCertificate {
        let challenge = account_server
            .registration_challenge(certificate_signing_key)
            .await
            .expect("Could not get registration challenge");

        let registration_message = Registration::new_signed(hw_privkey, pin_privkey, challenge)
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

    #[tokio::test]
    async fn test_register() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        // Verify the certificate
        let cert_data = cert
            .parse_and_verify_with_sub(&(&certificate_signing_key.verifying_key().await.unwrap()).into())
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

    impl WalletUserRepository for WalletUserTestRepo {
        type TransactionType = MockTransaction;

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
        async fn find_wallet_user_by_wallet_id(
            &self,
            _transaction: &Self::TransactionType,
            wallet_id: &str,
        ) -> Result<WalletUserQueryResult, PersistenceError> {
            Ok(WalletUserQueryResult::Found(Box::new(WalletUser {
                id: uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                wallet_id: wallet_id.to_string(),
                hw_pubkey: DerVerifyingKey(self.hw),
                encrypted_pin_pubkey: Encrypter::<VerifyingKey>::encrypt(
                    &MockPkcs11Client::<HsmError>::default(),
                    "encryption_key_1",
                    self.pin,
                )
                .await
                .unwrap(),
                unsuccessful_pin_entries: 0,
                last_unsuccessful_pin_entry: None,
                instruction_challenge: self.challenge.clone().map(|c| InstructionChallenge {
                    bytes: c,
                    expiration_date_time: Utc::now() + Duration::from_millis(15000),
                }),
                instruction_sequence_number: self.instruction_sequence_number,
            })))
        }
        async fn register_unsuccessful_pin_entry(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _is_blocked: bool,
            _datetime: DateTime<Utc>,
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
            _challenge: InstructionChallenge,
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
        async fn save_keys(
            &self,
            _transaction: &Self::TransactionType,
            _keys: WalletUserKeys,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
        async fn find_keys_by_identifiers(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            key_identifiers: &[String],
        ) -> Result<HashMap<String, WrappedKey>, PersistenceError> {
            Ok(key_identifiers
                .iter()
                .map(|id| {
                    (
                        id.clone(),
                        WrappedKey::new(SigningKey::random(&mut OsRng).to_bytes().to_vec()),
                    )
                })
                .collect())
        }
    }

    impl TransactionStarter for WalletUserTestRepo {
        type TransactionType = <MockTransactionStarter as TransactionStarter>::TransactionType;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            MockTransactionStarter.begin_transaction().await
        }
    }

    #[tokio::test]
    async fn test_check_pin() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();
        let instruction_result_signing_key = SoftwareEcdsaKey::new_random("instruction_result_signing_key".to_string());

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        let deps = WalletUserTestRepo {
            hw: hw_pubkey,
            pin: pin_pubkey,
            challenge: None,
            instruction_sequence_number: 42,
        };

        assert_matches!(
            account_server
                .instruction_challenge(
                    InstructionChallengeRequest {
                        request: SignedChallengeRequest::sign(9, &hw_privkey).await.unwrap(),
                        certificate: cert.clone(),
                    },
                    &deps,
                    &EpochGenerator,
                    &hsm,
                )
                .await
                .expect_err("should return instruction sequence number mismatch error"),
            ChallengeError::Validation(wallet_common::account::errors::Error::SequenceNumberMismatch)
        );

        let challenge = account_server
            .instruction_challenge(
                InstructionChallengeRequest {
                    request: SignedChallengeRequest::sign(43, &hw_privkey).await.unwrap(),
                    certificate: cert.clone(),
                },
                &deps,
                &EpochGenerator,
                &hsm,
            )
            .await
            .unwrap();

        assert_matches!(
            account_server
                .handle_instruction(
                    Instruction::new_signed(CheckPin, challenge.clone(), 43, &hw_privkey, &pin_privkey, cert.clone())
                        .await
                        .unwrap(),
                    &instruction_result_signing_key,
                    &MockGenerators,
                    &WalletUserTestRepo {
                        hw: hw_pubkey,
                        pin: pin_pubkey,
                        challenge: Some(challenge.clone()),
                        instruction_sequence_number: 43,
                    },
                    &FailingPinPolicy,
                    &hsm,
                )
                .await
                .expect_err("sequence number mismatch error should result in IncorrectPin error"),
            InstructionError::IncorrectPin(IncorrectPinData {
                attempts_left_in_round: _,
                is_final_round: _
            })
        );

        account_server
            .handle_instruction(
                Instruction::new_signed(CheckPin, challenge.clone(), 44, &hw_privkey, &pin_privkey, cert.clone())
                    .await
                    .unwrap(),
                &instruction_result_signing_key,
                &MockGenerators,
                &WalletUserTestRepo {
                    hw: hw_pubkey,
                    pin: pin_pubkey,
                    challenge: Some(challenge),
                    instruction_sequence_number: 2,
                },
                &TimeoutPinPolicy,
                &hsm,
            )
            .await
            .expect("should return instruction result");
    }

    #[tokio::test]
    async fn valid_wallet_certificate_should_verify() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        let challenge_request = InstructionChallengeRequest {
            request: SignedChallengeRequest::sign(1, &hw_privkey).await.unwrap(),
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
                &EpochGenerator,
                &hsm,
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
                &hsm,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn wrong_hw_key_should_not_validate() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        account_server
            .verify_wallet_certificate(
                &cert,
                &WalletUserTestRepo {
                    hw: *SigningKey::random(&mut OsRng).verifying_key(),
                    pin: pin_pubkey,
                    challenge: None,
                    instruction_sequence_number: 0,
                },
                &hsm,
            )
            .await
            .expect_err("Should not validate");
    }

    #[tokio::test]
    async fn wrong_pin_key_should_not_validate() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        account_server
            .verify_wallet_certificate(
                &cert,
                &WalletUserTestRepo {
                    hw: hw_pubkey,
                    pin: *SigningKey::random(&mut OsRng).verifying_key(),
                    challenge: None,
                    instruction_sequence_number: 0,
                },
                &hsm,
            )
            .await
            .expect_err("Should not validate");
    }

    #[tokio::test]
    async fn valid_challenge_should_verify() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        let mut repo = WalletUserTestRepo {
            hw: hw_pubkey,
            pin: pin_pubkey,
            challenge: None,
            instruction_sequence_number: 0,
        };

        let challenge_request = InstructionChallengeRequest {
            request: SignedChallengeRequest::sign(1, &hw_privkey).await.unwrap(),
            certificate: cert.clone(),
        };

        let challenge = account_server
            .instruction_challenge(challenge_request, &repo, &EpochGenerator, &hsm)
            .await
            .unwrap();

        repo.challenge = Some(challenge.clone());

        let tx = repo.begin_transaction().await.unwrap();
        let wallet_user = repo.find_wallet_user_by_wallet_id(&tx, "0").await.unwrap();
        tx.commit().await.unwrap();

        assert_matches!(
            wallet_user,
            WalletUserQueryResult::Found(user) if account_server
                .verify_instruction(
                    Instruction::new_signed(CheckPin, challenge, 44, &hw_privkey, &pin_privkey, cert.clone())
                        .await
                        .unwrap(),
                    &user,
                    &EpochGenerator,
                    &hsm,
                )
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn wrong_challenge_should_not_verify() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        let mut repo = WalletUserTestRepo {
            hw: hw_pubkey,
            pin: pin_pubkey,
            challenge: None,
            instruction_sequence_number: 0,
        };

        let challenge_request = InstructionChallengeRequest {
            request: SignedChallengeRequest::sign(1, &hw_privkey).await.unwrap(),
            certificate: cert.clone(),
        };

        let challenge = account_server
            .instruction_challenge(challenge_request, &repo, &EpochGenerator, &hsm)
            .await
            .unwrap();

        repo.challenge = Some(random_bytes(32));

        let tx = repo.begin_transaction().await.unwrap();
        let wallet_user = repo.find_wallet_user_by_wallet_id(&tx, "0").await.unwrap();
        tx.commit().await.unwrap();

        assert_matches!(
            wallet_user,
            WalletUserQueryResult::Found(user) if matches!(
                account_server.verify_instruction(
                    Instruction::new_signed(CheckPin, challenge, 44, &hw_privkey, &pin_privkey, cert.clone())
                        .await
                        .unwrap(),
                    &user,
                    &EpochGenerator,
                    &hsm,
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
    async fn expired_challenge_should_not_verify() {
        let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

        let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let hw_pubkey = *hw_privkey.verifying_key();
        let pin_pubkey = *pin_privkey.verifying_key();

        let cert = do_registration(
            &account_server,
            &hsm,
            &certificate_signing_key,
            &hw_privkey,
            &pin_privkey,
        )
        .await;

        let repo = WalletUserTestRepo {
            hw: hw_pubkey,
            pin: pin_pubkey,
            challenge: None,
            instruction_sequence_number: 0,
        };

        let challenge_request = InstructionChallengeRequest {
            request: SignedChallengeRequest::sign(1, &hw_privkey).await.unwrap(),
            certificate: cert.clone(),
        };

        let challenge = account_server
            .instruction_challenge(challenge_request, &repo, &EpochGenerator, &hsm)
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
                account_server
                    .verify_instruction(
                        Instruction::new_signed(CheckPin, challenge, 44, &hw_privkey, &pin_privkey, cert.clone())
                            .await
                            .unwrap(),
                        &user,
                        &EpochGenerator,
                        &hsm,
                    )
                    .await,
                Err(InstructionValidationError::ChallengeTimeout)
            );
        }
    }
}
