use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::ConnectionTrait;
use uuid::Uuid;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
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
