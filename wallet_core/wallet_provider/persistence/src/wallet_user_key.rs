use p256::ecdsa::SigningKey;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set};
use std::collections::HashMap;

use wallet_provider_domain::{model::wallet_user::WalletUserKeysCreate, repository::PersistenceError};

use crate::{entity::wallet_user_key, PersistenceConnection};

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn create_keys<S, T>(db: &T, create: WalletUserKeysCreate) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let models = create
        .keys
        .into_iter()
        .map(|(id, identifier, key)| wallet_user_key::ActiveModel {
            id: Set(id),
            wallet_user_id: Set(create.wallet_user_id),
            identifier: Set(identifier),
            private_key_der: Set(key.to_bytes().to_vec()),
        })
        .collect::<Vec<_>>();

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
) -> Result<HashMap<String, SigningKey>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_user_key::Entity::find()
        .select_only()
        .column(wallet_user_key::Column::Identifier)
        .column(wallet_user_key::Column::PrivateKeyDer)
        .filter(
            wallet_user_key::Column::WalletUserId
                .eq(wallet_user_id)
                .and(wallet_user_key::Column::Identifier.is_in(identifiers)),
        )
        .into_tuple::<(String, Vec<u8>)>()
        .all(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    result
        .into_iter()
        .map(|(identifier, private_key_der)| {
            let signing_key =
                SigningKey::from_slice(&private_key_der).map_err(PersistenceError::SigningKeyConversion)?;
            Ok((identifier, signing_key))
        })
        .collect()
}
