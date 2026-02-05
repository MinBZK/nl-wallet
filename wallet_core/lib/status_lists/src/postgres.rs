use std::collections::HashMap;
use std::future;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use futures::StreamExt;
use futures::future::join_all;
use futures::future::try_join_all;
use itertools::Itertools;
use rand::seq::SliceRandom;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DatabaseTransaction;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::JoinType;
use sea_orm::NotSet;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
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
use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use crypto::EcdsaKeySend;
use http_utils::urls::BaseUrlError;
use jwt::error::JwtError;
use measure::measure;
use token_status_list::status_list::PackedStatusList;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_service::BatchIsRevoked;
use token_status_list::status_list_service::RevocationError;
use token_status_list::status_list_service::StatusListRevocationService;
use token_status_list::status_list_service::StatusListService;
use token_status_list::status_list_service::StatusListServices;
use token_status_list::status_list_token::StatusListToken;
use token_status_list::status_list_token::StatusListTokenBuilder;
use tokio::task::AbortHandle;
use tokio::task::JoinError;
use tokio::task::JoinHandle;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

use crate::config::StatusListConfig;
use crate::config::StatusListConfigs;
use crate::entity::attestation_batch;
use crate::entity::attestation_batch_list_indices;
use crate::entity::attestation_type;
use crate::entity::status_list;
use crate::entity::status_list_item;
use crate::flag::Flag;
use crate::publish::LockVersion;
use crate::publish::PublishLockError;
use crate::refresh::RefreshControl;

/// Length of the external id for status lists used in the url (alphanumeric characters)
const EXTERNAL_ID_SIZE: usize = 12;

/// Number of tries to create status list while obtaining a status claim.
const IN_FLIGHT_CREATE_TRIES: usize = 5;

/// Flag name (as a key) used in the database to revoke all status lists
const FLAG_NAME_REVOKE_ALL: &str = "revoke_all";

/// Maximal concurrent publish when revoke_all is called
const REVOKE_ALL_MAX_CONCURRENT: usize = 16;

/// StatusListService implementation using Postgres for multiple attestation types.
///
/// See [PostgresStatusListService] for more.
#[derive(Debug)]
pub struct PostgresStatusListServices<K>(HashMap<String, PostgresStatusListService<K>>);

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
#[derive(Debug)]
pub struct PostgresStatusListService<K> {
    connection: DatabaseConnection,
    /// ID of the attestation type in the DB
    attestation_type_id: i16,
    config: Arc<StatusListConfig<K>>,
    revoke_all_flag: Flag,
}

