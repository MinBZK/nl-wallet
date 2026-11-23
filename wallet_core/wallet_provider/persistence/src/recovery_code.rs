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
use utils::vec_at_least::VecNonEmptyUnique;
use wallet_provider_domain::model::wallet_user::RecoveryCode;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::recovery_code;

/// Denies all `recovery_codes` in one statement. Codes are inserted in sorted order, so concurrent batches lock rows in
/// the same order.
pub async fn deny_many<S, T>(db: &T, recovery_codes: &VecNonEmptyUnique<RecoveryCode>) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    // Sort recovery codes, so concurrent batches lock rows in the same order.
    let mut sorted = recovery_codes.iter().collect::<Vec<_>>();
    sorted.sort();

    let models = sorted
        .into_iter()
        .cloned()
        .map(|recovery_code| recovery_code::ActiveModel {
            id: NotSet,
            recovery_code: Set(recovery_code.into()),
            is_denied: Set(true),
        });

    recovery_code::Entity::insert_many(models)
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

/// Checks whether a recovery_code is denied.
/// Adds an explicit row in the table if it doesn't yet exists, to lock the row, whereas `find_is_denied` does
/// not lock the row.
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

/// Returns whether the recovery code is denied, without locking or writing.
pub async fn find_is_denied<S, T>(db: &T, recovery_code: &RecoveryCode) -> Result<bool, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    recovery_code::Entity::find()
        .select_only()
        .select_column(recovery_code::Column::IsDenied)
        .filter(recovery_code::Column::RecoveryCode.eq(recovery_code.as_ref()))
        .into_tuple::<bool>()
        .one(db.connection())
        .await
        .map(|is_denied| is_denied.unwrap_or_default())
        .map_err(PersistenceError::Execution)
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
