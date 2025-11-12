use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use uuid::Uuid;

use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_user;
use crate::entity::wallet_user_wua;

pub async fn create<S, T>(db: &T, wallet_user_id: Uuid, wua_id: Uuid) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_wua::ActiveModel {
        wallet_user_id: Set(wallet_user_id),
        wua_id: Set(wua_id),
    }
    .insert(db.connection())
    .await
    .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

    Ok(())
}

pub async fn wua_ids_for_wallet<S, T>(db: &T, wallet_id: String) -> Result<Vec<Uuid>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    Ok(wallet_user_wua::Entity::find()
        .join(JoinType::InnerJoin, wallet_user_wua::Relation::WalletUser.def())
        .filter(wallet_user::Column::WalletId.eq(wallet_id.to_owned()))
        .all(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?
        .iter()
        .map(|model| model.wua_id)
        .collect())
}