// Manually implement Clone as derived Clone uses incorrect bounds:
// https://github.com/rust-lang/rust/issues/26925#issue-94161444
impl<K> Clone for PostgresStatusListService<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
            attestation_type_id: self.attestation_type_id,
            config: self.config.clone(),
            revoke_all_flag: self.revoke_all_flag.clone(),
        }
    }
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

    #[error("io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("io error for `{0}`: {1}")]
    IOWithPath(PathBuf, #[source] std::io::Error),

    #[error("could write JWT: {0}")]
    JWT(#[from] JwtError),

    #[error("no status lists in config")]
    NoStatusLists,

    #[error("no status list available and could not create one")]
    NoStatusListAvailable,

    #[error("could not lock for publish: {0}")]
    PublishLock(#[from] PublishLockError),

    #[error("too many claims requested: {0}")]
    TooManyClaimsRequested(usize),

    #[error("unknown attestation type: {0}")]
    UnknownAttestationType(String),
}

impl<K> StatusListServices for PostgresStatusListServices<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    type Error = StatusListServiceError;

    async fn obtain_status_claims(
        &self,
        attestation_type: &str,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
        tracing::debug!(
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

        service
            .obtain_status_claims_and_scheduled_tasks(batch_id, expires, copies)
            .await
            .map(|(claims, _)| claims)
    }
}

impl<K> StatusListRevocationService for PostgresStatusListServices<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    async fn revoke_attestation_batches(&self, batch_ids: Vec<Uuid>) -> Result<(), RevocationError> {
        // Each service is responsible for revoking and publishing the status list it is configured for.
        try_join_all(
            self.services()
                .map(|service| async { service.revoke_attestation_batches(batch_ids.clone()).await }),
        )
        .await?;

        Ok(())
    }

    async fn get_attestation_batch(&self, batch_id: Uuid) -> Result<BatchIsRevoked, RevocationError> {
        self.first_service().get_attestation_batch(batch_id).await
    }

    async fn list_attestation_batches(&self) -> Result<Vec<BatchIsRevoked>, RevocationError> {
        self.first_service().list_attestation_batches().await
    }
}

impl<K> PostgresStatusListServices<K> {
    pub async fn try_new(
        connection: DatabaseConnection,
        configs: StatusListConfigs<K>,
    ) -> Result<Self, StatusListServiceError> {
        if configs.as_ref().is_empty() {
            return Err(StatusListServiceError::NoStatusLists);
        }

        let attestation_type_ids = initialize_attestation_type_ids(&connection, configs.types()).await?;
        let services = configs
            .into_iter()
            .map(|(attestation_type, config)| {
                let attestation_type_id = *attestation_type_ids
                    .get(&attestation_type)
                    .expect("attestation_type_ids should have entry for initialized types");
                let service = PostgresStatusListService {
                    connection: connection.clone(),
                    attestation_type_id,
                    config: Arc::new(config),
                    revoke_all_flag: Flag::new(connection.clone(), FLAG_NAME_REVOKE_ALL.to_string()),
                };
                (attestation_type, service)
            })
            .collect();
        Ok(PostgresStatusListServices(services))
    }
}

impl<K> PostgresStatusListServices<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    pub async fn initialize_lists(&self) -> Result<Vec<JoinHandle<()>>, StatusListServiceError> {
        let results = try_join_all(self.0.values().map(|service| service.initialize_lists())).await?;
        Ok(results.into_iter().flat_map(|tasks| tasks.into_iter()).collect())
    }

    fn services(&self) -> impl Iterator<Item = &PostgresStatusListService<K>> {
        self.0.values()
    }

    fn first_service(&self) -> &PostgresStatusListService<K> {
        // in the constructor we ensure that at least one service is present
        self.0.values().next().expect("at least one service should be present")
    }

    pub fn start_refresh_jobs(&self) -> Vec<AbortHandle> {
        self.0.values().map(|service| service.start_refresh_job()).collect()
    }
}

impl<K> StatusListService for PostgresStatusListService<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    type Error = StatusListServiceError;

    async fn obtain_status_claims(
        &self,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
        tracing::debug!("Obtaining status claims with {} copies", copies);
        self.obtain_status_claims_and_scheduled_tasks(batch_id, expires, copies)
            .await
            .map(|(claims, _)| claims)
    }
}

impl<K> StatusListRevocationService for PostgresStatusListService<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    async fn revoke_attestation_batches(&self, batch_ids: Vec<Uuid>) -> Result<(), RevocationError> {
        // Find batches and status_lists for this service by batch_ids
        let batches: Vec<(i64, i64, String, i32)> = attestation_batch::Entity::find()
            .select_only()
            .column(attestation_batch::Column::Id)
            .select_column(status_list::Column::Id)
            .select_column(status_list::Column::ExternalId)
            .select_column(status_list::Column::Size)
            .inner_join(status_list::Entity)
            .filter(
                attestation_batch::Column::BatchId
                    .is_in(batch_ids)
                    .and(status_list::Column::AttestationTypeId.eq(self.attestation_type_id)),
            )
            .into_tuple()
            .all(&self.connection)
            .await
            .map_err(|e| RevocationError::InternalError(Box::new(e)))?;

        let (attestation_batch_ids, status_list_info): (Vec<_>, Vec<_>) = batches
            .into_iter()
            .map(|(batch_id, status_list_id, external_id, size)| (batch_id, (status_list_id, external_id, size)))
            .unzip();

        // Update revocation for all batches
        attestation_batch::Entity::update_many()
            .col_expr(attestation_batch::Column::IsRevoked, Expr::value(true))
            .filter(attestation_batch::Column::Id.is_in(attestation_batch_ids))
            .exec(&self.connection)
            .await
            .map_err(|e| RevocationError::InternalError(Box::new(e)))?;

