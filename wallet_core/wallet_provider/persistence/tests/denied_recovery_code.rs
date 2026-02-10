use crypto::utils::random_string;
use wallet_provider_persistence::denied_recovery_code;
use wallet_provider_persistence::test::WalletDeviceVendor;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_persistence::test::db_from_env;
use wallet_provider_persistence::wallet_user::find_wallet_user_by_wallet_id;
use wallet_provider_persistence::wallet_user::store_recovery_code;

#[tokio::test]
async fn test_insert_denied_recovery_code() {
    let recovery_code = random_string(64);

    let db = db_from_env().await.expect("Could not connect to database");

    // verify it does not exist before insertion
    let exists = denied_recovery_code::exists(&db, &recovery_code).await.unwrap();
    assert!(!exists);

    denied_recovery_code::insert(&db, recovery_code.clone())
        .await
        .expect("should be able to insert denied recovery code");

    let exists = denied_recovery_code::exists(&db, &recovery_code).await.unwrap();
    assert!(exists);

    // verify idempotency
    denied_recovery_code::insert(&db, recovery_code.clone())
        .await
        .expect("should be able to insert denied recovery code");
    let exists = denied_recovery_code::exists(&db, &recovery_code).await.unwrap();
    assert!(exists);

    // there's no minimum length
    let small = random_string(1);
    denied_recovery_code::insert(&db, small.clone())
        .await
        .expect("should be able to insert denied recovery code");

    let exists = denied_recovery_code::exists(&db, &small).await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn test_recovery_code_on_deny_list() {
    let db = db_from_env().await.expect("Could not connect to database");

    let wallet_id1 = random_string(32);
    let wallet_id2 = random_string(32);

    create_wallet_user_with_random_keys(&db, WalletDeviceVendor::Apple, wallet_id1.clone()).await;
    create_wallet_user_with_random_keys(&db, WalletDeviceVendor::Apple, wallet_id2.clone()).await;

    let recovery_code = random_string(64);
    store_recovery_code(&db, &wallet_id1, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");

    // before adding to deny list it should be false
    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id1)
        .await
        .unwrap()
        .unwrap_found();
    assert!(!wallet_user.recovery_code_on_deny_list);

    // add recovery code to deny list
    denied_recovery_code::insert(&db, recovery_code.clone())
        .await
        .expect("insert deny");

    // after insertion it should be true
    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id1)
        .await
        .unwrap()
        .unwrap_found();
    assert!(wallet_user.recovery_code_on_deny_list);

    // other user remains false (has no recovery code)
    let other = find_wallet_user_by_wallet_id(&db, &wallet_id2)
        .await
        .unwrap()
        .unwrap_found();
    assert!(!other.recovery_code_on_deny_list);
}
