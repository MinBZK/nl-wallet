use std::collections::HashSet;
use std::path::Path;

use assert_matches::assert_matches;
use chrono::DateTime;
use chrono::Utc;
use config::Config;
use config::File;
use futures::future::try_join_all;
use indexmap::IndexMap;
use itertools::Itertools;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::sea_query::Expr;
use url::Url;
use uuid::Uuid;

use crypto::utils::random_string;
use http_utils::urls::BaseUrl;
use status_lists::entity::attestation_batch;
use status_lists::entity::attestation_type;
use status_lists::entity::status_list;
use status_lists::entity::status_list_item;
use status_lists::postgres::PostgresStatusClaimService;
use status_lists::postgres::StatusListLocation;
use status_lists::settings::StatusListsSettings;
use token_status_list::status_claim::StatusClaim;
use token_status_list::status_claim::StatusListClaim;
use token_status_list::status_service::StatusClaimService;
use utils::date_time_seconds::DateTimeSeconds;
use utils::path::prefix_local_path;

async fn connection_and_settings(
    list_size: u32,
    create_threshold: u32,
) -> anyhow::Result<(DatabaseConnection, StatusListsSettings)> {
    let settings: StatusListsSettings = Config::builder()
        .set_default("list_size", list_size)?
        .set_default("create_threshold", create_threshold)?
        .add_source(File::from(prefix_local_path(Path::new("status_lists.toml")).as_ref()).required(true))
        .build()?
        .try_deserialize()?;
    let connection = server_utils::store::postgres::new_connection(settings.clone().storage_url.unwrap()).await?;
    Ok((connection, settings))
}

async fn create_status_list_service(
    connection: &DatabaseConnection,
    settings: StatusListsSettings,
    attestation_types: usize,
) -> anyhow::Result<(Vec<String>, PostgresStatusClaimService)> {
    let attestation_types = (0..attestation_types).map(|_| random_string(20)).collect::<Vec<_>>();

    let service = PostgresStatusClaimService::try_new(connection.clone(), settings, &attestation_types).await?;
    try_join_all(service.initialize_lists().await?.into_iter()).await?;

    Ok((attestation_types, service))
}

async fn recreate_status_list_service(
    connection: &DatabaseConnection,
    settings: StatusListsSettings,
    attestation_types: &[String],
) -> anyhow::Result<PostgresStatusClaimService> {
    let service = PostgresStatusClaimService::try_new(connection.clone(), settings, attestation_types).await?;
    try_join_all(service.initialize_lists().await?.into_iter()).await?;

    Ok(service)
}

async fn attestation_ids(connection: &DatabaseConnection, names: &[String]) -> IndexMap<String, i16> {
    attestation_type::Entity::find()
        .select_only()
        .column(attestation_type::Column::Name)
        .column(attestation_type::Column::Id)
        .filter(attestation_type::Column::Name.is_in(names))
        .order_by_asc(attestation_type::Column::Id)
        .into_tuple()
        .all(connection)
        .await
        .unwrap()
        .into_iter()
        .collect::<IndexMap<_, _>>()
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

async fn fetch_status_list(connection: &DatabaseConnection, attestation_type_id: i16) -> Vec<status_list::Model> {
    status_list::Entity::find()
        .filter(status_list::Column::AttestationTypeId.eq(attestation_type_id))
        .order_by_asc(status_list::Column::NextSequenceNo)
        .all(connection)
        .await
        .unwrap()
}

async fn update_availability_of_status_list(
    connection: &DatabaseConnection,
    attestation_type_id: i16,
    availability: i32,
) {
    // Make second list empty
    let result = status_list::Entity::update_many()
        .col_expr(status_list::Column::Available, Expr::value(availability))
        .filter(status_list::Column::AttestationTypeId.eq(attestation_type_id))
        .exec(connection)
        .await
        .unwrap();
    match result.rows_affected {
        0 => panic!(
            "Not updated availability of status list for attestation type {}",
            attestation_type_id
        ),
        1 => (),
        no => panic!(
            "Updated {} status lists for attestation type {}",
            no, attestation_type_id
        ),
    }
}

async fn fetch_attestation_batches(
    connection: &DatabaseConnection,
    status_lists: &[status_list::Model],
) -> Vec<attestation_batch::Model> {
    let ids = status_lists.iter().map(|list| list.id).collect::<HashSet<_>>();
    attestation_batch::Entity::find()
        .order_by_asc(attestation_batch::Column::Id)
        .all(connection)
        .await
        .unwrap()
        .into_iter()
        .filter(|batch| {
            let locations =
                serde_json::from_value::<Vec<StatusListLocation>>(batch.status_list_locations.clone()).unwrap();
            locations.iter().any(|l| ids.contains(&l.list_id))
        })
        .collect()
}

#[tokio::test]
async fn test_service_initializes_status_lists() {
    let (connection, settings) = connection_and_settings(10, 1).await.unwrap();
    let (attestation_types, _) = create_status_list_service(&connection, settings, 1).await.unwrap();

    let db_types = attestation_type::Entity::find()
        .filter(attestation_type::Column::Name.is_in(attestation_types))
        .all(&connection)
        .await
        .unwrap();
    assert_eq!(db_types.len(), 1);
    assert_eq!(db_types[0].next_sequence_no, 10);

    let db_lists = fetch_status_list(&connection, db_types[0].id).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 10, 10, 10, false).await;
}

