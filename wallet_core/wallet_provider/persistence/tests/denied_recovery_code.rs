use std::sync::Arc;

use crypto::utils::random_string;
use tokio::time::Duration;
use tokio::time::sleep;

use wallet_provider_domain::repository::Committable;
use wallet_provider_persistence::denied_recovery_code;
use wallet_provider_persistence::test::WalletDeviceVendor;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_persistence::test::db_from_env;
use wallet_provider_persistence::transaction;
use wallet_provider_persistence::wallet_user::find_wallet_user_by_wallet_id;
use wallet_provider_persistence::wallet_user::store_recovery_code;

#[tokio::test]
async fn test_insert_denied_recovery_code() {
    let recovery_code = random_string(64);

    let db = db_from_env().await.expect("Could not connect to database");

    // verify it does not exist before insertion
    let exists = denied_recovery_code::exists(&db, recovery_code.clone()).await.unwrap();
    assert!(!exists);

    denied_recovery_code::insert(&db, recovery_code.clone())
        .await
        .expect("should be able to insert denied recovery code");

    let exists = denied_recovery_code::exists(&db, recovery_code.clone()).await.unwrap();
    assert!(exists);

    // verify idempotency
    denied_recovery_code::insert(&db, recovery_code.clone())
        .await
        .expect("should be able to insert denied recovery code");
    let exists = denied_recovery_code::exists(&db, recovery_code).await.unwrap();
    assert!(exists);

    // there's no minimum length
    let small = random_string(1);
    denied_recovery_code::insert(&db, small.clone())
        .await
        .expect("should be able to insert denied recovery code");

    let exists = denied_recovery_code::exists(&db, small).await.unwrap();
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

/// This test attempts to expose a race condition / non-repeatable read:
/// - One task begins a transaction and reads whether a recovery code is on the deny list twice within the same
///   transaction, with a sleep in between reads.
/// - Another task inserts the recovery code into the deny list (outside the transaction) while the first task is
///   sleeping.
/// The test fails if the two reads within the same transaction return different results for the same recovery code. It
/// will not fail if the database provides repeatable reads, but should also not fail if the database provides only read
/// committed isolation level, which is the default isolation level for Postgres.
#[tokio::test]
async fn test_denied_recovery_code_repeatable_reads() {
    let db = db_from_env().await.expect("Could not connect to database");
    let db = Arc::new(db);

    let recovery_code = random_string(64);

    // Ensure the code is not present before starting the test
    let exists = denied_recovery_code::exists(&*db, recovery_code.clone()).await.unwrap();
    assert!(!exists, "Recovery code should not exist before the test starts");

    // Spawn the reader task which starts a transaction and does two reads with a sleep in between.
    let db_clone = Arc::clone(&db);
    let reader_recovery_code = recovery_code.clone();
    let reader = tokio::spawn(async move {
        let tx = transaction::begin_transaction(&db_clone)
            .await
            .expect("could not begin transaction");

        // First read inside transaction
        let first = denied_recovery_code::exists(&tx, reader_recovery_code.clone())
            .await
            .unwrap();

        // Sleep to allow the inserter to run
        sleep(Duration::from_millis(100)).await;

        // Second read inside the same transaction
        let second = denied_recovery_code::exists(&tx, reader_recovery_code).await.unwrap();

        tx.commit().await.expect("could not commit transaction");

        (first, second)
    });

    // Spawn the inserter task which will wait a bit and then insert the recovery code
    let inserter = tokio::spawn(async move {
        let tx = transaction::begin_transaction(&db)
            .await
            .expect("could not begin transaction");

        // Ensure we run the insert while the reader is sleeping between reads.
        sleep(Duration::from_millis(50)).await;

        denied_recovery_code::insert(&tx, recovery_code).await.unwrap();

        tx.commit().await.expect("could not commit transaction");
    });

    // Wait for both tasks to complete
    let (reader_res, inserter_res) = tokio::join!(reader, inserter);
    let (first, second) = reader_res.expect("reader task should not panic");
    inserter_res.expect("inserter task should not panic");

    // The test should fail if the two reads differ
    assert_eq!(first, second, "Detected non-repeatable read for the same recovery code",);
}
