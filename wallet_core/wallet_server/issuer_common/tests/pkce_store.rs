use db_test::DbSetup;
use db_test::connection_from_url;
use issuer_common::pkce_store::IssuerPkceStore;
use openid4vc::pkce::test::test_pkce_store;
use sea_orm::ConnectionTrait;
use sea_orm::DbBackend;
use sea_orm::Statement;
use server_utils::store::StoreConnection;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_issuer_pkce_store() {
    let db_setup = DbSetup::create().await;
    let database_connection = connection_from_url(db_setup.pid_issuer_url()).await;

    let store = IssuerPkceStore::new(StoreConnection::Postgres(database_connection.clone()));

    test_pkce_store(store, async |_store| {
        database_connection
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT COUNT(*) FROM "pkce_flow""#,
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
