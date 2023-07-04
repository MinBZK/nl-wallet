use std::env;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serial_test::serial;
use tokio::sync::OnceCell;
use uuid::Uuid;

use wallet_provider_domain::{
    model::wallet_user::WalletUserCreate,
    repository::{Committable, PersistenceError},
};
use wallet_provider_persistence::{
    database::{Db, PersistenceConnection},
    entity::wallet_user,
    transaction,
    wallet_user_repository::create_wallet_user,
};

static DB: OnceCell<Db> = OnceCell::const_new();

async fn db_from_env() -> Result<&'static Db, PersistenceError> {
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

async fn find_wallet_user(db: &Db, id: Uuid) -> Option<wallet_user::Model> {
    wallet_user::Entity::find()
        .filter(wallet_user::Column::Id.eq(id))
        .one(db.connection())
        .await
        .expect("Could not fetch wallet user")
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user() {
    let db = db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    create_wallet_user(
        db,
        WalletUserCreate {
            id: wallet_user_id,
            wallet_id: "wallet123".to_string(),
            hw_pubkey: "pubkey".to_string(),
        },
    )
    .await
    .expect("Could not create wallet user");

    let wallet_user = find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!("wallet123", wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_commit() {
    let db = db_from_env().await.expect("Could not connect to database");

    let transaction = transaction::begin_transaction(db)
        .await
        .expect("Could not begin transaction");

    let wallet_user_id = Uuid::new_v4();
    create_wallet_user(
        &transaction,
        WalletUserCreate {
            id: wallet_user_id,
            wallet_id: "wallet456".to_string(),
            hw_pubkey: "pubkey".to_string(),
        },
    )
    .await
    .expect("Could not create wallet user with transaction");

    let maybe_wallet_user = find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());

    transaction
        .commit()
        .await
        .expect("Could not commit wallet user transaction");

    let wallet_user = find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!("wallet456", wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_rollback() {
    let db = db_from_env().await.expect("Could not connect to database");
    let wallet_user_id = Uuid::new_v4();

    {
        let transaction = transaction::begin_transaction(db)
            .await
            .expect("Could not begin transaction");

        create_wallet_user(
            &transaction,
            WalletUserCreate {
                id: wallet_user_id,
                wallet_id: "wallet456".to_string(),
                hw_pubkey: "pubkey".to_string(),
            },
        )
        .await
        .expect("Could not create wallet user with transaction");
    }

    let maybe_wallet_user = find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());
}
