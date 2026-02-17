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

use wallet_provider_domain::model::wallet_user::RecoveryCode;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::recovery_code;

pub async fn insert<S, T>(db: &T, recovery_code: RecoveryCode) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = recovery_code::ActiveModel {
        id: NotSet,
        recovery_code: Set(recovery_code.into()),
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

pub async fn is_denied<S, T>(db: &T, recovery_code: RecoveryCode) -> Result<bool, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = recovery_code::ActiveModel {
        id: NotSet,
        recovery_code: Set(recovery_code.into()),
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

pub async fn set_allowed<S, T>(db: &T, recovery_code: &RecoveryCode) -> Result<bool, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = recovery_code::Entity::update_many()
        .col_expr(recovery_code::Column::IsDenied, Expr::value(false))
        .filter(recovery_code::Column::RecoveryCode.eq(recovery_code.as_ref()))
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(result.rows_affected == 1)
}

pub async fn list<S, T>(db: &T) -> Result<Vec<RecoveryCode>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    Ok(recovery_code::Entity::find()
        .select_only()
        .select_column(recovery_code::Column::RecoveryCode)
        .filter(recovery_code::Column::IsDenied.eq(true))
        .into_tuple::<String>()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .into_iter()
        .map(Into::into)
        .collect())
}
