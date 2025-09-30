use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use futures::future;
use itertools::Itertools;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tracing::warn;
use uuid::Uuid;

use attestation_data::constants::PID_RECOVERY_CODE;
use crypto::keys::EcdsaKey;
use crypto::p256_der::DerSignature;
use hsm::model::encrypter::Encrypter;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::pop::JwtPopClaims;
use jwt::wua::WuaDisclosure;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::CancelTransfer;
use wallet_account::messages::instructions::ChangePinCommit;
use wallet_account::messages::instructions::ChangePinRollback;
use wallet_account::messages::instructions::ChangePinStart;
use wallet_account::messages::instructions::CheckPin;
use wallet_account::messages::instructions::CompleteTransfer;
use wallet_account::messages::instructions::ConfirmTransfer;
use wallet_account::messages::instructions::DiscloseRecoveryCode;
use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
use wallet_account::messages::instructions::DiscloseRecoveryCodeResult;
use wallet_account::messages::instructions::GetTransferStatus;
use wallet_account::messages::instructions::PerformIssuance;
use wallet_account::messages::instructions::PerformIssuanceResult;
use wallet_account::messages::instructions::PerformIssuanceWithWua;
use wallet_account::messages::instructions::PerformIssuanceWithWuaResult;
use wallet_account::messages::instructions::ReceiveWalletPayload;
use wallet_account::messages::instructions::ReceiveWalletPayloadResult;
use wallet_account::messages::instructions::SendWalletPayload;
use wallet_account::messages::instructions::Sign;
use wallet_account::messages::instructions::SignResult;
use wallet_account::messages::instructions::StartPinRecovery;
use wallet_account::messages::transfer::TransferSessionState;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::wallet_user::TransferSession;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;
use wscd::Poa;
use wscd::poa::POA_JWT_TYP;

use crate::account_server::InstructionError;
use crate::account_server::InstructionValidationError;
use crate::account_server::UserState;
use crate::wallet_certificate::PinKeyChecks;
use crate::wua_issuer::WuaIssuer;

pub trait ValidateInstruction {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_change_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)?;
        validate_no_pin_recovery_in_progress(wallet_user)?;
        Ok(())
    }
}

fn validate_no_pin_change_in_progress(wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
    if wallet_user.pin_change_in_progress() {
        return Err(InstructionValidationError::PinChangeInProgress);
    }

    Ok(())
}

fn validate_no_transfer_in_progress(wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
    if wallet_user.transfer_in_progress() {
        return Err(InstructionValidationError::TransferInProgress);
    }

    Ok(())
}

fn validate_transfer_in_progress(wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
    if !wallet_user.transfer_in_progress() {
        return Err(InstructionValidationError::NoTransferInProgress);
    }

    Ok(())
}

fn validate_no_pin_recovery_in_progress(wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
    if matches!(wallet_user.state, WalletUserState::RecoveringPin) {
        return Err(InstructionValidationError::PinRecoveryInProgress);
    }

    Ok(())
}

fn validate_transfer_instruction(wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
    if wallet_user.recovery_code.is_none() {
        return Err(InstructionValidationError::MissingRecoveryCode);
    };

    validate_no_pin_change_in_progress(wallet_user)?;
    validate_no_pin_recovery_in_progress(wallet_user)?;

    Ok(())
}

impl ValidateInstruction for ChangePinStart {}
impl ValidateInstruction for PerformIssuance {}
impl ValidateInstruction for PerformIssuanceWithWua {}
impl ValidateInstruction for DiscloseRecoveryCode {}

impl ValidateInstruction for Sign {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_change_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)?;
        validate_no_pin_recovery_in_progress(wallet_user)?;

        if self
            .messages_with_identifiers
            .iter()
            .any(|(msg, _)| is_poa_message(msg))
        {
            let user = &wallet_user.id;
            warn!("user {user} attempted to sign a PoA via the Sign instruction instead of ConstructPoa");
            return Err(InstructionValidationError::PoaMessage);
        }

        Ok(())
    }
}

impl ValidateInstruction for ChangePinCommit {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_recovery_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)
    }
}

impl ValidateInstruction for ChangePinRollback {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_recovery_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)
    }
}

impl ValidateInstruction for CheckPin {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_change_in_progress(wallet_user)?;
        validate_no_pin_recovery_in_progress(wallet_user)
    }
}

impl ValidateInstruction for StartPinRecovery {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_change_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)
    }
}

impl ValidateInstruction for ConfirmTransfer {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        if wallet_user.recovery_code.is_none() {
            return Err(InstructionValidationError::MissingRecoveryCode);
        };

        validate_no_pin_change_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)?;
        validate_no_pin_recovery_in_progress(wallet_user)?;

        Ok(())
    }
}

impl ValidateInstruction for CancelTransfer {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_transfer_instruction(wallet_user)
    }
}

impl ValidateInstruction for GetTransferStatus {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_transfer_instruction(wallet_user)
    }
}

impl ValidateInstruction for SendWalletPayload {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_transfer_instruction(wallet_user)?;
        validate_transfer_in_progress(wallet_user)
    }
}

impl ValidateInstruction for ReceiveWalletPayload {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_transfer_instruction(wallet_user)?;
        validate_transfer_in_progress(wallet_user)
    }
}

impl ValidateInstruction for CompleteTransfer {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_transfer_instruction(wallet_user)?;
        validate_transfer_in_progress(wallet_user)
    }
}

impl ValidateInstruction for DiscloseRecoveryCodePinRecovery {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        validate_no_pin_change_in_progress(wallet_user)?;
        validate_no_transfer_in_progress(wallet_user)
    }
}

pub struct PinCheckOptions {
    pub allow_for_blocked_users: bool,
    pub key_checks: PinKeyChecks,
}

impl Default for PinCheckOptions {
    fn default() -> Self {
        Self {
            allow_for_blocked_users: false,
            key_checks: PinKeyChecks::AllChecks,
        }
    }
}

pub trait PinChecks {
    fn pin_checks_options() -> PinCheckOptions {
        Default::default()
    }
}

impl PinChecks for ChangePinStart {}
impl PinChecks for PerformIssuance {}
impl PinChecks for PerformIssuanceWithWua {}
impl PinChecks for DiscloseRecoveryCode {}
impl PinChecks for DiscloseRecoveryCodePinRecovery {}
impl PinChecks for ChangePinCommit {}
impl PinChecks for ChangePinRollback {}
impl PinChecks for CheckPin {}
impl PinChecks for Sign {}
impl PinChecks for SendWalletPayload {}

impl PinChecks for StartPinRecovery {
    fn pin_checks_options() -> PinCheckOptions {
        PinCheckOptions {
            // Blocked users should be able to reset their PIN, so don't reject blocked users.
            allow_for_blocked_users: true,

            // This instruction is signed with the user's new PIN key, whose HMAC is not in the certificate.
            key_checks: PinKeyChecks::SkipCertificateMatching,
        }
    }
}

pub trait HandleInstruction {
    type Result: Serialize;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>;
}

impl HandleInstruction for CheckPin {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        _wallet_user: &WalletUser,
        _generators: &G,
        _user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<(), InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        Ok(())
    }
}

