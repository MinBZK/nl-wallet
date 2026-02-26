use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use futures::future::try_join_all;
use p256::ecdsa::SigningKey;
use rstest::rstest;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use tokio::net::TcpListener;
use url::Url;
use uuid::Uuid;

use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use db_test::DbSetup;
use db_test::connection_from_url;
use status_lists::config::StatusListConfig;
use status_lists::entity::attestation_batch;
use status_lists::postgres::PostgresStatusListService;
use status_lists::publish::PublishDir;
use status_lists::revoke::create_revocation_router;
use token_status_list::status_list_service::StatusListRevocationService;
use utils::num::NonZeroU31;
use utils::num::U31;

async fn setup_revocation_server<L>(service: Arc<L>) -> anyhow::Result<Url>
where
    L: StatusListRevocationService + Send + Sync + 'static,
{
    let (router, _) = create_revocation_router(service);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    Ok(format!("http://127.0.0.1:{}/revoke/", port).parse()?)
}

pub async fn fetch_attestation_batch(
    connection: &DatabaseConnection,
    batch_id: Uuid,
) -> Option<attestation_batch::Model> {
    attestation_batch::Entity::find()
        .filter(attestation_batch::Column::BatchId.eq(batch_id))
        .one(connection)
        .await
        .unwrap()
}

async fn setup_revocation_test(
    db_setup: &DbSetup,
    publish_dir: PublishDir,
) -> (Arc<PostgresStatusListService<SigningKey>>, Url) {
    let ca = Ca::generate_issuer_mock_ca().unwrap();

    let key_pair = ca.generate_status_list_mock().unwrap();

    let config = StatusListConfig {
        list_size: NonZeroU31::try_new(100).unwrap(),
        create_threshold: U31::ZERO, // no new lists are needed during test
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl: None,
        base_url: "https://example.com/".parse().unwrap(),
        publish_dir,
        key_pair,
    };

    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let service = PostgresStatusListService::try_new(connection.clone(), &random_string(20), config)
        .await
        .unwrap();
    try_join_all(service.initialize_lists().await.unwrap().into_iter())
        .await
        .unwrap();

    let service = Arc::new(service);
    let revoke_endpoint = setup_revocation_server(Arc::clone(&service)).await.unwrap();

    (service, revoke_endpoint)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
#[case(&[Uuid::new_v4()])]
#[case(&[Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()])]
async fn test_revoke_batch(#[case] batch: &[Uuid]) {
    let db_setup = DbSetup::create().await;
    let publish_dir = tempfile::tempdir().unwrap();

    let (service, revocation_endpoint) = setup_revocation_test(
        &db_setup,
        PublishDir::try_new(publish_dir.path().to_path_buf()).unwrap(),
    )
    .await;

    join_all(batch.iter().map(async |id| {
        let tasks = service
            .obtain_status_claims_and_scheduled_tasks(*id, None, NonZeroUsize::MIN)
            .await
            .unwrap()
            .1;

        join_all(tasks).await;
    }))
    .await;

    let response = reqwest::Client::new()
        .post(revocation_endpoint.clone())
        .json(&batch)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // assert that all batches in the list are revoked
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    assert!(
        join_all(batch.iter().map(async |batch_id| {
            let revocation_status = fetch_attestation_batch(&connection, *batch_id).await.unwrap();
            revocation_status.is_revoked
        }))
        .await
        .into_iter()
        .all(|b| b)
    );

    // test idempotency
    let response = reqwest::Client::new()
        .post(revocation_endpoint)
        .json(&batch)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_revoke_non_existing_batch_should_return_success() {
    let db_setup = DbSetup::create().await;
    let publish_dir = tempfile::tempdir().unwrap();

    let (_, revocation_endpoint) = setup_revocation_test(
        &db_setup,
        PublishDir::try_new(publish_dir.path().to_path_buf()).unwrap(),
    )
    .await;

    let uuid = Uuid::new_v4();

    let response = reqwest::Client::new()
        .post(revocation_endpoint)
        .json(&[uuid])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}
