use std::collections::HashMap;

use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePublicKey;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::prelude::Expr;
use uuid::Uuid;

use hsm::model::wrapped_key::WrappedKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_user_key;
use crate::entity::wallet_user_key::IsBlockedModel;

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn create_keys<S, T>(db: &T, create: WalletUserKeys) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let models = create
        .keys
        .into_iter()
        .map(|key_create| {
            Ok(wallet_user_key::ActiveModel {
                id: Set(key_create.wallet_user_key_id),
                wallet_user_id: Set(create.wallet_user_id),
                identifier: Set(key_create.key_identifier),
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

pub async fn is_blocked_key<S, T>(db: &T, wallet_user_id: Uuid, key: VerifyingKey) -> Result<Option<bool>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let blocked_query_result: Vec<IsBlockedModel> = wallet_user_key::Entity::find()
        .column(wallet_user_key::Column::IsBlocked)
        .filter(
            wallet_user_key::Column::WalletUserId
                .eq(wallet_user_id)
                .and(wallet_user_key::Column::PublicKey.eq(key.to_public_key_der()?.into_vec())),
        )
        .into_partial_model()
        .all(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    let is_blocked = blocked_query_result
        .into_iter()
        .fold(None, |acc, is_blocked| match acc {
            None => Some(is_blocked.is_blocked),
            _ => panic!("multiple identical public keys found"),
        });

    Ok(is_blocked)
}

pub async fn unblock_blocked_keys<S, T>(db: &T, wallet_user_id: Uuid) -> std::result::Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_key::Entity::update_many()
        .col_expr(wallet_user_key::Column::IsBlocked, Expr::value(false))
        .filter(
            wallet_user_key::Column::WalletUserId
                .eq(wallet_user_id)
                .and(wallet_user_key::Column::IsBlocked.eq(true)),
        )
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

pub async fn delete_blocked_keys<S, T>(db: &T, wallet_user_id: Uuid) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_key::Entity::delete_many()
        .filter(
            wallet_user_key::Column::WalletUserId
                .eq(wallet_user_id)
                .and(wallet_user_key::Column::IsBlocked.eq(true)),
        )
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
        .filter(
            wallet_user_key::Column::WalletUserId
                .eq(wallet_user_id)
                .and(wallet_user_key::Column::IsBlocked.eq(false))
                .and(wallet_user_key::Column::Identifier.is_in(identifiers)),
        )
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
