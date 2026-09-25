use chrono::DateTime;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::DbBackend;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::Statement;
use sea_orm::TransactionTrait;
use sea_orm::prelude::Expr;
use tracing::info;
use wallet_provider_domain::model::admin_portal_session::AdminPortalLoginAttempt;
use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::admin_portal_login_attempt;
use crate::entity::admin_portal_user_session;

/// Maximum rows deleted per statement during cleanup, to bound lock duration and DB load. See
/// `issuer_common::par_store`/`state_bridge_store` for the same pattern.
const CLEANUP_BATCH_SIZE: u64 = 1_000;

pub async fn insert_login_attempt<S, T>(
    db: &T,
    state: String,
    login_attempt: AdminPortalLoginAttempt,
) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = admin_portal_login_attempt::ActiveModel {
        state: Set(state),
        nonce: Set(login_attempt.nonce),
        code_verifier: Set(login_attempt.code_verifier),
        created_at: Set(login_attempt.created_at.into()),
    };

    admin_portal_login_attempt::Entity::insert(model)
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(())
}

pub async fn take_login_attempt<S, T>(db: &T, state: &str) -> Result<Option<AdminPortalLoginAttempt>, PersistenceError>
where
    S: ConnectionTrait + TransactionTrait,
    T: PersistenceConnection<S>,
{
    let tx = db.connection().begin().await.map_err(PersistenceError::Transaction)?;

    let Some(model) = admin_portal_login_attempt::Entity::find_by_id(state.to_string())
        .lock_exclusive()
        .one(&tx)
        .await
        .map_err(PersistenceError::Execution)?
    else {
        return Ok(None);
    };

    admin_portal_login_attempt::Entity::delete_by_id(state.to_string())
        .exec(&tx)
        .await
        .map_err(PersistenceError::Execution)?;

    tx.commit().await.map_err(PersistenceError::Transaction)?;

    Ok(Some(AdminPortalLoginAttempt {
        nonce: model.nonce,
        code_verifier: model.code_verifier,
        created_at: model.created_at.into(),
    }))
}

pub async fn cleanup_expired_login_attempts<S, T>(db: &T, created_before: DateTime<Utc>) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let mut total_deleted: u64 = 0;

    // Deletes login attempts created before `created_before`, in bounded batches so a single statement never
    // holds a large lock or scans the whole backlog. `FOR UPDATE SKIP LOCKED` skips rows currently locked by a
    // concurrent `take_login_attempt`; the loop drains the rest, stopping once a batch removes fewer rows than the
    // limit (nothing left to delete). This also allows multiple pods to run concurrent cleanup tasks safely.
    loop {
        let result = db
            .connection()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                WITH rows_to_delete AS (
                    SELECT state
                    FROM admin_portal_login_attempt
                    WHERE created_at < $1
                    LIMIT $2
                    FOR UPDATE SKIP LOCKED
                )
                DELETE FROM admin_portal_login_attempt ala
                USING rows_to_delete
                WHERE ala.state = rows_to_delete.state
                "#,
                [created_before.into(), (CLEANUP_BATCH_SIZE as i64).into()],
            ))
            .await
            .map_err(PersistenceError::Execution)?;

        total_deleted += result.rows_affected();
        if result.rows_affected() < CLEANUP_BATCH_SIZE {
            break;
        }
    }

    if total_deleted > 0 {
        info!("Deleted {total_deleted} expired admin portal login attempt(s) from storage");
    }

    Ok(())
}

pub async fn insert_user_session<S, T>(
    db: &T,
    id: String,
    session: AdminPortalUserSession,
) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = admin_portal_user_session::ActiveModel {
        id: Set(id),
        display_name: Set(session.display_name),
        roles: Set(session.roles),
        id_token: Set(session.id_token),
        expires_at: Set(session.expires_at.into()),
    };

    admin_portal_user_session::Entity::insert(model)
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(())
}

/// Extends the session's expiry to `new_expires_at`, provided it has not already expired as of `now`.
pub async fn touch_user_session<S, T>(
    db: &T,
    id: &str,
    now: DateTime<Utc>,
    new_expires_at: DateTime<Utc>,
) -> Result<Option<AdminPortalUserSession>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = admin_portal_user_session::Entity::update_many()
        .col_expr(
            admin_portal_user_session::Column::ExpiresAt,
            Expr::value(new_expires_at),
        )
        .filter(admin_portal_user_session::Column::Id.eq(id))
        .filter(admin_portal_user_session::Column::ExpiresAt.gt(now))
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    if result.rows_affected == 0 {
        return Ok(None);
    }

    let model = admin_portal_user_session::Entity::find_by_id(id.to_string())
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(model.map(|model| AdminPortalUserSession {
        display_name: model.display_name,
        roles: model.roles,
        id_token: model.id_token,
        expires_at: model.expires_at.into(),
    }))
}

pub async fn take_user_session<S, T>(db: &T, id: &str) -> Result<Option<AdminPortalUserSession>, PersistenceError>
where
    S: ConnectionTrait + TransactionTrait,
    T: PersistenceConnection<S>,
{
    let tx = db.connection().begin().await.map_err(PersistenceError::Transaction)?;

    let Some(model) = admin_portal_user_session::Entity::find_by_id(id.to_string())
        .lock_exclusive()
        .one(&tx)
        .await
        .map_err(PersistenceError::Execution)?
    else {
        return Ok(None);
    };

    admin_portal_user_session::Entity::delete_by_id(id.to_string())
        .exec(&tx)
        .await
        .map_err(PersistenceError::Execution)?;

    tx.commit().await.map_err(PersistenceError::Transaction)?;

    Ok(Some(AdminPortalUserSession {
        display_name: model.display_name,
        roles: model.roles,
        id_token: model.id_token,
        expires_at: model.expires_at.into(),
    }))
}

pub async fn cleanup_expired_user_sessions<S, T>(db: &T, now: DateTime<Utc>) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let mut total_deleted: u64 = 0;

    // Deletes sessions that expired at or before `now`, in bounded batches so a single statement never holds a
    // large lock or scans the whole backlog. `FOR UPDATE SKIP LOCKED` skips rows currently locked by a concurrent
    // `take_user_session`/`touch_user_session`; the loop drains the rest, stopping once a batch removes fewer rows
    // than the limit (nothing left to delete). This also allows multiple pods to run concurrent cleanup tasks safely.
    loop {
        let result = db
            .connection()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                WITH rows_to_delete AS (
                    SELECT id
                    FROM admin_portal_user_session
                    WHERE expires_at <= $1
                    LIMIT $2
                    FOR UPDATE SKIP LOCKED
                )
                DELETE FROM admin_portal_user_session aus
                USING rows_to_delete
                WHERE aus.id = rows_to_delete.id
                "#,
                [now.into(), (CLEANUP_BATCH_SIZE as i64).into()],
            ))
            .await
            .map_err(PersistenceError::Execution)?;

        total_deleted += result.rows_affected();
        if result.rows_affected() < CLEANUP_BATCH_SIZE {
            break;
        }
    }

    if total_deleted > 0 {
        info!("Deleted {total_deleted} expired admin portal session(s) from storage");
    }

    Ok(())
}
