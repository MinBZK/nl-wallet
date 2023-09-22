use serial_test::serial;
use uuid::Uuid;

use wallet_common::generator::Generator;
use wallet_provider_domain::{repository::Committable, EpochGenerator};
use wallet_provider_persistence::{transaction, wallet_user::register_unsuccessful_pin_entry};

pub mod common;

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    common::create_wallet_user_with_random_keys(db, wallet_user_id, wallet_id.clone()).await;

    let wallet_user = common::find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_commit() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let transaction = transaction::begin_transaction(db)
        .await
        .expect("Could not begin transaction");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    common::create_wallet_user_with_random_keys(&transaction, wallet_user_id, wallet_id.clone()).await;

    let maybe_wallet_user = common::find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());

    transaction
        .commit()
        .await
        .expect("Could not commit wallet user transaction");

    let wallet_user = common::find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_rollback() {
    let db = common::db_from_env().await.expect("Could not connect to database");
    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    {
        let transaction = transaction::begin_transaction(db)
            .await
            .expect("Could not begin transaction");

        common::create_wallet_user_with_random_keys(&transaction, wallet_user_id, wallet_id).await;
    }

    let maybe_wallet_user = common::find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_register_unsuccessful_pin_entry() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    common::create_wallet_user_with_random_keys(db, wallet_user_id, wallet_id.clone()).await;

    let before = common::find_wallet_user(db, wallet_user_id).await.unwrap();
    assert!(before.last_unsuccessful_pin.is_none());

    register_unsuccessful_pin_entry(db, &wallet_id, false, EpochGenerator.generate())
        .await
        .expect("Could register unsuccessful pin entry");

    let after = common::find_wallet_user(db, wallet_user_id).await.unwrap();

    assert_eq!(before.pin_entries + 1, after.pin_entries);
    assert_eq!(EpochGenerator.generate(), after.last_unsuccessful_pin.unwrap());
}