impl HandleInstruction for ChangePinCommit {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        user_state
            .repositories
            .commit_pin_change(&tx, &wallet_user.wallet_id)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

pub(super) async fn perform_issuance_with_wua<T, R, H>(
    instruction: PerformIssuance,
    user_state: &UserState<R, H, impl WuaIssuer>,
) -> Result<(PerformIssuanceWithWuaResult, Vec<WrappedKey>, (WrappedKey, String)), InstructionError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let (issuance_result, wua_disclosure, wrapped_keys, wua_key_and_id) =
        perform_issuance(instruction, true, user_state).await?;

    let issuance_result = PerformIssuanceWithWuaResult {
        issuance_result,
        wua_disclosure: wua_disclosure.unwrap(),
    };

    // unwrap: `perform_issuance()` included a WUA since we passed it `true` above.
    Ok((issuance_result, wrapped_keys, wua_key_and_id.unwrap()))
}

/// Helper for the [`PerformIssuance`] and [`PerformIssuanceWithWua`] instruction handlers.
pub async fn perform_issuance<T, R, H>(
    instruction: PerformIssuance,
    issue_wua: bool,
    user_state: &UserState<R, H, impl WuaIssuer>,
) -> Result<
    (
        PerformIssuanceResult,
        Option<WuaDisclosure>,
        Vec<WrappedKey>,
        Option<(WrappedKey, String)>,
    ),
    InstructionError,
>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let (key_ids, wrapped_keys): (Vec<_>, Vec<_>) = user_state
        .wallet_user_hsm
        .generate_wrapped_keys(&user_state.wrapping_key_identifier, instruction.key_count)
        .await?
        .into_inner()
        .into_iter()
        .multiunzip();

    // Instantiate some VecNonEmpty's that we need below. Safe because generate_wrapped_keys() returns VecNonEmpty.
    let key_ids: VecNonEmpty<_> = key_ids.try_into().unwrap();
    let attestation_keys = wrapped_keys
        .iter()
        .map(|wrapped_key| attestation_key(wrapped_key, user_state))
        .collect_vec()
        .try_into()
        .unwrap();

    // The JWT claims to be signed in the PoPs and the PoA.
    let claims = JwtPopClaims::new(instruction.nonce, NL_WALLET_CLIENT_ID.to_string(), instruction.aud);

    let (wua_key_and_id, wua_disclosure) = if issue_wua {
        let (key, key_id, wua_disclosure) = wua(&claims, user_state).await?;
        (Some((key, key_id)), Some(wua_disclosure))
    } else {
        (None, None)
    };

    let pops = issuance_pops(&attestation_keys, &claims).await?;

    let key_count_including_wua = if issue_wua {
        instruction.key_count.get() + 1
    } else {
        instruction.key_count.get()
    };
    let poa = if key_count_including_wua > 1 {
        let wua_attestation_key = wua_key_and_id.as_ref().map(|(key, _)| attestation_key(key, user_state));
        Some(
            // Unwrap is safe because we're operating on the output of `generate_wrapped_keys()`
            // which returns `VecNonEmpty`
            Poa::new(
                attestation_keys
                    .iter()
                    .chain(wua_attestation_key.as_ref())
                    .collect_vec()
                    .try_into()
                    .unwrap(),
                claims,
            )
            .await?,
        )
    } else {
        None
    };

    let issuance_result = PerformIssuanceResult {
        key_identifiers: key_ids,
        pops,
        poa,
    };

    Ok((issuance_result, wua_disclosure, wrapped_keys, wua_key_and_id))
}

async fn persist_issuance_keys<T, R, H>(
    wrapped_keys: Vec<WrappedKey>,
    key_ids: Vec<String>,
    wua_key_and_id: Option<(WrappedKey, String)>,
    wallet_user: &WalletUser,
    uuid_generator: &impl Generator<Uuid>,
    user_state: &UserState<R, H, impl WuaIssuer>,
) -> Result<(), InstructionError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    // Assemble the keys to be stored in the database
    let db_keys = wrapped_keys
        .into_iter()
        .zip(key_ids)
        .chain(wua_key_and_id.into_iter())
        .map(|(key, key_identifier)| WalletUserKey {
            wallet_user_key_id: uuid_generator.generate(),
            key_identifier,
            key,
        })
        .collect();

    // Save the keys in the database
    let tx = user_state.repositories.begin_transaction().await?;
    user_state
        .repositories
        .save_keys(
            &tx,
            WalletUserKeys {
                wallet_user_id: wallet_user.id,
                keys: db_keys,
            },
        )
        .await?;
    tx.commit().await?;

    Ok(())
}

async fn wua<T, R, H>(
    claims: &JwtPopClaims,
    user_state: &UserState<R, H, impl WuaIssuer>,
) -> Result<(WrappedKey, String, WuaDisclosure), InstructionError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let (wua_wrapped_key, wua_key_id, wua) = user_state
        .wua_issuer
        .issue_wua()
        .await
        .map_err(|e| InstructionError::WuaIssuance(Box::new(e)))?;

    let wua_disclosure = SignedJwt::sign(claims, &attestation_key(&wua_wrapped_key, user_state))
        .await
        .map_err(InstructionError::PopSigning)?
        .into();

    Ok((wua_wrapped_key, wua_key_id, WuaDisclosure::new(wua, wua_disclosure)))
}

async fn issuance_pops<H>(
    attestation_keys: &VecNonEmpty<HsmCredentialSigningKey<'_, H>>,
    claims: &JwtPopClaims,
) -> Result<VecNonEmpty<UnverifiedJwt<JwtPopClaims, HeaderWithJwk>>, InstructionError>
where
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let pops = future::try_join_all(attestation_keys.iter().map(|attestation_key| async {
        let jwt = SignedJwt::sign_with_jwk(claims, attestation_key)
            .await
            .map_err(InstructionError::PopSigning)?
            .into();

        Ok::<_, InstructionError>(jwt)
    }))
    .await?
    .try_into()
    .unwrap(); // Safe because we're iterating over attestation_keys which is VecNonEmpty

    Ok(pops)
}

fn attestation_key<'a, T, R, H>(
    wrapped_key: &'a WrappedKey,
    user_state: &'a UserState<R, H, impl WuaIssuer>,
) -> HsmCredentialSigningKey<'a, H>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    HsmCredentialSigningKey {
        hsm: &user_state.wallet_user_hsm,
        wrapped_key,
        wrapping_key_identifier: &user_state.wrapping_key_identifier,
    }
}

impl HandleInstruction for PerformIssuance {
    type Result = PerformIssuanceResult;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let (issuance_result, _, wrapped_keys, _) = perform_issuance(self, false, user_state).await?;

        persist_issuance_keys(
            wrapped_keys,
            issuance_result.key_identifiers.as_ref().to_vec(),
            None,
            wallet_user,
            generators,
            user_state,
        )
        .await?;

        Ok(issuance_result)
    }
}

impl HandleInstruction for PerformIssuanceWithWua {
    type Result = PerformIssuanceWithWuaResult;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let (issuance_with_wua_result, wrapped_keys, wua_key_and_id) =
            perform_issuance_with_wua(self.issuance_instruction, user_state).await?;