        // Publish new status list
        try_join_all(status_list_info.into_iter().unique_by(|(id, _, _)| *id).map(
            |(list_id, external_id, size)| async move {
                let size = size.try_into().expect("size should be non-zero");
                self.publish_status_list(list_id, external_id.as_str(), size).await
            },
        ))
        .await
        .map_err(|e| RevocationError::InternalError(Box::new(e)))?;

        Ok(())
    }

    async fn get_attestation_batch(&self, batch_id: Uuid) -> Result<BatchIsRevoked, RevocationError> {
        attestation_batch::Entity::find()
            .filter(attestation_batch::Column::BatchId.eq(batch_id))
            .select_only()
            .select_column(attestation_batch::Column::BatchId)
            .select_column(attestation_batch::Column::IsRevoked)
            .into_tuple()
            .one(&self.connection)
            .await
            .map_err(|e| RevocationError::InternalError(Box::new(e)))?
            .map(|(batch_id, is_revoked)| BatchIsRevoked { batch_id, is_revoked })
            .ok_or_else(|| RevocationError::BatchIdNotFound(batch_id))
    }

    async fn list_attestation_batches(&self) -> Result<Vec<BatchIsRevoked>, RevocationError> {
        Ok(attestation_batch::Entity::find()
            .select_only()
            .select_column(attestation_batch::Column::BatchId)
            .select_column(attestation_batch::Column::IsRevoked)
            .into_tuple()
            .all(&self.connection)
            .await
            .map_err(|e| RevocationError::InternalError(Box::new(e)))?
            .into_iter()
            .map(|(batch_id, is_revoked)| BatchIsRevoked { batch_id, is_revoked })
            .collect())
    }
}

impl<K> PostgresStatusListService<K> {
    pub async fn try_new(
        connection: DatabaseConnection,
        attestation_type: &str,
        config: StatusListConfig<K>,
    ) -> Result<Self, StatusListServiceError> {
        let revoke_all_flag = Flag::new(connection.clone(), FLAG_NAME_REVOKE_ALL.to_string());
        Self::try_new_with_flag(connection, attestation_type, config, revoke_all_flag).await
    }

    pub async fn try_new_with_flag(
        connection: DatabaseConnection,
        attestation_type: &str,
        config: StatusListConfig<K>,
        revoke_all_flag: Flag,
    ) -> Result<Self, StatusListServiceError> {
        let attestation_types = vec![attestation_type];
        let attestation_type_ids = initialize_attestation_type_ids(&connection, attestation_types).await?;

        let attestation_type_id = *attestation_type_ids
            .get(attestation_type)
            .expect("attestation_type_ids should have entry for initialized types");

        Ok(Self {
            connection,
            attestation_type_id,
            config: Arc::new(config),
            revoke_all_flag,
        })
    }
}

