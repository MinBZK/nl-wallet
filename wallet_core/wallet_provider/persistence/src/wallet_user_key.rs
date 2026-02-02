use std::collections::HashMap;

use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePublicKey;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::DerivePartialModel;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::ModelTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::QueryTrait;
use sea_orm::Set;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::SelectStatement;
use uuid::Uuid;

use crypto::p256_der::verifying_key_sha256;
use hsm::model::wrapped_key::WrappedKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_user_key;

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn persist_keys<S, T>(db: &T, create: WalletUserKeys) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let models = create
        .keys
        .into_iter()
        .map(|key_create| {
            let key_identifier = verifying_key_sha256(key_create.key.public_key());

            Ok(wallet_user_key::ActiveModel {
                id: Set(key_create.wallet_user_key_id),
                wallet_user_id: Set(create.wallet_user_id),
                batch_id: Set(create.batch_id),
                identifier: Set(key_identifier),
                public_key: Set(key_create.key.public_key().to_public_key_der()?.into_vec()),
                encrypted_private_key: Set(key_create.key.wrapped_private_key().to_vec()),
                is_blocked: Set(key_create.is_blocked),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    wallet_user_key::Entity::insert_many(models)
        .exec(db.connection())
        .await
        .map(|_| ())
        .map_err(|e| PersistenceError::Execution(e.into()))
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "<wallet_user_key::Model as ModelTrait>::Entity")]
pub struct IsBlockedModel {
    pub is_blocked: bool,
}

pub async fn is_blocked_key<S, T>(db: &T, wallet_user_id: Uuid, key: VerifyingKey) -> Result<Option<bool>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let key_identifier = verifying_key_sha256(&key);

    let blocked_query_result: Option<IsBlockedModel> = wallet_user_key::Entity::find()
        .select_only()
        .column(wallet_user_key::Column::IsBlocked)
        .filter(wallet_user_key::Column::WalletUserId.eq(wallet_user_id))
        .filter(wallet_user_key::Column::Identifier.eq(key_identifier))
        .into_partial_model()
        .one(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(blocked_query_result.map(|query_result| query_result.is_blocked))
}

fn select_batch_id(wallet_user_id: Uuid, key_identifier: String) -> SelectStatement {
    wallet_user_key::Entity::find()
        .select_only()
        .column(wallet_user_key::Column::BatchId)
        .filter(wallet_user_key::Column::WalletUserId.eq(wallet_user_id))
        .filter(wallet_user_key::Column::Identifier.eq(key_identifier))
        .into_query()
}

pub async fn unblock_blocked_keys_in_same_batch<S, T>(db: &T, wallet_user_id: Uuid, key: VerifyingKey) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let key_identifier = verifying_key_sha256(&key);

    wallet_user_key::Entity::update_many()
        .col_expr(wallet_user_key::Column::IsBlocked, Expr::value(false))
        .filter(wallet_user_key::Column::IsBlocked.eq(true))
        .filter(wallet_user_key::Column::BatchId.in_subquery(select_batch_id(wallet_user_id, key_identifier)))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

pub async fn delete_blocked_keys_in_same_batch<S, T>(db: &T, wallet_user_id: Uuid, key: VerifyingKey) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let key_identifier = verifying_key_sha256(&key);

    wallet_user_key::Entity::delete_many()
        .filter(wallet_user_key::Column::IsBlocked.eq(true))
        .filter(wallet_user_key::Column::BatchId.in_subquery(select_batch_id(wallet_user_id, key_identifier)))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

pub async fn delete_all_blocked_keys<S, T>(db: &T, wallet_user_id: Uuid) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_key::Entity::delete_many()
        .filter(wallet_user_key::Column::IsBlocked.eq(true))
        .filter(wallet_user_key::Column::WalletUserId.eq(wallet_user_id))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

pub async fn delete_all_keys<S, T>(db: &T, wallet_user_ids: Vec<Uuid>) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_key::Entity::delete_many()
        .filter(wallet_user_key::Column::WalletUserId.is_in(wallet_user_ids))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

/// Retrieves all active (i.e. unblocked) keys for user `[wallet_user_id]` by `[identifiers]`.
pub async fn find_active_keys_by_identifiers<S, T>(
    db: &T,
    wallet_user_id: Uuid,
    identifiers: &[String],
) -> Result<HashMap<String, WrappedKey>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_key::Entity::find()
        .select_only()
        .column(wallet_user_key::Column::Identifier)
        .column(wallet_user_key::Column::EncryptedPrivateKey)
        .column(wallet_user_key::Column::PublicKey)
        .filter(wallet_user_key::Column::Identifier.is_in(identifiers))
        .filter(wallet_user_key::Column::WalletUserId.eq(wallet_user_id))
        .filter(wallet_user_key::Column::IsBlocked.eq(false))
        .into_tuple::<(String, Vec<u8>, Vec<u8>)>()
        .all(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?
        .into_iter()
        .map(|(id, key_data, public_key)| {
            Ok((
                id,
                WrappedKey::new(key_data, VerifyingKey::from_public_key_der(&public_key)?),
            ))
        })
        .collect()
}

pub async fn move_keys<S, T>(db: &T, from_wallet_user_id: Uuid, to_wallet_user_id: Uuid) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_key::Entity::update_many()
        .col_expr(wallet_user_key::Column::WalletUserId, Expr::value(to_wallet_user_id))
        .filter(wallet_user_key::Column::WalletUserId.eq(from_wallet_user_id))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}
