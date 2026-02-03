use sea_orm::ActiveValue::NotSet;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::denied_recovery_code;

pub async fn create<S, T>(db: &T, recovery_code: String) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = denied_recovery_code::ActiveModel {
        id: NotSet,
        recovery_code: Set(recovery_code),
    };
    denied_recovery_code::Entity::insert(model)
        .on_conflict(
            // this is to support idempotency; a recovery code can only be on the list once
            OnConflict::column(denied_recovery_code::Column::RecoveryCode)
                .do_nothing()
                .to_owned(),
        )
        .on_empty_do_nothing()
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(())
}
