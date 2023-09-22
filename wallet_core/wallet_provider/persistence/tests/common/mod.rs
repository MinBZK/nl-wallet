use std::env;

use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use tokio::sync::OnceCell;
use uuid::Uuid;

use wallet_provider_domain::{model::wallet_user::WalletUserCreate, repository::PersistenceError};
use wallet_provider_persistence::{
    database::Db, entity::wallet_user, wallet_user::create_wallet_user, PersistenceConnection,
};

static DB: OnceCell<Db> = OnceCell::const_new();

pub async fn db_from_env() -> Result<&'static Db, PersistenceError> {
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish(),
    );

    DB.get_or_try_init(|| async {
        Db::new(
            &env::var("WALLET_PROVIDER_DATABASE__HOST").unwrap_or("localhost".to_string()),
            &env::var("WALLET_PROVIDER_DATABASE__NAME").unwrap_or("wallet_provider".to_string()),
            Some(&env::var("WALLET_PROVIDER_DATABASE__USERNAME").unwrap_or("postgres".to_string())),
            Some(&env::var("WALLET_PROVIDER_DATABASE__PASSWORD").unwrap_or("postgres".to_string())),
        )
        .await
    })
    .await
}

pub async fn create_wallet_user_with_random_keys<S, T>(db: &T, id: Uuid, wallet_id: String)
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    create_wallet_user(
        db,
        WalletUserCreate {
            id,
            wallet_id,
            hw_pubkey: *SigningKey::random(&mut OsRng).verifying_key(),
            pin_pubkey: *SigningKey::random(&mut OsRng).verifying_key(),
        },
    )
    .await
    .expect("Could not create wallet user");
}

pub async fn find_wallet_user<S, T>(db: &T, id: Uuid) -> Option<wallet_user::Model>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::find()
        .filter(wallet_user::Column::Id.eq(id))
        .one(db.connection())
        .await
        .expect("Could not fetch wallet user")
}
