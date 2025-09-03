use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Utc;
use futures::try_join;
use itertools::Itertools;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QueryResult;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::StatementBuilder;
use sea_orm::TransactionTrait;
use sea_orm::sea_query::Alias;
use sea_orm::sea_query::BinOper;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::IntoColumnRef;
use sea_orm::sea_query::Query;
use sea_query::OnConflict;
use sea_query::Order;
use sea_query::SimpleExpr;
use tokio::fs;
use tracing::warn;
use uuid::Uuid;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use entity::attestation;
use entity::attestation::TypeMetadataModel;
use entity::attestation_copy;
use entity::attestation_copy::AttestationFormat;
use entity::disclosure_event;
use entity::disclosure_event::EventStatus;
use entity::disclosure_event_attestation;
use entity::issuance_event;
use entity::issuance_event_attestation;
use entity::keyed_data;
use mdoc::utils::serialization::cbor_deserialize;
use mdoc::utils::serialization::cbor_serialize;
use openid4vc::issuance_session::CredentialWithMetadata;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::issuance_session::IssuedCredentialCopies;
use platform_support::hw_keystore::PlatformEncryptionKey;
use sd_jwt::hasher::Sha256Hasher;
use sd_jwt::sd_jwt::VerifiedSdJwt;

use crate::AttestationIdentity;
use crate::AttestationPresentation;
use crate::DisclosureStatus;

use super::AttestationFormatQuery;
use super::Storage;
use super::StorageError;
use super::StorageResult;
use super::StorageState;
use super::StoredAttestation;
use super::attestation_copy::StoredAttestationCopy;
use super::data::KeyedData;
use super::database::Database;
use super::database::SqliteUrl;
use super::event_log::WalletEvent;
use super::key_file;
use super::sql_cipher_key::SqlCipherKey;

const DATABASE_NAME: &str = "wallet";
const KEY_FILE_SUFFIX: &str = "_db";
const DATABASE_FILE_EXT: &str = "db";
const KEY_IDENTIFIER_PREFIX: &str = "keyfile_";

fn key_file_alias_for_name(database_name: &str) -> String {
    // Append suffix to database name to get key file alias
    format!("{database_name}{KEY_FILE_SUFFIX}")
}

fn key_identifier_for_key_file(alias: &str) -> String {
    format!("{KEY_IDENTIFIER_PREFIX}{alias}")
}

/// This is the implementation of [`Storage`] as used by the [`crate::Wallet`]. Its responsibilities are:
///
/// * Managing the lifetime of one or more [`Database`] instances by combining its functionality with encrypted key
///   files. This also includes deleting the database and key file when the [`clear`] method is called.
/// * Communicating the current state of the database through the [`state`] method.
/// * Executing queries on the database by accepting / returning data structures that are used by [`crate::Wallet`].
#[derive(Debug)]
pub struct DatabaseStorage<K> {
    storage_path: PathBuf,
    open_database: Option<OpenDatabaseStorage<K>>,
}

#[derive(Debug)]
struct OpenDatabaseStorage<K> {
    database: Database,
    key_file_key: K,
}

impl AttestationFormatQuery {
    fn attestation_format(self) -> Option<AttestationFormat> {
        match self {
            AttestationFormatQuery::Any => None,
            AttestationFormatQuery::MsoMdoc => Some(AttestationFormat::Mdoc),
            AttestationFormatQuery::SdJwt => Some(AttestationFormat::SdJwt),
        }
    }
}

impl<K> DatabaseStorage<K> {
    pub fn new(storage_path: PathBuf) -> Self {
        DatabaseStorage {
            storage_path,
            open_database: None,
        }
    }

    // Helper method, should be called before accessing database.
    fn database(&self) -> StorageResult<&Database> {
        let database = &self.open_database.as_ref().ok_or(StorageError::NotOpened)?.database;

        Ok(database)
    }

    fn database_path_for_name(&self, name: &str) -> PathBuf {
        // Get path to database as "<storage_path>/<name>.db"
        self.storage_path.join(format!("{name}.{DATABASE_FILE_EXT}"))
    }

    async fn execute_query<S>(&self, query: S) -> StorageResult<Option<QueryResult>>
    where
        S: StatementBuilder,
    {
        let connection = self.database()?.connection();
        let query = connection.get_database_backend().build(&query);
        let query_result = connection.query_one(query).await?;
        Ok(query_result)
    }

    /// Returns a [`SimpleExpr`], comparing whether `timestamp_column` is newer than 31 days.
    fn newer_than_31_days(timestamp_column: impl IntoColumnRef) -> SimpleExpr {
        SimpleExpr::Binary(
            Box::new(SimpleExpr::Column(timestamp_column.into_column_ref())),
            BinOper::GreaterThan,
            Box::new(SimpleExpr::Custom("DATETIME('now', '-31 day')".to_owned())),
        )
    }

    async fn query_unique_attestations(
        &self,
        condition: Option<Condition>,
    ) -> StorageResult<Vec<StoredAttestationCopy>> {
        let database = self.database()?;

        // As this query only contains one `MIN()` aggregate function, the columns that
        // do not appear in the `GROUP BY` clause are taken from whichever `mdoc_copy`
        // row has the lowest disclosure count. This uses the "bare columns in aggregate
        // queries" feature that SQLite provides.
        //
        // See: https://www.sqlite.org/lang_select.html#bare_columns_in_an_aggregate_query
        let select = attestation_copy::Entity::find()
            .select_only()
            .inner_join(attestation::Entity)
            .column(attestation_copy::Column::Id)
            .column(attestation_copy::Column::AttestationId)
            .column(attestation_copy::Column::Attestation)
            .column(attestation_copy::Column::AttestationFormat)
            .column(attestation::Column::TypeMetadata)
            .column_as(attestation_copy::Column::DisclosureCount.min(), "disclosure_count")
            .group_by(attestation_copy::Column::AttestationId)
            .order_by(attestation_copy::Column::Id, Order::Asc);

        let select = match condition {
            Some(condition) => select.filter(condition),
            None => select,
        };

        let copies: Vec<(Uuid, Uuid, Vec<u8>, AttestationFormat, TypeMetadataModel)> =
            select.into_tuple().all(database.connection()).await?;

        let attestations = copies
            .into_iter()
            .map(
                |(attestation_copy_id, attestation_id, attestation_bytes, attestation_format, metadata)| {
                    let attestation = match attestation_format {
                        AttestationFormat::Mdoc => {
                            let mdoc = cbor_deserialize(attestation_bytes.as_slice())?;
                            StoredAttestation::MsoMdoc { mdoc: Box::new(mdoc) }
                        }
                        AttestationFormat::SdJwt => {
                            let sd_jwt = VerifiedSdJwt::dangerous_parse_unverified(
                                // Since we put utf-8 bytes into the database, we are certain we also get them out.
                                String::from_utf8(attestation_bytes).unwrap().as_str(),
                                &Sha256Hasher,
                            )?;
                            StoredAttestation::SdJwt {
                                sd_jwt: Box::new(sd_jwt),
                            }
                        }
                    };

                    let normalized_metadata = metadata.documents.to_normalized()?;

                    let stored_copy = StoredAttestationCopy {
                        attestation_id,
                        attestation_copy_id,
                        attestation,
                        normalized_metadata,
                    };

                    Ok(stored_copy)
                },
            )
            .collect::<Result<_, StorageError>>()?;

        Ok(attestations)
    }