        persist_issuance_keys(
            wrapped_keys,
            issuance_with_wua_result
                .issuance_result
                .key_identifiers
                .as_ref()
                .to_vec(),
            Some(wua_key_and_id),
            wallet_user,
            generators,
            user_state,
        )
        .await?;

        Ok(issuance_with_wua_result)
    }
}

impl HandleInstruction for Sign {
    type Result = SignResult;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let (data, identifiers): (Vec<_>, Vec<_>) = self.messages_with_identifiers.into_iter().unzip();

        let tx = user_state.repositories.begin_transaction().await?;
        let found_keys = user_state
            .repositories
            .find_keys_by_identifiers(
                &tx,
                wallet_user.id,
                &identifiers.clone().into_iter().flatten().collect::<Vec<_>>(),
            )
            .await?;
        tx.commit().await?;

        let signatures = future::try_join_all(identifiers.iter().zip(data).map(|(identifiers, data)| async {
            let data = Arc::new(data);
            future::try_join_all(identifiers.iter().map(|identifier| async {
                let wrapped_key = found_keys.get(identifier).cloned().unwrap();
                user_state
                    .wallet_user_hsm
                    .sign_wrapped(&user_state.wrapping_key_identifier, wrapped_key, Arc::clone(&data))
                    .await
                    .map(DerSignature::from)
            }))
            .await
        }))
        .await?;

        // A PoA should be generated only if the unique keys, i.e. the keys referenced in the instruction
        // after deduplication, count two or more.
        if found_keys.len() < 2 {
            Ok(SignResult { signatures, poa: None })
        } else {
            // We have to feed a Vec of references to `Poa::new()`, so we need to iterate twice to construct that.
            let keys = found_keys
                .values()
                .map(|wrapped_key| attestation_key(wrapped_key, user_state))
                .collect_vec();
            let keys = keys.iter().collect_vec().try_into().unwrap(); // We know there are at least two keys
            let claims = JwtPopClaims::new(self.poa_nonce, NL_WALLET_CLIENT_ID.to_string(), self.poa_aud);
            let poa = Poa::new(keys, claims).await?;

            Ok(SignResult {
                signatures,
                poa: Some(poa),
            })
        }
    }
}

impl HandleInstruction for DiscloseRecoveryCode {
    type Result = DiscloseRecoveryCodeResult;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let verified_sd_jwt = self
            .recovery_code_disclosure
            .into_verified_against_trust_anchors(&user_state.pid_issuer_trust_anchors, &TimeGenerator)?;

        let recovery_code = match verified_sd_jwt
            .as_ref()
            .to_disclosed_object()?
            .remove(PID_RECOVERY_CODE)
        {
            Some(Value::String(recovery_code)) => recovery_code,
            _ => return Err(InstructionError::MissingRecoveryCode),
        };

        let tx = user_state.repositories.begin_transaction().await?;

        // Check here as well to prevent failure for retried wallet request
        match wallet_user.recovery_code.as_ref() {
            None => {
                user_state
                    .repositories
                    .store_recovery_code(&tx, &wallet_user.wallet_id, recovery_code.clone())
                    .await?;
            }
            // This is a retried request
            Some(stored) if recovery_code.as_str() == stored => {}
            _ => return Err(InstructionError::InvalidRecoveryCode),
        }

        let transfer_session_id = if let Some(transfer_session_id) = user_state
            .repositories
            .find_transfer_session_id_by_destination_wallet_user_id(&tx, wallet_user.id)
            .await?
        {
            Some(transfer_session_id)
        } else if user_state
            .repositories
            .has_multiple_active_accounts_by_recovery_code(&tx, &recovery_code)
            .await?
        {
            let transfer_session_id = Uuid::new_v4();
            user_state
                .repositories
                .create_transfer_session(
                    &tx,
                    wallet_user.id,
                    transfer_session_id,
                    self.app_version,
                    generators.generate(),
                )
                .await?;
            Some(transfer_session_id)
        } else {
            None
        };

        tx.commit().await?;

        Ok(DiscloseRecoveryCodeResult { transfer_session_id })
    }
}

impl HandleInstruction for DiscloseRecoveryCodePinRecovery {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let verified_sd_jwt = self
            .recovery_code_disclosure
            .into_verified_against_trust_anchors(&user_state.pid_issuer_trust_anchors, &TimeGenerator)?;

        let recovery_code = match verified_sd_jwt
            .as_ref()
            .to_disclosed_object()?
            .remove(PID_RECOVERY_CODE)
        {
            Some(Value::String(recovery_code)) => recovery_code,
            _ => return Err(InstructionError::MissingRecoveryCode),
        };

        // Idempotency check
        if wallet_user.state == WalletUserState::Active && wallet_user.recovery_code.as_ref() == Some(&recovery_code) {
            return Ok(());
        }

        // The PID that was just received has to belong to the same person as the wallet,
        // which is the case only if they have the same recovery code.
        if wallet_user.recovery_code != Some(recovery_code) {
            return Err(InstructionError::InvalidRecoveryCode);
        }

        let tx = user_state.repositories.begin_transaction().await?;

        user_state.repositories.recover_pin(&tx, &wallet_user.wallet_id).await?;

        tx.commit().await?;

        Ok(())
    }
}

impl HandleInstruction for ConfirmTransfer {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        let transfer_session = check_transfer_instruction_prerequisites(
            &tx,
            &user_state.repositories,
            self.transfer_session_id,
            wallet_user,
        )
        .await?;

        if transfer_session.destination_wallet_app_version < self.app_version {
            return Err(InstructionError::AppVersionMismatch {
                source_version: self.app_version,
                destination_version: transfer_session.destination_wallet_app_version,
            });
        }

