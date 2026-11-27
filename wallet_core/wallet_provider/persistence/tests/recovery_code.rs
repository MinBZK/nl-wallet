use std::collections::HashSet;
use std::sync::Arc;

use crypto::utils::random_string;
use db_test::DbSetup;
use rstest::rstest;
use tokio::time::Duration;
use tokio::time::sleep;
use utils::vec_at_least::VecNonEmptyUnique;
use utils::vec_nonempty;
use wallet_provider_domain::model::wallet_user::RecoveryCode;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::repository::Committable;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::recovery_code;
use wallet_provider_persistence::test::WalletDeviceVendor;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_persistence::test::db_from_setup;
use wallet_provider_persistence::transaction;
use wallet_provider_persistence::wallet_user::find_wallet_user_by_wallet_id;
use wallet_provider_persistence::wallet_user::store_recovery_code;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_insert_recovery_code() {
    let recovery_code: RecoveryCode = random_string(64).into();

    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    // verify it does not exist before insertion
    let is_denied = recovery_code::is_denied(&db, recovery_code.clone()).await.unwrap();
    assert!(!is_denied);

    recovery_code::deny_many(&db, &vec_nonempty![recovery_code.clone()].into())
        .await
        .expect("should be able to insert recovery code");

    let is_denied = recovery_code::is_denied(&db, recovery_code.clone()).await.unwrap();
    assert!(is_denied);

    // verify idempotency
    recovery_code::deny_many(&db, &vec_nonempty![recovery_code.clone()].into())
        .await
        .expect("should be able to insert recovery code");
    let is_denied = recovery_code::is_denied(&db, recovery_code).await.unwrap();
    assert!(is_denied);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_deny_many_inserts_and_updates_recovery_codes() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let new_code: RecoveryCode = random_string(64).into();
    let allowed_code: RecoveryCode = random_string(64).into();
    let denied_code: RecoveryCode = random_string(64).into();
    let unrelated_code: RecoveryCode = random_string(64).into();

    assert!(!recovery_code::is_denied(&db, allowed_code.clone()).await.unwrap());
    assert!(!recovery_code::is_denied(&db, unrelated_code.clone()).await.unwrap());
    recovery_code::deny_many(&db, &vec_nonempty![denied_code.clone()].into())
        .await
        .unwrap();
    assert!(!recovery_code::find_is_denied(&db, &new_code).await.unwrap());

    let codes = vec_nonempty![new_code, denied_code, allowed_code].into();
    recovery_code::deny_many(&db, &codes).await.unwrap();

    for code in &codes {
        assert!(recovery_code::find_is_denied(&db, code).await.unwrap());
    }
    assert!(!recovery_code::find_is_denied(&db, &unrelated_code).await.unwrap(),);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_recovery_code_is_denied() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let wallet_id1: WalletId = random_string(32).into();
    let wallet_id2: WalletId = random_string(32).into();

    create_wallet_user_with_random_keys(&db, WalletDeviceVendor::Apple, wallet_id1.clone()).await;
    create_wallet_user_with_random_keys(&db, WalletDeviceVendor::Apple, wallet_id2.clone()).await;

    let recovery_code: RecoveryCode = random_string(64).into();
    store_recovery_code(&db, &wallet_id1, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");

    // before denying the recovery code it should be false
    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id1)
        .await
        .unwrap()
        .unwrap_found();
    assert!(!wallet_user.recovery_code_is_denied);

    // set recovery code to denied
    recovery_code::deny_many(&db, &vec_nonempty![recovery_code.clone()].into())
        .await
        .expect("insert deny");

    // after insertion it should be true
    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id1)
        .await
        .unwrap()
        .unwrap_found();
    assert!(wallet_user.recovery_code_is_denied);

    // other user remains false (has no recovery code)
    let other = find_wallet_user_by_wallet_id(&db, &wallet_id2)
        .await
        .unwrap()
        .unwrap_found();
    assert!(!other.recovery_code_is_denied);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_find_recovery_code_is_denied() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let recovery_code: RecoveryCode = random_string(64).into();

    // unknown recovery code has no record, and does not create one
    let is_denied = recovery_code::find_is_denied(&db, &recovery_code).await.unwrap();
    assert!(!is_denied);
    let is_denied = recovery_code::find_is_denied(&db, &recovery_code).await.unwrap();
    assert!(!is_denied);

    // a recovery code that is not denied has a record with `is_denied` false
    recovery_code::is_denied(&db, recovery_code.clone()).await.unwrap();
    let is_denied = recovery_code::find_is_denied(&db, &recovery_code).await.unwrap();
    assert!(!is_denied);

    // denied recovery code
    recovery_code::deny_many(&db, &vec_nonempty![recovery_code.clone()].into())
        .await
        .unwrap();
    let is_denied = recovery_code::find_is_denied(&db, &recovery_code).await.unwrap();
    assert!(is_denied);
}

