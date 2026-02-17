use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use assert_matches::assert_matches;
use chrono::DateTime;
use chrono::Utc;
use futures::future::try_join_all;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use rstest::rstest;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::sea_query::Expr;
use tempfile::TempDir;
use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use db_test::DbSetup;
use db_test::connection_from_url;
use jwt::DEFAULT_VALIDATIONS;
use jwt::EcdsaDecodingKey;
use jwt::SignedJwt;
use status_lists::config::StatusListConfig;
use status_lists::config::StatusListConfigs;
use status_lists::entity::attestation_batch;
use status_lists::entity::attestation_batch_list_indices;
use status_lists::entity::attestation_type;
use status_lists::entity::status_list;
use status_lists::entity::status_list_item;
use status_lists::flag::Flag;
use status_lists::postgres::PostgresStatusListService;
use status_lists::postgres::PostgresStatusListServices;
use status_lists::publish::PublishDir;
use token_status_list::status_list::Bits;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_service::StatusListRevocationService;
use token_status_list::status_list_service::StatusListService;
use token_status_list::status_list_token::StatusListToken;
use token_status_list::status_list_token::TOKEN_STATUS_LIST_JWT_TYP;
use utils::date_time_seconds::DateTimeSeconds;
use utils::num::NonZeroU31;
use utils::num::U31;

async fn create_status_list_service(
    ca: &Ca,
    connection: &DatabaseConnection,
    list_size: i32,
    create_threshold: i32,
    ttl: Option<Duration>,
    publish_dir: &TempDir,
) -> anyhow::Result<(
    String,
    StatusListConfig<SigningKey>,
    Flag,
    PostgresStatusListService<SigningKey>,
)> {
    let attestation_type = random_string(20);
    let config = StatusListConfig {
        list_size: NonZeroU31::try_new(list_size)?,
        create_threshold: U31::try_new(create_threshold)?,
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl,
        base_url: format!("https://example.com/tsl/{}", attestation_type)
            .as_str()
            .parse()?,
        publish_dir: PublishDir::try_new(publish_dir.path().to_path_buf())?,
        key_pair: ca.generate_status_list_mock()?,
    };
    let revoke_all_flag = Flag::new(connection.clone(), attestation_type.to_string());
    let service = PostgresStatusListService::try_new_with_flag(
        connection.clone(),
        &attestation_type,
        config.clone(),
        revoke_all_flag.clone(),
    )
    .await?;
    try_join_all(service.initialize_lists().await?.into_iter()).await?;

    Ok((attestation_type, config, revoke_all_flag, service))
}

async fn recreate_status_list_service(
    connection: &DatabaseConnection,
    attestation_type: &str,
    config: StatusListConfig<SigningKey>,
    revoke_all_flag: Flag,
) -> anyhow::Result<PostgresStatusListService<SigningKey>> {
    let service =
        PostgresStatusListService::try_new_with_flag(connection.clone(), attestation_type, config, revoke_all_flag)
            .await?;
    try_join_all(service.initialize_lists().await?.into_iter()).await?;

    Ok(service)
}

/// Clean publish dir such that all files are deleted and the lock files are truncated
async fn clean_publish_dir(path: PathBuf) {
    tokio::task::spawn_blocking(move || {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.extension() == Some(OsStr::new("lock")) {
                // Lock files always need to be intact (otherwise the lock cannot be guaranteed).
                // Truncating it, instead of deleting the file
                std::fs::write(path, "").unwrap();
            } else {
                std::fs::remove_file(path).unwrap();
            }
        }
    })
    .await
    .unwrap();
}

async fn attestation_type_id(connection: &DatabaseConnection, name: &str) -> i16 {
    attestation_type::Entity::find()
        .select_only()
        .column(attestation_type::Column::Id)
        .filter(attestation_type::Column::Name.eq(name))
        .into_tuple()
        .one(connection)
        .await
        .unwrap()
        .unwrap()
}