impl<K> PostgresStatusListService<K>
where
    K: EcdsaKeySend + Sync + 'static,
{
    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    pub async fn obtain_status_claims_and_scheduled_tasks(
        &self,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<(VecNonEmpty<StatusClaim>, Vec<JoinHandle<()>>), StatusListServiceError> {
        let copies = copies.get();

        // If issuer requests more copies than the size of a complete status list,
        // this is a configuration issue.
        if copies > self.config.list_size.as_usize() {
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
            lists_with_items.iter().map(|(list, items)| {
                let indices = items.iter().map(|item| item.index).collect();
                (list.id, indices)
            }),
        )
        .await?;
        tx.commit().await?;

        // Schedule housekeeping after committing
        let tasks = self.schedule_housekeeping(lists_with_items.iter().map(|(list, _)| list));

        // Convert into StatusClaim using the base url
        let claims = lists_with_items
            .into_iter()
            .flat_map(|(list, items)| {
                let url = self.config.base_url.join(&list.external_id);
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

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    async fn fetch_exclusive_available_status_lists_or_create(
        &self,
        copies: usize,
    ) -> Result<(DatabaseTransaction, Vec<status_list::Model>), StatusListServiceError> {
        let mut tries = IN_FLIGHT_CREATE_TRIES;
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

            if tries == IN_FLIGHT_CREATE_TRIES {
                tracing::warn!(
                    "Creating status list in flight for attestation type ID {}, increase create_threshold or list_size",
                    self.attestation_type_id,
                )
            } else if tries == 0 {
                return Err(StatusListServiceError::NoStatusListAvailable);
            }
            tries -= 1;

            let max_next_sequence_no = if let Some(max) = lists.iter().map(|list| list.next_sequence_no).max() {
                max
            } else {
                status_list::Entity::find()
                    .select_only()
                    .column_as(status_list::Column::NextSequenceNo.max(), "max_next_sequence_no")
                    .filter(status_list::Column::AttestationTypeId.eq(self.attestation_type_id))
                    .group_by(status_list::Column::AttestationTypeId)
                    .into_tuple::<Option<i64>>()
                    .one(&tx)
                    .await?
                    .flatten()
                    .unwrap_or_default()
            };

            let _ = self
                .create_status_list(max_next_sequence_no, true)
                .await
                .inspect_err(|err| {
                    tracing::error!(
                        "Error creating status list in flight for attestation type ID {}: {:?}",
                        self.attestation_type_id,
                        err
                    );
                })?;
        }
    }

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    async fn fetch_status_list_items(
        &self,
        tx: &DatabaseTransaction,
        mut lists: Vec<status_list::Model>,
        num_copies: u64,
    ) -> Result<Vec<(status_list::Model, Vec<status_list_item::Model>)>, StatusListServiceError> {
        let start_sequence_no = lists
            .iter()
            .map(|list| list.next_sequence_no - i64::from(list.available))
            .min();

        let items = status_list_item::Entity::find()
            .filter(status_list_item::Column::AttestationTypeId.eq(self.attestation_type_id))
            .filter(status_list_item::Column::SequenceNo.gte(start_sequence_no))
            .order_by_asc(status_list_item::Column::SequenceNo)
            .limit(num_copies)
            .all(tx)
            .await?;
        if items.len() != num_copies as usize {
            panic!(
                "Insufficient number of items in status list: fetched: {}, requested: {}",
                items.len(),
                num_copies,
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

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    async fn create_attestation_batch(
        tx: &DatabaseTransaction,
        batch_id: Uuid,
        expiration_date: Option<NaiveDate>,
        list_indices: impl Iterator<Item = (i64, Vec<i32>)>,
    ) -> Result<(), StatusListServiceError> {
        let model = attestation_batch::ActiveModel {
            id: NotSet,
            batch_id: Set(batch_id),
            expiration_date: Set(expiration_date),
            is_revoked: Set(false),
        };
        let attestation_batch_id = attestation_batch::Entity::insert(model).exec(tx).await?.last_insert_id;

        let models = list_indices.map(|(list_id, indices)| attestation_batch_list_indices::ActiveModel {
            attestation_batch_id: Set(attestation_batch_id),
            status_list_id: Set(list_id),
            indices: Set(indices),
        });
        attestation_batch_list_indices::Entity::insert_many(models)
            .exec(tx)
            .await?;

        Ok(())
    }

    /// Creates new status list if not already created.
    ///
    /// The `next_sequence_no` is used to ensure only a single new list is created.
    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
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
        let external_id = crypto::utils::random_string(EXTERNAL_ID_SIZE);
        let list_size = self.config.list_size.into_inner();
        let new_next_sequence_no = attestation_type.next_sequence_no + i64::from(list_size);
        let list = status_list::ActiveModel {
            id: NotSet,
            attestation_type_id: Set(self.attestation_type_id),
            external_id: Set(external_id.clone()),
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

        // Publish status list
        let service = self.clone();
        let publish = tokio::spawn(async move { service.publish_new_status_list(&external_id).await });

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
            "Inserted items did not match calculated sequence number",
        );
        let mut attestation_type = attestation_type.into_active_model();
        attestation_type.next_sequence_no = Set(new_next_sequence_no);
        attestation_type::Entity::update(attestation_type).exec(&tx).await?;

        // Wait for publish to complete before committing
        publish.await??;

        tx.commit().await?;
        Ok(true)
    }

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    pub async fn initialize_lists(&self) -> Result<Vec<JoinHandle<()>>, StatusListServiceError> {
        tracing::info!("Initializing status lists for ID {}", self.attestation_type_id);

        // Fetch all lists that still have list items in the database
        let lists = status_list::Entity::find()
            .filter(status_list::Column::AttestationTypeId.eq(self.attestation_type_id))
            .filter(
                status_list::Column::Id.in_subquery(
                    Query::select()
                        .expr(Expr::column(status_list_item::Column::StatusListId))
                        .from(status_list_item::Entity)
                        .to_owned(),
                ),
            )
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

            tracing::info!(
                "Schedule creation of status list items for list ID {}",
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
                tracing::info!("Schedule deletion of status list items for list ID {}", list.id);

                let connection = self.connection.clone();
                let list_id = list.id;
                tasks.push(tokio::spawn(Self::delete_status_list_items(connection, list_id)));
            }
            if list.available < self.config.create_threshold.into_inner() {
                tracing::info!(
                    "Schedule creation of status list items for attestation type ID {}",
                    list.attestation_type_id,
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
            Ok(created) if created => tracing::info!(
                "Created status list for attestation type ID {}",
                self.attestation_type_id,
            ),
            Err(err) => tracing::warn!(
                "Failed to create status list for attestation type ID {}: {}",
                self.attestation_type_id,
                err,
            ),
            _ => {}
        };
    }

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    async fn delete_status_list_items(connection: DatabaseConnection, id: i64) {
        let result = status_list_item::Entity::delete_many()
            .filter(status_list_item::Column::StatusListId.eq(id))
            .exec(&connection)
            .await;

        if let Err(err) = result {
            tracing::warn!("Failed to delete status list items of {}: {}", id, err);
        }
    }

    pub fn start_refresh_job(&self) -> AbortHandle {
        let service = self.clone();
        // Create control before spawning as the constructor can panic on incompatible settings
        let refresh_control = RefreshControl::new(self.config.refresh_threshold);
        tokio::spawn(async move {
            tracing::info!(
                "Starting refresh job for attestation type ID {}",
                service.attestation_type_id
            );
            loop {
                // Wrap in separate spawn job to catch panics
                let job_service = service.clone();
                match tokio::spawn(async move { job_service.refresh_status_lists(&refresh_control).await }).await {
                    Ok(delay) => {
                        tracing::debug!(
                            "Next refresh of status lists scheduled in {}s for attestation type ID {}",
                            delay.as_secs(),
                            service.attestation_type_id
                        );
                        tokio::time::sleep(delay).await
                    }
                    Err(err) => {
                        tracing::error!(
                            "Join error on refresh job for attestation type ID {}: {:?}",
                            service.attestation_type_id,
                            err
                        );
                        tokio::time::sleep(refresh_control.next_refresh_delay([])).await;
                    }
                };
            }
        })
        .abort_handle()
    }

    async fn refresh_status_lists(&self, refresh_control: &RefreshControl) -> Duration {
        tracing::debug!(
            "Refreshing status lists for attestation type ID {}",
            self.attestation_type_id
        );

        // Get all lists
        let lists = match status_list::Entity::find()
            .filter(status_list::Column::AttestationTypeId.eq(self.attestation_type_id))
            .all(&self.connection)
            .await
        {
            Ok(lists) => lists,
            Err(err) => {
                tracing::warn!(
                    "Could not fetch status lists from DB for attestation ID {}: {}",
                    self.attestation_type_id,
                    err
                );
                return refresh_control.next_refresh_delay([]);
            }
        };

        // Republish if necessary
        let expiries = join_all(lists.into_iter().map(|list| async move {
            let path = self.config.publish_dir.jwt_path(&list.external_id);
            let mut expiry = read_token_expiry(&path)
                .await
                .inspect_err(|err| tracing::warn!("Could not read expiry from `{}`: {}", path.display(), err))
                // Ignore error is ok because it is just logged with WARN
                .ok();

            if expiry.is_none_or(|exp| refresh_control.should_refresh(exp)) {
                tracing::info!("Republishing status list for ID {}", list.id);
                let size = list.size.try_into().expect("size should be non-zero");

                match self.publish_status_list(list.id, &list.external_id, size).await {
                    // Always read token expiry as it can be changed by another instance
                    Ok(_) => {
                        expiry = read_token_expiry(&path)
                            .await
                            .inspect_err(|err| {
                                tracing::error!(
                                    "Could not read expiry from just published token `{}`: {}",
                                    path.display(),
                                    err
                                )
                            })
                            // Ignore error is ok because it is just logged with ERROR
                            .ok()
                    }
                    Err(err) => tracing::warn!("Failed to refresh status list for ID {}: {}", list.id, err),
                }
            }
            expiry
        }))
        .await;

        // Calculate delay for next job: if one or more expiry cannot be read,
        // even after republishing, default to empty list.
        refresh_control.next_refresh_delay(expiries.into_iter().collect::<Option<Vec<_>>>().unwrap_or_default())
    }

    async fn publish_new_status_list(&self, external_id: &str) -> Result<(), StatusListServiceError> {
        // Build new status list
        let expires = Utc::now() + self.config.expiry;
        let sub = self.config.base_url.join(external_id);
        let packed = if self.revoke_all_flag.is_set().await? {
            PackedStatusList::all_invalid(self.config.list_size.as_usize())
        } else {
            PackedStatusList::new(self.config.list_size.as_usize())
        };
        let token = StatusListToken::builder(sub, packed)
            .exp(Some(expires))
            .ttl(self.config.ttl)
            .sign(&self.config.key_pair)
            .await?;

        // Write to disk
        let publish_lock = self.config.publish_dir.lock_for(external_id);
        let jwt_path = self.config.publish_dir.jwt_path(external_id);
        tokio::task::spawn_blocking(move || {
            // create because a new status list external id can be reused if the transaction fails
            publish_lock.create(expires)?;
            std::fs::write(&jwt_path, token.as_ref().serialization())
                .map_err(|err| StatusListServiceError::IOWithPath(jwt_path, err))
        })
        .await?
    }

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    async fn publish_status_list(
        &self,
        list_id: i64,
        external_id: &str,
        size: usize,
    ) -> Result<bool, StatusListServiceError> {
        if self.revoke_all_flag.is_set().await? {
            self.publish_status_list_all_revoked(external_id, size).await
        } else {
            self.publish_status_list_from_db(list_id, external_id, size).await
        }
    }

    async fn publish_status_list_from_db(
        &self,
        list_id: i64,
        external_id: &str,
        size: usize,
    ) -> Result<bool, StatusListServiceError> {
        // Fetch all revoked attestation for this status list
        let result: Vec<Vec<i32>> = attestation_batch_list_indices::Entity::find()
            .join(
                JoinType::InnerJoin,
                attestation_batch_list_indices::Relation::AttestationBatch.def(),
            )
            .select_only()
            .select_column(attestation_batch_list_indices::Column::Indices)
            .filter(attestation_batch_list_indices::Column::StatusListId.eq(list_id))
            .filter(attestation_batch::Column::IsRevoked.eq(true))
            .into_tuple()
            .all(&self.connection)
            .await?;

        let expires = Utc::now() + self.config.expiry;
        let version = LockVersion::from(result.len(), expires);
        self.config
            .publish_dir
            .lock_for(external_id)
            .with_lock_if_newer(version, async || {
                // Build packed status list
                let sub = self.config.base_url.join(external_id);
                let builder = tokio::task::spawn_blocking(move || {
                    let mut status_list = StatusList::new(size);
                    for index in result.into_iter().flatten() {
                        status_list.insert(index as usize, StatusType::Invalid);
                    }
                    StatusListToken::builder(sub, status_list.pack())
                })
                .await?;

                self.sign_and_write_token(builder, expires, external_id).await
            })
            .await
    }

    async fn publish_status_list_all_revoked(
        &self,
        external_id: &str,
        size: usize,
    ) -> Result<bool, StatusListServiceError> {
        let expires = Utc::now() + self.config.expiry;
        let version = LockVersion::from(usize::MAX, expires);
        self.config
            .publish_dir
            .lock_for(external_id)
            .with_lock_if_newer(version, async || {
                let sub = self.config.base_url.join(external_id);
                let builder = StatusListToken::builder(sub, PackedStatusList::all_invalid(size));
                self.sign_and_write_token(builder, expires, external_id).await
            })
            .await
    }

    async fn sign_and_write_token(
        &self,
        builder: StatusListTokenBuilder,
        expires: DateTime<Utc>,
        external_id: &str,
    ) -> Result<(), StatusListServiceError> {
        // Sign
        let token = builder
            .exp(Some(expires))
            .ttl(self.config.ttl)
            .sign(&self.config.key_pair)
            .await?;

        // Write to a tempfile and atomically move via rename
        let jwt_path = self.config.publish_dir.jwt_path(external_id);
        let tmp_path = self.config.publish_dir.tmp_path(external_id);
        tokio::task::spawn_blocking(move || {
            let buf = token.as_ref().serialization();
            std::fs::write(&tmp_path, buf).map_err(|err| StatusListServiceError::IOWithPath(tmp_path.clone(), err))?;
            std::fs::rename(&tmp_path, &jwt_path).map_err(|err| StatusListServiceError::IOWithPath(jwt_path, err))
        })
        .await??;

        Ok(())
    }

    #[measure(name = "nlwallet_status_list_operations", "service" => "status_lists")]
    pub async fn revoke_all(&self) -> Result<(), StatusListServiceError> {
        // First set revoke all to ensure new lists are also created with invalid status
        self.revoke_all_flag.set().await?;

        let service = self.clone();
        let mut stream = status_list::Entity::find()
            .select_only()
            .select_column(status_list::Column::ExternalId)
            .select_column(status_list::Column::Size)
            .into_tuple::<(String, i32)>()
            .stream(&self.connection)
            .await?
            .map(async |result| match result {
                Ok((external_id, size)) => {
                    service
                        .publish_status_list_all_revoked(&external_id, size as usize)
                        .await
                }
                Err(err) => future::ready(Err(StatusListServiceError::Db(err))).await,
            })
            .buffer_unordered(REVOKE_ALL_MAX_CONCURRENT);

        let mut result = Ok(());
        while let Some(job_result) = stream.next().await {
            if let Err(err) = job_result {
                tracing::error!("Error publishing list: {:?}", err);
                result = Err(err);
            }
        }
        result
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

#[derive(Debug, thiserror::Error)]
enum TokenReadError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
    #[error("no expiry set")]
    NoExpiry,
}

async fn read_token_expiry(path: &Path) -> Result<DateTime<Utc>, TokenReadError> {
    let token = tokio::fs::read_to_string(path).await?.parse::<StatusListToken>()?;
    // Trusting the files this service writes
    let (_, claims) = token.as_ref().dangerous_parse_unverified()?;
    claims.exp.ok_or(TokenReadError::NoExpiry)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;

    use crypto::server_keys::generate::Ca;
    use utils::num::NonZeroU31;
    use utils::num::U31;

    use crate::publish::PublishDir;

    use super::*;

    fn mock_service() -> PostgresStatusListService<SigningKey> {
        PostgresStatusListService {
            connection: DatabaseConnection::default(),
            attestation_type_id: 1,
            config: StatusListConfig {
                list_size: NonZeroU31::MIN,
                create_threshold: U31::ONE,
                expiry: Duration::from_secs(3600),
                refresh_threshold: Duration::from_secs(600),
                ttl: None,
                base_url: "https://example.com/tsl".parse().unwrap(),
                publish_dir: PublishDir::try_new(std::env::temp_dir()).unwrap(),
                key_pair: Ca::generate_issuer_mock_ca()
                    .unwrap()
                    .generate_status_list_mock()
                    .unwrap(),
            }
            .into(),
            revoke_all_flag: Flag::new(DatabaseConnection::default(), FLAG_NAME_REVOKE_ALL.to_string()),
        }
    }

    #[tokio::test]
    async fn test_service_obtain_status_claims_uninitialized_attestation_type() {
        let service = PostgresStatusListServices([("pid".to_string(), mock_service())].into());

        let batch_id = Uuid::new_v4();
        let result = service
            .obtain_status_claims("invalid", batch_id, None, NonZeroUsize::MIN)
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