/// This test attempts to expose a race condition / non-repeatable read:
/// - One task begins a transaction and reads whether a recovery code is denied twice within the same transaction, with
///   a sleep in between reads.
/// - Another task inserts the recovery code as denied into the table (outside the transaction) while the first task is
///   sleeping.
/// The test fails if the two reads within the same transaction return different results for the same recovery code. It
/// will not fail if the database provides repeatable reads, but should also not fail if the database provides only read
/// committed isolation level, which is the default isolation level for Postgres.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_recovery_code_repeatable_reads() {
    let db_setup = DbSetup::create().await;
    let db = Arc::new(db_from_setup(&db_setup).await);

    let recovery_code: RecoveryCode = random_string(64).into();

    // Ensure the code is not present before starting the test
    let is_denied = recovery_code::is_denied(&*db, recovery_code.clone()).await.unwrap();
    assert!(!is_denied, "Recovery code should not exist before the test starts");

    // Spawn the reader task which starts a transaction and does two reads with a sleep in between.
    let db_clone = Arc::clone(&db);
    let reader_recovery_code = recovery_code.clone();
    let reader = tokio::spawn(async move {
        let tx = transaction::begin_transaction(&db_clone)
            .await
            .expect("could not begin transaction");

        // First read inside transaction
        let first = recovery_code::is_denied(&tx, reader_recovery_code.clone())
            .await
            .unwrap();

        // Sleep to allow the inserter to run
        sleep(Duration::from_millis(100)).await;

        // Second read inside the same transaction
        let second = recovery_code::is_denied(&tx, reader_recovery_code).await.unwrap();

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

        recovery_code::deny_many(&tx, &vec_nonempty![recovery_code.clone()].into())
            .await
            .unwrap();

        tx.commit().await.expect("could not commit transaction");
    });

    // Wait for both tasks to complete
    let (reader_res, inserter_res) = tokio::join!(reader, inserter);
    let (first, second) = reader_res.expect("reader task should not panic");
    inserter_res.expect("inserter task should not panic");

    // The test should fail if the two reads differ
    assert_eq!(first, second, "Detected non-repeatable read for the same recovery code",);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_remove_recovery_code() {
    let recovery_code = RecoveryCode::from(random_string(64));

    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let single_recovery_code = vec_nonempty![recovery_code.clone()].into();
    recovery_code::set_allowed_many(&db, &single_recovery_code)
        .await
        .expect("should be able to set non-denied recovery code to allowed");

    recovery_code::deny_many(&db, &single_recovery_code)
        .await
        .expect("should be able to insert denied recovery code");

    let is_denied = recovery_code::is_denied(&db, recovery_code.clone()).await.unwrap();
    assert!(is_denied);

    recovery_code::set_allowed_many(&db, &single_recovery_code)
        .await
        .expect("should be able to set denied recovery code to allowed");

    let is_denied = recovery_code::is_denied(&db, recovery_code.clone()).await.unwrap();
    assert!(!is_denied);

    // verify idempotency
    recovery_code::set_allowed_many(&db, &single_recovery_code)
        .await
        .expect("should be able to set non-denied recovery code to allowed");

    let is_denied = recovery_code::is_denied(&db, recovery_code).await.unwrap();
    assert!(!is_denied);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_recovery_code() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let recovery_code: RecoveryCode = random_string(64).into();
    let recovery_codes = recovery_code::list(&db)
        .await
        .expect("should be able to list denied recovery code");

    assert!(!recovery_codes.contains(&recovery_code));

    recovery_code::deny_many(&db, &vec_nonempty![recovery_code.clone()].into())
        .await
        .expect("should be able to insert denied recovery code");

    let recovery_codes = recovery_code::list(&db)
        .await
        .expect("should be able to list denied recovery code");

    assert!(recovery_codes.contains(&recovery_code));

    // inserting another should result in both being listed
    let another_recovery_code: RecoveryCode = random_string(64).into();
    recovery_code::deny_many(&db, &vec_nonempty![another_recovery_code.clone()].into())
        .await
        .expect("should be able to insert denied recovery code");

    let recovery_codes = recovery_code::list(&db)
        .await
        .expect("should be able to list denied recovery code");

    assert!(
        [&recovery_code, &another_recovery_code]
            .iter()
            .all(|code| recovery_codes.contains(code))
    );

    // removing the first should result in only the second being listed
    recovery_code::set_allowed_many(&db, &vec_nonempty![recovery_code.clone()].into())
        .await
        .expect("should be able to set denied recovery code to allowed");

    let recovery_codes = recovery_code::list(&db)
        .await
        .expect("should be able to list denied recovery code");
    assert!(!recovery_codes.contains(&recovery_code));
    assert!(recovery_codes.contains(&another_recovery_code));

    // removing the second should result in an empty list
    recovery_code::set_allowed_many(&db, &vec_nonempty![another_recovery_code.clone()].into())
        .await
        .expect("should be able to set another denied recovery code to allowed");

    let recovery_codes = recovery_code::list(&db)
        .await
        .expect("should be able to list denied recovery code");

    assert!(
        [&recovery_code, &another_recovery_code]
            .iter()
            .all(|code| !recovery_codes.contains(code))
    );
}

