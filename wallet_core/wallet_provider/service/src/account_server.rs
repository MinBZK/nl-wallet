use std::error::Error;
use std::time::Duration;

use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use futures::try_join;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::pkcs8::DecodePublicKey;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use tracing::debug;
use tracing::warn;
use uuid::Uuid;

use android_attest::android_crl;
use android_attest::android_crl::GoogleRevocationListClient;
use android_attest::android_crl::RevocationStatusList;
use android_attest::certificate_chain::verify_google_key_attestation_with_time;
use android_attest::certificate_chain::GoogleKeyAttestationError;
use android_attest::play_integrity::client::PlayIntegrityClient;
use android_attest::play_integrity::client::PlayIntegrityError;
use android_attest::play_integrity::integrity_verdict::IntegrityVerdict;
use android_attest::play_integrity::verification::IntegrityVerdictVerificationError;
use android_attest::play_integrity::verification::VerifiedIntegrityVerdict;
use android_attest::play_integrity::verification::VerifyPlayStore;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AppIdentifier;
use apple_app_attest::AssertionCounter;
use apple_app_attest::AttestationEnvironment;
use apple_app_attest::VerifiedAttestation;
use wallet_common::account::errors::Error as AccountError;
use wallet_common::account::messages::auth::Registration;
use wallet_common::account::messages::auth::RegistrationAttestation;
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
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::account::signed::ChallengeResponse;
use wallet_common::account::signed::ChallengeResponsePayload;
use wallet_common::account::signed::SequenceNumberComparison;
use wallet_common::generator::Generator;
use wallet_common::jwt::EcdsaDecodingKey;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtError;
use wallet_common::jwt::JwtSubject;
use wallet_common::keys::poa::PoaError;
use wallet_common::utils;
use wallet_provider_domain::model::encrypted::Encrypted;
use wallet_provider_domain::model::encrypter::Decrypter;
use wallet_provider_domain::model::encrypter::Encrypter;
use wallet_provider_domain::model::hsm::Hsm;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::pin_policy::PinPolicyEvaluation;
use wallet_provider_domain::model::pin_policy::PinPolicyEvaluator;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
use wallet_provider_domain::model::wallet_user::WalletUserAttestationCreate;
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
    #[error("could not extract trust anchor from provided Apple certificate")]
    AppleCertificate(#[from] webpki::Error),
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
pub enum AndroidKeyAttestationError {
    #[error("could not decode public key from leaf certificate: {0}")]
    LeafPublicKey(#[source] p256::pkcs8::spki::Error),
    #[error("could not obtain Google certificate revocation list")]
    CrlClient,
    #[error("android key attestation verification failed: {0}")]
    Verification(#[from] GoogleKeyAttestationError),
    #[error("certificate chain contains at least one revoked certificate")]
    RevokedCertificates,
}

#[derive(Debug, thiserror::Error)]
pub enum AndroidAppAttestationError {
    #[error("could not decode integrity toking using Play Integrity API")]
    DecodeIntegrityToken,
    #[error("validation if integrity verdict failed: {0}")]
    IntegrityVerdict(#[source] IntegrityVerdictVerificationError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("registration challenge UTF-8 decoding error: {0}")]
    ChallengeDecoding(#[source] std::string::FromUtf8Error),
    #[error("registration challenge validation error: {0}")]
    ChallengeValidation(#[source] JwtError),
    #[error("validation of Apple key and/or app attestation failed: {0}")]
    AppleAttestation(#[from] apple_app_attest::AttestationError),
    #[error("validation of Google key attestation failed: {0}")]
    AndroidKeyAttestation(#[from] AndroidKeyAttestationError),
    #[error("validation of Google app attestation failed: {0}")]
    AndroidAppAttestation(#[from] AndroidAppAttestationError),
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
    WteIssuance(#[source] Box<dyn Error + Send + Sync + 'static>),
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

pub struct AppleAttestationConfiguration {
    pub app_identifier: AppIdentifier,
    pub environment: AttestationEnvironment,
    pub trust_anchors: Vec<TrustAnchor<'static>>,
}

impl AppleAttestationConfiguration {
    pub fn new(
        team_identifier: String,
        bundle_identifier: String,
        environment: AttestationEnvironment,
        trust_anchors: Vec<TrustAnchor<'static>>,
    ) -> Self {
        let app_identifier = AppIdentifier::new(team_identifier, bundle_identifier);

        Self {
            app_identifier,
            environment,
            trust_anchors,
        }
    }
}

#[trait_variant::make(Send)]
pub trait GoogleCrlProvider {
    async fn get_crl(&self) -> Result<RevocationStatusList, android_crl::Error>;
}

impl GoogleCrlProvider for GoogleRevocationListClient {
    async fn get_crl(&self) -> Result<RevocationStatusList, android_crl::Error> {
        self.get().await
    }
}

#[trait_variant::make(Send)]
pub trait IntegrityTokenDecoder {
    type Error: Error + Send + Sync + 'static;

    async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), Self::Error>;
}

impl IntegrityTokenDecoder for PlayIntegrityClient {
    type Error = PlayIntegrityError;

    async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), Self::Error> {
        self.decode_token(integrity_token).await
    }
}

pub struct AndroidAttestationConfiguration {
    pub root_public_keys: Vec<RootPublicKey>,
    pub package_name: String,
    pub verify_play_store: VerifyPlayStore,
}

pub struct AccountServerKeys {
    pub wallet_certificate_signing_pubkey: EcdsaDecodingKey,
    pub encryption_key_identifier: String,
    pub pin_public_disclosure_protection_key_identifier: String,
}

pub struct AccountServer<GRC = GoogleRevocationListClient, PIC = PlayIntegrityClient> {
    pub name: String,
    instruction_challenge_timeout: Duration,
    keys: AccountServerKeys,
    pub apple_config: AppleAttestationConfiguration,
    pub android_config: AndroidAttestationConfiguration,
    google_crl_client: GRC,
    play_integrity_client: PIC,
}

pub struct UserState<R, H, W> {
    pub repositories: R,
    pub wallet_user_hsm: H,
    pub wte_issuer: W,
}

impl<R, H, W> UserState<R, H, W> {
    pub fn new(repositories: R, wallet_user_hsm: H, wte_issuer: W) -> Self {
        Self {
            repositories,
            wallet_user_hsm,
            wte_issuer,
        }
    }
}

impl<GRC, PIC> AccountServer<GRC, PIC> {
    pub fn new(
        name: String,
        instruction_challenge_timeout: Duration,
        keys: AccountServerKeys,
        apple_config: AppleAttestationConfiguration,
        android_config: AndroidAttestationConfiguration,
        google_crl_client: GRC,
        play_integrity_client: PIC,
    ) -> Result<Self, AccountServerInitError> {
        Ok(AccountServer {
            instruction_challenge_timeout,
            name,
            keys,
            apple_config,
            android_config,
            google_crl_client,
            play_integrity_client,
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
                wallet_id: utils::random_string(32),
                random: utils::random_bytes(32),
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
        registration_message: ChallengeResponse<Registration>,
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<WalletCertificate, RegistrationError>
    where
        GRC: GoogleCrlProvider,
        PIC: IntegrityTokenDecoder,
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

        debug!("Verifying challenge and extracting wallet id");

        let challenge = &unverified.challenge;
        let wallet_id =
            Self::verify_registration_challenge(&self.keys.wallet_certificate_signing_pubkey, challenge)?.wallet_id;

        debug!("Validating attestation and checking signed registration against the provided hardware and pin keys");

        let attestation_timestamp = Utc::now();
        let challenge_hash = utils::sha256(challenge);
        let sequence_number_comparison = SequenceNumberComparison::EqualTo(0);
        let DerVerifyingKey(pin_pubkey) = unverified.payload.pin_pubkey;

        let (hw_pubkey, attestation) = match unverified.payload.attestation {
            RegistrationAttestation::Apple { data } => {
                debug!("Validating Apple key and app attestation");

                let (_, hw_pubkey) = VerifiedAttestation::parse_and_verify_with_time(
                    &data,
                    &self.apple_config.trust_anchors,
                    attestation_timestamp,
                    &challenge_hash,
                    &self.apple_config.app_identifier,
                    self.apple_config.environment,
                )?;

                debug!("Checking registration signatures");

                let attestation = registration_message
                    .parse_and_verify_apple(
                        challenge,
                        sequence_number_comparison,
                        &hw_pubkey,
                        &self.apple_config.app_identifier,
                        AssertionCounter::default(),
                        &pin_pubkey,
                    )
                    .map(|(_, assertion_counter)| WalletUserAttestationCreate::Apple {
                        data,
                        assertion_counter,
                    })
                    .map_err(RegistrationError::MessageValidation)?;

                (hw_pubkey, attestation)
            }
            RegistrationAttestation::Google {
                certificate_chain,
                integrity_token,
            } => {
                debug!("Validating Android key attestation");

                // Verify the certificate chain according to the google key attestation verification rules
                let crl = self.google_crl_client.get_crl().await.map_err(|error| {
                    warn!("could not obtain Google certificate revocation list: {0}", error);

                    AndroidKeyAttestationError::CrlClient
                })?;
                let attested_key_chain = certificate_chain
                    .as_ref()
                    .iter()
                    .map(|cert| CertificateDer::from_slice(cert))
                    .collect::<Vec<_>>();
                let leaf_certificate = verify_google_key_attestation_with_time(
                    &attested_key_chain,
                    &self.android_config.root_public_keys,
                    &crl,
                    &challenge_hash,
                    attestation_timestamp,
                )
                .map_err(|error| match error {
                    GoogleKeyAttestationError::RevokedCertificates(revocation_log) => {
                        warn!(
                            "found revoked certificates while verifying Android attested key certificate chain: {}",
                            revocation_log.join(" ")
                        );

                        AndroidKeyAttestationError::RevokedCertificates
                    }
                    error => AndroidKeyAttestationError::Verification(error),
                })?;

                // Extract the leaf certificate's verifying key
                let hw_pubkey = VerifyingKey::from_public_key_der(leaf_certificate.public_key().raw)
                    .map_err(AndroidKeyAttestationError::LeafPublicKey)?;

                debug!("Validating Android app attestation");

                let (integrity_verdict, integrity_verdict_json) = self
                    .play_integrity_client
                    .decode_token(&integrity_token)
                    .await
                    .map_err(|error| {
                        warn!("could not decode integrity token using Play Integrity API: {0}", error);

                        AndroidAppAttestationError::DecodeIntegrityToken
                    })?;

                let _ = VerifiedIntegrityVerdict::verify_with_time(
                    integrity_verdict,
                    &self.android_config.package_name,
                    &BASE64_STANDARD.encode(&challenge_hash),
                    &self.android_config.verify_play_store,
                    attestation_timestamp,
                )
                .map_err(AndroidAppAttestationError::IntegrityVerdict)?;

                debug!("Checking registration signatures");

                let attestation = registration_message
                    .parse_and_verify_google(challenge, sequence_number_comparison, &hw_pubkey, &pin_pubkey)
                    .map(|_| WalletUserAttestationCreate::Android {
                        certificate_chain: certificate_chain.into_inner(),
                        integrity_verdict_json,
                    })
                    .map_err(RegistrationError::MessageValidation)?;

                (hw_pubkey, attestation)
            }
        };

        debug!("Starting database transaction");

        let encrypted_pin_pubkey = Encrypter::encrypt(
            &user_state.wallet_user_hsm,
            &self.keys.encryption_key_identifier,
            pin_pubkey,
        )
        .await?;

        let tx = user_state.repositories.begin_transaction().await?;

        debug!("Creating new wallet user");

        let uuid = user_state
            .repositories
            .create_wallet_user(
                &tx,
                WalletUserCreate {
                    wallet_id: wallet_id.clone(),
                    hw_pubkey,
                    encrypted_pin_pubkey,
                    attestation_date_time: attestation_timestamp,
                    attestation,
                },
            )
            .await?;

        debug!("Generating new wallet certificate for user {}", uuid);

        let wallet_certificate = new_wallet_certificate(
            self.name.clone(),
            &self.keys.pin_public_disclosure_protection_key_identifier,
            certificate_signing_key,
            wallet_id,
            hw_pubkey,
            &pin_pubkey,
            &user_state.wallet_user_hsm,
        )
        .await?;

        tx.commit().await?;

        Ok(wallet_certificate)
    }

    pub async fn instruction_challenge<T, R, H>(
        &self,
        challenge_request: InstructionChallengeRequest,
        time_generator: &impl Generator<DateTime<Utc>>,
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<Vec<u8>, ChallengeError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Decrypter<VerifyingKey, Error = HsmError> + Hsm<Error = HsmError>,
    {
        debug!("Parse certificate and retrieving wallet user");
        let (user, claims) = parse_claims_and_retrieve_wallet_user(
            &challenge_request.certificate,
            &self.keys.wallet_certificate_signing_pubkey,
            &user_state.repositories,
        )
        .await?;

        debug!("Parsing and verifying challenge request for user {}", user.id);

        let sequence_number_comparison = SequenceNumberComparison::LargerThan(user.instruction_sequence_number);
        let DerVerifyingKey(hw_pubkey) = &user.hw_pubkey;
        let (request, assertion_counter) = match user.attestation {
            WalletUserAttestation::Apple { assertion_counter } => challenge_request
                .request
                .parse_and_verify_apple(
                    &claims.wallet_id,
                    sequence_number_comparison,
                    hw_pubkey,
                    &self.apple_config.app_identifier,
                    assertion_counter,
                )
                .map(|(request, assertion_counter)| (request, Some(assertion_counter))),
            WalletUserAttestation::Android => challenge_request
                .request
                .parse_and_verify_google(&claims.wallet_id, sequence_number_comparison, hw_pubkey)
                .map(|request| (request, None)),
        }?;

        debug!("Verifying wallet certificate");
        verify_wallet_certificate_public_keys(
            claims,
            &self.keys.pin_public_disclosure_protection_key_identifier,
            &self.keys.encryption_key_identifier,
            &user.hw_pubkey,
            if request.instruction_name == ChangePinRollback::NAME {
                user.encrypted_previous_pin_pubkey.unwrap_or(user.encrypted_pin_pubkey)
            } else {
                user.encrypted_pin_pubkey
            },
            &user_state.wallet_user_hsm,
        )
        .await?;

        debug!("Challenge request valid, persisting generated challenge and incremented sequence number");
        let challenge = InstructionChallenge {
            bytes: utils::random_bytes(32),
            expiration_date_time: time_generator.generate() + self.instruction_challenge_timeout,
        };

        debug!("Starting database transaction");
        let tx = user_state.repositories.begin_transaction().await?;

        let instruction_update = user_state
            .repositories
            .update_instruction_challenge_and_sequence_number(
                &tx,
                &user.wallet_id,
                challenge.clone(),
                request.sequence_number,
            );

        if let Some(assertion_counter) = assertion_counter {
            let update_assertion_counter =
                user_state
                    .repositories
                    .update_apple_assertion_counter(&tx, &user.wallet_id, assertion_counter);
            try_join!(instruction_update, update_assertion_counter,)?;
        } else {
            instruction_update.await?;
        }

        tx.commit().await?;

        debug!("Responding with generated challenge");
        Ok(challenge.bytes)
    }

    pub async fn handle_instruction<T, R, I, IR, G, H>(
        &self,
        instruction: Instruction<I>,
        instruction_result_signing_key: &impl InstructionResultSigningKey,
        generators: &G,
        pin_policy: &impl PinPolicyEvaluator,
        user_state: &UserState<R, H, impl WteIssuer>,
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
            .verify_and_extract_instruction(instruction, generators, pin_policy, user_state, |wallet_user| {
                wallet_user.encrypted_pin_pubkey.clone()
            })
            .await?;

        let instruction_result = instruction_payload.handle(&wallet_user, generators, user_state).await?;

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
        pin_policy: &impl PinPolicyEvaluator,
        user_state: &UserState<R, H, impl WteIssuer>,
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
            .verify_and_extract_instruction(instruction, generators, pin_policy, user_state, |wallet_user| {
                wallet_user.encrypted_pin_pubkey.clone()
            })
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

        let encrypted_pin_pubkey = Encrypter::encrypt(
            &user_state.wallet_user_hsm,
            &self.keys.encryption_key_identifier,
            pin_pubkey,
        )
        .await?;

        let tx = user_state.repositories.begin_transaction().await?;

        user_state
            .repositories
            .change_pin(&tx, wallet_user.wallet_id.as_str(), encrypted_pin_pubkey)
            .await?;

        let wallet_certificate = new_wallet_certificate(
            self.name.clone(),
            &self.keys.pin_public_disclosure_protection_key_identifier,
            signing_keys.1,
            wallet_user.wallet_id,
            wallet_user.hw_pubkey.0,
            &pin_pubkey,
            &user_state.wallet_user_hsm,
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
        pin_policy: &impl PinPolicyEvaluator,
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<InstructionResult<()>, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
        H: WalletUserHsm<Error = HsmError> + Hsm<Error = HsmError> + Decrypter<VerifyingKey, Error = HsmError>,
    {
        let (wallet_user, _) = self
            .verify_and_extract_instruction(instruction, generators, pin_policy, user_state, |wallet_user| {
                wallet_user
                    .encrypted_previous_pin_pubkey
                    .clone()
                    .unwrap_or(wallet_user.encrypted_pin_pubkey.clone())
            })
            .await?;

        debug!(
            "Starting database transaction and instruction handling process for user {}",
            &wallet_user.id
        );

        let tx = user_state.repositories.begin_transaction().await?;

        user_state
            .repositories
            .rollback_pin_change(&tx, wallet_user.wallet_id.as_str())
            .await?;

        tx.commit().await?;

        self.sign_instruction_result(instruction_result_signing_key, ()).await
    }

    async fn verify_and_extract_instruction<T, R, I, G, H, F>(
        &self,
        instruction: Instruction<I>,
        generators: &G,
        pin_policy: &impl PinPolicyEvaluator,
        user_state: &UserState<R, H, impl WteIssuer>,
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
            &self.keys.wallet_certificate_signing_pubkey,
            &self.keys.pin_public_disclosure_protection_key_identifier,
            &self.keys.encryption_key_identifier,
            pin_pubkey,
            user_state,
        )
        .await?;

        debug!(
            "Starting database transaction and instruction handling process for user {}",
            &wallet_user.id
        );

        let tx = user_state.repositories.begin_transaction().await?;

        debug!("Clearing instruction challenge");

        user_state
            .repositories
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

        let verification_result = self
            .verify_instruction(instruction, &wallet_user, generators, &user_state.wallet_user_hsm)
            .await;

        match verification_result {
            Ok((challenge_response_payload, assertion_counter)) => {
                debug!("Instruction successfully verified, validating instruction");

                challenge_response_payload.payload.validate_instruction(&wallet_user)?;

                debug!("Instruction successfully validated, resetting pin retries");

                let reset_pin_entries = user_state
                    .repositories
                    .reset_unsuccessful_pin_entries(&tx, &wallet_user.wallet_id);

                debug!(
                    "Updating instruction sequence number to {}",
                    challenge_response_payload.sequence_number
                );

                let update_sequence_number = user_state.repositories.update_instruction_sequence_number(
                    &tx,
                    &wallet_user.wallet_id,
                    challenge_response_payload.sequence_number,
                );

                if let Some(assertion_counter) = assertion_counter {
                    let update_assertion_counter = user_state.repositories.update_apple_assertion_counter(
                        &tx,
                        &wallet_user.wallet_id,
                        assertion_counter,
                    );
                    try_join!(reset_pin_entries, update_sequence_number, update_assertion_counter)?;
                } else {
                    try_join!(reset_pin_entries, update_sequence_number)?;
                }

                tx.commit().await?;

                Ok((wallet_user, challenge_response_payload.payload))
            }
            Err(validation_error) => {
                let error = if matches!(validation_error, InstructionValidationError::VerificationFailed(_)) {
                    debug!("Instruction validation failed, registering unsuccessful pin entry");

                    user_state
                        .repositories
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
        &self,
        instruction: Instruction<I>,
        wallet_user: &WalletUser,
        time_generator: &impl Generator<DateTime<Utc>>,
        verifying_key_decrypter: &D,
    ) -> Result<(ChallengeResponsePayload<I>, Option<AssertionCounter>), InstructionValidationError>
    where
        I: InstructionAndResult,
        D: Decrypter<VerifyingKey, Error = HsmError>,
    {
        let challenge = Self::verify_instruction_challenge(wallet_user, time_generator)?;

        let pin_pubkey = verifying_key_decrypter
            .decrypt(
                &self.keys.encryption_key_identifier,
                wallet_user.encrypted_pin_pubkey.clone(),
            )
            .await?;

        let sequence_number_comparison = SequenceNumberComparison::LargerThan(wallet_user.instruction_sequence_number);
        let DerVerifyingKey(hw_pubkey) = &wallet_user.hw_pubkey;
        let (parsed, assertion_counter) = match wallet_user.attestation {
            WalletUserAttestation::Apple { assertion_counter } => instruction
                .instruction
                .parse_and_verify_apple(
                    &challenge.bytes,
                    sequence_number_comparison,
                    hw_pubkey,
                    &self.apple_config.app_identifier,
                    assertion_counter,
                    &pin_pubkey,
                )
                .map(|(parsed, assertion_counter)| (parsed, Some(assertion_counter))),
            WalletUserAttestation::Android => instruction
                .instruction
                .parse_and_verify_google(&challenge.bytes, sequence_number_comparison, hw_pubkey, &pin_pubkey)
                .map(|parsed| (parsed, None)),
        }
        .map_err(InstructionValidationError::VerificationFailed)?;

        Ok((parsed, assertion_counter))
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
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use p256::ecdsa::SigningKey;

    use android_attest::mock_chain::MockCaChain;
    use apple_app_attest::MockAttestationCa;
    use wallet_common::apple::MockAppleAttestedKey;
    use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;
    use wallet_provider_persistence::repositories::mock::WalletUserTestRepo;

    use crate::wallet_certificate;
    use crate::wte_issuer::mock::MockWteIssuer;

    use super::*;

    pub static MOCK_APPLE_CA: LazyLock<MockAttestationCa> = LazyLock::new(MockAttestationCa::generate);
    pub static MOCK_GOOGLE_CA_CHAIN: LazyLock<MockCaChain> = LazyLock::new(|| MockCaChain::generate(1));

    pub type MockAccountServer = AccountServer<RevocationStatusList, MockPlayIntegrityClient>;

    #[derive(Clone, Copy)]
    pub enum AttestationType {
        Apple,
        Google,
    }

    #[derive(Clone, Copy)]
    pub enum AttestationCa<'a> {
        Apple(&'a MockAttestationCa),
        Google(&'a MockCaChain),
    }

    impl GoogleCrlProvider for RevocationStatusList {
        async fn get_crl(&self) -> Result<RevocationStatusList, android_crl::Error> {
            Ok(self.clone())
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("mock play integrity client error to be used in tests")]
    pub struct MockPlayIntegrityClientError {}

    pub struct MockPlayIntegrityClient {
        pub package_name: String,
        pub verify_play_store: VerifyPlayStore,
        pub has_error: bool,
    }

    impl MockPlayIntegrityClient {
        pub fn new(package_name: String, verify_play_store: VerifyPlayStore) -> Self {
            Self {
                package_name,
                verify_play_store,
                has_error: false,
            }
        }
    }

    impl IntegrityTokenDecoder for MockPlayIntegrityClient {
        type Error = MockPlayIntegrityClientError;

        async fn decode_token(&self, integrity_token: &str) -> Result<(IntegrityVerdict, String), Self::Error> {
            if self.has_error {
                return Err(MockPlayIntegrityClientError {});
            }

            // For testing, assume that the mock integrity token simply equals the request hash.
            let verdict = IntegrityVerdict::new_mock(
                self.package_name.clone(),
                integrity_token.to_string(),
                self.verify_play_store.clone(),
            );
            let json = serde_json::to_string(&verdict).unwrap();

            Ok((verdict, json))
        }
    }

    pub fn setup_account_server(
        certificate_signing_pubkey: &VerifyingKey,
        crl: RevocationStatusList,
    ) -> MockAccountServer {
        let integrity_client = MockPlayIntegrityClient::new(
            "com.example.app".to_string(),
            VerifyPlayStore::Verify {
                play_store_certificate_hashes: HashSet::from([utils::random_bytes(16)]),
            },
        );

        AccountServer::new(
            "mock_account_server".into(),
            Duration::from_millis(15000),
            AccountServerKeys {
                wallet_certificate_signing_pubkey: certificate_signing_pubkey.into(),
                encryption_key_identifier: wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER.to_string(),
                pin_public_disclosure_protection_key_identifier:
                    wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER.to_string(),
            },
            AppleAttestationConfiguration {
                app_identifier: AppIdentifier::new_mock(),
                environment: AttestationEnvironment::Development,
                trust_anchors: vec![MOCK_APPLE_CA.trust_anchor().to_owned()],
            },
            AndroidAttestationConfiguration {
                root_public_keys: vec![RootPublicKey::Rsa(MOCK_GOOGLE_CA_CHAIN.root_public_key.clone())],
                package_name: integrity_client.package_name.clone(),
                verify_play_store: integrity_client.verify_play_store.clone(),
            },
            crl,
            integrity_client,
        )
        .unwrap()
    }

    pub type MockUserState = UserState<WalletUserTestRepo, MockPkcs11Client<HsmError>, MockWteIssuer>;

    pub fn user_state<R>(
        repositories: R,
        wallet_user_hsm: MockPkcs11Client<HsmError>,
    ) -> UserState<R, MockPkcs11Client<HsmError>, MockWteIssuer> {
        UserState::<R, MockPkcs11Client<HsmError>, MockWteIssuer> {
            repositories,
            wallet_user_hsm,
            wte_issuer: MockWteIssuer,
        }
    }

    #[derive(Debug)]
    pub enum MockHardwareKey {
        Apple(MockAppleAttestedKey),
        Google(SigningKey),
    }

    impl MockHardwareKey {
        pub fn verifying_key(&self) -> &VerifyingKey {
            match self {
                Self::Apple(attested_key) => attested_key.verifying_key(),
                Self::Google(signing_key) => signing_key.verifying_key(),
            }
        }

        pub async fn sign_instruction_challenge<I>(
            &self,
            wallet_id: String,
            instruction_sequence_number: u64,
            certificate: WalletCertificate,
        ) -> InstructionChallengeRequest
        where
            I: InstructionAndResult,
        {
            match self {
                Self::Apple(attested_key) => {
                    InstructionChallengeRequest::new_apple::<I>(
                        wallet_id,
                        instruction_sequence_number,
                        attested_key,
                        certificate,
                    )
                    .await
                }
                Self::Google(signing_key) => {
                    InstructionChallengeRequest::new_google::<I>(
                        wallet_id,
                        instruction_sequence_number,
                        signing_key,
                        certificate,
                    )
                    .await
                }
            }
            .unwrap()
        }

        pub async fn sign_instruction<T>(
            &self,
            instruction: T,
            challenge: Vec<u8>,
            instruction_sequence_number: u64,
            pin_privkey: &SigningKey,
            certificate: WalletCertificate,
        ) -> Instruction<T>
        where
            T: Serialize + DeserializeOwned,
        {
            match self {
                Self::Apple(attested_key) => {
                    Instruction::new_apple(
                        instruction,
                        challenge,
                        instruction_sequence_number,
                        attested_key,
                        pin_privkey,
                        certificate,
                    )
                    .await
                }
                Self::Google(signing_key) => {
                    Instruction::new_google(
                        instruction,
                        challenge,
                        instruction_sequence_number,
                        signing_key,
                        pin_privkey,
                        certificate,
                    )
                    .await
                }
            }
            .unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use base64::prelude::*;
    use chrono::DateTime;
    use chrono::TimeZone;
    use chrono::Utc;
    use hmac::digest::crypto_common::rand_core::OsRng;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rstest::rstest;
    use uuid::uuid;

    use android_attest::attestation_extension::key_description::KeyDescription;
    use android_attest::mock_chain::MockCaChain;
    use android_attest::play_integrity::verification::VerifyPlayStore;
    use apple_app_attest::AssertionCounter;
    use apple_app_attest::AssertionError;
    use apple_app_attest::AssertionValidationError;
    use apple_app_attest::MockAttestationCa;
    use wallet_common::account::errors::Error as AccountError;
    use wallet_common::account::messages::auth::WalletCertificate;
    use wallet_common::account::messages::errors::IncorrectPinData;
    use wallet_common::account::messages::instructions::ChangePinCommit;
    use wallet_common::account::messages::instructions::ChangePinRollback;
    use wallet_common::account::messages::instructions::ChangePinStart;
    use wallet_common::account::messages::instructions::CheckPin;
    use wallet_common::account::messages::instructions::InstructionAndResult;
    use wallet_common::account::messages::instructions::InstructionResult;
    use wallet_common::account::signed::ChallengeResponse;
    use wallet_common::apple::MockAppleAttestedKey;
    use wallet_common::generator::Generator;
    use wallet_common::jwt::EcdsaDecodingKey;
    use wallet_common::keys::EcdsaKey;
    use wallet_common::utils;
    use wallet_provider_domain::generator::mock::MockGenerators;
    use wallet_provider_domain::model::encrypted::Encrypted;
    use wallet_provider_domain::model::encrypter::Encrypter;
    use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;
    use wallet_provider_domain::model::wallet_user::InstructionChallenge;
    use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
    use wallet_provider_domain::model::FailingPinPolicy;
    use wallet_provider_domain::model::TimeoutPinPolicy;
    use wallet_provider_domain::repository::Committable;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_domain::repository::TransactionStarter;
    use wallet_provider_domain::repository::WalletUserRepository;
    use wallet_provider_domain::EpochGenerator;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;
    use wallet_provider_persistence::repositories::mock::WalletUserTestRepo;

    use crate::hsm::HsmError;
    use crate::keys::WalletCertificateSigningKey;
    use crate::wallet_certificate;
    use crate::wallet_certificate::mock::setup_hsm;
    use crate::wallet_certificate::mock::WalletCertificateSetup;
    use crate::wallet_certificate::verify_wallet_certificate;
    use crate::wte_issuer::mock::MockWteIssuer;

    use super::mock;
    use super::mock::AttestationCa;
    use super::mock::AttestationType;
    use super::mock::MockAccountServer;
    use super::mock::MockHardwareKey;
    use super::mock::MockPlayIntegrityClient;
    use super::mock::MockUserState;
    use super::mock::MOCK_APPLE_CA;
    use super::mock::MOCK_GOOGLE_CA_CHAIN;
    use super::AndroidAppAttestationError;
    use super::ChallengeError;
    use super::InstructionError;
    use super::InstructionValidationError;
    use super::RegistrationError;
    use super::UserState;

    async fn do_registration(
        account_server: &MockAccountServer,
        certificate_signing_key: &impl WalletCertificateSigningKey,
        pin_privkey: &SigningKey,
        attestation_ca: AttestationCa<'_>,
    ) -> Result<(WalletCertificate, MockHardwareKey, MockPkcs11Client<HsmError>), RegistrationError> {
        let challenge = account_server
            .registration_challenge(certificate_signing_key)
            .await
            .expect("Could not get registration challenge");

        let challenge_hash = utils::sha256(&challenge);
        let (registration_message, hw_privkey) = match attestation_ca {
            AttestationCa::Apple(apple_mock_ca) => {
                let (attested_key, attestation_data) = MockAppleAttestedKey::new_with_attestation(
                    apple_mock_ca,
                    &challenge_hash,
                    account_server.apple_config.environment,
                    account_server.apple_config.app_identifier.clone(),
                );
                let registration_message =
                    ChallengeResponse::new_apple(&attested_key, attestation_data, pin_privkey, challenge)
                        .await
                        .expect("Could not sign new Apple attested registration");

                (registration_message, MockHardwareKey::Apple(attested_key))
            }
            AttestationCa::Google(android_mock_ca_chain) => {
                let integrity_token = BASE64_STANDARD.encode(&challenge_hash);
                let key_description = KeyDescription::new_valid_mock(challenge_hash);

                let (attested_certificate_chain, attested_private_key) =
                    android_mock_ca_chain.generate_attested_leaf_certificate(&key_description);
                let registration_message = ChallengeResponse::new_google(
                    &attested_private_key,
                    attested_certificate_chain.try_into().unwrap(),
                    integrity_token,
                    pin_privkey,
                    challenge,
                )
                .await
                .expect("Could not sign new Google attested registration");

                (registration_message, MockHardwareKey::Google(attested_private_key))
            }
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_create_wallet_user()
            .returning(|_, _| Ok(uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08")));

        let hsm = setup_hsm().await;
        let user_state = UserState {
            repositories: wallet_user_repo,
            wallet_user_hsm: hsm,
            wte_issuer: MockWteIssuer,
        };

        account_server
            .register(certificate_signing_key, registration_message, &user_state)
            .await
            .map(|wallet_certificate| (wallet_certificate, hw_privkey, user_state.wallet_user_hsm))
    }

    async fn setup_and_do_registration(
        attestation_type: AttestationType,
    ) -> (
        WalletCertificateSetup,
        MockAccountServer,
        MockHardwareKey,
        WalletCertificate,
        MockUserState,
    ) {
        let setup = WalletCertificateSetup::new().await;
        let account_server = mock::setup_account_server(&setup.signing_pubkey, Default::default());

        let attestation_ca = match attestation_type {
            AttestationType::Apple => AttestationCa::Apple(&MOCK_APPLE_CA),
            AttestationType::Google => AttestationCa::Google(&MOCK_GOOGLE_CA_CHAIN),
        };

        let (cert, hw_privkey, hsm) =
            do_registration(&account_server, &setup.signing_key, &setup.pin_privkey, attestation_ca)
                .await
                .expect("Could not process registration message at account server");

        let apple_assertion_counter = match attestation_type {
            AttestationType::Apple => Some(AssertionCounter::from(1)),
            AttestationType::Google => None,
        };

        let repo = WalletUserTestRepo {
            hw_pubkey: *hw_privkey.verifying_key(),
            encrypted_pin_pubkey: setup.encrypted_pin_pubkey.clone(),
            previous_encrypted_pin_pubkey: None,
            challenge: None,
            instruction_sequence_number: 0,
            apple_assertion_counter,
        };

        let user_state = mock::user_state(repo, hsm);

        (setup, account_server, hw_privkey, cert, user_state)
    }

    async fn do_instruction_challenge<I>(
        account_server: &MockAccountServer,
        hw_privkey: &MockHardwareKey,
        wallet_certificate: WalletCertificate,
        instruction_sequence_number: u64,
        user_state: &MockUserState,
    ) -> Result<Vec<u8>, ChallengeError>
    where
        I: InstructionAndResult,
    {
        let instruction_challenge = hw_privkey
            .sign_instruction_challenge::<I>(
                wallet_certificate.dangerous_parse_unverified().unwrap().1.wallet_id,
                instruction_sequence_number,
                wallet_certificate,
            )
            .await;

        account_server
            .instruction_challenge(instruction_challenge, &EpochGenerator, user_state)
            .await
    }

    async fn do_check_pin(
        account_server: &MockAccountServer,
        pin_privkey: &SigningKey,
        hw_privkey: &MockHardwareKey,
        wallet_certificate: WalletCertificate,
        instruction_result_signing_key: &SigningKey,
        user_state: &mut MockUserState,
    ) -> Result<InstructionResult<()>, anyhow::Error> {
        let challenge = do_instruction_challenge::<CheckPin>(
            account_server,
            hw_privkey,
            wallet_certificate.clone(),
            43,
            user_state,
        )
        .await?;

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            instruction_sequence_number: 43,
            apple_assertion_counter: match hw_privkey {
                MockHardwareKey::Apple(attested_key) => Some(AssertionCounter::from(*attested_key.next_counter() - 1)),
                MockHardwareKey::Google(_) => None,
            },
            ..user_state.repositories.clone()
        };

        let instruction_error = account_server
            .handle_instruction(
                hw_privkey
                    .sign_instruction(CheckPin, challenge.clone(), 43, pin_privkey, wallet_certificate.clone())
                    .await,
                instruction_result_signing_key,
                &MockGenerators,
                &FailingPinPolicy,
                user_state,
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

        user_state.repositories = WalletUserTestRepo {
            instruction_sequence_number: 2,
            ..user_state.repositories.clone()
        };

        let result = account_server
            .handle_instruction(
                hw_privkey
                    .sign_instruction(CheckPin, challenge, 44, pin_privkey, wallet_certificate.clone())
                    .await,
                instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                user_state,
            )
            .await?;

        Ok(result)
    }

    async fn do_pin_change_start(
        account_server: &MockAccountServer,
        wallet_certificate_setup: &WalletCertificateSetup,
        hw_privkey: &MockHardwareKey,
        wallet_certificate: WalletCertificate,
        instruction_result_signing_key: &SigningKey,
        user_state: &mut MockUserState,
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
            hw_privkey,
            wallet_certificate.clone(),
            43,
            user_state,
        )
        .await
        .unwrap();

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            instruction_sequence_number: 2,
            ..user_state.repositories.clone()
        };

        let pop_pin_pubkey = new_pin_privkey.try_sign(challenge.as_slice()).await.unwrap();

        let new_certificate_result = account_server
            .handle_change_pin_start_instruction(
                hw_privkey
                    .sign_instruction(
                        ChangePinStart {
                            pin_pubkey: new_pin_pubkey.into(),
                            pop_pin_pubkey: pop_pin_pubkey.into(),
                        },
                        challenge.clone(),
                        44,
                        &wallet_certificate_setup.pin_privkey,
                        wallet_certificate.clone(),
                    )
                    .await,
                (instruction_result_signing_key, &wallet_certificate_setup.signing_key),
                &MockGenerators,
                &TimeoutPinPolicy,
                user_state,
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
    #[rstest]
    async fn test_register(
        #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
    ) {
        let (setup, account_server, hw_privkey, cert, user_state) = setup_and_do_registration(attestation_type).await;

        let cert_data = cert
            .parse_and_verify_with_sub(&setup.signing_key.verifying_key().into())
            .expect("Could not parse and verify wallet certificate");
        assert_eq!(cert_data.iss, account_server.name);
        assert_eq!(&cert_data.hw_pubkey.0, hw_privkey.verifying_key());

        verify_wallet_certificate(
            &cert,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
            &user_state,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[rstest]
    async fn test_register_invalid_apple_attestation() {
        let setup = WalletCertificateSetup::new().await;
        let account_server = mock::setup_account_server(&setup.signing_pubkey, Default::default());

        // Have a `MockAppleAttestedKey` be generated under a different CA to make the attestation validation fail.
        let other_apple_mock_ca = MockAttestationCa::generate();

        let error = do_registration(
            &account_server,
            &setup.signing_key,
            &setup.pin_privkey,
            AttestationCa::Apple(&other_apple_mock_ca),
        )
        .await
        .map(|_| ()) // the return value MockPkcs11Client doesn't implement Debug, so discard it
        .expect_err("registering with an invalid Apple attestation should fail");

        assert_matches!(error, RegistrationError::AppleAttestation(_));
    }

    #[tokio::test]
    #[rstest]
    async fn test_register_invalid_android_key_attestation() {
        let setup = WalletCertificateSetup::new().await;
        let account_server = mock::setup_account_server(&setup.signing_pubkey, Default::default());

        // Generate the Google certificate chain using a different set of CAs to make the attestation validation fail.
        let other_android_mock_ca_chain = MockCaChain::generate(1);

        let error = do_registration(
            &account_server,
            &setup.signing_key,
            &setup.pin_privkey,
            AttestationCa::Google(&other_android_mock_ca_chain),
        )
        .await
        .map(|_| ())
        .expect_err("registering with an invalid Android attestation should fail");

        assert_matches!(error, RegistrationError::AndroidKeyAttestation(_));
    }

    #[tokio::test]
    #[rstest]
    async fn test_register_android_play_integrity_client_error() {
        let setup = WalletCertificateSetup::new().await;
        let mut account_server = mock::setup_account_server(&setup.signing_pubkey, Default::default());

        // Have the Play Integrity client return an error.
        account_server.play_integrity_client.has_error = true;

        let error = do_registration(
            &account_server,
            &setup.signing_key,
            &setup.pin_privkey,
            AttestationCa::Google(&MOCK_GOOGLE_CA_CHAIN),
        )
        .await
        .map(|_| ())
        .expect_err("registering should fail when the Play Integrity API fails to decode the token");

        assert_matches!(
            error,
            RegistrationError::AndroidAppAttestation(AndroidAppAttestationError::DecodeIntegrityToken)
        );
    }

    #[tokio::test]
    #[rstest]
    async fn test_register_invalid_android_integrity_verdict() {
        let setup = WalletCertificateSetup::new().await;
        let mut account_server = mock::setup_account_server(&setup.signing_pubkey, Default::default());

        // Have the Play Integrity API expect a different package name.
        account_server.play_integrity_client =
            MockPlayIntegrityClient::new("com.example.other".to_string(), VerifyPlayStore::NoVerify);

        let error = do_registration(
            &account_server,
            &setup.signing_key,
            &setup.pin_privkey,
            AttestationCa::Google(&MOCK_GOOGLE_CA_CHAIN),
        )
        .await
        .map(|_| ())
        .expect_err("registering with an invalid Android Integrity Verdict should fail");

        assert_matches!(
            error,
            RegistrationError::AndroidAppAttestation(AndroidAppAttestationError::IntegrityVerdict(_))
        );
    }

    #[tokio::test]
    #[rstest]
    async fn test_challenge_request_error_signature_type_mismatch(
        #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
    ) {
        let (_setup, account_server, _hw_privkey, cert, user_state) = setup_and_do_registration(attestation_type).await;

        // Create a hardware key that is the opposite type of the one used during registration.
        let wrong_hw_privkey = match attestation_type {
            AttestationType::Apple => MockHardwareKey::Google(SigningKey::random(&mut OsRng)),
            AttestationType::Google => MockHardwareKey::Apple(MockAppleAttestedKey::new_random(
                account_server.apple_config.app_identifier.clone(),
            )),
        };

        let error = do_instruction_challenge::<CheckPin>(&account_server, &wrong_hw_privkey, cert, 43, &user_state)
            .await
            .expect_err(
                "requesting a challenge with a different signature type than used during registration should fail",
            );

        assert_matches!(
            error,
            ChallengeError::Validation(wallet_common::account::errors::Error::SignatureTypeMismatch { .. })
        )
    }

    #[tokio::test]
    #[rstest]
    async fn test_challenge_request_error_apple_assertion_counter() {
        let (_setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(AttestationType::Apple).await;
        user_state.repositories.apple_assertion_counter = Some(AssertionCounter::from(200));

        let error = do_instruction_challenge::<CheckPin>(&account_server, &hw_privkey, cert, 43, &user_state)
            .await
            .expect_err(
                "requesting a challenge with a different signature type than used during registration should fail",
            );

        assert_matches!(
            error,
            ChallengeError::Validation(wallet_common::account::errors::Error::AssertionVerification(
                AssertionError::Validation(AssertionValidationError::CounterTooLow { .. })
            ))
        )
    }

    #[tokio::test]
    #[rstest]
    async fn valid_instruction_challenge_should_verify(
        #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
    ) {
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(attestation_type).await;

        let challenge_request = hw_privkey
            .sign_instruction_challenge::<CheckPin>(
                cert.dangerous_parse_unverified().unwrap().1.wallet_id,
                1,
                cert.clone(),
            )
            .await;

        let challenge = account_server
            .instruction_challenge(challenge_request, &EpochGenerator, &user_state)
            .await
            .unwrap();

        user_state.repositories.challenge = Some(challenge.clone());

        let tx = user_state.repositories.begin_transaction().await.unwrap();
        let wallet_user = user_state
            .repositories
            .find_wallet_user_by_wallet_id(&tx, "0")
            .await
            .unwrap();
        tx.commit().await.unwrap();

        if let WalletUserQueryResult::Found(user) = wallet_user {
            let instruction = hw_privkey
                .sign_instruction(CheckPin, challenge, 44, &setup.pin_privkey, cert)
                .await;
            let _ = account_server
                .verify_instruction(instruction, &user, &EpochGenerator, &user_state.wallet_user_hsm)
                .await
                .expect("instruction should be valid");
        } else {
            panic!("user should be found")
        }
    }

    #[tokio::test]
    #[rstest]
    async fn wrong_instruction_challenge_should_not_verify(
        #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
    ) {
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(attestation_type).await;

        let challenge_request = hw_privkey
            .sign_instruction_challenge::<CheckPin>(
                cert.dangerous_parse_unverified().unwrap().1.wallet_id,
                1,
                cert.clone(),
            )
            .await;

        let challenge = account_server
            .instruction_challenge(challenge_request, &EpochGenerator, &user_state)
            .await
            .unwrap();

        user_state.repositories.challenge = Some(utils::random_bytes(32));

        let tx = user_state.repositories.begin_transaction().await.unwrap();
        let wallet_user = user_state
            .repositories
            .find_wallet_user_by_wallet_id(&tx, "0")
            .await
            .unwrap();
        tx.commit().await.unwrap();

        if let WalletUserQueryResult::Found(user) = wallet_user {
            let instruction = hw_privkey
                .sign_instruction(CheckPin, challenge, 44, &setup.pin_privkey, cert)
                .await;
            let error = account_server
                .verify_instruction(instruction, &user, &EpochGenerator, &user_state.wallet_user_hsm)
                .await
                .expect_err("instruction should not be valid");

            match attestation_type {
                AttestationType::Apple => {
                    assert_matches!(
                        error,
                        InstructionValidationError::VerificationFailed(AccountError::AssertionVerification(
                            AssertionError::Validation(AssertionValidationError::ChallengeMismatch)
                        ))
                    );
                }
                AttestationType::Google => {
                    assert_matches!(
                        error,
                        InstructionValidationError::VerificationFailed(
                            wallet_common::account::errors::Error::ChallengeMismatch
                        )
                    );
                }
            };
        } else {
            panic!("user should be found")
        }
    }

    struct ExpiredAtEpochGeneretor;

    impl Generator<DateTime<Utc>> for ExpiredAtEpochGeneretor {
        fn generate(&self) -> DateTime<Utc> {
            Utc.timestamp_nanos(-1)
        }
    }

    #[tokio::test]
    #[rstest]
    async fn expired_instruction_challenge_should_not_verify(
        #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
    ) {
        let (setup, account_server, hw_privkey, cert, user_state) = setup_and_do_registration(attestation_type).await;

        let challenge_request = hw_privkey
            .sign_instruction_challenge::<CheckPin>(
                cert.dangerous_parse_unverified().unwrap().1.wallet_id,
                1,
                cert.clone(),
            )
            .await;

        let challenge = account_server
            .instruction_challenge(challenge_request, &EpochGenerator, &user_state)
            .await
            .unwrap();

        let tx = user_state.repositories.begin_transaction().await.unwrap();
        let wallet_user = user_state
            .repositories
            .find_wallet_user_by_wallet_id(&tx, "0")
            .await
            .unwrap();

        if let WalletUserQueryResult::Found(mut user) = wallet_user {
            user.instruction_challenge = Some(InstructionChallenge {
                bytes: challenge.clone(),
                expiration_date_time: ExpiredAtEpochGeneretor.generate(),
            });

            let instruction = hw_privkey
                .sign_instruction(CheckPin, challenge, 44, &setup.pin_privkey, cert)
                .await;
            let error = account_server
                .verify_instruction(instruction, &user, &EpochGenerator, &user_state.wallet_user_hsm)
                .await
                .expect_err("instruction should not be valid");

            assert_matches!(error, InstructionValidationError::ChallengeTimeout);
        } else {
            panic!("user should be found")
        }
    }

    #[tokio::test]
    #[rstest]
    async fn test_check_pin(
        #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
    ) {
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(attestation_type).await;
        user_state.repositories.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let challenge_error =
            do_instruction_challenge::<CheckPin>(&account_server, &hw_privkey, cert.clone(), 9, &user_state)
                .await
                .expect_err("should return instruction sequence number mismatch error");

        assert_matches!(
            challenge_error,
            ChallengeError::Validation(wallet_common::account::errors::Error::SequenceNumberMismatch)
        );

        let instruction_result = do_check_pin(
            &account_server,
            &setup.pin_privkey,
            &hw_privkey,
            cert,
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await
        .expect("should return unit instruction result");

        instruction_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result");
    }

    #[tokio::test]
    async fn test_change_pin_start_commit() {
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(AttestationType::Google).await;
        user_state.repositories.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let (new_pin_privkey, _new_pin_pubkey, encrypted_new_pin_pubkey, new_cert) = do_pin_change_start(
            &account_server,
            &setup,
            &hw_privkey,
            cert.clone(),
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await;

        verify_wallet_certificate(
            &new_cert,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
            &user_state,
        )
        .await
        .expect_err("verifying with the old pin_pubkey should fail");

        user_state.repositories.encrypted_pin_pubkey = encrypted_new_pin_pubkey.clone();

        verify_wallet_certificate(
            &new_cert,
            &EcdsaDecodingKey::from(&setup.signing_pubkey),
            wallet_certificate::mock::PIN_PUBLIC_DISCLOSURE_PROTECTION_KEY_IDENTIFIER,
            wallet_certificate::mock::ENCRYPTION_KEY_IDENTIFIER,
            |wallet_user| wallet_user.encrypted_pin_pubkey.clone(),
            &user_state,
        )
        .await
        .expect("verifying with the new pin_pubkey should succeed");

        let challenge = do_instruction_challenge::<ChangePinCommit>(
            &account_server,
            &hw_privkey,
            new_cert.clone(),
            45,
            &user_state,
        )
        .await
        .unwrap();

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
            ..user_state.repositories.clone()
        };

        account_server
            .handle_instruction(
                hw_privkey
                    .sign_instruction(
                        ChangePinCommit {},
                        challenge.clone(),
                        46,
                        &setup.pin_privkey,
                        cert.clone(),
                    )
                    .await,
                &instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
            )
            .await
            .expect_err("should fail for old pin");

        user_state.repositories = WalletUserTestRepo {
            encrypted_pin_pubkey: encrypted_new_pin_pubkey.clone(),
            previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
            challenge: Some(challenge.clone()),
            ..user_state.repositories.clone()
        };
        let instruction_result = account_server
            .handle_instruction(
                hw_privkey
                    .sign_instruction(
                        ChangePinCommit {},
                        challenge.clone(),
                        46,
                        &new_pin_privkey,
                        new_cert.clone(),
                    )
                    .await,
                &instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
            )
            .await
            .expect("should return instruction result");

        instruction_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result");

        user_state.repositories = WalletUserTestRepo {
            encrypted_pin_pubkey: encrypted_new_pin_pubkey.clone(),
            previous_encrypted_pin_pubkey: None,
            challenge: Some(challenge.clone()),
            ..user_state.repositories.clone()
        };
        account_server
            .handle_instruction(
                hw_privkey
                    .sign_instruction(ChangePinCommit {}, challenge, 46, &new_pin_privkey, new_cert.clone())
                    .await,
                &instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
            )
            .await
            .expect("committing double should succeed");

        do_check_pin(
            &account_server,
            &new_pin_privkey,
            &hw_privkey,
            new_cert,
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await
        .expect("should be able to send CheckPin instruction with the new certificate");
    }

    #[tokio::test]
    async fn test_change_pin_start_invalid_pop() {
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(AttestationType::Google).await;
        user_state.repositories.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let new_pin_privkey = SigningKey::random(&mut OsRng);
        let new_pin_pubkey = *new_pin_privkey.verifying_key();

        let challenge =
            do_instruction_challenge::<ChangePinStart>(&account_server, &hw_privkey, cert.clone(), 43, &user_state)
                .await
                .unwrap();

        let pop_pin_pubkey = new_pin_privkey
            .try_sign(utils::random_bytes(32).as_slice())
            .await
            .unwrap();

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            instruction_sequence_number: 2,
            ..user_state.repositories
        };

        let error = account_server
            .handle_change_pin_start_instruction(
                hw_privkey
                    .sign_instruction(
                        ChangePinStart {
                            pin_pubkey: new_pin_pubkey.into(),
                            pop_pin_pubkey: pop_pin_pubkey.into(),
                        },
                        challenge,
                        44,
                        &setup.pin_privkey,
                        cert.clone(),
                    )
                    .await,
                (&instruction_result_signing_key, &setup.signing_key),
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
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
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(AttestationType::Google).await;
        user_state.repositories.instruction_sequence_number = 42;

        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let (new_pin_privkey, _new_pin_pubkey, _encrypted_new_pin_pubkey, new_cert) = do_pin_change_start(
            &account_server,
            &setup,
            &hw_privkey,
            cert.clone(),
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await;

        let challenge =
            do_instruction_challenge::<ChangePinRollback>(&account_server, &hw_privkey, cert.clone(), 45, &user_state)
                .await
                .unwrap();

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
            ..user_state.repositories.clone()
        };
        account_server
            .handle_change_pin_rollback_instruction(
                hw_privkey
                    .sign_instruction(
                        ChangePinRollback {},
                        challenge.clone(),
                        46,
                        &new_pin_privkey,
                        new_cert.clone(),
                    )
                    .await,
                &instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
            )
            .await
            .expect_err("should fail for new pin");

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            previous_encrypted_pin_pubkey: Some(setup.encrypted_pin_pubkey.clone()),
            ..user_state.repositories.clone()
        };
        account_server
            .handle_change_pin_rollback_instruction(
                hw_privkey
                    .sign_instruction(
                        ChangePinRollback {},
                        challenge.clone(),
                        46,
                        &setup.pin_privkey,
                        cert.clone(),
                    )
                    .await,
                &instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
            )
            .await
            .expect("should succeed for old pin");

        user_state.repositories = WalletUserTestRepo {
            challenge: Some(challenge.clone()),
            previous_encrypted_pin_pubkey: None,
            ..user_state.repositories.clone()
        };
        let instruction_result = account_server
            .handle_change_pin_rollback_instruction(
                hw_privkey
                    .sign_instruction(ChangePinRollback {}, challenge, 47, &setup.pin_privkey, cert.clone())
                    .await,
                &instruction_result_signing_key,
                &MockGenerators,
                &TimeoutPinPolicy,
                &user_state,
            )
            .await
            .expect("should return instruction result for old pin");

        instruction_result
            .parse_and_verify_with_sub(&instruction_result_signing_key.verifying_key().into())
            .expect("Could not parse and verify instruction result");

        do_check_pin(
            &account_server,
            &new_pin_privkey,
            &hw_privkey,
            new_cert,
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await
        .expect_err("should not be able to send CheckPin instruction with new certificate");

        do_check_pin(
            &account_server,
            &setup.pin_privkey,
            &hw_privkey,
            cert,
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await
        .expect("should be able to send CheckPin instruction with old certificate");
    }

    #[tokio::test]
    async fn test_change_pin_no_other_instructions_allowed() {
        let (setup, account_server, hw_privkey, cert, mut user_state) =
            setup_and_do_registration(AttestationType::Google).await;
        user_state.repositories.instruction_sequence_number = 42;
        let instruction_result_signing_key = SigningKey::random(&mut OsRng);

        let (_new_pin_privkey, _new_pin_pubkey, encrypted_new_pin_pubkey, _new_cert) = do_pin_change_start(
            &account_server,
            &setup,
            &hw_privkey,
            cert.clone(),
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await;

        user_state.repositories.previous_encrypted_pin_pubkey = Some(encrypted_new_pin_pubkey);
        let error = do_check_pin(
            &account_server,
            &setup.pin_privkey,
            &hw_privkey,
            cert,
            &instruction_result_signing_key,
            &mut user_state,
        )
        .await
        .expect_err("other instructions than change_pin_commit and change_pin_rollback are not allowed");
        assert_eq!(
            "instruction validation error: pin change is in progress",
            error.to_string()
        );
    }
}
