use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;

use audit_log::audited;
use audit_log::model::AuditLog;
use audit_log::model::FromAuditLogError;
use hsm::model::Hsm;
use hsm::service::HsmError;
use token_status_list::status_list_service::StatusListRevocationService;
use utils::generator::Generator;
use wallet_account::RevocationCode;
use wallet_account::messages::errors::RevocationReason;
use wallet_provider_domain::model::QueryResult;
use wallet_provider_domain::model::wallet_flag::WalletFlag;
use wallet_provider_domain::model::wallet_user::RecoveryCode;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::model::wallet_user::WalletUserIsRevoked;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletFlagRepository;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::account_server::UserState;
use crate::flags::WalletFlags;
use crate::wua_issuer::WuaIssuer;

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("persistence error: {0}")]
    Storage(#[from] PersistenceError),

    #[error("flag error: {0}")]
    Flag(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("error revoking WUA: {0}")]
    WuaRevocation(#[from] token_status_list::status_list_service::RevocationError),

    #[error("wallet ID not found: {0:?}")]
    WalletIdsNotFound(HashSet<WalletId>),

    #[error("error signing hmac for revocation code: {0}")]
    RevocationCodeHmac(#[source] HsmError),

    #[error("revocation code not found: {0}")]
    RevocationCodeNotFound(String),

    #[error("recovery code not found: {0}")]
    RecoveryCodeNotFound(RecoveryCode),

    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl FromAuditLogError for RevocationError {
    fn from_audit_log_error(audit_log_error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

#[audited]
pub async fn revoke_wallet_by_revocation_code<T, R, F, H>(
    #[audit] revocation_code: RevocationCode,
    revocation_code_key_identifier: &str,
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<DateTime<Utc>, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Hsm<Error = HsmError>,
{
    let revocation_reason = RevocationReason::UserRequest;

    let revocation_code_hmac = user_state
        .wallet_user_hsm
        .sign_hmac(revocation_code_key_identifier, revocation_code.as_ref().as_bytes())
        .await
        .map_err(RevocationError::RevocationCodeHmac)?;

    let tx = user_state.repositories.begin_transaction().await?;

    let wallet_user_result = user_state
        .repositories
        .find_wallet_user_by_revocation_code(&tx, revocation_code_hmac.as_slice())
        .await?;

    let QueryResult::Found(wallet_user) = wallet_user_result else {
        return Err(RevocationError::RevocationCodeNotFound(revocation_code.into()));
    };

    // Idempotency: if the wallet is already revoked, use the existing revocation datetime.
    let revocation_date_time = if wallet_user.state == WalletUserState::Revoked {
        wallet_user
            .revocation_registration
            .expect("revoked wallet user must have a revocation_registration")
            .date_time
    } else {
        time.generate()
    };

    let wua_ids = user_state
        .repositories
        .revoke_wallet_users(&tx, vec![wallet_user.id], revocation_reason, revocation_date_time)
        .await?;

    // Commit transaction before updating status list
    tx.commit().await?;

    user_state
        .status_list_service
        .revoke_attestation_batches(wua_ids)
        .await?;

    Ok(revocation_date_time)
}

#[audited]
pub async fn revoke_wallets_by_recovery_code<T, R, F, H>(
    #[audit] recovery_code: &RecoveryCode,
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<usize, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    system_revoke_wallets_by_recovery_code(recovery_code, user_state, time).await
}

pub async fn system_revoke_wallets_by_recovery_code<T, R, F, H>(
    recovery_code: &RecoveryCode,
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<usize, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let revocation_reason = RevocationReason::AdminRequest;
    let revocation_date_time = time.generate();

    let tx = user_state.repositories.begin_transaction().await?;

    user_state
        .repositories
        .deny_recovery_code(&tx, recovery_code.to_owned())
        .await?;

    let wallet_user_ids = user_state
        .repositories
        .find_wallet_user_ids_by_recovery_code(&tx, recovery_code)
        .await?;

    let found_wallet_count = wallet_user_ids.len();
    let wua_ids = user_state
        .repositories
        .revoke_wallet_users(&tx, wallet_user_ids, revocation_reason, revocation_date_time)
        .await?;

    // Commit transaction before updating status list
    tx.commit().await?;

    user_state
        .status_list_service
        .revoke_attestation_batches(wua_ids)
        .await?;

    Ok(found_wallet_count)
}

#[audited]
pub async fn revoke_wallets_by_wallet_id<T, R, F, H>(
    #[audit] wallet_ids: &HashSet<WalletId>,
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let revocation_reason = RevocationReason::AdminRequest;
    let revocation_date_time = time.generate();

    let tx = user_state.repositories.begin_transaction().await?;

    let found_wallets = user_state
        .repositories
        .find_wallet_user_id_by_wallet_ids(&tx, wallet_ids)
        .await?;

    if found_wallets.len() != wallet_ids.len() {
        let not_found_ids: HashSet<WalletId> = wallet_ids
            .difference(&found_wallets.into_keys().collect())
            .cloned()
            .collect();
        return Err(RevocationError::WalletIdsNotFound(not_found_ids));
    }

    // revoke all users
    let wua_ids = user_state
        .repositories
        .revoke_wallet_users(
            &tx,
            found_wallets.into_values().collect_vec(),
            revocation_reason,
            revocation_date_time,
        )
        .await?;

    // Commit transaction before updating status list
    tx.commit().await?;

    // Revoke WUA attestations of all successfully revoked wallets
    user_state
        .status_list_service
        .revoke_attestation_batches(wua_ids)
        .await?;

    Ok(())
}

#[audited]
pub async fn revoke_solution<R, F, H>(
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<(), RevocationError>
where
    F: WalletFlags,
{
    user_state
        .flags
        .set_solution_revoked()
        .await
        .map_err(|err| RevocationError::Flag(Box::new(err)))?;
    user_state.status_list_service.republish_all(false).await?;
    Ok(())
}

pub async fn restore_solution<R, F, H>(
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
) -> Result<(), RevocationError>
where
    R: WalletFlagRepository,
{
    user_state
        .repositories
        .clear_flag(WalletFlag::SolutionRevoked)
        .await
        .map_err(|err| RevocationError::Flag(Box::new(err)))?;
    user_state.status_list_service.republish_all(true).await?;
    Ok(())
}

pub async fn list_wallets<T, R, F, H>(
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
) -> Result<Vec<WalletUserIsRevoked>, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let tx = user_state.repositories.begin_transaction().await?;
    let wallet_ids = user_state.repositories.list_wallets(&tx).await?;

    tx.commit().await?;

    Ok(wallet_ids)
}

pub async fn list_denied_recovery_codes<T, R, F, H>(
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
) -> Result<Vec<RecoveryCode>, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let tx = user_state.repositories.begin_transaction().await?;
    let wallet_ids = user_state.repositories.list_denied_recovery_codes(&tx).await?;

    tx.commit().await?;

    Ok(wallet_ids)
}

pub async fn remove_denied_recovery_code<T, R, F, H>(
    user_state: &UserState<R, F, H, impl WuaIssuer, impl StatusListRevocationService>,
    recovery_code: &RecoveryCode,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let tx = user_state.repositories.begin_transaction().await?;
    let removed = user_state.repositories.allow_recovery_code(&tx, recovery_code).await?;

    tx.commit().await?;

    if !removed {
        return Err(RevocationError::RecoveryCodeNotFound(recovery_code.to_owned()));
    }

    Ok(())
}