/// The denial state of a recovery code in the database, for tests of batch operations.
#[derive(Debug, Clone, Copy)]
enum CodeState {
    /// Stored and denied.
    Denied,
    /// Stored and allowed.
    Allowed,
    /// Not stored.
    Unknown,
}

/// Returns a new recovery code in `state`.
async fn create_recovery_code(db: &Db, state: CodeState) -> RecoveryCode {
    let recovery_code: RecoveryCode = random_string(64).into();
    match state {
        CodeState::Denied => {
            recovery_code::deny_many(db, &vec_nonempty![recovery_code.clone()].into())
                .await
                .expect("should insert denied recovery code");
        }
        CodeState::Allowed => {
            // Since recovery_code is just created as a random string, it should not yet exist before, so this will
            // allow it.
            recovery_code::is_denied(db, recovery_code.clone())
                .await
                .expect("should insert recovery code");
        }
        CodeState::Unknown => {}
    }
    recovery_code
}

#[rstest]
#[case::only_denied(&[CodeState::Denied])]
#[case::mixed(&[CodeState::Denied, CodeState::Allowed, CodeState::Unknown, CodeState::Denied])]
#[case::none_denied(&[CodeState::Allowed, CodeState::Unknown])]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_all_denied_in_returns_denied_codes_of_input(#[case] states: &[CodeState]) {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let mut input = Vec::new();
    let mut expected = HashSet::new();
    for state in states {
        let recovery_code = create_recovery_code(&db, *state).await;
        if matches!(state, CodeState::Denied) {
            expected.insert(recovery_code.clone());
        }
        input.push(recovery_code);
    }

    let denied = recovery_code::find_denied_recovery_codes(&db, &VecNonEmptyUnique::try_from(input).unwrap())
        .await
        .expect("should list denied recovery codes");

    assert_eq!(denied, expected);
}

#[rstest]
#[case::denied_codes(&[CodeState::Denied, CodeState::Denied], 2)]
#[case::unknown_code_is_not_counted(&[CodeState::Denied, CodeState::Unknown], 1)]
#[case::already_allowed_code_is_not_counted(&[CodeState::Denied, CodeState::Allowed], 1)]
#[case::only_unknown_codes(&[CodeState::Unknown], 0)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_set_allowed_many_allows_requested_codes_only(#[case] states: &[CodeState], #[case] expected_count: u64) {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    // A denied code that is not requested must stay denied.
    let not_requested = create_recovery_code(&db, CodeState::Denied).await;

    let mut requested = vec![];
    for state in states {
        requested.push(create_recovery_code(&db, *state).await);
    }

    let count = recovery_code::set_allowed_many(&db, &VecNonEmptyUnique::try_from(requested.clone()).unwrap())
        .await
        .expect("should allow recovery codes");

    // The count is the number of changed records, so already allowed codes must not be counted.
    assert_eq!(count, expected_count);
    for recovery_code in &requested {
        let is_denied = recovery_code::is_denied(&db, recovery_code.clone())
            .await
            .expect("should query recovery code");
        assert!(!is_denied, "requested recovery code should be allowed");
    }
    let is_denied = recovery_code::is_denied(&db, not_requested)
        .await
        .expect("should query recovery code");
    assert!(is_denied, "recovery code that was not requested should stay denied");
}