    fn combine_events(
        issuance_events: Vec<(issuance_event::Model, issuance_event_attestation::Model)>,
        disclosure_events: Vec<(disclosure_event::Model, Option<disclosure_event_attestation::Model>)>,
    ) -> StorageResult<Vec<WalletEvent>> {
        // Collect issuance events into a Vec of WalletEvents
        let mut wallet_events = issuance_events
            .into_iter()
            .map(|(event, event_attestation)| {
                let attestation =
                    serde_json::from_value::<AttestationPresentation>(event_attestation.attestation_presentation)?;

                Ok(WalletEvent::Issuance {
                    id: event.id,
                    attestation: Box::new(attestation),
                    timestamp: event.timestamp,
                    renewed: event_attestation.renewed,
                })
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        // Deduplicate disclosure events and add the related attestations
        let disclosure_events = disclosure_events.into_iter().try_fold(
            HashMap::<Uuid, WalletEvent>::new(),
            |mut acc, (event, att_opt)| {
                // Lookup wallet_event or create new one
                let wallet_event = acc.entry(event.id).or_insert_with(|| {
                    // Unwrapping here is safe since the certificate has been parsed before
                    let reader_certificate = BorrowingCertificate::from_der(event.relying_party_certificate).unwrap();
                    let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)
                        .unwrap()
                        .unwrap();

                    WalletEvent::Disclosure {
                        id: event.id,
                        attestations: Default::default(),
                        timestamp: event.timestamp,
                        reader_certificate: Box::new(reader_certificate),
                        reader_registration: Box::new(reader_registration),
                        status: event.status,
                        r#type: event.r#type.into(),
                    }
                });

                // Add related attestation to the wallet_event if it exists
                if let Some(att) = att_opt {
                    let attestation = serde_json::from_value::<AttestationPresentation>(att.attestation_presentation)?;

                    let WalletEvent::Disclosure { attestations, .. } = wallet_event else {
                        panic!("should always be a disclosure event");
                    };
                    attestations.push(attestation);
                }

                StorageResult::Ok(acc)
            },
        )?;

        // Merge issuance and disclosure events
        wallet_events.append(&mut disclosure_events.into_values().collect());

        // Sort by timestamp descending
        wallet_events.sort_by(|a, b| b.timestamp().cmp(a.timestamp()));

        Ok(wallet_events)
    }
}

impl<K> DatabaseStorage<K>
where
    K: PlatformEncryptionKey,
{
    /// This helper method uses [`get_or_create_key_file`] and the utilities in [`platform_support`]
    /// to construct a [`SqliteUrl`] and a [`SqlCipherKey`], which in turn are used to create a [`Database`]
    /// instance.
    async fn open_encrypted_database(&self, name: &str) -> StorageResult<OpenDatabaseStorage<K>> {
        let key_file_alias = key_file_alias_for_name(name);
        let key_file_key_identifier = key_identifier_for_key_file(&key_file_alias);
        let database_path = self.database_path_for_name(name);

        // Get or create the encryption key for the key file contents. The identifier used
        // for this should be globally unique. If this is not the case, the same database is
        // being opened multiple times, which is a programmer error and should result in a panic.
        let key_file_key =
            K::new_unique(&key_file_key_identifier).expect("database key file key identifier is already in use");

        // Get database key of the correct length including a salt, stored in encrypted file.
        let key_bytes = key_file::get_or_create_key_file(
            &self.storage_path,
            &key_file_alias,
            &key_file_key,
            SqlCipherKey::size_with_salt(),
        )
        .await?;
        let key = SqlCipherKey::try_from(key_bytes.as_slice())?;

        // Open database at the path, encrypted using the key
        let database = Database::open(SqliteUrl::File(database_path), key).await?;
        let open_database = OpenDatabaseStorage { database, key_file_key };

        Ok(open_database)
    }
}

impl<K> Storage for DatabaseStorage<K>
where
    K: PlatformEncryptionKey,
{
    /// Indicate whether there is no database on disk, there is one but it is unopened
    /// or the database is currently open.
    async fn state(&self) -> StorageResult<StorageState> {
        if self.open_database.is_some() {
            return Ok(StorageState::Opened);
        }

        let database_path = self.database_path_for_name(DATABASE_NAME);

        if fs::try_exists(database_path).await? {
            return Ok(StorageState::Unopened);
        }

        Ok(StorageState::Uninitialized)
    }

    /// Load a database, creating a new key file and database file if necessary.
    async fn open(&mut self) -> StorageResult<()> {
        if self.open_database.is_some() {
            return Err(StorageError::AlreadyOpened);
        }

        let open_database = self.open_encrypted_database(DATABASE_NAME).await?;
        self.open_database.replace(open_database);

        Ok(())
    }

    /// Clear the contents of the database by closing it and removing both database and key file.
    async fn clear(&mut self) {
        // Take the Database from the Option<> so that close_and_delete() can consume it.
        if let Some(open_database) = self.open_database.take() {
            if let Err(error) = open_database.database.close_and_delete().await {
                warn!("Could not close and delete database: {}", error);
            }

            let key_file_alias = key_file_alias_for_name(DATABASE_NAME);
            if let Err(error) = key_file::delete_key_file(&self.storage_path, &key_file_alias).await {
                warn!("Could not delete database key file: {}", error);
            }

            if let Err(error) = open_database.key_file_key.delete().await {
                warn!("Could not delete database key file key: {}", error);
            }
        }
    }

    /// Get data entry from the key-value table, if present.
    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>> {
        let database = self.database()?;

        let data = keyed_data::Entity::find_by_id(D::KEY)
            .one(database.connection())
            .await?
            .map(|m| serde_json::from_value::<D>(m.data))
            .transpose()?;

        Ok(data)
    }

    /// Insert data entry in the key-value table, which will return an error when one is already present.
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        let database = self.database()?;

        let _ = keyed_data::ActiveModel {
            key: Set(D::KEY.to_string()),
            data: Set(serde_json::to_value(data)?),
        }
        .insert(database.connection())
        .await?;

        Ok(())
    }

    /// Update data entry in the key-value table using the provided key,
    /// inserting the data if it is not already present.
    async fn upsert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        let database = self.database()?;

        let model = keyed_data::ActiveModel {
            key: Set(D::KEY.to_string()),
            data: Set(serde_json::to_value(data)?),
        };
        keyed_data::Entity::insert(model)
            .on_conflict(
                OnConflict::column(keyed_data::Column::Key)
                    .update_column(keyed_data::Column::Data)
                    .to_owned(),
            )
            .exec(database.connection())
            .await?;

