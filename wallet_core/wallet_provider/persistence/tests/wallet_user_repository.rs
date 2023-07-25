use std::env;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serial_test::serial;
use tokio::sync::OnceCell;
use uuid::Uuid;

use wallet_provider_domain::{
    generator::Generator,
    model::wallet_user::WalletUserCreate,
    repository::{Committable, PersistenceError},
    EpochGenerator,
};
use wallet_provider_persistence::{
    database::Db,
    entity::wallet_user,
    transaction,
    wallet_user_repository::{create_wallet_user, register_unsuccessful_pin_entry},
    PersistenceConnection,
};

static DB: OnceCell<Db> = OnceCell::const_new();

async fn db_from_env() -> Result<&'static Db, PersistenceError> {
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

async fn find_wallet_user<S, T>(db: &T, id: Uuid) -> Option<wallet_user::Model>
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

async fn do_create_wallet_user<S, T>(db: &T, id: Uuid, wallet_id: String)
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    create_wallet_user(
        db,
        WalletUserCreate {
            id,
            wallet_id,
            hw_pubkey_der: "hw_pubkey".as_bytes().to_vec(),
            pin_pubkey_der: "pin_pubkey".as_bytes().to_vec(),
        },
    )
    .await
    .expect("Could not create wallet user");
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user() {
    let db = db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    do_create_wallet_user(db, wallet_user_id, wallet_id.clone()).await;

    let wallet_user = find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
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
    let wallet_id = Uuid::new_v4().to_string();

    do_create_wallet_user(&transaction, wallet_user_id, wallet_id.clone()).await;

    let maybe_wallet_user = find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());

    transaction
        .commit()
        .await
        .expect("Could not commit wallet user transaction");

    let wallet_user = find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_rollback() {
    let db = db_from_env().await.expect("Could not connect to database");
    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    {
        let transaction = transaction::begin_transaction(db)
            .await
            .expect("Could not begin transaction");

        do_create_wallet_user(&transaction, wallet_user_id, wallet_id).await;
    }

    let maybe_wallet_user = find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_register_unsuccessful_pin_entry() {
    let db = db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    do_create_wallet_user(db, wallet_user_id, wallet_id.clone()).await;

    let before = find_wallet_user(db, wallet_user_id).await.unwrap();
    assert!(before.last_unsuccessful_pin.is_none());

    register_unsuccessful_pin_entry(db, &wallet_id, false, EpochGenerator.generate())
        .await
        .expect("Could register unsuccessful pin entry");

    let after = find_wallet_user(db, wallet_user_id).await.unwrap();

    assert_eq!(before.pin_entries + 1, after.pin_entries);
    assert_eq!(EpochGenerator.generate(), after.last_unsuccessful_pin.unwrap());
}
