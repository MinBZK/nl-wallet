use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use assert_matches::assert_matches;
use chrono::DateTime;
use chrono::Utc;
use config::Config;
use config::File;
use futures::future::try_join_all;
use p256::ecdsa::SigningKey;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::sea_query::Expr;
use serde::Deserialize;
use tempfile::TempDir;
use url::Url;
use uuid::Uuid;

use crypto::EcdsaKey;
use crypto::EcdsaKeySend;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use jwt::DEFAULT_VALIDATIONS;
use jwt::EcdsaDecodingKey;
use jwt::UnverifiedJwt;
use server_utils::keys::PrivateKeyVariant;
use status_lists::config::StatusListConfig;
use status_lists::config::StatusListConfigs;
use status_lists::entity::attestation_batch;
use status_lists::entity::attestation_batch_list_indices;
use status_lists::entity::attestation_type;
use status_lists::entity::status_list;
use status_lists::entity::status_list_item;
use status_lists::postgres::PostgresStatusListService;
use status_lists::postgres::PostgresStatusListServices;
use status_lists::publish::PublishDir;
use token_status_list::status_claim::StatusClaim;
use token_status_list::status_claim::StatusListClaim;
use token_status_list::status_list::Bits;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_token::StatusListToken;
use token_status_list::status_list_token::TOKEN_STATUS_LIST_JWT_TYP;
use utils::date_time_seconds::DateTimeSeconds;
use utils::num::NonZeroU31;
use utils::path::prefix_local_path;

#[derive(Debug, Clone, Deserialize)]
struct TestSettings {
    storage_url: Url,
}

async fn connection_from_settings() -> anyhow::Result<DatabaseConnection> {
    let settings: TestSettings = Config::builder()
        .add_source(File::from(prefix_local_path(Path::new("status_lists.toml")).as_ref()).required(true))
        .build()?
        .try_deserialize()?;
    let connection = server_utils::store::postgres::new_connection(settings.storage_url).await?;
    Ok(connection)
}

async fn private_key_variant(pair: KeyPair<SigningKey>) -> KeyPair<PrivateKeyVariant> {
    KeyPair::new(
        PrivateKeyVariant::Software(pair.private_key().clone()),
        pair.certificate().clone(),
    )
    .await
    .unwrap()
}

async fn create_status_list_service(
    ca: &Ca,
    connection: &DatabaseConnection,
    list_size: i32,
    create_threshold: i32,
    publish_dir: &TempDir,
) -> anyhow::Result<(String, StatusListConfig, PostgresStatusListService<PrivateKeyVariant>)> {
    let attestation_type = random_string(20);
    let config = StatusListConfig {
        list_size: NonZeroU31::try_new(list_size)?,
        create_threshold: NonZeroU31::try_new(create_threshold)?,
        base_url: format!("https://example.com/tsl/{}", attestation_type)
            .as_str()
            .parse()?,
        publish_dir: PublishDir::try_new(publish_dir.path().to_path_buf())?,
        key_pair: private_key_variant(ca.generate_status_list_mock()?).await,
    };
    let service = PostgresStatusListService::try_new(connection.clone(), &attestation_type, config.clone()).await?;
    try_join_all(service.initialize_lists().await?.into_iter()).await?;

    Ok((attestation_type, config, service))
}

async fn recreate_status_list_service(
    connection: &DatabaseConnection,
    attestation_type: &str,
    config: StatusListConfig,
) -> anyhow::Result<PostgresStatusListService<impl EcdsaKeySend>> {
    let service = PostgresStatusListService::try_new(connection.clone(), attestation_type, config).await?;
    try_join_all(service.initialize_lists().await?.into_iter()).await?;

    Ok(service)
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
        ((next_sequence_no - size as i64)..next_sequence_no).collect::<Vec<_>>(),
    );

    let mut indices = items.iter().map(|item| item.index).collect::<Vec<_>>();
    indices.sort();
    assert_eq!(indices, (0..size).collect::<Vec<_>>());

    items
}

async fn assert_empty_published_list(config: &StatusListConfig, list: &status_list::Model) {
    assert_published_list(config, list, vec![]).await
}

