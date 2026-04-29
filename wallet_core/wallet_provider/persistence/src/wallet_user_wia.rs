use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use uuid::Uuid;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_user_wia;

pub async fn create<S, T>(db: &T, wallet_user_id: Uuid, wia_id: Uuid) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_wia::ActiveModel {
        wallet_user_id: Set(wallet_user_id),
        wia_id: Set(wia_id),
    }
    .insert(db.connection())
    .await
    .map_err(PersistenceError::Execution)?;

    Ok(())
}

pub async fn list_wia_ids<S, T>(db: &T) -> Result<Vec<Uuid>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_wia::Entity::find()
        .select_only()
        .column(wallet_user_wia::Column::WiaId)
        .into_tuple()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)
}

pub async fn find_wia_ids_for_wallet_users<S, T>(
    db: &T,
    wallet_user_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_wia::Entity::find()
        .select_only()
        .column(wallet_user_wia::Column::WiaId)
        .filter(wallet_user_wia::Column::WalletUserId.is_in(wallet_user_ids))
        .into_tuple()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)
}
