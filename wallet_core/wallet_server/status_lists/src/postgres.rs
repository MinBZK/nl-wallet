use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use derive_more::From;
use derive_more::Into;
use futures::future::try_join_all;
use itertools::Itertools;
use rand::seq::SliceRandom;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DatabaseTransaction;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::NotSet;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::SelectColumns;
use sea_orm::Set;
use sea_orm::TransactionTrait;
use sea_orm::TryInsertResult;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::LockBehavior;
use sea_orm::sea_query::LockType;
use sea_orm::sea_query::OnConflict;
use sea_orm::sea_query::Query;
use sea_orm::sqlx::types::chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use http_utils::urls::BaseUrl;
use http_utils::urls::BaseUrlError;
use token_status_list::status_claim::StatusClaim;
use token_status_list::status_claim::StatusListClaim;
use token_status_list::status_list_service::StatusListService;
use tokio::task::JoinError;
use tokio::task::JoinHandle;
use utils::date_time_seconds::DateTimeSeconds;
use utils::ints::NonZeroU31;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;

use crate::config::StatusListConfig;
use crate::config::StatusListConfigs;
use crate::entity::attestation_batch;
use crate::entity::attestation_type;
use crate::entity::status_list;
use crate::entity::status_list_item;

/// Length of the external id for status lists used in the url (alphanumeric characters)
const STATUS_LIST_EXTERNAL_ID_SIZE: usize = 12;

/// Number of tries to create status list while obtaining a status claim.
const STATUS_LIST_IN_FLIGHT_CREATE_TRIES: usize = 5;

/// StatusListService implementation using Postgres for multiple attestation types.
///
/// See [PostgresStatusListService] for more.
#[derive(Debug, Clone)]
pub struct PostgresStatusListServices(HashMap<String, PostgresStatusListService>);

/// StatusListService implementation using Postgres.
///
/// StatusListService tries to obtain status lists locations with minimal write queries.
/// This is implemented by creating all items for every list upfront (in the background).
/// When the create_threshold is too low, this will happen in flight.
///
/// When a status list is depleted the deletion of the items (which are not necessary anymore)
/// will be scheduled in the background.
///
/// On creation the service will schedule housekeeping for all lists that still have list items.
///
/// The items of the status list have a sequence number on total order per attestation type. This
/// simplifies the queries to fetch the available items. The next sequence number of the status list
/// is the exclusive end of the sequence numbers used for that status list and the start of a new
/// status list. This next sequence number is also stored on the attestation type to detect a
/// concurrent creation of the list by a separate instance.
#[derive(Debug, Clone)]
pub struct PostgresStatusListService {
    connection: DatabaseConnection,
    /// ID of the attestation type in the DB
    attestation_type_id: i16,

    list_size: NonZeroU31,
    create_threshold: NonZeroU31,