async fn assert_published_list(
    config: &StatusListConfig,
    list: &status_list::Model,
    revoked: impl IntoIterator<Item = usize>,
) {
    let path = config.publish_dir.jwt_path(&list.external_id);
    let status_list_token = tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(path).expect("published file not found");
        let mut buffer = String::new();
        std::io::BufReader::new(file).read_to_string(&mut buffer).unwrap();
        buffer.parse::<StatusListToken>().unwrap()
    })
    .await
    .unwrap();

    let verifying_key = EcdsaDecodingKey::from(&config.key_pair.verifying_key().await.unwrap());
    let (header, claims) = UnverifiedJwt::from(status_list_token)
        .parse_and_verify(&verifying_key, &DEFAULT_VALIDATIONS)
        .unwrap();
    assert_eq!(header.inner().typ, TOKEN_STATUS_LIST_JWT_TYP);

    let bits = *claims.status_list.bits();
    assert_eq!(bits, Bits::One);

    let published = claims.status_list.unpack();
    let mut expected = StatusList::new_aligned(config.list_size.as_usize(), bits);
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

#[tokio::test]
async fn test_service_initializes_status_lists() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _) = create_status_list_service(&ca, &connection, 10, 1, &publish_dir)
        .await
        .unwrap();

    let attestation_type = attestation_type::Entity::find()
        .filter(attestation_type::Column::Name.eq(attestation_type))
        .one(&connection)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(attestation_type.next_sequence_no, 10);

    let db_lists = fetch_status_list(&connection, attestation_type.id).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 10, 10, 10, false).await;
    assert_empty_published_list(&config, &db_lists[0]).await;
}

#[tokio::test]
async fn test_service_initializes_multiple_status_lists() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();

    let private_key = private_key_variant(ca.generate_status_list_mock().unwrap()).await;
    let publish_dir = tempfile::tempdir().unwrap();
    let configs: StatusListConfigs = (0..2)
        .map(|_| {
            let attestation_type = random_string(20);
            let config = StatusListConfig {
                list_size: NonZeroU31::try_new(4).unwrap(),
                create_threshold: NonZeroU31::try_new(1).unwrap(),
                base_url: "https://example.com/tsl".parse().unwrap(),
                publish_dir: PublishDir::try_new(publish_dir.path().to_path_buf()).unwrap(),
                key_pair: private_key.clone(),
            };
            (attestation_type, config)
        })
        .collect::<HashMap<_, _>>()
        .into();
    let service = PostgresStatusListServices::try_new(connection.clone(), configs.clone())
        .await
        .unwrap();
    try_join_all(service.initialize_lists().await.unwrap()).await.unwrap();

    let attestation_types = attestation_type::Entity::find()
        .filter(attestation_type::Column::Name.is_in(configs.as_ref().keys()))
        .all(&connection)
        .await
        .unwrap();

    for attestation_type in attestation_types {
        let db_lists = fetch_status_list(&connection, attestation_type.id).await;
        assert_eq!(db_lists.len(), 1);
        assert_status_list_items(&connection, &db_lists[0], 4, 4, 4, false).await;
        assert_empty_published_list(&configs.as_ref()[&attestation_type.name], &db_lists[0]).await;
    }
}

#[tokio::test]
async fn test_service_initializes_schedule_housekeeping_empty() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _) = create_status_list_service(&ca, &connection, 5, 2, &publish_dir)
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
    let _ = recreate_status_list_service(&connection, &attestation_type, config).await;

    // Check for empty list if new one is created and properly cleaned up
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 0, 5, 5, true).await;
    assert_status_list_items(&connection, &db_lists[1], 6, 6, 11, false).await;
}

#[tokio::test]
async fn test_service_initializes_schedule_housekeeping_almost_empty() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _) = create_status_list_service(&ca, &connection, 5, 2, &publish_dir)
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
    let _ = recreate_status_list_service(&connection, &attestation_type, config).await;

    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 1, 5, 5, false).await;
    assert_status_list_items(&connection, &db_lists[1], 7, 7, 12, false).await;
}

#[tokio::test]
async fn test_service_initializes_schedule_housekeeping_full() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, _) = create_status_list_service(&ca, &connection, 5, 2, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Recreate list with large list size
    let _ = recreate_status_list_service(&connection, &attestation_type, config).await;

    // Full list should still be same
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 5, 5, 5, false).await;
}

