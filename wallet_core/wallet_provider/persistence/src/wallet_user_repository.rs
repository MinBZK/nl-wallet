use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait};

use wallet_provider_domain::{model::wallet_user::WalletUserCreate, repository::PersistenceError};

use crate::{entity::wallet_user, PersistenceConnection};

pub async fn create_wallet_user<S, T>(db: &T, user: WalletUserCreate) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::ActiveModel {
        id: Set(user.id),
        wallet_id: Set(user.wallet_id),
        hw_pubkey: Set(user.hw_pubkey),
    }
    .insert(db.connection())
    .await
    .map(|_| ())
    .map_err(|e| PersistenceError::Execution(e.into()))
}
