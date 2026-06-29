use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use db_test::DbSetup;
use db_test::connection_from_url;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use openid4vc::store::Store;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbBackend;
use sea_orm::Statement;
use utils::generator::mock::MockTimeGenerator;

/// Verify that `cleanup` removes only expired rows and drains backlogs larger than a single batch
/// (the `FOR UPDATE SKIP LOCKED` loop), leaving still-valid rows untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_issuer_state_bridge_store_cleanup_drains_in_batches() {
    let db_setup = DbSetup::create().await;
    let database_connection = connection_from_url(db_setup.pid_issuer_url()).await;

    let now = DateTime::from_timestamp_secs(1_000_000_000).unwrap();
    let time_generator = MockTimeGenerator::new(now);
    let store = IssuerStateBridgeStore::<serde_json::Value, _>::new_postgres_with_time_generator(
        database_connection.clone(),
        time_generator,
    );

    // More than one batch (CLEANUP_BATCH_SIZE == 1000) worth of expired rows, plus a few valid ones.
    let expired_count: i64 = 1_005;
    let valid_count: i64 = 3;

    insert_rows(
        &database_connection,
        "expired",
        expired_count,
        now - Duration::seconds(1),
    )
    .await;
    insert_rows(&database_connection, "valid", valid_count, now + Duration::hours(1)).await;
    assert_eq!(
        count_rows(&database_connection).await,
        (expired_count + valid_count) as usize
    );

    store.cleanup().await.unwrap();

    assert_eq!(count_rows(&database_connection).await, valid_count as usize);
}

async fn insert_rows(connection: &DatabaseConnection, prefix: &str, count: i64, expires_at: DateTime<Utc>) {
    connection
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO state_bridge (bridge_key, entry, expires_at)
            SELECT $1 || '-' || g, '{}'::jsonb, $2
            FROM generate_series(1, $3::bigint) AS g
            "#,
            [prefix.into(), expires_at.into(), count.into()],
        ))
        .await
        .unwrap();
}

async fn count_rows(connection: &DatabaseConnection) -> usize {
    connection
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            r#"SELECT COUNT(*) FROM "state_bridge""#,
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<i64>(0)
        .unwrap()
        .try_into()
        .unwrap()
}
