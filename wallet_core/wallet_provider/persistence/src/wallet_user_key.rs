use std::collections::HashMap;

use p256::{
    ecdsa::VerifyingKey,
    pkcs8::{DecodePublicKey, EncodePublicKey},
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set};

use wallet_provider_domain::{
    model::{wallet_user::WalletUserKeys, wrapped_key::WrappedKey},
    repository::PersistenceError,
};

use crate::{entity::wallet_user_key, PersistenceConnection};

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
            })
        })
        .collect::<Result<Vec<_>>>()?;

    wallet_user_key::Entity::insert_many(models)
        .exec(db.connection())
        .await
        .map(|_| ())
        .map_err(|e| PersistenceError::Execution(e.into()))
}

pub async fn find_keys_by_identifiers<S, T>(
    db: &T,
    wallet_user_id: uuid::Uuid,
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