    base_url: BaseUrl,
    _publish_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum StatusListServiceError {
    #[error("base url error: {0}")]
    BaseUrl(#[from] BaseUrlError),

    #[error("database error: {0}")]
    Db(#[from] DbErr),

    #[error("could not randomize indices: {0}")]
    Indices(#[from] JoinError),

    #[error("invalid publish dir: {0}")]
    InvalidPublishDir(PathBuf),

    #[error("io error for `{0}`: {1}")]
    IO(PathBuf, #[source] std::io::Error),

    #[error("could not serialize / deserialize: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("no status list available and could not create one")]
    NoStatusListAvailable(),

    #[error("too many claims requested: {0}")]
    TooManyClaimsRequested(usize),

    #[error("unknown attestation type: {0}")]
    UnknownAttestationType(String),
}

#[derive(Debug, Clone, PartialEq, From, Into, Serialize, Deserialize)]
#[serde(into = "(i64,u32)", try_from = "(i64,u32)")]
pub struct StatusListLocation {
    pub list_id: i64,
    pub index: u32,
}

impl StatusListService for PostgresStatusListServices {
    type Error = StatusListServiceError;

    async fn obtain_status_claims(
        &self,
        attestation_type: &str,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
        log::debug!(
            "Obtaining status claims for {} with {} copies",
            attestation_type,
            copies
        );

        let service = self
            .0
            .get(attestation_type)
            .ok_or(StatusListServiceError::UnknownAttestationType(
                attestation_type.to_string(),
            ))?;

        service.obtain_status_claims(batch_id, expires, copies).await
    }
}

impl PostgresStatusListServices {
    pub async fn try_new(
        connection: DatabaseConnection,
        configs: StatusListConfigs,
    ) -> Result<Self, StatusListServiceError> {
        let attestation_type_ids = initialize_attestation_type_ids(&connection, configs.types()).await?;
        let publish_dirs = try_join_all(
            configs
                .as_ref()
                .values()
                .map(|config| check_publish_dir(config.publish_dir.as_path())),
        )
        .await?;
        let services = configs
            .into_iter()
            .zip(publish_dirs.into_iter())
            .map(|((attestation_type, config), publish_dir)| {
                let attestation_type_id = *attestation_type_ids
                    .get(&attestation_type)
                    .expect("attestation_type_ids should have entry for initialized types");
                let service = PostgresStatusListService {
                    connection: connection.clone(),
                    attestation_type_id,
                    list_size: config.list_size,
                    create_threshold: config.create_threshold,
                    base_url: config.base_url,
                    _publish_dir: publish_dir,
                };
                (attestation_type, service)
            })
            .collect();
        Ok(PostgresStatusListServices(services))
    }
    pub async fn initialize_lists(&self) -> Result<Vec<JoinHandle<()>>, StatusListServiceError> {
        let results = try_join_all(self.0.values().map(|service| service.initialize_lists())).await?;
        Ok(results.into_iter().flat_map(|tasks| tasks.into_iter()).collect())
    }
}

impl PostgresStatusListService {
    pub async fn try_new(
        connection: DatabaseConnection,
        attestation_type: &str,
        config: StatusListConfig,
    ) -> Result<Self, StatusListServiceError> {
        let attestation_types = vec![attestation_type];
        let attestation_type_ids = initialize_attestation_type_ids(&connection, attestation_types).await?;

        let attestation_type_id = *attestation_type_ids
            .get(attestation_type)
            .expect("attestation_type_ids should have entry for initialized types");

        Ok(Self {
            connection,
            attestation_type_id,
            list_size: config.list_size,
            create_threshold: config.create_threshold,
            base_url: config.base_url,
            _publish_dir: check_publish_dir(&config.publish_dir).await?,
        })
    }

    pub async fn obtain_status_claims(
        &self,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, StatusListServiceError> {
        self.obtain_status_claims_and_scheduled_tasks(batch_id, expires, copies)
            .await
            .map(|(claims, _)| claims)
    }

    pub async fn obtain_status_claims_and_scheduled_tasks(
        &self,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<(VecNonEmpty<StatusClaim>, Vec<JoinHandle<()>>), StatusListServiceError> {
        let copies = copies.get();

        // If issuer requests more copies than the size of a complete status list,
        // this is a configuration issue.
        if copies > self.list_size.into_inner() as usize {
            return Err(StatusListServiceError::TooManyClaimsRequested(copies));
        }

        // Get status lists with exclusive lock or create if not available
        let (tx, lists) = self.fetch_exclusive_available_status_lists_or_create(copies).await?;

        // Get the next status list items and store them in the attestation_batch table
        let lists_with_items = self.fetch_status_list_items(&tx, lists, copies as u64).await?;
        Self::create_attestation_batch(
            &tx,
            batch_id,
            expires.map(|d| DateTime::from(d).date_naive()),
            lists_with_items
                .iter()
                .flat_map(|(list, items)| {
                    items.iter().map(|item| StatusListLocation {
                        list_id: list.id,
                        index: item.index as u32,
                    })
                })
                .collect(),
        )
        .await?;
        tx.commit().await?;

        // Schedule housekeeping after committing
        let tasks = self.schedule_housekeeping(lists_with_items.iter().map(|(list, _)| list));

        // Convert into StatusClaim using the base url
        let claims = lists_with_items
            .into_iter()
            .flat_map(|(list, items)| {
                let url = self.base_url.join(&list.external_id);
                items.into_iter().map(move |item| {
                    StatusClaim::StatusList(StatusListClaim {
                        idx: item.index as u32,
                        uri: url.clone(),
                    })
                })
            })
            .collect::<Vec<_>>();

        Ok((claims.try_into().unwrap(), tasks))
    }

    async fn fetch_exclusive_available_status_lists_or_create(
        &self,
        copies: usize,
    ) -> Result<(DatabaseTransaction, Vec<status_list::Model>), StatusListServiceError> {
        let mut tries = STATUS_LIST_IN_FLIGHT_CREATE_TRIES;
        loop {
            // Always restart transaction (e.g. level was set to repeatable read)
            let tx = self.connection.begin().await?;
            let lists = status_list::Entity::find()
                .filter(status_list::Column::AttestationTypeId.eq(self.attestation_type_id))
                .filter(status_list::Column::Available.gt(0))
                // Use a lock because we use and update availability afterward
                .lock_exclusive()
                .all(&tx)
                .await?;

            // If the `create_threshold` is large enough compared to the requested claim_size and
            // concurrent requests, this should always be true. If this is false, the
            // `create_threshold` should be increased.
            let available: usize = lists.iter().map(|list| list.available as usize).sum();
            if available >= copies {
                return Ok((tx, lists));
            }

            if tries == STATUS_LIST_IN_FLIGHT_CREATE_TRIES {
                log::warn!(
                    "Creating status list in flight, increase threshold to at least {}",
                    copies
                );
            } else if tries == 0 {
                return Err(StatusListServiceError::NoStatusListAvailable());
            }
            tries -= 1;

            let next_sequence_no = lists.iter().map(|list| list.next_sequence_no).max().unwrap_or_default();
            if !self.create_status_list(next_sequence_no, true).await? {
                log::warn!("Failed to create status list in flight");
            }
        }
    }

    async fn fetch_status_list_items(
        &self,
        tx: &DatabaseTransaction,
        mut lists: Vec<status_list::Model>,
        num_copies: u64,
    ) -> Result<Vec<(status_list::Model, Vec<status_list_item::Model>)>, StatusListServiceError> {
        let start_sequence_no = lists
            .iter()
            .map(|list| list.next_sequence_no - list.available as i64)
            .min();

        let items = status_list_item::Entity::find()
            .filter(status_list_item::Column::AttestationTypeId.eq(self.attestation_type_id))
            .filter(status_list_item::Column::SequenceNo.gte(start_sequence_no))
            .order_by_asc(status_list_item::Column::SequenceNo)
            .limit(num_copies)
            .into_model::<status_list_item::Model>()
            .all(tx)
            .await?;
        if items.len() != num_copies as usize {
            panic!(
                "Insufficient number of items in status list: fetched: {}, requested: {}",
                items.len(),
                num_copies
            );
        }

        // The items are ordered by sequence_no which implies ordering by list_id in the way they
        // are created by, which means `chunk_by` can be used instead of `into_group_map`.
        let mut list_with_items = Vec::with_capacity(lists.len());
        for (list_id, chunk) in &items.into_iter().chunk_by(|item| item.status_list_id) {
            let list = lists.remove(
                lists
                    .iter()
                    .position(|list| list.id == list_id)
                    .unwrap_or_else(|| panic!("List with ID {} not found", list_id)),
            );
            let items = chunk.collect::<Vec<_>>();
            list_with_items.push((list, items));
        }

        // Update availability of status lists (cannot be done in ChunkBy as it is not Send)
        for (list, items) in &mut list_with_items {
            list.available -= items.len() as i32;
            if list.available < 0 {
                panic!("More list items than available in status list for ID {}", list.id);
            }
            let update_result = status_list::Entity::update_many()
                .col_expr(status_list::Column::Available, Expr::value(list.available))
                .filter(status_list::Column::Id.eq(list.id))
                .exec(tx)
                .await?;
            if update_result.rows_affected != 1 {
                panic!("Status list update affected none or multiple rows for ID {}", list.id);
            }
        }

        Ok(list_with_items)
    }

    async fn create_attestation_batch(
        tx: &DatabaseTransaction,
        batch_id: Uuid,
        expiration_date: Option<NaiveDate>,
        locations: Vec<StatusListLocation>,
    ) -> Result<(), StatusListServiceError> {
        let model = attestation_batch::ActiveModel {
            id: NotSet,
            batch_id: Set(batch_id),
            expiration_date: Set(expiration_date),
            is_revoked: Set(false),
            status_list_locations: Set(serde_json::to_value(locations)?),
        };
        attestation_batch::Entity::insert(model).exec(tx).await?;

        Ok(())
    }

    /// Creates new status list if not already created.
    ///
    /// The `next_sequence_no` is used to ensure only a single new list is created.
    async fn create_status_list(
        &self,
        next_sequence_no: i64,
        wait_for_lock: bool,
    ) -> Result<bool, StatusListServiceError> {
        let tx = self.connection.begin().await?;

        // Get exclusive lock on attestation type
        let mut query =
            attestation_type::Entity::find().filter(attestation_type::Column::Id.eq(self.attestation_type_id));
        query = match wait_for_lock {
            false => query.lock_with_behavior(LockType::Update, LockBehavior::SkipLocked),
            true => query.lock_exclusive(),
        };
        let attestation_type = match (query.one(&tx).await?, wait_for_lock) {
            (None, false) => return Ok(false),
            (Some(attestation_type), _) => attestation_type,
            _ => panic!("Missing attestation type for ID {}", self.attestation_type_id),
        };

        // Status list was created by someone else
        if attestation_type.next_sequence_no != next_sequence_no {
            return Ok(false);
        }

        // Create new list
        let list_size = self.list_size.into_inner();
        let new_next_sequence_no = attestation_type.next_sequence_no + list_size as i64;
        let list = status_list::ActiveModel {
            id: NotSet,
            attestation_type_id: Set(self.attestation_type_id),
            external_id: Set(crypto::utils::random_string(STATUS_LIST_EXTERNAL_ID_SIZE)),
            available: Set(list_size),
            size: Set(list_size),
            next_sequence_no: Set(new_next_sequence_no),
        };
        let list_id = match status_list::Entity::insert(list)
            .on_conflict(
                OnConflict::column(status_list::Column::ExternalId)
                    .do_nothing()
                    .to_owned(),
            )
            .on_empty_do_nothing()
            .exec(&tx)
            .await?
        {
            TryInsertResult::Inserted(inserted) => inserted.last_insert_id,
            _ => return Ok(false),
        };

        // Create new list items
        let indices = tokio::task::spawn_blocking(move || {
            let mut indices = (0..list_size).collect::<Vec<_>>();
            indices.shuffle(&mut rand::thread_rng());
            indices
        })
        .await?;

        // Insert items into batches limited by u16::MAX params
        let mut next_sequence_no = next_sequence_no as usize;
        for chunk in indices.chunks((u16::MAX / 4) as usize) {
            let items = chunk
                .iter()
                .enumerate()
                .map(|(k, index)| status_list_item::ActiveModel {
                    attestation_type_id: Set(self.attestation_type_id),
                    sequence_no: Set((next_sequence_no + k) as i64),
                    status_list_id: Set(list_id),
                    index: Set(*index),
                })
                .collect::<Vec<_>>();
            next_sequence_no += items.len();
            status_list_item::Entity::insert_many(items).exec(&tx).await?;
        }

        // Update next sequence no of attestation type
        assert_eq!(
            next_sequence_no, new_next_sequence_no as usize,
            "Inserted items did not match calculated sequence number"
        );
        let mut attestation_type = attestation_type.into_active_model();
        attestation_type.next_sequence_no = Set(new_next_sequence_no);
        attestation_type::Entity::update(attestation_type).exec(&tx).await?;

        tx.commit().await?;
        Ok(true)
    }

    pub async fn initialize_lists(&self) -> Result<Vec<JoinHandle<()>>, StatusListServiceError> {
        log::info!("Initializing status lists");

        // Fetch all lists that still have list items in the database
        let lists = status_list::Entity::find()
            .filter(status_list::Column::AttestationTypeId.eq(self.attestation_type_id))
            .filter(
                status_list::Column::Id.in_subquery(
                    Query::select()
                        .distinct()
                        .expr(Expr::column(status_list_item::Column::StatusListId))
                        .from(status_list_item::Entity)
                        .to_owned(),
                ),
            )
            .into_model::<status_list::Model>()
            .all(&self.connection)
            .await?;

        // Create status lists if all lists for this attestation type are full
        if lists.is_empty() {
            let next_sequence_no = attestation_type::Entity::find_by_id(self.attestation_type_id)
                .select_only()
                .select_column(attestation_type::Column::NextSequenceNo)
                .into_tuple()
                .one(&self.connection)
                .await?
                .unwrap_or_else(|| panic!("Missing attestation type for ID {}", self.attestation_type_id));

            log::info!(
                "Schedule creation of status list items for {}",
                self.attestation_type_id
            );
            let service = self.clone();
            let task = tokio::spawn(async move { service.create_status_list_in_background(next_sequence_no).await });
            Ok(vec![task])
        } else {
            Ok(self.schedule_housekeeping(&lists))
        }
    }

    fn schedule_housekeeping<'a>(
        &self,
        lists: impl IntoIterator<Item = &'a status_list::Model>,
    ) -> Vec<JoinHandle<()>> {
        let mut tasks = Vec::new();
        for list in lists {
            if list.available == 0 {
                log::info!("Schedule deletion of status list items for ID {}", list.id);

                let connection = self.connection.clone();
                let list_id = list.id;
                tasks.push(tokio::spawn(Self::delete_status_list_items(connection, list_id)));
            }
            if list.available <= self.create_threshold.into_inner() {
                log::info!(
                    "Schedule creation of status list items for attestation type ID {}",
                    list.attestation_type_id
                );

                let service = self.clone();
                let next_sequence_no = list.next_sequence_no;
                tasks.push(tokio::spawn(async move {
                    service.create_status_list_in_background(next_sequence_no).await
                }));
            }
        }
        tasks
    }

    async fn create_status_list_in_background(&self, next_sequence_no: i64) {
        // No wait, as this can be spawned multiple times, e.g.
        //  - when multiple instances are initializing the list
        //  - when the threshold is hit and the status list is not created yet.
        // Waiting will only hog connections from the DB pool waiting for the lock.
        match self.create_status_list(next_sequence_no, false).await {
            Ok(created) if created => log::info!(
                "Created status list for attestation type ID {}",
                self.attestation_type_id
            ),
            Err(err) => log::warn!("Failed to create status list: {}", err),
            _ => {}
        };
    }

    async fn delete_status_list_items(connection: DatabaseConnection, id: i64) {
        let result = status_list_item::Entity::delete_many()
            .filter(status_list_item::Column::StatusListId.eq(id))
            .exec(&connection)
            .await;

        if let Err(err) = result {
            log::warn!("Failed to delete status list items of {}: {}", id, err);
        }
    }
}

async fn initialize_attestation_type_ids(
    connection: &DatabaseConnection,
    attestation_types: Vec<&str>,
) -> Result<HashMap<String, i16>, DbErr> {
    let map = fetch_attestation_type_ids(connection, attestation_types.iter().copied()).await?;
    let insert = attestation_types
        .iter()
        .filter_map(|attestation_type| match map.get(*attestation_type) {
            None => Some(attestation_type::ActiveModel {
                id: NotSet,
                name: Set(attestation_type.to_string()),
                next_sequence_no: Set(0),
            }),
            _ => None,
        });
    match attestation_type::Entity::insert_many(insert)
        .on_conflict(
            OnConflict::column(attestation_type::Column::Name)
                .do_nothing()
                .to_owned(),
        )
        .on_empty_do_nothing()
        .exec(connection)
        .await?
    {
        TryInsertResult::Empty => Ok(map),
        _ => {
            let map = fetch_attestation_type_ids(connection, attestation_types).await?;
            Ok(map)
        }
    }
}

async fn fetch_attestation_type_ids(
    connection: &DatabaseConnection,
    attestation_types: impl IntoIterator<Item = &str>,
) -> Result<HashMap<String, i16>, DbErr> {
    attestation_type::Entity::find()
        .filter(attestation_type::Column::Name.is_in(attestation_types))
        .all(connection)
        .await
        .map(|models| {
            models
                .into_iter()
                .map(|model| (model.name, model.id))
                .collect::<HashMap<_, _>>()
        })
}

async fn check_publish_dir(publish_dir: &Path) -> Result<PathBuf, StatusListServiceError> {
    let publish_path = prefix_local_path(publish_dir);
    let metadata = tokio::fs::metadata(&publish_path)
        .await
        .map_err(|err| StatusListServiceError::IO(publish_dir.to_path_buf(), err))?;
    if !metadata.is_dir() {
        return Err(StatusListServiceError::InvalidPublishDir(publish_dir.to_path_buf()));
    }
    Ok(publish_path.into_owned())
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    fn mock_service() -> PostgresStatusListService {
        PostgresStatusListService {
            connection: DatabaseConnection::default(),
            attestation_type_id: 1,
            list_size: 1.try_into().unwrap(),
            create_threshold: 1.try_into().unwrap(),
            base_url: "https://example.com/tsl".parse().unwrap(),
            _publish_dir: std::env::temp_dir(),
        }
    }

    #[tokio::test]
    async fn test_service_obtain_status_claims_uninitialized_attestation_type() {
        let service = PostgresStatusListServices([("pid".to_string(), mock_service())].into());

        let batch_id = Uuid::new_v4();
        let result = service
            .obtain_status_claims("invalid", batch_id, None, 1.try_into().unwrap())
            .await;
        assert_matches!(result, Err(StatusListServiceError::UnknownAttestationType(attestation_type)) if attestation_type == "invalid");
    }

    #[tokio::test]
    async fn test_service_obtain_status_claims_too_many_copies() {
        let service = mock_service();

        let batch_id = Uuid::new_v4();
        let result = service
            .obtain_status_claims_and_scheduled_tasks(batch_id, None, 3.try_into().unwrap())
            .await;
        assert_matches!(result, Err(StatusListServiceError::TooManyClaimsRequested(size)) if size == 3);
    }
}
