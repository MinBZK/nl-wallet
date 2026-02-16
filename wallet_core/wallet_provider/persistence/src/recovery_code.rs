use sea_orm::ActiveValue::NotSet;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::recovery_code;

pub async fn insert<S, T>(db: &T, recovery_code: String) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = recovery_code::ActiveModel {
        id: NotSet,
        recovery_code: Set(recovery_code),
        is_denied: Set(true),
    };

    recovery_code::Entity::insert(model)
        .on_conflict(
            OnConflict::column(recovery_code::Column::RecoveryCode)
                .update_column(recovery_code::Column::IsDenied)
                .to_owned(),
        )
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(())
}

pub async fn is_denied<S, T>(db: &T, recovery_code: String) -> Result<bool, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = recovery_code::ActiveModel {
        id: NotSet,
        recovery_code: Set(recovery_code),
        is_denied: Set(false),
    };

    let result = recovery_code::Entity::insert(model)
        .on_conflict(
            OnConflict::column(recovery_code::Column::RecoveryCode)
                // hack to get a lock on the row
                .update_column(recovery_code::Column::RecoveryCode)
                .to_owned(),
        )
        .exec_with_returning(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(result.is_denied)
}