        Ok(())
    }

    async fn delete_data<D: KeyedData>(&mut self) -> StorageResult<()> {
        let database = self.database()?;

        keyed_data::Entity::delete_by_id(D::KEY.to_string())
            .exec(database.connection())
            .await?;

        Ok(())
    }

    async fn insert_credentials(
        &mut self,
        timestamp: DateTime<Utc>,
        credentials: Vec<(CredentialWithMetadata, AttestationPresentation)>,
    ) -> StorageResult<()> {
        let issuance_event_id = Uuid::now_v7();

        let issuance_event = issuance_event::ActiveModel {
            id: Set(issuance_event_id),
            timestamp: Set(timestamp),
        };

        // Construct a vec of tuples of 1 `attestation` and 1 or more `attestation_copy` models to be inserted
        // into the database.
        let attestation_models = credentials
            .into_iter()
            .map(
                |(
                    CredentialWithMetadata {
                        copies,
                        attestation_type,
                        metadata_documents,
                    },
                    attestation_presentation,
                )| {
                    let attestation_id = Uuid::now_v7();

                    let attestation_model = attestation::ActiveModel {
                        id: Set(attestation_id),
                        attestation_type: Set(attestation_type),
                        type_metadata: Set(TypeMetadataModel::new(metadata_documents)),
                    };

                    let copy_models = create_attestation_copy_models(attestation_id, copies)?;

                    let issuance_event_attestation = issuance_event_attestation::ActiveModel {
                        id: Set(Uuid::now_v7()),
                        issuance_event_id: Set(issuance_event_id),
                        attestation_id: Set(Some(attestation_id)),
                        attestation_presentation: Set(serde_json::to_value(attestation_presentation)?),
                        renewed: Set(false),
                    };

                    Ok((attestation_model, copy_models, issuance_event_attestation))
                },
            )
            .collect::<Result<Vec<_>, StorageError>>()?;

        // Make two separate vecs out of the vec of tuples.
        let (attestation_models, copy_models, issuance_event_attestations): (Vec<_>, Vec<_>, Vec<_>) =
            itertools::MultiUnzip::multiunzip(attestation_models.into_iter());

        let transaction = self.database()?.connection().begin().await?;

        attestation::Entity::insert_many(attestation_models)
            .exec(&transaction)
            .await?;
        attestation_copy::Entity::insert_many(copy_models.into_iter().flatten())
            .exec(&transaction)
            .await?;
        issuance_event::Entity::insert(issuance_event)
            .exec(&transaction)
            .await?;
        issuance_event_attestation::Entity::insert_many(issuance_event_attestations)
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn update_credentials(
        &mut self,
        timestamp: DateTime<Utc>,
        credentials: Vec<(IssuedCredentialCopies, AttestationPresentation)>,
    ) -> StorageResult<()> {
        let issuance_event_id = Uuid::now_v7();

        let issuance_event = issuance_event::ActiveModel {
            id: Set(issuance_event_id),
            timestamp: Set(timestamp),
        };

        let transaction = self.database()?.connection().begin().await?;

        let mut issuance_event_attestations = Vec::with_capacity(credentials.len());

        for (copies, attestation_presentation) in credentials {
            let AttestationIdentity::Fixed { id: attestation_id } = attestation_presentation.identity else {
                return Err(StorageError::EventEphemeralIdentity);
            };

            let issuance_event_attestation = issuance_event_attestation::ActiveModel {
                id: Set(Uuid::now_v7()),
                issuance_event_id: Set(issuance_event_id),
                attestation_id: Set(Some(attestation_id)),
                attestation_presentation: Set(serde_json::to_value(attestation_presentation)?),
                renewed: Set(true),
            };
            issuance_event_attestations.push(issuance_event_attestation);

            attestation_copy::Entity::delete_many()
                .filter(attestation_copy::Column::AttestationId.eq(attestation_id))
                .exec(&transaction)
                .await?;

            attestation_copy::Entity::insert_many(create_attestation_copy_models(attestation_id, copies)?)
                .exec(&transaction)
                .await?;
        }

        issuance_event::Entity::insert(issuance_event)
            .exec(&transaction)
            .await?;
        issuance_event_attestation::Entity::insert_many(issuance_event_attestations)
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn increment_attestation_copies_usage_count(&mut self, attestation_copy_ids: Vec<Uuid>) -> StorageResult<()> {
        attestation_copy::Entity::update_many()
            .col_expr(
                attestation_copy::Column::DisclosureCount,
                Expr::col(attestation_copy::Column::DisclosureCount).add(1),
            )
            .filter(attestation_copy::Column::Id.is_in(attestation_copy_ids))
            .exec(self.database()?.connection())
            .await?;

        Ok(())
    }

    async fn fetch_unique_attestations(&self) -> StorageResult<Vec<StoredAttestationCopy>> {
        self.query_unique_attestations(None).await
    }

    async fn fetch_unique_attestations_by_type<'a>(
        &'a self,
        attestation_types: &HashSet<&'a str>,
        format_query: AttestationFormatQuery,
    ) -> StorageResult<Vec<StoredAttestationCopy>> {
        let condition = Condition::all();

        let attestation_types_iter = attestation_types.iter().copied();
        let condition = condition.add(attestation::Column::AttestationType.is_in(attestation_types_iter));

        let condition = match format_query.attestation_format() {
            Some(attestation_format) => {
                condition.add(attestation_copy::Column::AttestationFormat.eq(attestation_format))
            }
            None => condition,
        };

        self.query_unique_attestations(Some(condition)).await
    }

    async fn has_any_attestations_with_type(&self, attestation_type: &str) -> StorageResult<bool> {
        let select_statement = Query::select()
            .column((attestation::Entity, attestation::Column::Id))
            .from(attestation::Entity)
            .and_where(Expr::col(attestation::Column::AttestationType).eq(attestation_type))
            .take();

        let exists_query = Query::select()
            .expr_as(Expr::exists(select_statement), Alias::new("attestation_type_exists"))
            .to_owned();

        let exists_result = self.execute_query(exists_query).await?;
        let exists = exists_result
            .map(|result| result.try_get("", "attestation_type_exists"))
            .transpose()?
            .unwrap_or(false);

        Ok(exists)
    }

    async fn log_disclosure_event(
        &mut self,
        timestamp: DateTime<Utc>,
        proposed_attestation_presentations: Vec<AttestationPresentation>,
        reader_certificate: BorrowingCertificate,
        status: DisclosureStatus,
        r#type: DisclosureType,
    ) -> StorageResult<()> {
        let transaction = self.database()?.connection().begin().await?;

        let event_id = Uuid::now_v7();

        let disclosure_event = disclosure_event::ActiveModel {
            id: Set(event_id),
            timestamp: Set(timestamp),
            relying_party_certificate: Set(reader_certificate.to_vec()),
            status: Set(status),
            r#type: Set(r#type.into()),
        };

        let disclosure_event_attestations = proposed_attestation_presentations
            .into_iter()
            .map(|attestation_presentation| {
                let attestation_id = match &attestation_presentation.identity {
                    AttestationIdentity::Ephemeral => None,
                    AttestationIdentity::Fixed { id } => Some(*id),
                }
                .ok_or(StorageError::EventEphemeralIdentity)?;

                Ok(disclosure_event_attestation::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    disclosure_event_id: Set(event_id),
                    attestation_id: Set(Some(attestation_id)),
                    attestation_presentation: Set(serde_json::to_value(attestation_presentation)?),
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;

        disclosure_event::Entity::insert(disclosure_event)
            .exec(&transaction)
            .await?;

        if !disclosure_event_attestations.is_empty() {
            disclosure_event_attestation::Entity::insert_many(disclosure_event_attestations)
                .exec(&transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let fetch_issuance_events = issuance_event::Entity::find()
            .inner_join(issuance_event_attestation::Entity)
            .select_also(issuance_event_attestation::Entity)
            .order_by_desc(issuance_event::Column::Timestamp)
            .all(connection);

        let fetch_disclosure_events = disclosure_event::Entity::find()
            .left_join(disclosure_event_attestation::Entity)
            .select_also(disclosure_event_attestation::Entity)
            .order_by_desc(disclosure_event::Column::Timestamp)
            .all(connection);

        let (issuance_events, disclosure_events) = try_join!(fetch_issuance_events, fetch_disclosure_events)?;

        let issuance_events = issuance_events
            .into_iter()
            .map(|(event, att)|
                // Unwrap is safe here because of the inner join above
                (event, att.unwrap()))
            .collect_vec();

        Self::combine_events(issuance_events, disclosure_events)
    }

    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let fetch_issuance_events = issuance_event::Entity::find()
            .inner_join(issuance_event_attestation::Entity)
            .select_also(issuance_event_attestation::Entity)
            .filter(Self::newer_than_31_days(issuance_event::Column::Timestamp))
            .order_by_desc(issuance_event::Column::Timestamp)
            .all(connection);

        let fetch_disclosure_events = disclosure_event::Entity::find()
            .left_join(disclosure_event_attestation::Entity)
            .select_also(disclosure_event_attestation::Entity)
            .filter(Self::newer_than_31_days(disclosure_event::Column::Timestamp))
            .order_by_desc(disclosure_event::Column::Timestamp)
            .all(connection);

        let (issuance_events, disclosure_events) = try_join!(fetch_issuance_events, fetch_disclosure_events)?;

        let issuance_events = issuance_events
            .into_iter()
            .map(|(event, att)|
                // Unwrap is safe here because of the inner join above
                (event, att.unwrap()))
            .collect_vec();

        Self::combine_events(issuance_events, disclosure_events)
    }

    async fn fetch_wallet_events_by_attestation_id(&self, attestation_id: Uuid) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let fetch_issuance_events = issuance_event::Entity::find()
            .inner_join(issuance_event_attestation::Entity)
            .select_also(issuance_event_attestation::Entity)
            .filter(issuance_event_attestation::Column::AttestationId.eq(attestation_id))
            .order_by_desc(issuance_event::Column::Timestamp)
            .all(connection);

        let fetch_disclosure_events = disclosure_event::Entity::find()
            .left_join(disclosure_event_attestation::Entity)
            .select_also(disclosure_event_attestation::Entity)
            .filter(disclosure_event_attestation::Column::AttestationId.eq(attestation_id))
            .order_by_desc(disclosure_event::Column::Timestamp)
            .all(connection);

        let (issuance_events, disclosure_events) = try_join!(fetch_issuance_events, fetch_disclosure_events)?;

        let issuance_events = issuance_events
            .into_iter()
            .map(|(event, att)|
                // Unwrap is safe here because of the inner join above
                (event, att.unwrap()))
            .collect_vec();

        Self::combine_events(issuance_events, disclosure_events)
    }

    // TODO (PVW-4135): Fix logic to uniquely identify an RP, since its certificate may change.
    async fn did_share_data_with_relying_party(&self, certificate: &BorrowingCertificate) -> StorageResult<bool> {
        let select_statement = Query::select()
            .column((disclosure_event::Entity, disclosure_event::Column::Id))
            .from(disclosure_event_attestation::Entity)
            .inner_join(
                disclosure_event::Entity,
                Expr::col((disclosure_event::Entity, disclosure_event::Column::Id)).eq(Expr::col((
                    disclosure_event_attestation::Entity,
                    disclosure_event_attestation::Column::DisclosureEventId,
                ))),
            )
            .and_where(Expr::col(disclosure_event::Column::RelyingPartyCertificate).eq(certificate.as_ref()))
            .and_where(Expr::col(disclosure_event::Column::Status).eq(EventStatus::Success))
            .limit(1)
            .take();

        let exists_query = Query::select()
            .expr_as(Expr::exists(select_statement), Alias::new("certificate_exists"))
            .to_owned();

        let exists_result = self.execute_query(exists_query).await?;
        let exists = exists_result
            .map(|result| result.try_get("", "certificate_exists"))
            .transpose()?
            .unwrap_or(false);

        Ok(exists)
    }
}

fn create_attestation_copy_models(
    attestation_id: Uuid,
    copies: IssuedCredentialCopies,
) -> StorageResult<Vec<attestation_copy::ActiveModel>> {
    copies
        .into_inner()
        .into_iter()
        .map(|credential| match credential {
            IssuedCredential::MsoMdoc(mdoc) => {
                let model = attestation_copy::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    attestation_id: Set(attestation_id),
                    attestation: Set(cbor_serialize(&mdoc)?),
                    attestation_format: Set(AttestationFormat::Mdoc),
                    ..Default::default()
                };

                Ok::<_, StorageError>(model)
            }
            IssuedCredential::SdJwt(sd_jwt) => {
                let model = attestation_copy::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    attestation_id: Set(attestation_id),
                    attestation: Set(sd_jwt.to_string().into_bytes()),
                    attestation_format: Set(AttestationFormat::SdJwt),
                    ..Default::default()
                };

                Ok(model)
            }
        })
        .try_collect()
}

#[cfg(any(test, feature = "wallet_deps"))]
pub mod in_memory_storage {
    use crypto::utils::random_bytes;

    use platform_support::hw_keystore::mock::MockHardwareEncryptionKey;

    use crate::storage::DatabaseStorage;
    use crate::storage::database::Database;
    use crate::storage::database::SqliteUrl;
    use crate::storage::database_storage::OpenDatabaseStorage;
    use crate::storage::sql_cipher_key::SqlCipherKey;

    pub async fn open_in_memory_database_storage() -> DatabaseStorage<MockHardwareEncryptionKey> {
        let mut storage = DatabaseStorage::<MockHardwareEncryptionKey>::new("storage_path".into());

        // Create a test database, override the database field on Storage.
        let key_bytes = random_bytes(SqlCipherKey::size_with_salt());
        let database = Database::open(SqliteUrl::InMemory, key_bytes.as_slice().try_into().unwrap())
            .await
            .expect("Could not open in-memory database");

        // Create an encryption key for the key file, which is not actually used,
        // but still needs to be present.
        let key_file_key = MockHardwareEncryptionKey::new_random("open_test_database_storage".to_string());

        storage.open_database = OpenDatabaseStorage { database, key_file_key }.into();

        storage
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::ops::Add;
    use std::ops::Sub;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use chrono::Duration;
    use chrono::TimeZone;
    use chrono::Utc;
    use itertools::Itertools;
    use tokio::fs;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::credential_payload::IntoCredentialPayload;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use attestation_data::x509::generate::mock::generate_reader_mock;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::utils::random_bytes;
    use mdoc::holder::Mdoc;
    use openid4vc::issuance_session::IssuedCredentialCopies;
    use platform_support::hw_keystore::mock::MockHardwareEncryptionKey;
    use platform_support::utils::PlatformUtilities;
    use platform_support::utils::mock::MockHardwareUtilities;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
    use wallet_account::messages::registration::WalletCertificate;

    use in_memory_storage::open_in_memory_database_storage;

    use crate::storage::data::RegistrationData;

    use super::*;

    static ISSUER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();

        generate_issuer_mock(&issuer_ca, IssuerRegistration::new_mock().into()).unwrap()
    });

    static READER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let reader_ca = Ca::generate_reader_mock_ca().unwrap();

        generate_reader_mock(&reader_ca, ReaderRegistration::new_mock().into()).unwrap()
    });

    #[test]
    fn test_key_file_alias_for_name() {
        assert_eq!(key_file_alias_for_name("test_database"), "test_database_db");
    }

    #[tokio::test]
    async fn test_database_open_encrypted_database_and_clear() {
        let mut storage =
            DatabaseStorage::<MockHardwareEncryptionKey>::new(MockHardwareUtilities::storage_path().await.unwrap());

        let name = "test_open_encrypted_database";
        let key_file_alias = key_file_alias_for_name(name);
        let key_file_identifier = key_identifier_for_key_file(&key_file_alias);
        let database_path = storage.database_path_for_name(name);

        // Make sure we start with a clean slate.
        _ = key_file::delete_key_file(&storage.storage_path, &key_file_alias).await;
        _ = fs::remove_file(database_path).await;

        // The key file encryption key should be absent.
        assert!(!MockHardwareEncryptionKey::identifier_exists(&key_file_identifier));

        // Open the encrypted database.
        let open_database = storage
            .open_encrypted_database(name)
            .await
            .expect("Could not open encrypted database");

        // The database should have opened a file at the expected path.
        let database_path = match open_database.database.url {
            SqliteUrl::File(ref path) => path.clone(),
            _ => panic!("Unexpected database URL"),
        };
        assert!(
            database_path
                .to_str()
                .unwrap()
                .contains("test_open_encrypted_database.db")
        );
        assert!(fs::try_exists(&database_path).await.unwrap());

        // The key file encryption key should be present.
        assert!(MockHardwareEncryptionKey::identifier_exists(&key_file_identifier));

        // Set the open database on the `DatabaseStorage` instance, then drop the storage.
        // Both the database file and the encryption key should still exist.
        storage.open_database = open_database.into();
        drop(storage);
        assert!(fs::try_exists(&database_path).await.unwrap());
        assert!(MockHardwareEncryptionKey::identifier_exists(&key_file_identifier));

        // Re-open the encrypted database, set it on the `DatabaseStorage`
        // instance and then call clear on it in order to delete the database.
        let mut storage =
            DatabaseStorage::<MockHardwareEncryptionKey>::new(MockHardwareUtilities::storage_path().await.unwrap());
        storage.open_database = storage
            .open_encrypted_database(name)
            .await
            .expect("Could not open encrypted database")
            .into();
        storage.clear().await;

        // The database file should be gone and the key file encryption key should be absent.
        assert!(!fs::try_exists(&database_path).await.unwrap());
        assert!(!MockHardwareEncryptionKey::identifier_exists(&key_file_identifier));
    }

    #[tokio::test]
    async fn test_database_keyed_storage() {
        let registration = RegistrationData {
            attested_key_identifier: "key_id".to_string(),
            pin_salt: vec![1, 2, 3, 4],
            wallet_id: "wallet_123".to_string(),
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid"),
        };

        let mut storage = open_in_memory_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Try to fetch the registration, none should be there.
        let no_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(no_registration.is_none());

        // Save the registration and fetch it again.
        storage
            .insert_data(&registration)
            .await
            .expect("Could not save registration");

        let fetched_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(fetched_registration.is_some());
        let fetched_registration = fetched_registration.unwrap();
        assert_eq!(fetched_registration.pin_salt, registration.pin_salt);
        assert_eq!(
            fetched_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Save the registration again, should result in an error.
        let save_result = storage.insert_data(&registration).await;
        assert!(save_result.is_err());

        // Upsert registration
        let new_salt = random_bytes(64);
        let updated_registration = RegistrationData {
            attested_key_identifier: "key_id".to_string(),
            pin_salt: new_salt,
            wallet_id: registration.wallet_id.clone(),
            wallet_certificate: registration.wallet_certificate.clone(),
        };
        storage
            .upsert_data(&updated_registration)
            .await
            .expect("Could not update registration");

        let fetched_after_update_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");
        assert!(fetched_after_update_registration.is_some());
        let fetched_after_update_registration = fetched_after_update_registration.unwrap();
        assert_eq!(
            fetched_after_update_registration.pin_salt,
            updated_registration.pin_salt
        );
        assert_eq!(
            fetched_after_update_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Delete registration
        storage
            .delete_data::<RegistrationData>()
            .await
            .expect("Could not delete registration");
        let no_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");
        assert!(no_registration.is_none());

        // Clear database, state should be uninitialized.
        storage.clear().await;

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Uninitialized));

        // Open the database again and test if upsert stores new data.
        let mut storage = open_in_memory_database_storage().await;
        storage
            .upsert_data(&registration)
            .await
            .expect("Could not upsert registration");

        let fetched_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(fetched_registration.is_some());
        let fetched_registration = fetched_registration.unwrap();
        assert_eq!(fetched_registration.pin_salt, registration.pin_salt);
    }

    #[tokio::test]
    async fn test_mdoc_storage() {
        let mut storage = open_in_memory_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let mdoc = Mdoc::new_mock().await;

        // The mock mdoc is never deserialized, so it contains `ProtectedHeader { original_data: None, .. }`.
        // When this mdoc is serialized, stored, fetched, and then deserialized again, it will contain
        // `ProtectedHeader { original_data: Some(..), .. }` so the equality check below will fail.
        // This line fixes that.
        let mdoc: Mdoc = cbor_deserialize(cbor_serialize(&mdoc).unwrap().as_slice()).unwrap();
        let credential = IssuedCredential::MsoMdoc(Box::new(mdoc.clone()));

        let issued_mdoc_copies = IssuedCredentialCopies::new_or_panic(
            vec![credential.clone(), credential.clone(), credential.clone()]
                .try_into()
                .unwrap(),
        );

        // Insert mdocs
        storage
            .insert_credentials(
                Utc::now(),
                vec![(
                    CredentialWithMetadata::new(
                        issued_mdoc_copies,
                        mdoc.mso.doc_type.clone(),
                        VerifiedTypeMetadataDocuments::nl_pid_example(),
                    ),
                    AttestationPresentation::new_mock(),
                )],
            )
            .await
            .expect("Could not insert attestations");

        let fetched_unique = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        // Only one unique `AttestationCopy` should be returned and it should match all copies.
        assert_eq!(fetched_unique.len(), 1);
        let attestation_copy1 = fetched_unique.first().unwrap();

        assert_matches!(
            &attestation_copy1.attestation,
            StoredAttestation::MsoMdoc { mdoc: stored } if **stored == mdoc
        );
        assert_eq!(
            attestation_copy1.normalized_metadata,
            NormalizedTypeMetadata::nl_pid_example()
        );

        // Only one unique `AttestationCopy` should be returned when querying
        // the attestation type, but not when the queried format is SD-JWT.
        let attestation_types = HashSet::from([mdoc.mso.doc_type.as_str()]);
        let fetched_unique_any = storage
            .fetch_unique_attestations_by_type(&attestation_types, AttestationFormatQuery::Any)
            .await
            .expect("Could not fetch unique attestations by type");

        let fetched_unique_mdoc = storage
            .fetch_unique_attestations_by_type(&attestation_types, AttestationFormatQuery::MsoMdoc)
            .await
            .expect("Could not fetch unique attestations by type");

        let fetched_unique_sd_jwt = storage
            .fetch_unique_attestations_by_type(&attestation_types, AttestationFormatQuery::SdJwt)
            .await
            .expect("Could not fetch unique attestations by type");

        let fetched_unique_other = storage
            .fetch_unique_attestations_by_type(&HashSet::from(["other"]), AttestationFormatQuery::Any)
            .await
            .expect("Could not fetch unique attestations by type");

        assert_eq!(fetched_unique_any.len(), 1);
        assert_matches!(
            &fetched_unique_any.first().unwrap().attestation,
            StoredAttestation::MsoMdoc { mdoc: stored } if **stored == mdoc
        );
        assert_eq!(fetched_unique_mdoc.len(), 1);
        assert_matches!(
            &fetched_unique_mdoc.first().unwrap().attestation,
            StoredAttestation::MsoMdoc { mdoc: stored } if **stored == mdoc
        );
        assert!(fetched_unique_sd_jwt.is_empty());
        assert!(fetched_unique_other.is_empty());

        // Increment the usage count for this attestation copy.
        storage
            .increment_attestation_copies_usage_count(vec![attestation_copy1.attestation_copy_id])
            .await
            .expect("Could not increment usage count for attestation copy");

        // Fetch unique attestations
        let fetched_unique_attestation_type = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        // One matching `AttestationCopy` should be returned and it should be a different copy than the fist one.
        assert_eq!(fetched_unique_attestation_type.len(), 1);
        let attestation_copy2 = fetched_unique_attestation_type.first().unwrap();
        assert_matches!(
            &attestation_copy2.attestation,
            StoredAttestation::MsoMdoc { mdoc: stored } if **stored == mdoc
        );
        assert_eq!(
            attestation_copy2.normalized_metadata,
            NormalizedTypeMetadata::nl_pid_example()
        );
        assert_ne!(
            attestation_copy1.attestation_copy_id,
            attestation_copy2.attestation_copy_id
        );

        // Increment the usage count for this mdoc.
        storage
            .increment_attestation_copies_usage_count(vec![attestation_copy2.attestation_copy_id])
            .await
            .expect("Could not increment usage count for attestation copy");

        // Fetch unique attestations twice, which should result in exactly the same
        // copy, since it is the last one that has a `usage_count` of 0.
        let fetched_unique_remaining1 = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");
        let fetched_unique_remaining2 = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        // Test that the copy identifiers are the same and that they
        // are different from the other two attestation copy identifiers.
        assert_eq!(fetched_unique_remaining1.len(), 1);
        let remaning_attestation_copy_id1 = fetched_unique_remaining1.first().unwrap().attestation_copy_id;
        assert_eq!(fetched_unique_remaining2.len(), 1);
        let remaning_attestation_copy_id2 = fetched_unique_remaining2.first().unwrap().attestation_copy_id;

        assert_eq!(remaning_attestation_copy_id1, remaning_attestation_copy_id2);
        assert_ne!(attestation_copy1.attestation_copy_id, remaning_attestation_copy_id1);
        assert_ne!(attestation_copy2.attestation_copy_id, remaning_attestation_copy_id1);
    }

    #[tokio::test]
    async fn test_sd_jwt_storage() {
        let mut storage = open_in_memory_database_storage().await;

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let sd_jwt = VerifiedSdJwt::pid_example(&ISSUER_KEY);
        let credential = IssuedCredential::SdJwt(Box::new(sd_jwt.clone()));

        let issued_copies = IssuedCredentialCopies::new_or_panic(
            vec![credential.clone(), credential.clone(), credential.clone()]
                .try_into()
                .unwrap(),
        );

        let attestation_type = sd_jwt.as_ref().claims().vct.as_ref().unwrap().to_owned();

        let attestations = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        assert!(attestations.is_empty());

        // Insert sd_jwts
        storage
            .insert_credentials(
                Utc::now(),
                vec![(
                    CredentialWithMetadata::new(
                        issued_copies,
                        attestation_type.clone(),
                        VerifiedTypeMetadataDocuments::nl_pid_example(),
                    ),
                    AttestationPresentation::new_mock(),
                )],
            )
            .await
            .expect("Could not insert mdocs");

        let fetched_unique = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        // Only one unique `AttestationCopy` should be returned and it should match all copies.
        assert_eq!(fetched_unique.len(), 1);
        let attestation_copy1 = fetched_unique.first().unwrap();

        assert_matches!(
            &attestation_copy1.attestation,
            StoredAttestation::SdJwt { sd_jwt: stored } if stored.as_ref() == &sd_jwt
        );
        assert_eq!(
            attestation_copy1.normalized_metadata,
            NormalizedTypeMetadata::nl_pid_example()
        );

        // Only one unique `AttestationCopy` should be returned when querying
        // the attestation type, but not when the queried format is mdoc.
        let attestation_types = HashSet::from([attestation_type.as_str()]);
        let fetched_unique_any = storage
            .fetch_unique_attestations_by_type(&attestation_types, AttestationFormatQuery::Any)
            .await
            .expect("Could not fetch unique attestations by type");

        let fetched_unique_mdoc = storage
            .fetch_unique_attestations_by_type(&attestation_types, AttestationFormatQuery::MsoMdoc)
            .await
            .expect("Could not fetch unique attestations by type");

        let fetched_unique_sd_jwt = storage
            .fetch_unique_attestations_by_type(&attestation_types, AttestationFormatQuery::SdJwt)
            .await
            .expect("Could not fetch unique attestations by type");

        let fetched_unique_other = storage
            .fetch_unique_attestations_by_type(&HashSet::from(["other"]), AttestationFormatQuery::Any)
            .await
            .expect("Could not fetch unique attestations by type");

        assert_eq!(fetched_unique_any.len(), 1);
        assert_matches!(
            &fetched_unique_any.first().unwrap().attestation,
            StoredAttestation::SdJwt { sd_jwt: stored } if stored.as_ref() == &sd_jwt
        );
        assert!(fetched_unique_mdoc.is_empty());
        assert_eq!(fetched_unique_sd_jwt.len(), 1);
        assert_matches!(
            &fetched_unique_sd_jwt.first().unwrap().attestation,
            StoredAttestation::SdJwt { sd_jwt: stored } if stored.as_ref() == &sd_jwt
        );
        assert!(fetched_unique_other.is_empty());
    }

    #[tokio::test]
    async fn test_insert_and_update_attestations() {
        let mut storage = open_in_memory_database_storage().await;

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Create issued_copies that will be inserted into the database
        let sd_jwt = VerifiedSdJwt::pid_example(&ISSUER_KEY);
        let credential = IssuedCredential::SdJwt(Box::new(sd_jwt.clone()));
        let issued_copies = IssuedCredentialCopies::new_or_panic(
            vec![credential.clone(), credential.clone(), credential.clone()]
                .try_into()
                .unwrap(),
        );

        let attestation_type = sd_jwt.as_ref().claims().vct.as_ref().unwrap().to_owned();

        let attestations = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        assert!(attestations.is_empty());

        let mut attestation_presentation = AttestationPresentation::new_mock();

        // Insert sd_jwt
        storage
            .insert_credentials(
                Utc::now(),
                vec![(
                    CredentialWithMetadata::new(
                        issued_copies.clone(),
                        attestation_type,
                        VerifiedTypeMetadataDocuments::nl_pid_example(),
                    ),
                    attestation_presentation.clone(),
                )],
            )
            .await
            .expect("Could not insert mdocs");

        let attestations = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        // One matching attestation should be returned
        assert_eq!(attestations.len(), 1);

        let inserted_attestation_copy = attestations.first().unwrap();
        let StoredAttestation::SdJwt {
            sd_jwt: inserted_attestation,
        } = &inserted_attestation_copy.attestation
        else {
            panic!("Attestation is not an SD-JWT")
        };

        // After insertion, there should be one issuance event
        let fetched_events = storage.fetch_wallet_events().await.unwrap();
        assert_eq!(fetched_events.len(), 1);

        // Create new issued_copies that will be updated
        let sd_jwt = VerifiedSdJwt::pid_example(&ISSUER_KEY);
        let credential = IssuedCredential::SdJwt(Box::new(sd_jwt.clone()));
        let issued_copies = IssuedCredentialCopies::new_or_panic(
            vec![credential.clone(), credential.clone(), credential.clone()]
                .try_into()
                .unwrap(),
        );

        attestation_presentation.identity = AttestationIdentity::Fixed {
            id: attestations[0].attestation_id,
        };

        // Update sd_jwt
        storage
            .update_credentials(Utc::now(), vec![(issued_copies, attestation_presentation)])
            .await
            .expect("Could not update sd-jwts");

        let attestations = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        // After updating, there should still be one attestation
        assert_eq!(attestations.len(), 1);

        let updated_attestation_copy = attestations.first().unwrap();
        let StoredAttestation::SdJwt {
            sd_jwt: updated_attestation,
        } = &updated_attestation_copy.attestation
        else {
            panic!("Attestation is not an SD-JWT")
        };

        assert_eq!(
            inserted_attestation_copy.attestation_id,
            updated_attestation_copy.attestation_id
        );
        assert_ne!(
            inserted_attestation_copy.attestation_copy_id,
            updated_attestation_copy.attestation_copy_id
        );
        assert_ne!(inserted_attestation, updated_attestation);

        // Incrementing the usage_count of the attestation ensures a new copy is returned next time unique attestations
        // are queried
        storage
            .increment_attestation_copies_usage_count(vec![updated_attestation_copy.attestation_copy_id])
            .await
            .expect("Could not increment usage count for attestation copy");

        let attestations = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations");

        let next_updated_attestation_copy = attestations.first().unwrap();
        let StoredAttestation::SdJwt {
            sd_jwt: next_updated_attestation,
        } = &next_updated_attestation_copy.attestation
        else {
            panic!("Attestation is not an SD-JWT")
        };

        assert_eq!(
            updated_attestation_copy.attestation_id,
            next_updated_attestation_copy.attestation_id,
        );
        assert_ne!(
            updated_attestation_copy.attestation_copy_id,
            next_updated_attestation_copy.attestation_copy_id,
        );
        assert_eq!(updated_attestation, next_updated_attestation);

        // And there should be two events, of which the most recent one should be a renew event
        let fetched_events = storage.fetch_wallet_events().await.unwrap();
        assert_eq!(fetched_events.len(), 2);
        assert_matches!(fetched_events[0], WalletEvent::Issuance { renewed, .. } if renewed);
    }

    #[tokio::test]
    async fn test_storing_disclosure_cancel_event() {
        let mut storage = open_in_memory_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();

        // No data shared with RP
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );

        // Log cancel event
        storage
            .log_disclosure_event(
                timestamp,
                vec![],
                READER_KEY.certificate().clone(),
                DisclosureStatus::Cancelled,
                DisclosureType::Regular,
            )
            .await
            .unwrap();

        let fetched_events = storage.fetch_wallet_events().await.unwrap();

        // Cancel event should exist
        assert_eq!(fetched_events.len(), 1);
        assert_eq!(fetched_events.first().unwrap().timestamp(), &timestamp);

        // Still no data shared with RP
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_storing_disclosure_error_event_without_data() {
        let mut storage = open_in_memory_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();

        // No data shared with RP
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );

        // Log error event
        storage
            .log_disclosure_event(
                timestamp,
                vec![],
                READER_KEY.certificate().clone(),
                DisclosureStatus::Error,
                DisclosureType::Regular,
            )
            .await
            .unwrap();

        let fetched_events = storage.fetch_wallet_events().await.unwrap();

        // Error event should exist
        assert_eq!(fetched_events.len(), 1);
        assert_eq!(fetched_events.first().unwrap().timestamp(), &timestamp);

        // Still no data shared with RP
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_storing_disclosure_error_event_with_data() {
        let mut storage = open_in_memory_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let issuance_timestamp = Utc::now();
        let disclosure_timestamp = Utc::now().add(Duration::days(1));
        let earlier_disclosure_timestamp = Utc::now().sub(Duration::days(32));

        // No data shared with RP
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );

        let sd_jwt = VerifiedSdJwt::pid_example(&ISSUER_KEY);
        let credential = IssuedCredential::SdJwt(Box::new(sd_jwt.clone()));

        let issued_copies = IssuedCredentialCopies::new_or_panic(
            vec![credential.clone(), credential.clone(), credential.clone()]
                .try_into()
                .unwrap(),
        );

        let attestation_type = sd_jwt.as_ref().claims().vct.as_ref().unwrap().to_owned();

        // Insert sd_jwt
        storage
            .insert_credentials(
                issuance_timestamp,
                vec![(
                    CredentialWithMetadata::new(
                        issued_copies,
                        attestation_type,
                        VerifiedTypeMetadataDocuments::nl_pid_example(),
                    ),
                    AttestationPresentation::new_mock(),
                )],
            )
            .await
            .expect("Could not insert mdocs");

        let StoredAttestationCopy {
            attestation: StoredAttestation::SdJwt { sd_jwt },
            attestation_id,
            ..
        } = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations")
            .first()
            .cloned()
            .unwrap()
        else {
            panic!("should fetch SD-JWT");
        };

        let normalized_metadata = VerifiedTypeMetadataDocuments::nl_pid_example().to_normalized().unwrap();

        let issuer_certificate = sd_jwt.as_ref().issuer_certificate();
        let issuer_registration = IssuerRegistration::from_certificate(issuer_certificate)
            .unwrap()
            .unwrap();

        let payload = sd_jwt
            .as_ref()
            .as_ref()
            .into_credential_payload(&normalized_metadata)
            .unwrap();
        let attestation = AttestationPresentation::create_from_attributes(
            AttestationIdentity::Fixed { id: attestation_id },
            normalized_metadata,
            issuer_registration.organization,
            &payload.previewable_payload.attributes,
        )
        .unwrap();

        storage
            .log_disclosure_event(
                disclosure_timestamp,
                vec![attestation.clone(), attestation.clone()],
                READER_KEY.certificate().clone(),
                DisclosureStatus::Error,
                DisclosureType::Regular,
            )
            .await
            .unwrap();
        storage
            .log_disclosure_event(
                earlier_disclosure_timestamp,
                vec![attestation.clone(), attestation],
                READER_KEY.certificate().clone(),
                DisclosureStatus::Error,
                DisclosureType::Regular,
            )
            .await
            .unwrap();

        let fetched_events = storage.fetch_wallet_events().await.unwrap();

        // Error event should exist
        assert_eq!(fetched_events.len(), 3);
        assert_eq!(fetched_events[0].timestamp(), &disclosure_timestamp);
        assert_matches!(&fetched_events[0], WalletEvent::Disclosure { attestations, .. } if attestations.len() == 2);
        assert_eq!(fetched_events[1].timestamp(), &issuance_timestamp);
        assert_eq!(fetched_events[2].timestamp(), &earlier_disclosure_timestamp);
        assert_matches!(&fetched_events[2], WalletEvent::Disclosure { attestations, .. } if attestations.len() == 2);

        // Still no data has been shared with RP, because we only consider Successful events
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );

        let events_by_attestation_id = storage
            .fetch_wallet_events_by_attestation_id(attestation_id)
            .await
            .unwrap();
        assert_eq!(events_by_attestation_id.len(), 3);

        // Recent events should not include the disclosure event with the 'earlier_disclosure_timestamp'
        let events = storage.fetch_recent_wallet_events().await.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(fetched_events[0].timestamp(), &disclosure_timestamp);
        assert_eq!(fetched_events[1].timestamp(), &issuance_timestamp);
    }

    #[tokio::test]
    async fn test_event_log_storage_ordering() {
        let mut storage = open_in_memory_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));
        test_history_ordering(&mut storage).await;
    }

    pub(crate) async fn test_history_ordering(storage: &mut impl Storage) {
        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let timestamp_older = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();
        let timestamp_even_older = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();

        let sd_jwt = VerifiedSdJwt::pid_example(&ISSUER_KEY);
        let credential = IssuedCredential::SdJwt(Box::new(sd_jwt.clone()));

        let issued_copies = IssuedCredentialCopies::new_or_panic(vec![credential.clone()].try_into().unwrap());
        let attestation_type = sd_jwt.as_ref().claims().vct.as_ref().unwrap().to_owned();

        // Insert sd_jwts
        storage
            .insert_credentials(
                timestamp,
                vec![
                    (
                        CredentialWithMetadata::new(
                            issued_copies.clone(),
                            attestation_type.clone(),
                            VerifiedTypeMetadataDocuments::nl_pid_example(),
                        ),
                        AttestationPresentation::new_mock(),
                    ),
                    (
                        CredentialWithMetadata::new(
                            issued_copies,
                            attestation_type,
                            VerifiedTypeMetadataDocuments::nl_pid_example(),
                        ),
                        AttestationPresentation::new_mock(),
                    ),
                ],
            )
            .await
            .expect("Could not insert mdocs");

        let attestations = storage
            .fetch_unique_attestations()
            .await
            .expect("Could not fetch unique attestations")
            .into_iter()
            .map(|attestation| {
                let StoredAttestationCopy {
                    attestation: StoredAttestation::SdJwt { sd_jwt },
                    attestation_id,
                    ..
                } = attestation
                else {
                    panic!("should fetch SD-JWT");
                };

                let normalized_metadata = VerifiedTypeMetadataDocuments::nl_pid_example().to_normalized().unwrap();

                let issuer_certificate = sd_jwt.as_ref().issuer_certificate();
                let issuer_registration = IssuerRegistration::from_certificate(issuer_certificate)
                    .unwrap()
                    .unwrap();

                let payload = sd_jwt
                    .as_ref()
                    .as_ref()
                    .into_credential_payload(&normalized_metadata)
                    .unwrap();
                AttestationPresentation::create_from_attributes(
                    AttestationIdentity::Fixed { id: attestation_id },
                    normalized_metadata,
                    issuer_registration.organization,
                    &payload.previewable_payload.attributes,
                )
                .unwrap()
            })
            .collect_vec();

        // No data shared with RP
        assert!(
            !storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );

        let attestation1 = attestations[0].clone();
        let attestation2 = attestations[1].clone();

        storage
            .log_disclosure_event(
                timestamp_even_older,
                vec![attestation1],
                READER_KEY.certificate().clone(),
                DisclosureStatus::Success,
                DisclosureType::Regular,
            )
            .await
            .unwrap();

        storage
            .log_disclosure_event(
                timestamp_older,
                vec![attestation2],
                READER_KEY.certificate().clone(),
                DisclosureStatus::Success,
                DisclosureType::Regular,
            )
            .await
            .unwrap();

        // Data has been shared with RP
        assert!(
            storage
                .did_share_data_with_relying_party(READER_KEY.certificate())
                .await
                .unwrap()
        );

        // Fetch and verify events are sorted descending by timestamp
        assert_eq!(
            storage
                .fetch_wallet_events()
                .await
                .unwrap()
                .iter()
                .map(|event| event.timestamp())
                .collect_vec(),
            vec![&timestamp, &timestamp, &timestamp_older, &timestamp_even_older]
        );
    }
}
