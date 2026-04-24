use std::sync::Arc;

use chrono::DateTime;
use db_test::DbSetup;
use db_test::connection_from_url;
use issuer_common::nonce_store::ProofNonceStore;
use openid4vc::nonce::store::test::test_nonce_store;
use sea_orm::ConnectionTrait;
use sea_orm::DbBackend;
use sea_orm::Statement;
use utils::generator::mock::MockTimeGenerator;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_proof_nonce_store() {
    let db_setup = DbSetup::create().await;
    let database_connection = connection_from_url(db_setup.issuer_common_url()).await;

    let time_generator = MockTimeGenerator::new(DateTime::from_timestamp_secs(1_000_000_000).unwrap());
    let mock_time = Arc::clone(&time_generator.time);

    let store = ProofNonceStore::new_postgres_with_time_generator(database_connection.clone(), time_generator);

    test_nonce_store(store, mock_time, async |_store| {
        database_connection
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT COUNT(*) FROM "proof_nonce""#,
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index::<i64>(0)
            .unwrap()
            .try_into()
            .unwrap()
    })
    .await
}