#[tokio::test]
async fn test_service_initializes_schedule_housekeeping() {
    let (connection, settings) = connection_and_settings(5, 2).await.unwrap();
    let (attestation_types, _) = create_status_list_service(&connection, settings.clone(), 3)
        .await
        .unwrap();

    let type_ids = attestation_ids(&connection, attestation_types.as_slice()).await;

    // Make list for first attestation type empty
    update_availability_of_status_list(&connection, *type_ids.get_index(0).unwrap().1, 0).await;

    // Make list for second attestation type almost empty
    update_availability_of_status_list(&connection, *type_ids.get_index(1).unwrap().1, 1).await;

    // Recreate list with large list size
    let settings = StatusListsSettings {
        list_size: 6.try_into().unwrap(),
        ..settings
    };
    let _ = recreate_status_list_service(&connection, settings.clone(), &type_ids.keys().cloned().collect_vec()).await;

    // Check for empty list if new one is created and properly cleaned up
    let db_lists = fetch_status_list(&connection, *type_ids.get_index(0).unwrap().1).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 0, 5, 5, true).await;
    assert_status_list_items(&connection, &db_lists[1], 6, 6, 11, false).await;

    // Check for almost empty list if new list is created, but items are still there
    let db_lists = fetch_status_list(&connection, *type_ids.get_index(1).unwrap().1).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 1, 5, 5, false).await;
    assert_status_list_items(&connection, &db_lists[1], 6, 6, 11, false).await;

    // Full list should still be same
    let db_lists = fetch_status_list(&connection, *type_ids.get_index(2).unwrap().1).await;
    assert_eq!(db_lists.len(), 1);
    assert_status_list_items(&connection, &db_lists[0], 5, 5, 5, false).await;
}

#[tokio::test]
async fn test_service_create_status_claims() {
    let (connection, settings) = connection_and_settings(9, 5).await.unwrap();
    let (attestation_types, service) = create_status_list_service(&connection, settings, 1).await.unwrap();

    let db_type_ids = attestation_ids(&connection, attestation_types.as_slice()).await;
    let attestation_type_id = *db_type_ids.get_index(0).unwrap().1;
    update_availability_of_status_list(&connection, attestation_type_id, 8).await;

    let batch_id = Uuid::new_v4();
    let base_url = "https://example.com/tsl".parse().unwrap();
    let expiration_date: DateTimeSeconds = Utc::now().into();
    let (claims, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(
            &attestation_types[0],
            batch_id,
            base_url,
            Some(expiration_date),
            2.try_into().unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tasks.len(), 0);

    let db_lists = fetch_status_list(&connection, attestation_type_id).await;
    assert_eq!(db_lists.len(), 1); // No new list creation scheduled
    let db_list_items = assert_status_list_items(&connection, &db_lists[0], 6, 9, 9, false).await;

    assert_eq!(claims.len(), 2.try_into().unwrap());
    assert_matches!(&claims[0], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_list_items[1].index as u32,
        uri: format!("https://example.com/tsl/{}", db_lists[0].external_id).parse().unwrap(),
    });
    assert_matches!(&claims[1], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_list_items[2].index as u32,
        uri: format!("https://example.com/tsl/{}", db_lists[0].external_id).parse().unwrap(),
    });

    let db_attestations = fetch_attestation_batches(&connection, &db_lists).await;
    assert_eq!(db_attestations.len(), 1);
    assert_eq!(db_attestations[0].batch_id, batch_id);
    assert_eq!(
        db_attestations[0].expiration_date,
        Some(DateTime::from(expiration_date).date_naive())
    );
    assert!(!db_attestations[0].is_revoked);

    let locations =
        serde_json::from_value::<Vec<StatusListLocation>>(db_attestations[0].status_list_locations.clone()).unwrap();
    assert_eq!(locations.len(), 2);
    assert_eq!(
        locations[0],
        StatusListLocation {
            list_id: db_lists[0].id,
            index: db_list_items[1].index as u32
        }
    );
    assert_eq!(
        locations[1],
        StatusListLocation {
            list_id: db_lists[0].id,
            index: db_list_items[2].index as u32
        }
    );
}

