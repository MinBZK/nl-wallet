use sea_orm::ActiveValue::NotSet;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::SelectColumns;
use sea_orm::Set;
use sea_orm::prelude::Expr;
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

pub async fn set_allowed<S, T>(db: &T, recovery_code: &str) -> Result<bool, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = recovery_code::Entity::update_many()
        .col_expr(recovery_code::Column::IsDenied, Expr::value(false))
        .filter(recovery_code::Column::RecoveryCode.eq(recovery_code))
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(result.rows_affected == 1)
}

pub async fn list<S, T>(db: &T) -> Result<Vec<String>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    recovery_code::Entity::find()
        .select_only()
        .select_column(recovery_code::Column::RecoveryCode)
        .filter(recovery_code::Column::IsDenied.eq(true))
        .into_tuple()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)
}