        user_state
            .repositories
            .confirm_wallet_transfer(
                &tx,
                wallet_user.id,
                transfer_session.destination_wallet_user_id,
                transfer_session.transfer_session_id,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl HandleInstruction for CancelTransfer {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        let transfer_session = check_transfer_instruction_prerequisites(
            &tx,
            &user_state.repositories,
            self.transfer_session_id,
            wallet_user,
        )
        .await?;

        if transfer_session.state == TransferSessionState::Success {
            return Err(InstructionError::AccountTransferIllegalState);
        }

        user_state
            .repositories
            .cancel_wallet_transfer(
                &tx,
                transfer_session.transfer_session_id,
                transfer_session.source_wallet_user_id,
                transfer_session.destination_wallet_user_id,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl HandleInstruction for GetTransferStatus {
    type Result = TransferSessionState;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        // TODO: PVW-4959: The database query used for checking the prerequisites is quite heavy
        // (since it retrieves the encrypted wallet data) and could be optimized if necessary.
        let transfer_session = check_transfer_instruction_prerequisites(
            &tx,
            &user_state.repositories,
            self.transfer_session_id,
            wallet_user,
        )
        .await?;

        tx.commit().await?;

        Ok(transfer_session.state)
    }
}

impl HandleInstruction for SendWalletPayload {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<(), InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        let transfer_session = check_transfer_instruction_prerequisites(
            &tx,
            &user_state.repositories,
            self.transfer_session_id,
            wallet_user,
        )
        .await?;

        // Make this instruction idempotent by checking if the payload has already been sent
        if transfer_session.encrypted_wallet_data.is_some()
            && transfer_session.state == TransferSessionState::ReadyForDownload
        {
            return Ok(());
        }

        if transfer_session.state != TransferSessionState::ReadyForTransfer {
            return Err(InstructionError::AccountTransferIllegalState);
        }

        user_state
            .repositories
            .store_wallet_transfer_data(&tx, transfer_session.transfer_session_id, self.payload)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl HandleInstruction for ReceiveWalletPayload {
    type Result = ReceiveWalletPayloadResult;

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<ReceiveWalletPayloadResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        let transfer_session = check_transfer_instruction_prerequisites(
            &tx,
            &user_state.repositories,
            self.transfer_session_id,
            wallet_user,
        )
        .await?;

        let TransferSession {
            state: TransferSessionState::ReadyForDownload,
            encrypted_wallet_data: Some(payload),
            ..
        } = transfer_session
        else {
            return Err(InstructionError::AccountTransferIllegalState);
        };

        tx.commit().await?;

        Ok(ReceiveWalletPayloadResult { payload })
    }
}

impl HandleInstruction for CompleteTransfer {
    type Result = ();

    async fn handle<T, R, H, G>(
        self,
        wallet_user: &WalletUser,
        _generators: &G,
        user_state: &UserState<R, H, impl WuaIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
        G: Generator<Uuid> + Generator<DateTime<Utc>>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        let transfer_session = check_transfer_instruction_prerequisites(
            &tx,
            &user_state.repositories,
            self.transfer_session_id,
            wallet_user,
        )
        .await?;

        match transfer_session.state {
            TransferSessionState::Canceled => {
                return Err(InstructionError::AccountTransferCanceled);
            }
            TransferSessionState::Success => {
                return Ok(());
            }
            TransferSessionState::ReadyForDownload => {}
            _ => return Err(InstructionError::AccountTransferIllegalState),
        }

        let Some(source_wallet_user_id) = transfer_session.source_wallet_user_id else {
            return Err(InstructionError::AccountTransferIllegalState);
        };

        user_state
            .repositories
            .complete_wallet_transfer(
                &tx,
                transfer_session.transfer_session_id,
                source_wallet_user_id,
                transfer_session.destination_wallet_user_id,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

async fn check_transfer_instruction_prerequisites<T, R>(
    tx: &T,
    repositories: &R,
    transfer_session_id: Uuid,
    wallet_user: &WalletUser,
) -> Result<TransferSession, InstructionError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let Some(transfer_session) = repositories
        .find_transfer_session_by_transfer_session_id(tx, transfer_session_id)
        .await?
    else {
        return Err(InstructionError::NoAccountTransferInProgress);
    };

    // recovery code of wallet_user (source) should match recovery code of transfer_session (destination)
    if wallet_user
        .recovery_code
        .as_ref()
        .expect("instruction validation fails if there is no recovery code")
        != &transfer_session.destination_wallet_recovery_code
    {
        return Err(InstructionError::AccountTransferWalletsMismatch);
    }

    Ok(transfer_session)
}

struct HsmCredentialSigningKey<'a, H> {
    hsm: &'a H,
    wrapped_key: &'a WrappedKey,
    wrapping_key_identifier: &'a str,
}

impl<H> PartialEq for HsmCredentialSigningKey<'_, H> {
    fn eq(&self, other: &Self) -> bool {
        self.wrapped_key == other.wrapped_key
    }
}

impl<H> Eq for HsmCredentialSigningKey<'_, H> {}

impl<H> Hash for HsmCredentialSigningKey<'_, H> {
    fn hash<HASH: Hasher>(&self, state: &mut HASH) {
        self.wrapped_key.hash(state);
    }
}

impl<H> EcdsaKey for HsmCredentialSigningKey<'_, H>
where
    H: WalletUserHsm<Error = HsmError>,
{
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.wrapped_key.public_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.hsm
            .sign_wrapped(
                self.wrapping_key_identifier,
                self.wrapped_key.clone(),
                Arc::new(msg.to_vec()),
            )
            .await
    }
}

/// Test if the `message` is the payload of a PoA JWT, i.e. `header.body` where `header` is the base64url encoding
/// of a JSON object containing `"typ":"poa+jwt"`.
///
/// This function must be used by the signing instructions to prevent a user from signing a PoA without
/// using the intended `ConstructPoa` instruction for that. It should therefore be resistant to "tricks"
/// such as include whitespace in the JSON or mess with the casing of the casing of the value of the `typ` field.
///
/// Since this function is executed for every single message that the WP signs for a wallet, before JSON deserialization
/// of the header we do a number of cheaper checks to return early if the passed message is clearly not a PoA JWT
/// payload.
fn is_poa_message(message: &[u8]) -> bool {
    // A JWT payload contains a single dot which is not located at the beginning of the string.
    let predicate = |&x| x == b'.';
    let dot_pos = match message.iter().position(predicate) {
        None | Some(0) => return false, // a string without dot, or whose first character is a dot is not a JWT payload
        Some(dot_pos) => {
            if message.iter().skip(dot_pos + 1).any(predicate) {
                return false; // a string with more than one dot is not a JWT payload
            }

            dot_pos
        }
    };

    let first_part = &message[0..dot_pos];
    let Ok(decoded) = BASE64_URL_SAFE_NO_PAD.decode(first_part) else {
        return false; // not a PoA in case of Base64url decoding errors
    };

    // We use a custom `Header` struct here as opposed to `jsonwebtoken::Header` so as to only deserialize
    // the `typ` field and not any of the other ones in `jsonwebtoken::Header`.
    #[derive(Deserialize)]
    struct Header {
        typ: String,
    }

    let Ok(header) = serde_json::from_slice::<Header>(&decoded) else {
        return false; // not a PoA in case of JSON deserialization errors
    };

    header.typ.to_ascii_lowercase() == POA_JWT_TYP
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    use assert_matches::assert_matches;
    use base64::prelude::*;
    use itertools::Itertools;
    use mockall::predicate;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::signature::Verifier;
    use rand::rngs::OsRng;
    use rstest::rstest;
    use semver::Version;
    use uuid::Uuid;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::constants::PID_RECOVERY_CODE;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::generate::Ca;
    use crypto::utils::random_bytes;
    use crypto::utils::random_string;
    use hsm::model::wrapped_key::WrappedKey;
    use jwt::Algorithm;
    use jwt::UnverifiedJwt;
    use jwt::Validation;
    use jwt::headers::HeaderWithJwk;
    use jwt::jwk::jwk_to_p256;
    use jwt::pop::JwtPopClaims;
    use jwt::wua::WuaDisclosure;
    use sd_jwt::sd_jwt::SdJwt;
    use sd_jwt::sd_jwt::UnverifiedSdJwt;
    use wallet_account::NL_WALLET_CLIENT_ID;
    use wallet_account::messages::instructions::CancelTransfer;
    use wallet_account::messages::instructions::ChangePinCommit;
    use wallet_account::messages::instructions::ChangePinRollback;
    use wallet_account::messages::instructions::ChangePinStart;
    use wallet_account::messages::instructions::CheckPin;
    use wallet_account::messages::instructions::CompleteTransfer;
    use wallet_account::messages::instructions::ConfirmTransfer;
    use wallet_account::messages::instructions::DiscloseRecoveryCode;
    use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
    use wallet_account::messages::instructions::GetTransferStatus;
    use wallet_account::messages::instructions::PerformIssuance;
    use wallet_account::messages::instructions::PerformIssuanceWithWua;
    use wallet_account::messages::instructions::ReceiveWalletPayload;
    use wallet_account::messages::instructions::SendWalletPayload;
    use wallet_account::messages::instructions::Sign;
    use wallet_account::messages::instructions::StartPinRecovery;
    use wallet_account::messages::transfer::TransferSessionState;
    use wallet_provider_domain::generator::mock::MockGenerators;
    use wallet_provider_domain::model::wallet_user;
    use wallet_provider_domain::model::wallet_user::TransferSession;
    use wallet_provider_domain::model::wallet_user::WalletUserState;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;
    use wscd::Poa;

    use crate::account_server::InstructionValidationError;
    use crate::account_server::mock;
    use crate::instructions::HandleInstruction;
    use crate::instructions::InstructionError;
    use crate::instructions::ValidateInstruction;
    use crate::instructions::is_poa_message;
    use crate::wallet_certificate::mock::setup_hsm;

    #[tokio::test]
    async fn should_handle_checkpin() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my_wrapping_key_identifier";

        let instruction = CheckPin {};
        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    MockTransactionalWalletUserRepository::new(),
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_sign() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let poa_nonce = Some("nonce".to_string());
        let poa_aud = "aud".to_string();

        let random_msg_1 = random_bytes(32);
        let random_msg_2 = random_bytes(32);
        let instruction = Sign {
            messages_with_identifiers: vec![
                (random_msg_1.clone(), vec!["key1".to_string(), "key2".to_string()]),
                (random_msg_2.clone(), vec!["key2".to_string()]),
            ],
            poa_nonce: poa_nonce.clone(),
            poa_aud: poa_aud.clone(),
        };
        let signing_key_1 = SigningKey::random(&mut OsRng);
        let signing_key_2 = SigningKey::random(&mut OsRng);
        let signing_key_1_bytes = signing_key_1.to_bytes().to_vec();
        let signing_key_2_bytes = signing_key_2.to_bytes().to_vec();
        let signing_key_1_public = *signing_key_1.verifying_key();
        let signing_key_2_public = *signing_key_2.verifying_key();

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();

        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));

        wallet_user_repo
            .expect_find_keys_by_identifiers()
            .withf(|_, _, key_identifiers| key_identifiers.contains(&"key1".to_string()))
            .return_once(move |_, _, _| {
                Ok(HashMap::from([
                    (
                        "key1".to_string(),
                        WrappedKey::new(signing_key_1_bytes, signing_key_1_public),
                    ),
                    (
                        "key2".to_string(),
                        WrappedKey::new(signing_key_2_bytes, signing_key_2_public),
                    ),
                ]))
            });

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();

        signing_key_1
            .verifying_key()
            .verify(&random_msg_1, result.signatures[0][0].as_inner())
            .unwrap();
        signing_key_2
            .verifying_key()
            .verify(&random_msg_1, result.signatures[0][1].as_inner())
            .unwrap();
        signing_key_2
            .verifying_key()
            .verify(&random_msg_2, result.signatures[1][0].as_inner())
            .unwrap();

        result
            .poa
            .unwrap()
            .verify(
                &[signing_key_1_public, signing_key_2_public],
                &poa_aud,
                &[NL_WALLET_CLIENT_ID.to_string()],
                poa_nonce.as_ref().unwrap(),
            )
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_disclose_recovery_code() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key =
            generate_issuer_mock_with_registration(&issuer_ca, IssuerRegistration::new_mock().into()).unwrap();
        let holder_key = SigningKey::random(&mut OsRng);
        let sd_jwt = SdJwt::example_pid_sd_jwt(&issuer_key, holder_key.verifying_key());

        let recovery_code_disclosure = sd_jwt
            .into_presentation_builder()
            .disclose(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .finish()
            .into();

        let instruction = DiscloseRecoveryCode {
            recovery_code_disclosure,
            app_version: Version::parse("1.0.0").unwrap(),
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_store_recovery_code()
            .returning(|_, _, _| Ok(()));
        wallet_user_repo
            .expect_find_transfer_session_id_by_destination_wallet_user_id()
            .returning(|_, _| Ok(None));
        wallet_user_repo
            .expect_has_multiple_active_accounts_by_recovery_code()
            .returning(|_, _| Ok(false));

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![issuer_ca.as_borrowing_trust_anchor().to_owned_trust_anchor()],
                ),
            )
            .await
            .unwrap();

        assert!(result.transfer_session_id.is_none());
    }

    #[tokio::test]
    async fn should_handle_disclose_recovery_code_with_multiple_accounts() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key =
            generate_issuer_mock_with_registration(&issuer_ca, IssuerRegistration::new_mock().into()).unwrap();
        let holder_key = SigningKey::random(&mut OsRng);
        let sd_jwt = SdJwt::example_pid_sd_jwt(&issuer_key, holder_key.verifying_key());

        let recovery_code_disclosure = sd_jwt
            .into_presentation_builder()
            .disclose(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .finish()
            .into();

        let instruction = DiscloseRecoveryCode {
            recovery_code_disclosure,
            app_version: Version::parse("1.0.0").unwrap(),
        };

        let transfer_session_id: Arc<Mutex<Option<Uuid>>> = Arc::new(Mutex::new(None));
        let transfer_session_id_clone = Arc::clone(&transfer_session_id);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_store_recovery_code()
            .returning(|_, _, _| Ok(()));
        wallet_user_repo
            .expect_find_transfer_session_id_by_destination_wallet_user_id()
            .returning(|_, _| Ok(None));
        wallet_user_repo
            .expect_has_multiple_active_accounts_by_recovery_code()
            .returning(|_, _| Ok(true));
        wallet_user_repo
            .expect_create_transfer_session()
            .withf(move |_, _, session_id, _, _| {
                transfer_session_id_clone.lock().unwrap().replace(*session_id);
                true
            })
            .returning(|_, _, _, _, _| Ok(()));

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![issuer_ca.as_borrowing_trust_anchor().to_owned_trust_anchor()],
                ),
            )
            .await
            .unwrap();

        assert_eq!(
            transfer_session_id.lock().unwrap().unwrap(),
            result.transfer_session_id.unwrap()
        );
    }

    #[tokio::test]
    async fn should_handle_disclose_recovery_code_with_multiple_accounts_idempotency() {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key =
            generate_issuer_mock_with_registration(&issuer_ca, IssuerRegistration::new_mock().into()).unwrap();
        let holder_key = SigningKey::random(&mut OsRng);
        let sd_jwt = SdJwt::example_pid_sd_jwt(&issuer_key, holder_key.verifying_key());

        let recovery_code_disclosure: UnverifiedSdJwt = sd_jwt
            .into_presentation_builder()
            .disclose(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .finish()
            .into();

        let instruction = DiscloseRecoveryCode {
            recovery_code_disclosure: recovery_code_disclosure.clone(),
            app_version: Version::parse("1.0.0").unwrap(),
        };

        let transfer_session_id: Arc<Mutex<Option<Uuid>>> = Arc::new(Mutex::new(None));
        let transfer_session_id_clone = Arc::clone(&transfer_session_id);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_store_recovery_code()
            .returning(|_, _, _| Ok(()));
        wallet_user_repo
            .expect_find_transfer_session_id_by_destination_wallet_user_id()
            .returning(|_, _| Ok(None));
        wallet_user_repo
            .expect_has_multiple_active_accounts_by_recovery_code()
            .returning(|_, _| Ok(true));
        wallet_user_repo
            .expect_create_transfer_session()
            .withf(move |_, _, session_id, _, _| {
                transfer_session_id_clone.lock().unwrap().replace(*session_id);
                true
            })
            .returning(|_, _, _, _, _| Ok(()));

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![issuer_ca.as_borrowing_trust_anchor().to_owned_trust_anchor()],
                ),
            )
            .await
            .unwrap();

        assert_eq!(
            transfer_session_id.lock().unwrap().unwrap(),
            result.transfer_session_id.unwrap()
        );

        let instruction = DiscloseRecoveryCode {
            recovery_code_disclosure,
            app_version: Version::parse("1.0.0").unwrap(),
        };

        let transfer_session_id_clone = Arc::clone(&transfer_session_id);

        wallet_user.recovery_code = Some("885ed8a2-f07a-4f77-a8df-2e166f5ebd36".to_string());
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_has_multiple_active_accounts_by_recovery_code()
            .returning(|_, _| Ok(true));
        wallet_user_repo
            .expect_find_transfer_session_id_by_destination_wallet_user_id()
            .returning(move |_, _| Ok(Some(transfer_session_id_clone.lock().unwrap().unwrap())));
        wallet_user_repo.expect_create_transfer_session().never();

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![issuer_ca.as_borrowing_trust_anchor().to_owned_trust_anchor()],
                ),
            )
            .await
            .unwrap();

        assert_eq!(
            transfer_session_id.lock().unwrap().unwrap(),
            result.transfer_session_id.unwrap()
        );
    }

    #[tokio::test]
    async fn should_handle_disclose_recovery_code_for_pin_recovery() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key =
            generate_issuer_mock_with_registration(&issuer_ca, IssuerRegistration::new_mock().into()).unwrap();
        let holder_key = SigningKey::random(&mut OsRng);
        let sd_jwt = SdJwt::example_pid_sd_jwt(&issuer_key, holder_key.verifying_key());

        let recovery_code_disclosure = sd_jwt
            .into_presentation_builder()
            .disclose(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .finish()
            .into();

        let instruction = DiscloseRecoveryCodePinRecovery {
            recovery_code_disclosure,
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo.expect_recover_pin().returning(|_, _| Ok(()));

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![issuer_ca.as_borrowing_trust_anchor().to_owned_trust_anchor()],
                ),
            )
            .await;

        assert_matches!(result, Ok(_));
    }

    #[tokio::test]
    async fn should_handle_disclose_recovery_code_for_pin_recovery_idempotency() {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key =
            generate_issuer_mock_with_registration(&issuer_ca, IssuerRegistration::new_mock().into()).unwrap();
        let holder_key = SigningKey::random(&mut OsRng);
        let sd_jwt = SdJwt::example_pid_sd_jwt(&issuer_key, holder_key.verifying_key());

        let recovery_code_disclosure = sd_jwt
            .into_presentation_builder()
            .disclose(
                &vec![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .finish()
            .into();

        let instruction = DiscloseRecoveryCodePinRecovery {
            recovery_code_disclosure,
        };

        wallet_user.state = WalletUserState::Active;
        wallet_user.recovery_code = Some("885ed8a2-f07a-4f77-a8df-2e166f5ebd36".to_string());
        let wallet_user_repo = MockTransactionalWalletUserRepository::new();

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![issuer_ca.as_borrowing_trust_anchor().to_owned_trust_anchor()],
                ),
            )
            .await;

        assert_matches!(result, Ok(_));
    }

    fn mock_jwt_payload(header: &str) -> Vec<u8> {
        (BASE64_URL_SAFE_NO_PAD.encode(header) + "." + &BASE64_URL_SAFE_NO_PAD.encode("{}")).into_bytes()
    }

    #[rstest]
    #[case(mock_jwt_payload(r#"{"typ":"poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#"{"typ":"poa+JWT"}"#), true)] // accept any casing of the field value
    #[case(mock_jwt_payload(r#"{"typ":"PoA+jWt"}"#), true)]
    #[case(mock_jwt_payload(r#"{"typ": "poa+jwt"}"#), true)] // whitespace in the JSON doesn't matter
    #[case(mock_jwt_payload(r#"{ "typ":"poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#" {"typ": "poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#"{ "typ": "poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#"{	"typ":"poa+jwt"}"#), true)]
    #[case(
        mock_jwt_payload(
            r#"{"typ"
:"poa+jwt"}"#
        ),
        true
    )]
    #[case(
        mock_jwt_payload(
            r#" {	"typ":
"poa+jwt"}"#
        ),
        true
    )]
    #[case(mock_jwt_payload(r#"{"Typ":"poa+jwt"}"#), false)] // a differently cased field name is a different field
    #[case(mock_jwt_payload(r#"{" typ":"poa+jwt"}"#), false)] // whitespace in the field name is a different field
    #[case(mock_jwt_payload(r#"{"typ":" poa+jwt"}"#), false)] // or in the field value
    #[case(mock_jwt_payload(r#"{"typ":"jwt"}"#), false)] // an ordinary JWT is not a PoA
    #[case(mock_jwt_payload(r#"{"typ":42}"#), false)] // Invalid JSON is not a PoA
    #[case(mock_jwt_payload(r#"{"typ"}"#), false)]
    #[case(".blah".to_string().into_bytes(), false)]
    #[case([".".to_string().into_bytes(), mock_jwt_payload(r#"{"typ":"jwt"}"#)].concat(), false)]
    #[case([mock_jwt_payload(r#"{"typ":"poa+jwt"}"#), ".blah".to_string().into_bytes()].concat(), false)]
    #[test]
    fn test_is_poa_message(#[case] msg: Vec<u8>, #[case] is_poa: bool) {
        assert_eq!(is_poa_message(&msg), is_poa);
    }

    #[tokio::test]
    async fn test_cannot_sign_poa_via_sign_instruction() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let instruction = Sign {
            messages_with_identifiers: vec![(mock_jwt_payload(r#"{"typ":"poa+jwt"}"#), vec!["key".to_string()])],
            poa_nonce: Some("nonce".to_string()),
            poa_aud: "aud".to_string(),
        };

        let err = instruction.validate_instruction(&wallet_user).unwrap_err();
        assert_matches!(err, InstructionValidationError::PoaMessage);
    }

    async fn perform_issuance<R, I: HandleInstruction<Result = R>>(instruction: I) -> R {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo.expect_save_keys().returning(|_, _| Ok(()));

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap()
    }

    fn validate_issuance(
        pops: &[UnverifiedJwt<JwtPopClaims, HeaderWithJwk>],
        poa: Option<Poa>,
        wua_with_disclosure: Option<&WuaDisclosure>,
    ) {
        let mut validations = Validation::new(Algorithm::ES256);
        validations.set_required_spec_claims(&["iss", "aud"]);
        validations.set_issuer(&[NL_WALLET_CLIENT_ID]);
        validations.set_audience(&[POP_AUD]);

        let keys = pops
            .iter()
            .map(|pop| {
                let (header, _) = pop.parse_and_verify_with_jwk(&validations).unwrap();

                header.verifying_key().unwrap()
            })
            .collect_vec();

        let wua_key = wua_with_disclosure.map(|wua_with_disclosure| {
            let wua_key = jwk_to_p256(
                &wua_with_disclosure
                    .wua()
                    .dangerous_parse_unverified()
                    .unwrap()
                    .1
                    .confirmation
                    .jwk,
            )
            .unwrap();

            wua_with_disclosure
                .wua_pop()
                .parse_and_verify(&(&wua_key).into(), &validations)
                .unwrap();

            wua_key
        });

        let keys = keys.into_iter().chain(wua_key).collect_vec();
        if keys.len() > 1 {
            poa.unwrap()
                .verify(&keys, POP_AUD, &[NL_WALLET_CLIENT_ID.to_string()], POP_NONCE)
                .unwrap();
        }
    }

    const POP_AUD: &str = "aud";
    const POP_NONCE: &str = "nonce";

    #[tokio::test]
    #[rstest]
    #[case(1)]
    #[case(2)]
    async fn should_handle_perform_issuance(#[case] key_count: usize) {
        let result = perform_issuance(PerformIssuance {
            key_count: key_count.try_into().unwrap(),
            aud: POP_AUD.to_string(),
            nonce: Some(POP_NONCE.to_string()),
        })
        .await;

        validate_issuance(result.pops.as_slice(), result.poa, None);
    }

    #[tokio::test]
    async fn should_handle_perform_issuance_with_wua() {
        let result = perform_issuance(PerformIssuanceWithWua {
            issuance_instruction: PerformIssuance {
                key_count: 1.try_into().unwrap(),
                aud: POP_AUD.to_string(),
                nonce: Some(POP_NONCE.to_string()),
            },
        })
        .await;

        validate_issuance(
            result.issuance_result.pops.as_slice(),
            result.issuance_result.poa,
            Some(&result.wua_disclosure),
        );
    }

    fn mock_change_pin_start_instruction() -> ChangePinStart {
        let privkey = SigningKey::random(&mut OsRng);
        let signature: Signature = privkey.sign("bla".as_bytes());
        ChangePinStart {
            pin_pubkey: (*privkey.verifying_key()).into(),
            pop_pin_pubkey: signature.into(),
        }
    }

    fn mock_sign_instruction() -> Sign {
        Sign {
            messages_with_identifiers: vec![],
            poa_nonce: None,
            poa_aud: "aud".to_string(),
        }
    }

    fn mock_start_pin_recovery_instruction() -> StartPinRecovery {
        StartPinRecovery {
            issuance_with_wua_instruction: PerformIssuanceWithWua {
                issuance_instruction: PerformIssuance {
                    key_count: 1.try_into().unwrap(),
                    aud: "aud".to_string(),
                    nonce: None,
                },
            },
            pin_pubkey: (*SigningKey::random(&mut OsRng).verifying_key()).into(),
        }
    }

    #[rstest]
    #[case(Box::new(CheckPin), false)]
    #[case(Box::new(mock_change_pin_start_instruction()), false)]
    #[case(Box::new(ChangePinCommit {}), false)]
    #[case(Box::new(ChangePinRollback {}), false)]
    #[case(Box::new(mock_sign_instruction()), false)]
    #[case(Box::new(mock_start_pin_recovery_instruction()), true)]
    fn test_instruction_validation_during_pin_recovery(
        #[case] instruction: Box<dyn ValidateInstruction>,
        #[case] should_succeed: bool,
    ) {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.state = WalletUserState::RecoveringPin;

        let result = instruction.validate_instruction(&wallet_user);

        if should_succeed {
            assert_matches!(result, Ok(()));
        } else {
            assert_matches!(result, Err(InstructionValidationError::PinRecoveryInProgress));
        }
    }

    #[tokio::test]
    async fn validating_confirm_transfer() {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some("recovery_code".to_string());
        let transfer_session_id = Uuid::new_v4();
        let app_version = Version::parse("1.0.0").unwrap();

        let instruction = ConfirmTransfer {
            transfer_session_id,
            app_version,
        };

        instruction.validate_instruction(&wallet_user).unwrap();
    }

    #[tokio::test]
    async fn validating_confirm_transfer_should_fail_if_source_is_transferring_itself() {
        let transfer_session_id = Uuid::new_v4();
        let app_version = Version::parse("1.0.0").unwrap();

        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some("recovery_code".to_string());
        wallet_user.state = WalletUserState::Transferring;

        let instruction = ConfirmTransfer {
            transfer_session_id,
            app_version,
        };

        let err = instruction
            .validate_instruction(&wallet_user)
            .expect_err("should fail if source is transferring itself");
        assert_matches!(err, InstructionValidationError::TransferInProgress);
    }

    #[tokio::test]
    async fn validating_confirm_transfer_should_fail_if_source_does_not_have_recovery_code() {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = None;
        let transfer_session_id = Uuid::new_v4();
        let app_version = Version::parse("1.0.0").unwrap();

        let instruction = ConfirmTransfer {
            transfer_session_id,
            app_version,
        };

        let err = instruction
            .validate_instruction(&wallet_user)
            .expect_err("should fail if source does not have recovery code");
        assert_matches!(err, InstructionValidationError::MissingRecoveryCode);
    }

    fn example_transfer_session(transfer_session_id: Uuid, state: TransferSessionState) -> TransferSession {
        TransferSession {
            id: Uuid::new_v4(),
            source_wallet_user_id: Some(Uuid::new_v4()),
            destination_wallet_user_id: Uuid::new_v4(),
            transfer_session_id,
            destination_wallet_recovery_code: String::from("recovery_code"),
            destination_wallet_app_version: Version::parse("1.9.8").unwrap(),
            state,
            encrypted_wallet_data: None,
        }
    }

    #[tokio::test]
    async fn should_handle_confirm_transfer() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));
        wallet_user.state = WalletUserState::Active;

        let transfer_session_id = Uuid::new_v4();
        let app_version = Version::parse("1.0.0").unwrap();

        let transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Created);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo
            .expect_confirm_wallet_transfer()
            .returning(|_, _, _, _| Ok(()));

        let instruction = ConfirmTransfer {
            transfer_session_id,
            app_version,
        };

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_confirm_transfer_should_fail_for_different_recovery_code() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));
        wallet_user.state = WalletUserState::Active;

        let transfer_session_id = Uuid::new_v4();
        let app_version = Version::parse("1.0.0").unwrap();

        let mut transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Created);
        transfer_session.destination_wallet_recovery_code = String::from("different_recovery_code");

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));

        let instruction = ConfirmTransfer {
            transfer_session_id,
            app_version,
        };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when wallet_user does not have a recovery code");

        assert_matches!(err, InstructionError::AccountTransferWalletsMismatch);
    }

    #[tokio::test]
    async fn should_handle_confirm_transfer_error_when_not_in_progress() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.state = WalletUserState::Active;

        let transfer_session_id = Uuid::new_v4();
        let app_version = Version::parse("1.0.0").unwrap();
        let instruction = ConfirmTransfer {
            transfer_session_id,
            app_version,
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(None));

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(None));

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(None));

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when a transfer session is not in progress");

        assert_matches!(err, InstructionError::NoAccountTransferInProgress)
    }

    #[tokio::test]
    async fn should_handle_confirm_transfer_wrong_app_version() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let mut transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Created);
        transfer_session.destination_wallet_app_version = Version::parse("1.2.3").unwrap();

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .returning(move |_, _| Ok(Some(transfer_session.clone())));

        let instruction = ConfirmTransfer {
            transfer_session_id: Uuid::new_v4(),
            app_version: Version::parse("2.2.3").unwrap(),
        };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when app version is wrong");

        assert_matches!(err, InstructionError::AppVersionMismatch { .. });
    }

    #[tokio::test]
    #[rstest]
    #[case(Box::new(CancelTransfer { transfer_session_id: Uuid::new_v4() }))]
    #[case(Box::new(GetTransferStatus { transfer_session_id: Uuid::new_v4() }))]
    async fn validating_transfer_instruction(#[case] instruction: Box<dyn ValidateInstruction>) {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some("recovery_code".to_string());
        instruction.validate_instruction(&wallet_user).unwrap();
    }

    #[tokio::test]
    #[rstest]
    #[case(Box::new(CancelTransfer { transfer_session_id: Uuid::new_v4() }))]
    #[case(Box::new(GetTransferStatus { transfer_session_id: Uuid::new_v4() }))]
    async fn validating_transfer_instruction_should_fail_if_source_does_not_have_recovery_code(
        #[case] instruction: Box<dyn ValidateInstruction>,
    ) {
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = None;

        let err = instruction
            .validate_instruction(&wallet_user)
            .expect_err("should fail if source does not have recovery code");
        assert_matches!(err, InstructionValidationError::MissingRecoveryCode);
    }

    #[tokio::test]
    async fn should_handle_cancel_transfer() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let mut transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Created);
        transfer_session.encrypted_wallet_data = Some(random_string(32));

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo
            .expect_cancel_wallet_transfer()
            .returning(|_, _, _, _| Ok(()));

        let instruction = CancelTransfer { transfer_session_id };

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_cancel_transfer_when_already_completed() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let mut transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Success);
        transfer_session.encrypted_wallet_data = Some(random_string(32));

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo
            .expect_cancel_wallet_transfer()
            .returning(|_, _, _, _| Ok(()));

        let instruction = CancelTransfer { transfer_session_id };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when the transfer session has already been completed");

        assert_matches!(err, InstructionError::AccountTransferIllegalState);
    }

    #[tokio::test]
    async fn should_handle_get_transfer_state() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();

        let transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::ReadyForTransfer);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));

        let instruction = GetTransferStatus { transfer_session_id };

        let state = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();

        assert_eq!(state, TransferSessionState::ReadyForTransfer);
    }

    #[tokio::test]
    async fn should_handle_send_wallet_payload() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::ReadyForTransfer);

        let payload = random_string(32);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo
            .expect_store_wallet_transfer_data()
            .with(
                predicate::always(),
                predicate::eq(transfer_session_id),
                predicate::eq(payload.clone()),
            )
            .returning(move |_, _, _| Ok(()));

        let instruction = SendWalletPayload {
            transfer_session_id,
            payload,
        };

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_send_wallet_payload_idempotency() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session =
            example_transfer_session(transfer_session_id, TransferSessionState::ReadyForDownload);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo.expect_store_wallet_transfer_data().never();

        let instruction = SendWalletPayload {
            transfer_session_id,
            payload,
        };

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_send_wallet_payload_for_illegal_state() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Canceled);

        let payload = random_string(32);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo.expect_store_wallet_transfer_data().never();

        let instruction = SendWalletPayload {
            transfer_session_id,
            payload,
        };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when transfer session is not ReadyForTransfer");

        assert_matches!(err, InstructionError::AccountTransferIllegalState)
    }

    #[tokio::test]
    async fn should_handle_receive_wallet_payload() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session =
            example_transfer_session(transfer_session_id, TransferSessionState::ReadyForDownload);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));

        let instruction = ReceiveWalletPayload { transfer_session_id };

        let result = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();

        assert_eq!(result.payload, payload);
    }

    #[tokio::test]
    async fn should_handle_receive_wallet_payload_for_illegal_state() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session =
            example_transfer_session(transfer_session_id, TransferSessionState::ReadyForTransfer);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));

        let instruction = ReceiveWalletPayload { transfer_session_id };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail for illegal transfer state");

        assert_matches!(err, InstructionError::AccountTransferIllegalState)
    }

    #[tokio::test]
    async fn should_handle_receive_wallet_payload_for_empty_wallet_data() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::ReadyForDownload);

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));

        let instruction = ReceiveWalletPayload { transfer_session_id };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail for empty wallet data");

        assert_matches!(err, InstructionError::AccountTransferIllegalState)
    }

    #[tokio::test]
    async fn should_handle_complete_transfer() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session =
            example_transfer_session(transfer_session_id, TransferSessionState::ReadyForDownload);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo
            .expect_complete_wallet_transfer()
            .returning(|_, _, _, _| Ok(()));

        let instruction = CompleteTransfer { transfer_session_id };

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_complete_transfer_when_already_canceled() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Canceled);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo
            .expect_complete_wallet_transfer()
            .returning(|_, _, _, _| Ok(()));

        let instruction = CompleteTransfer { transfer_session_id };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when the transfer session has already been canceled");

        assert_matches!(err, InstructionError::AccountTransferCanceled);
    }

    #[tokio::test]
    async fn should_handle_complete_transfer_wrong_state() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session =
            example_transfer_session(transfer_session_id, TransferSessionState::ReadyForTransfer);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo.expect_complete_wallet_transfer().never();

        let instruction = CompleteTransfer { transfer_session_id };

        let err = instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .expect_err("should fail when the transfer session has the wrong state");

        assert_matches!(err, InstructionError::AccountTransferIllegalState);
    }

    #[tokio::test]
    async fn should_handle_complete_transfer_idempotency() {
        let wrapping_key_identifier = "my-wrapping-key-identifier";
        let mut wallet_user = wallet_user::mock::wallet_user_1();
        wallet_user.recovery_code = Some(String::from("recovery_code"));

        let transfer_session_id = Uuid::new_v4();
        let payload = random_string(32);

        let mut transfer_session = example_transfer_session(transfer_session_id, TransferSessionState::Success);
        transfer_session.encrypted_wallet_data = Some(payload.clone());

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_transfer_session_by_transfer_session_id()
            .with(predicate::always(), predicate::eq(transfer_session_id))
            .returning(move |_, _| Ok(Some(transfer_session.clone())));
        wallet_user_repo.expect_complete_wallet_transfer().never();

        let instruction = CompleteTransfer { transfer_session_id };

        instruction
            .handle(
                &wallet_user,
                &MockGenerators,
                &mock::user_state(
                    wallet_user_repo,
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                    vec![],
                ),
            )
            .await
            .unwrap();
    }
}