async fn assert_status_list_items(
    connection: &DatabaseConnection,
    list: &status_list::Model,
    available: i32,
    size: i32,
    next_sequence_no: i64,
    deleted_items: bool,
) -> Vec<status_list_item::Model> {
    assert_eq!(list.available, available);
    assert_eq!(list.size, size);
    assert_eq!(list.next_sequence_no, next_sequence_no);

    let items = status_list_item::Entity::find()
        .filter(status_list_item::Column::StatusListId.eq(list.id))
        .order_by_asc(status_list_item::Column::SequenceNo)
        .all(connection)
        .await
        .unwrap();

    if deleted_items {
        assert_eq!(items.len(), 0);
        return Vec::new();
    }

    assert_eq!(items.len(), size as usize);
    assert_eq!(
        items.iter().map(|item| item.sequence_no).collect::<Vec<_>>(),
        ((next_sequence_no - i64::from(size))..next_sequence_no).collect::<Vec<_>>(),
    );

    let mut indices = items.iter().map(|item| item.index).collect::<Vec<_>>();
    indices.sort();
    assert_eq!(indices, (0..size).collect::<Vec<_>>());

    items
}

async fn assert_empty_published_list(config: &StatusListConfig<SigningKey>, list: &status_list::Model) {
    assert_published_list(config, list, vec![]).await
}

async fn assert_published_list(
    config: &StatusListConfig<SigningKey>,
    list: &status_list::Model,
    revoked: impl IntoIterator<Item = usize>,
) {
    let path = config.publish_dir.jwt_path(&list.external_id);
    let status_list_token = tokio::fs::read_to_string(path)
        .await
        .unwrap()
        .parse::<StatusListToken>()
        .unwrap();

    let verifying_key = EcdsaDecodingKey::from(&config.key_pair.verifying_key().await.unwrap());
    let (header, claims) = status_list_token
        .as_ref()
        .parse_and_verify(&verifying_key, &DEFAULT_VALIDATIONS)
        .unwrap();
    assert_eq!(header.inner().typ, TOKEN_STATUS_LIST_JWT_TYP);

    let bits = *claims.status_list.bits();
    assert_eq!(bits, Bits::One);
    assert_eq!(claims.ttl, config.ttl);

    // Expiry should be less than configured in since time has increased
    let expiry_from_now = claims.exp.expect("expiry should be set") - Utc::now();
    assert_eq!(
        expiry_from_now
            .to_std()
            .expect("expiry should be in future")
            .saturating_sub(config.expiry),
        Duration::ZERO
    );

    let published = claims.status_list.unpack();
    let mut expected = StatusList::new_aligned(list.size as usize, bits);
    for index in revoked {
        expected.insert(index, StatusType::Invalid);
    }
    assert_eq!(published, expected);
}

async fn fetch_status_list(connection: &DatabaseConnection, type_id: i16) -> Vec<status_list::Model> {
    status_list::Entity::find()
        .filter(status_list::Column::AttestationTypeId.eq(type_id))
        .order_by_asc(status_list::Column::NextSequenceNo)
        .all(connection)
        .await
        .unwrap()
}

async fn update_availability_of_status_list(connection: &DatabaseConnection, type_id: i16, availability: i32) {
    // Make second list empty
    let result = status_list::Entity::update_many()
        .col_expr(status_list::Column::Available, Expr::value(availability))
        .filter(status_list::Column::AttestationTypeId.eq(type_id))
        .exec(connection)
        .await
        .unwrap();
    match result.rows_affected {
        0 => panic!(
            "Not updated availability of status list for attestation type {}",
            type_id
        ),
        1 => (),
        no => panic!("Updated {} status lists for attestation type {}", no, type_id),
    }
}

