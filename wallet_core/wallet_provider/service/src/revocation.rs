use chrono::Utc;
use futures::future::try_join_all;
use uuid::Uuid;

use token_status_list::status_list_service::StatusListRevocationService;
use wallet_provider_domain::model::wallet_user::QueryResult;
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

    #[error("wallet ID not found: {0}")]
    WalletIdNotFound(String),
}

pub async fn revoke_wallets<T, R, H>(
    wallet_ids: Vec<String>,
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let revocation_reason = RevocationReason::AdminRequest;
    let revocation_date_time = Utc::now();
    let wua_ids: Vec<Uuid> = try_join_all(wallet_ids.iter().map(async |wallet_id| {
        let tx = user_state.repositories.begin_transaction().await?;

        let found = user_state
            .repositories
            .find_wallet_user_id_by_wallet_id(&tx, wallet_id)
            .await?;

        let result = match found {
            QueryResult::Found(wallet_user_id) => Ok(user_state
                .repositories
                .revoke_wallet_user(&tx, *wallet_user_id, revocation_reason, revocation_date_time)
                .await?),
            QueryResult::NotFound => Err(RevocationError::WalletIdNotFound(wallet_id.to_owned())),
        };
        tx.commit().await?;

        result
    }))
    .await?
    .into_iter()
    .flatten()
    .collect();

    // Revoke WUA attestations of all successfully revoked wallets
    user_state
        .status_list_service
        .revoke_attestation_batches(wua_ids)
        .await?;

    Ok(())
}

pub async fn revoke_all_wallets<T, R, H>(
    user_state: &UserState<R, H, impl WuaIssuer, impl StatusListRevocationService>,
) -> Result<(), RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    let revocation_reason = RevocationReason::WalletSolutionCompromised;
    let revocation_date_time = Utc::now();

    // TODO rewrite this method (PVW-5299)
    let tx = user_state.repositories.begin_transaction().await?;
    let wallet_user_ids = user_state.repositories.list_wallet_user_ids(&tx).await?;
    tx.commit().await?;

    let wua_ids: Vec<Uuid> = try_join_all(wallet_user_ids.into_iter().map(async |wallet_user_id| {
        let tx = user_state.repositories.begin_transaction().await?;
        let wua_ids = user_state
            .repositories
            .revoke_wallet_user(&tx, wallet_user_id, revocation_reason, revocation_date_time)
            .await?;
        tx.commit().await?;

        Result::<_, RevocationError>::Ok(wua_ids)
    }))
    .await?
    .into_iter()
    .flatten()
    .collect();

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
