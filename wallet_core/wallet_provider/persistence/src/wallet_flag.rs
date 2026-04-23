use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;
use wallet_provider_domain::model::wallet_flag::WalletFlag;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_flag;

pub async fn list_wallet_flags<S, T>(db: &T) -> Result<Vec<(WalletFlag, bool)>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    Ok(wallet_flag::Entity::find()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .into_iter()
        .map(|model| {
            (
                WalletFlag::try_from(model.name.as_str()).expect("invalid flag name"),
                model.value,
            )
        })
        .collect())
}

pub async fn get_wallet_flag<S, T>(db: &T, name: WalletFlag) -> Result<bool, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    Ok(wallet_flag::Entity::find()
        .select_only()
        .column(wallet_flag::Column::Value)
        .filter(wallet_flag::Column::Name.eq(name.to_string()))
        .into_tuple::<bool>()
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .unwrap_or(false))
}

pub async fn set_wallet_flag<S, T>(db: &T, name: WalletFlag, value: bool) -> Result<(), PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = wallet_flag::ActiveModel {
        name: Set(name.to_string()),
        value: Set(value),
    };
    wallet_flag::Entity::insert(model)
        .on_conflict(
            OnConflict::column(wallet_flag::Column::Name)
                .update_column(wallet_flag::Column::Value)
                .to_owned(),
        )
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;
    Ok(())
}
