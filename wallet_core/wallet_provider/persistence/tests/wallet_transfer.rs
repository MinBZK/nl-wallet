use assert_matches::assert_matches;
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use semver::Version;
use uuid::Uuid;

use crypto::utils::random_string;
use db_test::DbSetup;
use wallet_account::messages::transfer::TransferSessionState;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_persistence::PersistenceConnection;
use wallet_provider_persistence::entity::wallet_transfer;
use wallet_provider_persistence::test::WalletDeviceVendor;
use wallet_provider_persistence::wallet_transfer::create_transfer_session;
use wallet_provider_persistence::wallet_transfer::find_transfer_session_by_transfer_session_id;
use wallet_provider_persistence::wallet_transfer::find_transfer_session_id_by_destination_wallet_user_id;
use wallet_provider_persistence::wallet_transfer::set_wallet_transfer_data;
use wallet_provider_persistence::wallet_transfer::update_transfer_state;
use wallet_provider_persistence::wallet_user::store_recovery_code;

pub mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_transfer_session() {
    let db_setup = DbSetup::create().await;
    let (db, wallet_user_id, wallet_id, _) = common::create_test_user(&db_setup, WalletDeviceVendor::Apple).await;

    store_recovery_code(&db, &wallet_id, random_string(64).into())
        .await
        .expect("storing the recovery code should succeed");

    let transfer_session_id = Uuid::new_v4();
    let destination_wallet_app_version = Version::parse("1.0.0").unwrap();

    create_transfer_session(
        &db,
        wallet_user_id,
        transfer_session_id,
        destination_wallet_app_version.clone(),
        Utc::now(),
    )
    .await
    .unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(wallet_user_id, transfer_session.destination_wallet_user_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_find_transfer_session_by_transfer_session_id() {
    let db_setup = DbSetup::create().await;
    let (db, wallet_user_id, wallet_id, _) = common::create_test_user(&db_setup, WalletDeviceVendor::Apple).await;

    store_recovery_code(&db, &wallet_id, random_string(64).into())
        .await
        .expect("storing the recovery code should succeed");

    let transfer_session_id = Uuid::new_v4();
    let destination_wallet_app_version = Version::parse("1.2.3").unwrap();

    create_transfer_session(
        &db,
        wallet_user_id,
        transfer_session_id,
        destination_wallet_app_version.clone(),
        Utc::now(),
    )
    .await
    .unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(transfer_session_id, transfer_session.transfer_session_id);
    assert_eq!(
        destination_wallet_app_version,
        transfer_session.destination_wallet_app_version
    );

    assert!(
        find_transfer_session_by_transfer_session_id(&db, Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_find_transfer_session_id_by_destination_wallet_user_id() {
    let db_setup = DbSetup::create().await;
    let (db, wallet_user_id, _, _) = common::create_test_user(&db_setup, WalletDeviceVendor::Apple).await;

    let transfer_session_id = Uuid::new_v4();

    create_transfer_session(
        &db,
        wallet_user_id,
        transfer_session_id,
        Version::parse("1.2.3").unwrap(),
        Utc::now(),
    )
    .await
    .unwrap();

    let stored_transfer_session_id = find_transfer_session_id_by_destination_wallet_user_id(&db, wallet_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(stored_transfer_session_id, transfer_session_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_update_transfer_state() {
    let db_setup = DbSetup::create().await;
    let (db, wallet_user_id, wallet_id, _) = common::create_test_user(&db_setup, WalletDeviceVendor::Apple).await;

    store_recovery_code(&db, &wallet_id, random_string(64).into())
        .await
        .expect("storing the recovery code should succeed");

    let transfer_session_id = Uuid::new_v4();
    let destination_wallet_app_version = Version::parse("1.2.3").unwrap();

    create_transfer_session(
        &db,
        wallet_user_id,
        transfer_session_id,
        destination_wallet_app_version.clone(),
        Utc::now(),
    )
    .await
    .unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(transfer_session_id, transfer_session.transfer_session_id);
    assert_eq!(transfer_session.state, TransferSessionState::Created);

    update_transfer_state(&db, transfer_session_id, TransferSessionState::Paired)
        .await
        .unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(transfer_session_id, transfer_session.transfer_session_id);
    assert_eq!(transfer_session.state, TransferSessionState::Paired);

    let err = update_transfer_state(&db, Uuid::new_v4(), TransferSessionState::Success)
        .await
        .expect_err("Updating a non-existing transfer session should fail");
    assert_matches!(err, PersistenceError::NoRowsUpdated);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_set_wallet_transfer_data() {
    let db_setup = DbSetup::create().await;
    let (db, wallet_user_id, wallet_id, _) = common::create_test_user(&db_setup, WalletDeviceVendor::Apple).await;

    store_recovery_code(&db, &wallet_id, random_string(64).into())
        .await
        .expect("storing the recovery code should succeed");

    let transfer_session_id = Uuid::new_v4();

    wallet_transfer::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_wallet_user_id: Set(None),
        destination_wallet_user_id: Set(wallet_user_id),
        transfer_session_id: Set(transfer_session_id),
        destination_wallet_app_version: Set("1.2.3".to_string()),
        state: Set(TransferSessionState::Created.to_string()),
        created: Set(Utc::now().into()),
        encrypted_wallet_data: Set(Some(random_string(128))),
    }
    .insert(db.connection())
    .await
    .unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();

    assert!(transfer_session.encrypted_wallet_data.is_some());

    set_wallet_transfer_data(&db, transfer_session_id, None).await.unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();

    assert!(transfer_session.encrypted_wallet_data.is_none());

    set_wallet_transfer_data(&db, transfer_session_id, Some(random_string(32)))
        .await
        .unwrap();

    let transfer_session = find_transfer_session_by_transfer_session_id(&db, transfer_session_id)
        .await
        .unwrap()
        .unwrap();

    assert!(transfer_session.encrypted_wallet_data.is_some());

    let err = set_wallet_transfer_data(&db, Uuid::new_v4(), None)
        .await
        .expect_err("Updating a non-existing transfer session should fail");
    assert_matches!(err, PersistenceError::NoRowsUpdated);
}