async fn fetch_attestation_batches(
    connection: &DatabaseConnection,
    status_lists: &[status_list::Model],
) -> Vec<(attestation_batch::Model, Vec<attestation_batch_list_indices::Model>)> {
    let ids = status_lists.iter().map(|list| list.id);
    attestation_batch::Entity::find()
        .find_with_related(attestation_batch_list_indices::Entity)
        .filter(attestation_batch_list_indices::Column::StatusListId.is_in(ids))
        .order_by_asc(attestation_batch_list_indices::Column::StatusListId)
        .all(connection)
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_initializes_status_lists() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _, _) = create_status_list_service(&ca, &connection, 10, 1, None, &publish_dir)
        .await
        .unwrap();

    // Check if attestation type is correctly created
    let attestation_type = attestation_type::Entity::find()
        .filter(attestation_type::Column::Name.eq(attestation_type))
        .one(&connection)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(attestation_type.next_sequence_no, 10);

    // Check if status lists are correctly initialized
    let db_lists = fetch_status_list(&connection, attestation_type.id).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 10, 10, 10, false).await;
    assert_empty_published_list(&config, &db_lists[0]).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_multiple_services_initializes_status_lists_and_refresh_job() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;

    let key_pair = ca.generate_status_list_mock().unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let configs: StatusListConfigs<SigningKey> = (0..2)
        .zip_eq(itertools::repeat_n(key_pair, 2))
        .map(|(_, key_pair)| {
            let attestation_type = random_string(20);
            let config = StatusListConfig {
                list_size: NonZeroU31::try_new(4).unwrap(),
                create_threshold: U31::ONE,
                expiry: Duration::from_secs(3600),
                refresh_threshold: Duration::from_secs(600),
                ttl: None,
                base_url: "https://example.com/tsl".parse().unwrap(),
                publish_dir: PublishDir::try_new(publish_dir.path().to_path_buf()).unwrap(),
                key_pair,
            };
            (attestation_type, config)
        })
        .collect::<HashMap<_, _>>()
        .into();
    let service = PostgresStatusListServices::try_new(connection.clone(), configs.clone())
        .await
        .unwrap();
    try_join_all(service.initialize_lists().await.unwrap()).await.unwrap();

    // Check if attestation types are correctly created
    let attestation_types = attestation_type::Entity::find()
        .filter(attestation_type::Column::Name.is_in(configs.as_ref().keys()))
        .all(&connection)
        .await
        .unwrap();

    // Check if status lists are correctly initialized
    for attestation_type in &attestation_types {
        let db_lists = fetch_status_list(&connection, attestation_type.id).await;
        assert_eq!(db_lists.len(), 1);
        assert_status_list_items(&connection, &db_lists[0], 4, 4, 4, false).await;
        assert_empty_published_list(&configs.as_ref()[&attestation_type.name], &db_lists[0]).await;
    }

    // Clean published files and start refresh job
    clean_publish_dir(publish_dir.path().to_path_buf()).await;
    let refresh_handles = service.start_refresh_jobs();
    tokio::time::sleep(Duration::from_millis(100)).await;
    refresh_handles.into_iter().for_each(|handle| handle.abort());

    // Check if status lists are correctly republished
    for attestation_type in attestation_types {
        let db_lists = fetch_status_list(&connection, attestation_type.id).await;
        assert_eq!(db_lists.len(), 1);
        assert_empty_published_list(&configs.as_ref()[&attestation_type.name], &db_lists[0]).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_initializes_schedule_housekeeping_empty() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, _) =
        create_status_list_service(&ca, &connection, 5, 2, None, &publish_dir)
            .await
            .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Make list empty
    update_availability_of_status_list(&connection, type_id, 0).await;

    // Recreate list with large list size
    let config = StatusListConfig {
        list_size: 6.try_into().unwrap(),
        ..config
    };
    let _ = recreate_status_list_service(&connection, &attestation_type, config, revoke_all_flag).await;

    // Old list should be empty and new list should be created
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 0, 5, 5, true).await;
    assert_status_list_items(&connection, &db_lists[1], 6, 6, 11, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_initializes_schedule_housekeeping_almost_empty() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, _) =
        create_status_list_service(&ca, &connection, 5, 2, None, &publish_dir)
            .await
            .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Make list almost empty
    update_availability_of_status_list(&connection, type_id, 1).await;

    // Recreate list with large list size
    let config = StatusListConfig {
        list_size: 7.try_into().unwrap(),
        ..config
    };
    let _ = recreate_status_list_service(&connection, &attestation_type, config, revoke_all_flag).await;

    // New list should be created, but old one still has available items
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 1, 5, 5, false).await;
    assert_status_list_items(&connection, &db_lists[1], 7, 7, 12, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_initializes_schedule_housekeeping_full() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, _) =
        create_status_list_service(&ca, &connection, 5, 2, None, &publish_dir)
            .await
            .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Recreate list with large list size
    let _ = recreate_status_list_service(&connection, &attestation_type, config, revoke_all_flag).await;

    // Full list should still be same
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 5, 5, 5, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_create_status_claims() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _, service) = create_status_list_service(&ca, &connection, 9, 5, None, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;
    update_availability_of_status_list(&connection, type_id, 8).await;

    // Obtain claims for attestation batch
    let batch_id = Uuid::new_v4();
    let expiration_date: DateTimeSeconds = Utc::now().into();
    let (claims, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(batch_id, Some(expiration_date), 2.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(tasks.len(), 0);

    // Check if database is correctly updated
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1); // No new list creation scheduled
    let db_list_items = assert_status_list_items(&connection, &db_lists[0], 6, 9, 9, false).await;

    // Check if claims matches config and database
    assert_eq!(claims.len(), 2.try_into().unwrap());
    assert_matches!(&claims[0], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_list_items[1].index as u32,
        uri: config.base_url.join(&db_lists[0].external_id),
    });
    assert_matches!(&claims[1], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_list_items[2].index as u32,
        uri: config.base_url.join(&db_lists[0].external_id),
    });

    // Check if database attestation batch is correctly stored
    let db_attestations = fetch_attestation_batches(&connection, &db_lists).await;
    assert_eq!(db_attestations.len(), 1);

    let db_attestation = &db_attestations[0].0;
    assert_eq!(db_attestation.batch_id, batch_id);
    assert_eq!(
        db_attestation.expiration_date,
        Some(DateTime::from(expiration_date).date_naive())
    );
    assert!(!db_attestation.is_revoked);

    let db_list_indices = &db_attestations[0].1;
    assert_eq!(db_list_indices.len(), 1);
    assert_eq!(db_list_indices[0].status_list_id, db_lists[0].id);
    assert_eq!(
        db_list_indices[0].indices,
        vec![db_list_items[1].index, db_list_items[2].index]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_create_status_claims_creates_in_flight_if_needed() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _, service) = create_status_list_service(&ca, &connection, 8, 1, None, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;
    update_availability_of_status_list(&connection, type_id, 1).await;

    // Fetch items of current list now, since they will be deleted
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);
    let db_old_list_items = assert_status_list_items(&connection, &db_lists[0], 1, 8, 8, false).await;

    // Obtain claims for attestation batch
    let batch_id = Uuid::new_v4();
    let (claims, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(batch_id, None, 2.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(tasks.len(), 2);
    try_join_all(tasks.into_iter()).await.unwrap();

    // Check if database is correctly updated
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 0, 8, 8, true).await;
    assert_empty_published_list(&config, &db_lists[0]).await;
    let db_new_list_items = assert_status_list_items(&connection, &db_lists[1], 7, 8, 16, false).await;

    // Check if claims matches config and database
    assert_eq!(claims.len(), 2.try_into().unwrap());
    assert_matches!(&claims[0], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_old_list_items[7].index as u32,
        uri: config.base_url.join(&db_lists[0].external_id),
    });
    assert_matches!(&claims[1], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_new_list_items[0].index as u32,
        uri: config.base_url.join(&db_lists[1].external_id),
    });

    // Check if database attestation batch is correctly stored
    let db_attestations = fetch_attestation_batches(&connection, &db_lists).await;
    assert_eq!(db_attestations.len(), 1);

    let db_attestation = &db_attestations[0].0;
    assert_eq!(db_attestation.batch_id, batch_id);
    assert_eq!(db_attestation.expiration_date, None);
    assert!(!db_attestation.is_revoked);

    let db_list_indices = &db_attestations[0].1;
    assert_eq!(db_list_indices.len(), 2);
    assert_eq!(db_list_indices[0].status_list_id, db_lists[0].id);
    assert_eq!(db_list_indices[0].indices, vec![db_old_list_items[7].index]);
    assert_eq!(db_list_indices[1].status_list_id, db_lists[1].id);
    assert_eq!(db_list_indices[1].indices, vec![db_new_list_items[0].index]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_create_status_claims_concurrently() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _, service) =
        create_status_list_service(&ca, &connection, 24, 2, None, &publish_dir)
            .await
            .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Obtain claims for multiple attestation batches
    let concurrent = 7;
    let num_copies = 3.try_into().unwrap();
    let claims_per_batch =
        try_join_all((0..concurrent).map(|_| service.obtain_status_claims(Uuid::new_v4(), None, num_copies)))
            .await
            .unwrap();

    // Check if claims matches the database
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1); // No new list creation scheduled
    let mut db_list_items = assert_status_list_items(&connection, &db_lists[0], 3, 24, 24, false).await;

    let url = config.base_url.join(&db_lists[0].external_id);
    let db_claims = db_list_items
        .drain(0..(concurrent * num_copies.get()))
        .map(|item| {
            StatusClaim::StatusList(StatusListClaim {
                idx: item.index as u32,
                uri: url.clone(),
            })
        })
        .collect::<HashSet<_>>();
    assert_eq!(db_list_items.len(), 3);

    let claims = claims_per_batch.into_iter().flatten().collect::<HashSet<_>>();
    assert_eq!(claims, db_claims);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_revoke_attestation_batches_multiple_lists() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, _) =
        create_status_list_service(&ca, &connection, 4, 1, Some(Duration::from_secs(300)), &publish_dir)
            .await
            .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Ensure we have two lists and change list size for new list
    update_availability_of_status_list(&connection, type_id, 1).await;
    let config = StatusListConfig {
        list_size: 10.try_into().unwrap(),
        ..config
    };
    let service = recreate_status_list_service(&connection, attestation_type.as_ref(), config.clone(), revoke_all_flag)
        .await
        .unwrap();

    // Create status claims for attestation
    let batch_id = Uuid::new_v4();
    let claims = service
        .obtain_status_claims(batch_id, None, 3.try_into().unwrap())
        .await
        .unwrap();

    // Revoke all attestation
    service.revoke_attestation_batches(vec![batch_id]).await.unwrap();

    // Check if published list matches database
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);

    let list_urls = db_lists
        .iter()
        .map(|list| config.base_url.join(&list.external_id))
        .collect::<Vec<_>>();
    assert_published_list(
        &config,
        &db_lists[0],
        claims.iter().filter_map(|claim| match claim {
            StatusClaim::StatusList(list) if list_urls[0] == list.uri => Some(list.idx as usize),
            _ => None,
        }),
    )
    .await;
    assert_published_list(
        &config,
        &db_lists[1],
        claims.into_iter().filter_map(|claim| match claim {
            StatusClaim::StatusList(list) if list_urls[1] == list.uri => Some(list.idx as usize),
            _ => None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_revoke_attestation_batches_concurrently() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _, service) = create_status_list_service(&ca, &connection, 9, 1, None, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Obtain claims for multiple attestation batches
    let concurrent = 7;
    let batch_ids = (0..concurrent).map(|_| Uuid::new_v4()).collect_vec();
    let claims_per_batch = try_join_all(
        batch_ids
            .iter()
            .copied()
            .map(|batch_id| service.obtain_status_claims(batch_id, None, NonZeroUsize::MIN)),
    )
    .await
    .unwrap();

    // Revoke concurrently
    try_join_all(
        batch_ids
            .into_iter()
            .map(|batch_id| service.revoke_attestation_batches(vec![batch_id])),
    )
    .await
    .unwrap();

    // Check if published list matches database
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);

    assert_published_list(
        &config,
        &db_lists[0],
        claims_per_batch.into_iter().flat_map(|claims| {
            claims.into_iter().map(|claim| match claim {
                StatusClaim::StatusList(list) => list.idx as usize,
            })
        }),
    )
    .await;
}

async fn republish_list_with_expiry(path: &Path, key_pair: &KeyPair<impl EcdsaKey>, expiry: Option<DateTime<Utc>>) {
    let token: StatusListToken = tokio::fs::read_to_string(path).await.unwrap().parse().unwrap();
    let (_, mut claims) = token.as_ref().dangerous_parse_unverified().unwrap();
    claims.exp = expiry;
    let signed = SignedJwt::sign_with_certificate(&claims, key_pair).await.unwrap();
    tokio::fs::write(path, signed.into_unverified().serialization())
        .await
        .unwrap();
    let lock_path = path.with_extension("lock");
    tokio::fs::write(&lock_path, []).await.unwrap();
}

async fn wait_for_refresh(service: &PostgresStatusListService<SigningKey>, path: &Path) -> anyhow::Result<()> {
    let before = tokio::fs::metadata(path).await?.modified()?;
    let handle = service.start_refresh_job();
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        match tokio::fs::metadata(path).await?.modified() {
            Ok(current) if current > before => {
                handle.abort();
                return Ok(());
            }
            _ => {}
        }
    }
    handle.abort();
    Err(anyhow::Error::msg("Timeout waiting for refresh"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
#[case(None)]
#[case(Some(Utc::now()))]
async fn test_service_refresh_status_list_if_expired(#[case] expiry: Option<DateTime<Utc>>) {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _, service) = create_status_list_service(&ca, &connection, 3, 1, None, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);

    let path = publish_dir.path().join(format!("{}.jwt", db_lists[0].external_id));
    republish_list_with_expiry(&path, &config.key_pair, expiry).await;

    wait_for_refresh(&service, &path).await.unwrap();
    assert_published_list(&config, &db_lists[0], []).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_republish_status_list_with_revoke_all_set() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, service) =
        create_status_list_service(&ca, &connection, 5, 1, None, &publish_dir)
            .await
            .unwrap();

    // Check if status lists are correctly initialized
    let type_id = attestation_type_id(&connection, &attestation_type).await;
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 5, 5, 5, false).await;

    // Create status claims for attestation
    let batch_id = Uuid::new_v4();
    service
        .obtain_status_claims(batch_id, None, NonZeroUsize::MIN)
        .await
        .unwrap();

    // Set revoke all
    revoke_all_flag.set().await.unwrap();

    // Revoke all attestation to republish list
    service.revoke_attestation_batches(vec![batch_id]).await.unwrap();

    // Check if published list only contains invalid statuses
    assert_published_list(&config, &db_lists[0], 0..8).await;

    // Set revoke all again to check for idempotency
    revoke_all_flag.set().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_new_status_list_with_revoke_all_set() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, service) =
        create_status_list_service(&ca, &connection, 2, 1, None, &publish_dir)
            .await
            .unwrap();

    // Set revoke all
    revoke_all_flag.set().await.unwrap();

    // Create status claims for attestation to create new list
    let (_, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(Uuid::new_v4(), None, 2.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(tasks.len(), 2);
    try_join_all(tasks.into_iter()).await.unwrap();

    // Check if status lists are correctly initialized
    let type_id = attestation_type_id(&connection, &attestation_type).await;
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[1], 2, 2, 4, false).await;

    // Check if published list only contains invalid statuses
    assert_published_list(&config, &db_lists[1], 0..8).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_service_revoke_all() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db_setup = DbSetup::create_clean().await;
    let connection = connection_from_url(db_setup.status_lists_url()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, revoke_all_flag, _) =
        create_status_list_service(&ca, &connection, 2, 1, None, &publish_dir)
            .await
            .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Ensure we have two lists and change list size for new list
    update_availability_of_status_list(&connection, type_id, 0).await;
    let config = StatusListConfig {
        list_size: 9.try_into().unwrap(),
        ..config
    };
    let service = recreate_status_list_service(
        &connection,
        attestation_type.as_ref(),
        config.clone(),
        revoke_all_flag.clone(),
    )
    .await
    .unwrap();

    // Check if status lists are correctly initialized
    let type_id = attestation_type_id(&connection, &attestation_type).await;
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[1], 9, 9, 11, false).await;

    // Revoke all
    service.revoke_all().await.unwrap();

    // Check if revoke_all flag is set
    assert!(revoke_all_flag.is_set().await.unwrap());

    // Check if published lists only contains invalid statuses
    assert_published_list(&config, &db_lists[0], 0..8).await;
    assert_published_list(&config, &db_lists[1], 0..16).await;
}
