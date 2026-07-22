use chrono::DateTime;
use chrono::Duration;
use db_test::DbName;
use db_test::DbSetup;
use db_test::connection_from_url;
use issuer_common::par_store::IssuerParStore;
use openid4vc::par::test::test_par_store;
use openid4vc::store::Store;
use sea_orm::ConnectionTrait;
use sea_orm::DbBackend;
use sea_orm::Statement;
use server_utils::store::StoreConnection;
use utils::generator::mock::MockTimeGenerator;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_issuer_par_store() {
    let db_setup = DbSetup::create_clean_only([DbName::PidIssuer]).await;
    let database_connection = connection_from_url(db_setup.pid_issuer_url()).await;

    let store = IssuerParStore::new(StoreConnection::Postgres(database_connection.clone()));

    test_par_store(store, async |_store| count_rows(&database_connection).await).await
}

/// Verify that `cleanup` removes only expired rows and drains backlogs larger than a single batch
/// (the `FOR UPDATE SKIP LOCKED` loop), leaving still-valid rows untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_issuer_par_store_cleanup_drains_in_batches() {
    let db_setup = DbSetup::create_clean_only([DbName::PidIssuer]).await;
    let database_connection = connection_from_url(db_setup.pid_issuer_url()).await;

    let now = DateTime::from_timestamp_secs(1_000_000_000).unwrap();
    let time_generator = MockTimeGenerator::new(now);
    let store = IssuerParStore::new_postgres_with_time_generator(database_connection.clone(), time_generator);

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

async fn insert_rows(
    connection: &sea_orm::DatabaseConnection,
    prefix: &str,
    count: i64,
    expires_at: DateTime<chrono::Utc>,
) {
    connection
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO pushed_authorization_request (request_uri, data, expires_at)
            SELECT $1 || '-' || g, '{}'::jsonb, $2
            FROM generate_series(1, $3::bigint) AS g
            "#,
            [prefix.into(), expires_at.into(), count.into()],
        ))
        .await
        .unwrap();
}

async fn count_rows(connection: &sea_orm::DatabaseConnection) -> usize {
    connection
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            r#"SELECT COUNT(*) FROM "pushed_authorization_request""#,
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<i64>(0)
        .unwrap()
        .try_into()
        .unwrap()
}
