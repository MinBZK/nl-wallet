use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use audit_log::model::AuditLog;
use audit_log::model::PostgresAuditLogError;
use hsm::model::Hsm;
use hsm::service::HsmError;
use token_status_list::status_list_service::StatusListRevocationService;
use utils::generator::Generator;
use wallet_account::RevocationCode;
use wallet_account::messages::errors::RevocationReason;
use wallet_provider_domain::model::QueryResult;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::account_server::UserState;
use crate::wua_issuer::WuaIssuer;

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("persistence error: {0}")]
    Storage(#[from] PersistenceError),

    #[error("error revoking WUA: {0}")]
    WuaRevocation(#[from] token_status_list::status_list_service::RevocationError),

    #[error("wallet ID not found: {0:?}")]
    WalletIdsNotFound(HashSet<String>),

    #[error("error signing hmac for revocation code: {0}")]
    RevocationCodeHmac(#[source] HsmError),

    #[error("no wallet found with recovation code: {0}")]
    RevocationCodeNotFound(String),

    #[error("error while auditing: {0}")]
    AuditLog(#[from] PostgresAuditLogError),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevocationResult {
    revoked_at: DateTime<Utc>,
}

pub async fn revoke_wallet_by_revocation_code<T, R, H>(
    revocation_code: RevocationCode,
    revocation_code_key_identifier: &str,
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
    audit_log: &impl AuditLog<Error = PostgresAuditLogError>,
) -> Result<RevocationResult, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Hsm<Error = HsmError>,
{
    audit_log
        .audit(
            "revoke_wallet_by_revocation_code",
            json!({"revocation_code": revocation_code.to_string()}),
            async || {
                let revocation_reason = RevocationReason::UserRequest;
                let revocation_date_time = time.generate();

                let revocation_code_hmac = user_state
                    .wallet_user_hsm
                    .sign_hmac(revocation_code_key_identifier, revocation_code.as_ref().as_bytes())
                    .await
                    .map_err(RevocationError::RevocationCodeHmac)?;

                let tx = user_state.repositories.begin_transaction().await?;

                let wallet_user_id_result = user_state
                    .repositories
                    .find_wallet_user_id_by_revocation_code(&tx, revocation_code_hmac.as_slice())
                    .await?;

                let QueryResult::Found(wallet_user_id) = wallet_user_id_result else {
                    return Err(RevocationError::RevocationCodeNotFound(revocation_code.into()));
                };

                let wua_ids = user_state
                    .repositories
                    .revoke_wallet_users(&tx, vec![*wallet_user_id], revocation_reason, revocation_date_time)
                    .await?;

                tx.commit().await?;

                user_state
                    .status_list_service
                    .revoke_attestation_batches(wua_ids)
                    .await?;

                Ok(RevocationResult {
                    revoked_at: revocation_date_time,
                })
            },
        )
        .await
}

pub async fn revoke_wallets_by_recovery_code<T, R, H>(
    recovery_code: &str,
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Hsm<Error = HsmError>,
{
    let revocation_reason = RevocationReason::AdminRequest;
    let revocation_date_time = time.generate();

    let tx = user_state.repositories.begin_transaction().await?;

    user_state
        .repositories
        .add_recovery_code_to_deny_list(&tx, recovery_code.to_owned())
        .await?;

    let wallet_user_ids = user_state
        .repositories
        .find_wallet_user_ids_by_recovery_code(&tx, recovery_code)
        .await?;

    let wua_ids = user_state
        .repositories
        .revoke_wallet_users(&tx, wallet_user_ids, revocation_reason, revocation_date_time)
        .await?;

    tx.commit().await?;

    user_state
        .status_list_service
        .revoke_attestation_batches(wua_ids)
        .await?;

    Ok(())
}

pub async fn revoke_wallets_by_wallet_id<T, R, H>(
    wallet_ids: &HashSet<String>,
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
    audit_log: &impl AuditLog<Error = PostgresAuditLogError>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    audit_log
        .audit(
            "revoke_wallets_by_wallet_id",
            json!({"wallet_ids": wallet_ids}),
            async || {
                let revocation_reason = RevocationReason::AdminRequest;
                let revocation_date_time = time.generate();

                let tx = user_state.repositories.begin_transaction().await?;

                let found_wallets = user_state
                    .repositories
                    .find_wallet_user_id_by_wallet_ids(&tx, wallet_ids)
                    .await?;

                if found_wallets.len() != wallet_ids.len() {
                    let not_found_ids: HashSet<String> = wallet_ids
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

                tx.commit().await?;

                // Revoke WUA attestations of all successfully revoked wallets
                user_state
                    .status_list_service
                    .revoke_attestation_batches(wua_ids)
                    .await?;

                Ok(())
            },
        )
        .await
}

pub async fn revoke_all_wallets<T, R, H>(
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
    audit_log: &impl AuditLog<Error = PostgresAuditLogError>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    audit_log
        .audit("revoke_all_wallets", json!({}), async || {
            let revocation_reason = RevocationReason::WalletSolutionCompromised;
            let revocation_date_time = time.generate();

            // TODO rewrite this method (PVW-5299)
            let tx = user_state.repositories.begin_transaction().await?;
            let wallet_user_ids = user_state.repositories.list_wallet_user_ids(&tx).await?;
            let wua_ids = user_state
                .repositories
                .revoke_wallet_users(&tx, wallet_user_ids, revocation_reason, revocation_date_time)
                .await?;
            tx.commit().await?;

            // TODO consider adding a `revoke_all` method to `StatusListRevocationService` for efficiency (PVW-5299)
            user_state
                .status_list_service
                .revoke_attestation_batches(wua_ids)
                .await?;

            Ok(())
        })
        .await
}

pub async fn list_wallets<T, R, H>(
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
) -> Result<Vec<String>, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let tx = user_state.repositories.begin_transaction().await?;
    let wallet_ids = user_state.repositories.list_wallet_ids(&tx).await?;

    tx.commit().await?;

    Ok(wallet_ids)
}