#[tokio::test]
async fn test_service_create_status_claims() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, service) = create_status_list_service(&ca, &connection, 9, 5, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;
    update_availability_of_status_list(&connection, type_id, 8).await;

    let batch_id = Uuid::new_v4();
    let expiration_date: DateTimeSeconds = Utc::now().into();
    let (claims, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(batch_id, Some(expiration_date), 2.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(tasks.len(), 0);

    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1); // No new list creation scheduled
    let db_list_items = assert_status_list_items(&connection, &db_lists[0], 6, 9, 9, false).await;

    assert_eq!(claims.len(), 2.try_into().unwrap());
    assert_matches!(&claims[0], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_list_items[1].index as u32,
        uri: config.base_url.join(&db_lists[0].external_id),
    });
    assert_matches!(&claims[1], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_list_items[2].index as u32,
        uri: config.base_url.join(&db_lists[0].external_id),
    });

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

#[tokio::test]
async fn test_service_create_status_claims_creates_in_flight_if_needed() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, service) = create_status_list_service(&ca, &connection, 8, 1, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;
    update_availability_of_status_list(&connection, type_id, 1).await;

    // Fetch items of current list now, since they will be deleted
    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 1);
    let db_old_list_items = assert_status_list_items(&connection, &db_lists[0], 1, 8, 8, false).await;

    let batch_id = Uuid::new_v4();
    let (claims, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(batch_id, None, 2.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(tasks.len(), 2);
    try_join_all(tasks.into_iter()).await.unwrap();

    let db_lists = fetch_status_list(&connection, type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 0, 8, 8, true).await;
    assert_empty_published_list(&config, &db_lists[0]).await;
    let db_new_list_items = assert_status_list_items(&connection, &db_lists[1], 7, 8, 16, false).await;

    assert_eq!(claims.len(), 2.try_into().unwrap());
    assert_matches!(&claims[0], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_old_list_items[7].index as u32,
        uri: config.base_url.join(&db_lists[0].external_id),
    });
    assert_matches!(&claims[1], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_new_list_items[0].index as u32,
        uri: config.base_url.join(&db_lists[1].external_id),
    });

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

#[tokio::test]
async fn test_service_create_status_claims_concurrently() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, service) = create_status_list_service(&ca, &connection, 24, 2, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    let concurrent = 7;
    let num_copies = 3.try_into().unwrap();
    let claims_per_batch =
        try_join_all((0..concurrent).map(|_| service.obtain_status_claims(Uuid::new_v4(), None, num_copies)))
            .await
            .unwrap();

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

#[tokio::test]
async fn test_service_revoke_attestation_batch_multiple_lists() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, service) = create_status_list_service(&ca, &connection, 4, 1, &publish_dir)
        .await
        .unwrap();

    // Ensure we have two lists
    let type_id = attestation_type_id(&connection, &attestation_type).await;
    update_availability_of_status_list(&connection, type_id, 1).await;
    try_join_all(service.initialize_lists().await.unwrap()).await.unwrap();

    // Create status claims for attestation
    let batch_id = Uuid::new_v4();
    let claims = service
        .obtain_status_claims(batch_id, None, 3.try_into().unwrap())
        .await
        .unwrap();

    // Revoke all attestation
    service.revoke_attestation_batch(batch_id).await.unwrap();

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

#[tokio::test]
async fn test_service_revoke_attestation_batch_concurrently() {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let connection = connection_from_settings().await.unwrap();
    let publish_dir = tempfile::tempdir().unwrap();
    let (attestation_type, config, service) = create_status_list_service(&ca, &connection, 9, 1, &publish_dir)
        .await
        .unwrap();

    let type_id = attestation_type_id(&connection, &attestation_type).await;

    // Obtain claims for multiple attestation batches
    let concurrent = 7;
    let batch_ids = (0..concurrent).map(|_| Uuid::new_v4()).collect::<Vec<_>>();
    let claims_per_batch = try_join_all(
        batch_ids
            .iter()
            .copied()
            .map(|batch_id| service.obtain_status_claims(batch_id, None, 1.try_into().unwrap())),
    )
    .await
    .unwrap();

    // Revoke concurrently
    try_join_all(
        batch_ids
            .into_iter()
            .map(|batch_id| service.revoke_attestation_batch(batch_id)),
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
