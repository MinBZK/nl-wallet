use crypto::utils::random_string;
use wallet_provider_persistence::denied_recovery_code;
use wallet_provider_persistence::test::db_from_env;

#[tokio::test]
async fn test_insert_denied_recovery_code() {
    let recovery_code = random_string(64);

    let db = db_from_env().await.expect("Could not connect to database");

    denied_recovery_code::insert(&db, recovery_code.clone())
        .await
        .expect("should be able to insert denied recovery code");

    // verify idempotency
    denied_recovery_code::insert(&db, recovery_code)
        .await
        .expect("should be able to insert denied recovery code");

    // there's no minimum length
    denied_recovery_code::insert(&db, random_string(1))
        .await
        .expect("should be able to insert denied recovery code");
}
