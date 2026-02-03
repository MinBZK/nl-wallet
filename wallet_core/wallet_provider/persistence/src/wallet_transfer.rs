use chrono::DateTime;
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use sea_orm::Set;
use sea_orm::prelude::Expr;
use semver::Version;
use uuid::Uuid;

use wallet_account::messages::transfer::TransferSessionState;
use wallet_provider_domain::model::wallet_user::TransferSession;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_transfer;
use crate::entity::wallet_user;

pub async fn create_transfer_session<S, T>(
    db: &T,
    destination_wallet_user_id: Uuid,
    transfer_session_id: Uuid,
    destination_wallet_app_version: Version,
    created: DateTime<Utc>,
) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_transfer::ActiveModel {
        id: Set(Uuid::new_v4()),
        destination_wallet_user_id: Set(destination_wallet_user_id),
        source_wallet_user_id: Set(None),
        transfer_session_id: Set(transfer_session_id),
        destination_wallet_app_version: Set(destination_wallet_app_version.to_string()),
        state: Set(TransferSessionState::Created.to_string()),
        created: Set(created.into()),
        encrypted_wallet_data: Set(None),
    }
    .insert(db.connection())
    .await
    .map_err(PersistenceError::Execution)?;

    Ok(())
}

pub async fn find_transfer_session_by_transfer_session_id<S, T>(
    db: &T,
    transfer_session_id: Uuid,
) -> Result<Option<TransferSession>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    type TransferQueryResult = (Uuid, Uuid, Option<Uuid>, Uuid, String, String, Option<String>, String);

    let result: Option<TransferQueryResult> = wallet_transfer::Entity::find()
        .select_only()
        .column(wallet_transfer::Column::Id)
        .column(wallet_transfer::Column::DestinationWalletUserId)
        .column(wallet_transfer::Column::SourceWalletUserId)
        .column(wallet_transfer::Column::TransferSessionId)
        .column(wallet_transfer::Column::DestinationWalletAppVersion)
        .column(wallet_transfer::Column::State)
        .column(wallet_transfer::Column::EncryptedWalletData)
        .column(wallet_user::Column::RecoveryCode)
        .filter(wallet_transfer::Column::TransferSessionId.eq(transfer_session_id))
        .join(JoinType::InnerJoin, wallet_transfer::Relation::WalletUser2.def())
        .into_tuple()
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    let transfer_session = result.map(
        |(
            id,
            destination_wallet_user_id,
            source_wallet_user_id,
            transfer_session_id,
            destination_wallet_app_version,
            state,
            encrypted_wallet_data,
            destination_wallet_recovery_code,
        )| TransferSession {
            id,
            source_wallet_user_id,
            destination_wallet_user_id,
            destination_wallet_app_version: Version::parse(&destination_wallet_app_version)
                .expect("version from database should parse"),
            transfer_session_id,
            state: state.parse().expect("state from database should parse"),
            encrypted_wallet_data,
            destination_wallet_recovery_code,
        },
    );

    Ok(transfer_session)
}

pub async fn find_transfer_session_id_by_destination_wallet_user_id<S, T>(
    db: &T,
    destination_wallet_user_id: Uuid,
) -> Result<Option<Uuid>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_transfer::Entity::find()
        .select_only()
        .column(wallet_transfer::Column::TransferSessionId)
        .filter(wallet_transfer::Column::DestinationWalletUserId.eq(destination_wallet_user_id))
        .into_tuple()
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)
}

pub async fn update_transfer_state<S, T>(
    db: &T,
    transer_session_id: Uuid,
    transfer_session_state: TransferSessionState,
) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_transfer::Entity::update_many()
        .col_expr(
            wallet_transfer::Column::State,
            Expr::value(transfer_session_state.to_string()),
        )
        .filter(wallet_transfer::Column::TransferSessionId.eq(transer_session_id))
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_transfer`s with the same `transfer_session_id`"),
    }
}

pub async fn set_transfer_source<S, T>(
    db: &T,
    transer_session_id: Uuid,
    source_wallet_user_id: Uuid,
) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_transfer::Entity::update_many()
        .col_expr(
            wallet_transfer::Column::SourceWalletUserId,
            Expr::value(source_wallet_user_id),
        )
        .filter(wallet_transfer::Column::TransferSessionId.eq(transer_session_id))
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_transfer`s with the same `transfer_session_id`"),
    }
}

pub async fn set_wallet_transfer_data<S, T>(
    db: &T,
    transer_session_id: Uuid,
    encrypted_wallet_data: Option<String>,
) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_transfer::Entity::update_many()
        .col_expr(
            wallet_transfer::Column::EncryptedWalletData,
            encrypted_wallet_data.map_or(Expr::cust("null"), Expr::value),
        )
        .filter(wallet_transfer::Column::TransferSessionId.eq(transer_session_id))
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_transfer`s with the same `transfer_session_id`"),
    }
}
