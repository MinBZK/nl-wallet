use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use token_status_list::status_list_service::StatusListRevocationService;
use utils::generator::Generator;
use wallet_provider_domain::model::wallet_user::RevocationReason;
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
    WalletIdsNotFound(Vec<String>),
}

pub async fn revoke_wallets<T, R, H>(
    wallet_ids: &[String],
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let revocation_reason = RevocationReason::AdminRequest;
    let revocation_date_time = time.generate();

    let tx = user_state.repositories.begin_transaction().await?;

    let (wallet_user_ids, found_wallet_ids): (Vec<Uuid>, Vec<String>) = user_state
        .repositories
        .find_wallet_user_id_by_wallet_ids(&tx, wallet_ids)
        .await?
        .into_iter()
        .unzip();

    if found_wallet_ids.len() != wallet_ids.len() {
        let not_found_ids: Vec<String> = wallet_ids
            .iter()
            .filter(|id| !found_wallet_ids.contains(id))
            .cloned()
            .collect();
        return Err(RevocationError::WalletIdsNotFound(not_found_ids));
    }

    // revoke all users
    let wua_ids = user_state
        .repositories
        .revoke_wallet_users(&tx, wallet_user_ids, revocation_reason, revocation_date_time)
        .await?;

    tx.commit().await?;

    // Revoke WUA attestations of all successfully revoked wallets
    user_state
        .status_list_service
        .revoke_attestation_batches(wua_ids)
        .await?;

    Ok(())
}

pub async fn revoke_all_wallets<T, R, H>(
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
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