#[tokio::test]
async fn test_service_create_status_claims_creates_in_flight_if_needed() {
    let (connection, settings) = connection_and_settings(8, 1).await.unwrap();
    let (attestation_types, service) = create_status_list_service(&connection, settings, 1).await.unwrap();

    let db_type_ids = attestation_ids(&connection, attestation_types.as_slice()).await;
    let attestation_type_id = *db_type_ids.get_index(0).unwrap().1;
    update_availability_of_status_list(&connection, attestation_type_id, 1).await;

    // Fetch items of current list now, since they will be deleted
    let db_lists = fetch_status_list(&connection, attestation_type_id).await;
    assert_eq!(db_lists.len(), 1);
    let db_old_list_items = assert_status_list_items(&connection, &db_lists[0], 1, 8, 8, false).await;

    let batch_id = Uuid::new_v4();
    let base_url = "https://example.com/tsl".parse().unwrap();
    let (claims, tasks) = service
        .obtain_status_claims_and_scheduled_tasks(
            &attestation_types[0],
            batch_id,
            base_url,
            None,
            2.try_into().unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tasks.len(), 2);
    try_join_all(tasks.into_iter()).await.unwrap();

    let db_lists = fetch_status_list(&connection, attestation_type_id).await;
    assert_eq!(db_lists.len(), 2);
    assert_status_list_items(&connection, &db_lists[0], 0, 8, 8, true).await;
    let db_new_list_items = assert_status_list_items(&connection, &db_lists[1], 7, 8, 16, false).await;

    assert_eq!(claims.len(), 2.try_into().unwrap());
    assert_matches!(&claims[0], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_old_list_items[7].index as u32,
        uri: format!("https://example.com/tsl/{}", db_lists[0].external_id).parse().unwrap(),
    });
    assert_matches!(&claims[1], StatusClaim::StatusList(list) if *list == StatusListClaim {
        idx: db_new_list_items[0].index as u32,
        uri: format!("https://example.com/tsl/{}", db_lists[1].external_id).parse().unwrap(),
    });

    let db_attestations = fetch_attestation_batches(&connection, &db_lists).await;
    assert_eq!(db_attestations.len(), 1);
    assert_eq!(db_attestations[0].batch_id, batch_id);
    assert_eq!(db_attestations[0].expiration_date, None);
    assert!(!db_attestations[0].is_revoked);

    let locations =
        serde_json::from_value::<Vec<StatusListLocation>>(db_attestations[0].status_list_locations.clone()).unwrap();
    assert_eq!(locations.len(), 2);
    assert_eq!(
        locations[0],
        StatusListLocation {
            list_id: db_lists[0].id,
            index: db_old_list_items[7].index as u32
        }
    );
    assert_eq!(
        locations[1],
        StatusListLocation {
            list_id: db_lists[1].id,
            index: db_new_list_items[0].index as u32
        }
    );
}

#[tokio::test]
async fn test_service_create_status_claims_concurrently() {
    let (connection, settings) = connection_and_settings(24, 2).await.unwrap();
    let (attestation_types, service) = create_status_list_service(&connection, settings, 1).await.unwrap();

    let db_type_ids = attestation_ids(&connection, attestation_types.as_slice()).await;
    let attestation_type_id = *db_type_ids.get_index(0).unwrap().1;

    let concurrent = 7;
    let num_copies = 3.try_into().unwrap();
    let base_url: BaseUrl = "https://example.com/tsl".parse().unwrap();
    let claims_per_batch = try_join_all((0..concurrent).map(|_| {
        service.obtain_status_claims(
            &attestation_types[0],
            Uuid::new_v4(),
            base_url.clone(),
            None,
            num_copies,
        )
    }))
    .await
    .unwrap();

    let db_lists = fetch_status_list(&connection, attestation_type_id).await;
    assert_eq!(db_lists.len(), 1); // No new list creation scheduled
    let mut db_list_items = assert_status_list_items(&connection, &db_lists[0], 3, 24, 24, false).await;

    let url: Url = format!("https://example.com/tsl/{}", db_lists[0].external_id)
        .parse()
        .unwrap();
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
