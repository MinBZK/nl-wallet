use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::NotSet;
use sea_orm::ConnectionTrait;
use sea_orm::Set;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::denied_recovery_code;

pub async fn create<S, T>(db: &T, recovery_code: String) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    denied_recovery_code::ActiveModel {
        id: NotSet,
        recovery_code: Set(recovery_code),
    }
    .insert(db.connection())
    .await
    .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

    Ok(())
}
