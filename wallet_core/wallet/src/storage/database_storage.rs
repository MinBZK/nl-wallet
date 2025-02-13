use std::collections::HashSet;
use std::path::PathBuf;

use futures::try_join;
use sea_orm::sea_query::Alias;
use sea_orm::sea_query::BinOper;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::IntoColumnRef;
use sea_orm::sea_query::Query;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::IntoSimpleExpr;
use sea_orm::JoinType;
use sea_orm::ModelTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QueryResult;
use sea_orm::QuerySelect;
use sea_orm::RelationDef;
use sea_orm::RelationTrait;
use sea_orm::Select;
use sea_orm::Set;
use sea_orm::StatementBuilder;
use sea_orm::TransactionTrait;
use sea_query::OnConflict;
use sea_query::SimpleExpr;
use tokio::fs;
use tracing::warn;
use uuid::Uuid;

use entity::disclosure_history_event;
use entity::disclosure_history_event::EventStatus;
use entity::disclosure_history_event_doc_type;
use entity::history_doc_type;
use entity::issuance_history_event;
use entity::issuance_history_event_doc_type;
use entity::keyed_data;
use entity::mdoc;
use entity::mdoc_copy;
use nl_wallet_mdoc::utils::serialization::cbor_deserialize;
use nl_wallet_mdoc::utils::serialization::cbor_serialize;
use nl_wallet_mdoc::utils::serialization::CborError;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use openid4vc::credential::MdocCopies;
use platform_support::hw_keystore::PlatformEncryptionKey;

use super::data::KeyedData;
use super::database::Database;
use super::database::SqliteUrl;
use super::event_log::WalletEvent;
use super::event_log::WalletEventModel;
use super::key_file;
use super::sql_cipher_key::SqlCipherKey;
use super::Storage;
use super::StorageError;
use super::StorageResult;
use super::StorageState;
use super::StoredMdocCopy;

const DATABASE_NAME: &str = "wallet";
const KEY_FILE_SUFFIX: &str = "_db";
const DATABASE_FILE_EXT: &str = "db";
const KEY_IDENTIFIER_PREFIX: &str = "keyfile_";

fn key_file_alias_for_name(database_name: &str) -> String {
    // Append suffix to database name to get key file alias
    format!("{}{}", database_name, KEY_FILE_SUFFIX)
}

fn key_identifier_for_key_file(alias: &str) -> String {
    format!("{}{}", KEY_IDENTIFIER_PREFIX, alias)
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
        self.storage_path.join(format!("{}.{}", name, DATABASE_FILE_EXT))
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

    async fn query_unique_mdocs<F>(&self, transform_select: F) -> StorageResult<Vec<StoredMdocCopy>>
    where
        F: FnOnce(Select<mdoc_copy::Entity>) -> Select<mdoc_copy::Entity>,
    {
        let database = self.database()?;

        // As this query only contains one `MIN()` aggregate function, the columns that
        // do not appear in the `GROUP BY` clause are taken from whichever `mdoc_copy`
        // row has the lowest disclosure count. This uses the "bare columns in aggregate
        // queries" feature that SQLite provides.
        //
        // See: https://www.sqlite.org/lang_select.html#bare_columns_in_an_aggregate_query
        let select = mdoc_copy::Entity::find()
            .select_only()
            .columns([
                mdoc_copy::Column::Id,
                mdoc_copy::Column::MdocId,
                mdoc_copy::Column::Mdoc,
            ])
            .column_as(mdoc_copy::Column::DisclosureCount.min(), "disclosure_count")
            .group_by(mdoc_copy::Column::MdocId);

        let mdoc_copies = transform_select(select).all(database.connection()).await?;

        let mdocs = mdoc_copies
            .into_iter()
            .map(|model| {
                let mdoc = cbor_deserialize(model.mdoc.as_slice())?;
                let stored_mdoc_copy = StoredMdocCopy {
                    mdoc_id: model.mdoc_id,
                    mdoc_copy_id: model.id,
                    mdoc,
                };

                Ok(stored_mdoc_copy)
            })
            .collect::<Result<_, CborError>>()?;

        Ok(mdocs)
    }

    async fn insert_doc_types(
        connection: &impl ConnectionTrait,
        new_doc_type_entities: Vec<history_doc_type::Model>,
    ) -> Result<(), DbErr> {
        if !new_doc_type_entities.is_empty() {
            let new_doc_type_entities = new_doc_type_entities
                .into_iter()
                .map(history_doc_type::ActiveModel::from)
                .collect::<Vec<_>>();

            history_doc_type::Entity::insert_many(new_doc_type_entities)
                .exec(connection)
                .await?;
        }
        Ok(())
    }

    async fn query_history_events_by_doc_type<
        Entity: EntityTrait<Model = Model>,
        Model: ModelTrait<Entity = Entity>,
        TimestampColumn: IntoSimpleExpr,
    >(
        doc_type: &str,
        connection: &impl ConnectionTrait,
        event_relation: RelationDef,
        doc_type_relation: RelationDef,
        timestamp_column: TimestampColumn,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .join_rev(JoinType::InnerJoin, event_relation)
            .join(JoinType::InnerJoin, doc_type_relation)
            .filter(history_doc_type::Column::DocType.eq(doc_type))
            .order_by_desc(timestamp_column)
            .all(connection)
            .await
    }

    async fn insert_history_event_and_doc_type_mappings<
        EventEntity: EntityTrait,
        EventActiveModel: ActiveModelTrait<Entity = EventEntity>,
        EventDocTypeEntity: EntityTrait,
        EventDocTypeActiveModel: ActiveModelTrait<Entity = EventDocTypeEntity>,
        DocTypeMapper,
    >(
        connection: &impl ConnectionTrait,
        event_entity: EventActiveModel,
        new_doc_type_entities: Vec<history_doc_type::Model>,
        existing_doc_type_entities: Vec<history_doc_type::Model>,
        doc_type_mapper: DocTypeMapper,
    ) -> StorageResult<()>
    where
        DocTypeMapper: Fn((&EventActiveModel, Uuid)) -> EventDocTypeActiveModel,
    {
        // Prepare the event <-> doc_type mapping entities.
        // This is done before inserting the `event_entity`, in order to avoid cloning.
        let event_doc_type_entities = new_doc_type_entities
            .iter()
            .chain(existing_doc_type_entities.iter())
            .map(|doc_type| doc_type_mapper((&event_entity, doc_type.id)))
            .collect::<Vec<_>>();

        // Insert the event and the new doc_types simultaneously, as they are independent
        let insert_event = EventEntity::insert(event_entity).exec(connection);
        let insert_new_doc_types = Self::insert_doc_types(connection, new_doc_type_entities);
        try_join!(insert_event, insert_new_doc_types)?;

        // Insert the event <-> doc_type mappings
        if !event_doc_type_entities.is_empty() {
            EventDocTypeEntity::insert_many(event_doc_type_entities)
                .exec(connection)
                .await?;
        }
        Ok(())
    }

    fn combine_history_events(
        issuance_events: Vec<issuance_history_event::Model>,
        disclosure_events: Vec<disclosure_history_event::Model>,
    ) -> StorageResult<Vec<WalletEvent>> {
        let mut issuance_events: Vec<WalletEvent> = issuance_events
            .into_iter()
            .map(WalletEvent::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let mut disclosure_events: Vec<WalletEvent> = disclosure_events
            .into_iter()
            .map(WalletEvent::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        issuance_events.append(&mut disclosure_events);
        issuance_events.sort_by(|a, b| b.timestamp().cmp(a.timestamp()));
        Ok(issuance_events)
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

    async fn insert_mdocs(&mut self, mdocs: Vec<MdocCopies>) -> StorageResult<()> {
        // Construct a vec of tuples of 1 `mdoc` and 1 or more `mdoc_copy` models,
        // based on the unique `MdocCopies`, to be inserted into the database.
        let mdoc_models = mdocs
            .into_iter()
            .map(|mdoc_copies| {
                let mdoc_id = Uuid::new_v4();

                let copy_models = mdoc_copies
                    .as_ref()
                    .iter()
                    .map(|mdoc| {
                        let model = mdoc_copy::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            mdoc_id: Set(mdoc_id),
                            mdoc: Set(cbor_serialize(&mdoc)?),
                            ..Default::default()
                        };

                        Ok(model)
                    })
                    .collect::<Result<Vec<_>, CborError>>()?;

                // `mdoc_copies.cred_copies` is guaranteed to contain at least one value because of the filter() above.
                let doc_type = mdoc_copies.into_iter().next().unwrap().doc_type().clone();
                let mdoc_model = mdoc::ActiveModel {
                    id: Set(mdoc_id),
                    doc_type: Set(doc_type),
                };

                Ok((mdoc_model, copy_models))
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        // Make two separate vecs out of the vec of tuples.
        let (mdoc_models, copy_models): (Vec<_>, Vec<_>) = mdoc_models.into_iter().unzip();

        let transaction = self.database()?.connection().begin().await?;

        mdoc::Entity::insert_many(mdoc_models).exec(&transaction).await?;
        mdoc_copy::Entity::insert_many(copy_models.into_iter().flatten())
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn increment_mdoc_copies_usage_count(&mut self, mdoc_copy_ids: Vec<Uuid>) -> StorageResult<()> {
        mdoc_copy::Entity::update_many()
            .col_expr(
                mdoc_copy::Column::DisclosureCount,
                Expr::col(mdoc_copy::Column::DisclosureCount).add(1),
            )
            .filter(mdoc_copy::Column::Id.is_in(mdoc_copy_ids))
            .exec(self.database()?.connection())
            .await?;

        Ok(())
    }

    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<StoredMdocCopy>> {
        self.query_unique_mdocs(|select| select).await
    }

    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<StoredMdocCopy>> {
        let doc_types_iter = doc_types.iter().copied();

        self.query_unique_mdocs(move |select| {
            select
                .inner_join(mdoc::Entity)
                .filter(mdoc::Column::DocType.is_in(doc_types_iter))
        })
        .await
    }

    async fn has_any_mdocs_with_doctype(&self, doc_type: &str) -> StorageResult<bool> {
        let result = self.fetch_unique_mdocs_by_doctypes(&HashSet::from([doc_type])).await?;
        Ok(!result.is_empty())
    }

    async fn log_wallet_event(&mut self, event: WalletEvent) -> StorageResult<()> {
        let transaction = self.database()?.connection().begin().await?;

        let event_doc_types = event.associated_doc_types();

        // Find existing doc_type entities
        let existing_doc_type_entities = history_doc_type::Entity::find()
            .filter(history_doc_type::Column::DocType.is_in(event_doc_types.clone()))
            .all(&transaction)
            .await?;

        // Get Vec of existing doc_types
        let existing_doc_types = existing_doc_type_entities
            .iter()
            .map(|e| e.doc_type.as_str())
            .collect::<Vec<_>>();

        // Determine what new doc_type entries need to be inserted
        let new_doc_type_entities = event_doc_types
            .into_iter()
            .filter(|doc_type| !existing_doc_types.contains(doc_type))
            .map(|doc_type| history_doc_type::Model {
                id: Uuid::new_v4(),
                doc_type: doc_type.to_owned(),
            })
            .collect::<Vec<_>>();

        // Insert the history event
        match WalletEventModel::try_from(event)? {
            WalletEventModel::Issuance(event_entity) => {
                Self::insert_history_event_and_doc_type_mappings(
                    &transaction,
                    issuance_history_event::ActiveModel::from(event_entity),
                    new_doc_type_entities,
                    existing_doc_type_entities,
                    |(event, doc_type_id)| issuance_history_event_doc_type::ActiveModel {
                        issuance_history_event_id: event.id.clone(),
                        history_doc_type_id: Set(doc_type_id),
                    },
                )
                .await?;
            }
            WalletEventModel::Disclosure(event_entity) => {
                Self::insert_history_event_and_doc_type_mappings(
                    &transaction,
                    disclosure_history_event::ActiveModel::from(event_entity),
                    new_doc_type_entities,
                    existing_doc_type_entities,
                    |(event, doc_type_id)| disclosure_history_event_doc_type::ActiveModel {
                        disclosure_history_event_id: event.id.clone(),
                        history_doc_type_id: Set(doc_type_id),
                    },
                )
                .await?;
            }
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let fetch_issuance_events = issuance_history_event::Entity::find()
            .order_by_desc(issuance_history_event::Column::Timestamp)
            .all(connection);

        let fetch_disclosure_events = disclosure_history_event::Entity::find()
            .order_by_desc(disclosure_history_event::Column::Timestamp)
            .all(connection);

        let (issuance_events, disclosure_events) = try_join!(fetch_issuance_events, fetch_disclosure_events)?;

        Self::combine_history_events(issuance_events, disclosure_events)
    }

    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let fetch_issuance_events = issuance_history_event::Entity::find()
            .filter(Self::newer_than_31_days(issuance_history_event::Column::Timestamp))
            .order_by_desc(issuance_history_event::Column::Timestamp)
            .all(connection);

        let fetch_disclosure_events = disclosure_history_event::Entity::find()
            .filter(Self::newer_than_31_days(disclosure_history_event::Column::Timestamp))
            .order_by_desc(disclosure_history_event::Column::Timestamp)
            .all(connection);

        let (issuance_events, disclosure_events) = try_join!(fetch_issuance_events, fetch_disclosure_events)?;

        Self::combine_history_events(issuance_events, disclosure_events)
    }

    async fn fetch_wallet_events_by_doc_type(&self, doc_type: &str) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let fetch_issuance_events = Self::query_history_events_by_doc_type(
            doc_type,
            connection,
            issuance_history_event_doc_type::Relation::HistoryEvent.def(),
            issuance_history_event_doc_type::Relation::HistoryDocType.def(),
            issuance_history_event::Column::Timestamp,
        );
        let fetch_disclosure_events = Self::query_history_events_by_doc_type(
            doc_type,
            connection,
            disclosure_history_event_doc_type::Relation::HistoryEvent.def(),
            disclosure_history_event_doc_type::Relation::HistoryDocType.def(),
            disclosure_history_event::Column::Timestamp,
        );

        let (issuance_events, disclosure_events) = try_join!(fetch_issuance_events, fetch_disclosure_events)?;

        Self::combine_history_events(issuance_events, disclosure_events)
    }

    async fn did_share_data_with_relying_party(&self, certificate: &BorrowingCertificate) -> StorageResult<bool> {
        let select_statement = Query::select()
            .column(disclosure_history_event::Column::RelyingPartyCertificate)
            .from(disclosure_history_event::Entity)
            .and_where(Expr::col(disclosure_history_event::Column::RelyingPartyCertificate).eq(certificate.as_ref()))
            .and_where(Expr::col(disclosure_history_event::Column::Status).eq(EventStatus::Success))
            .and_where(Expr::col(disclosure_history_event::Column::Attributes).is_not_null())
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

#[cfg(test)]
pub(crate) mod tests {
    use std::mem;
    use std::sync::LazyLock;

    use chrono::TimeZone;
    use chrono::Utc;
    use tokio::fs;

    use nl_wallet_mdoc::holder::Mdoc;
    use nl_wallet_mdoc::server_keys::generate::Ca;
    use nl_wallet_mdoc::server_keys::KeyPair;
    use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
    use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
    use platform_support::utils::mock::MockHardwareUtilities;
    use platform_support::utils::PlatformUtilities;
    use wallet_common::account::messages::auth::WalletCertificate;
    use wallet_common::keys::mock_hardware::MockHardwareEncryptionKey;
    use wallet_common::utils::random_bytes;

    use crate::storage::data::RegistrationData;

    use super::*;

    const PID_DOCTYPE: &str = "com.example.pid";
    const ADDRESS_DOCTYPE: &str = "com.example.address";

    static ISSUER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        issuer_ca
            .generate_issuer_mock(IssuerRegistration::new_mock().into())
            .unwrap()
    });

    static READER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let reader_ca = Ca::generate_reader_mock_ca().unwrap();
        reader_ca
            .generate_reader_mock(ReaderRegistration::new_mock().into())
            .unwrap()
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
        assert!(database_path
            .to_str()
            .unwrap()
            .contains("test_open_encrypted_database.db"));
        assert!(fs::try_exists(&database_path).await.unwrap());

        // The key file encryption key should be present.
        assert!(MockHardwareEncryptionKey::identifier_exists(&key_file_identifier));

        // Set the open database on the `DatabaseStorage` instance, then drop the storage.
        // Both the database file and the encryption key should still exist.
        storage.open_database = open_database.into();
        mem::drop(storage);
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

    async fn open_test_database_storage() -> DatabaseStorage<MockHardwareEncryptionKey> {
        let mut storage =
            DatabaseStorage::<MockHardwareEncryptionKey>::new(MockHardwareUtilities::storage_path().await.unwrap());

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

    #[tokio::test]
    async fn test_database_keyed_storage() {
        let registration = RegistrationData {
            attested_key_identifier: "key_id".to_string(),
            pin_salt: vec![1, 2, 3, 4],
            wallet_id: "wallet_123".to_string(),
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid"),
        };

        let mut storage = open_test_database_storage().await;

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
        let mut storage = open_test_database_storage().await;
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
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Create MdocsMap from example Mdoc
        let mdoc = Mdoc::new_example_mock();
        let mdoc_copies = MdocCopies::try_from([mdoc.clone(), mdoc.clone(), mdoc].to_vec()).unwrap();

        // Insert mdocs
        storage
            .insert_mdocs(vec![mdoc_copies.clone()])
            .await
            .expect("Could not insert mdocs");

        // Fetch unique mdocs
        let fetched_unique = storage
            .fetch_unique_mdocs()
            .await
            .expect("Could not fetch unique mdocs");

        // Only one unique `Mdoc` should be returned and it should match all copies.
        assert_eq!(fetched_unique.len(), 1);
        let mdoc_copy1 = fetched_unique.first().unwrap();
        assert_eq!(&mdoc_copy1.mdoc, mdoc_copies.first());

        // Increment the usage count for this mdoc.
        storage
            .increment_mdoc_copies_usage_count(vec![mdoc_copy1.mdoc_copy_id])
            .await
            .expect("Could not increment usage count for mdoc copy");

        // Fetch unique mdocs based on doctype
        let fetched_unique_doctype = storage
            .fetch_unique_mdocs_by_doctypes(&HashSet::from(["foo", "org.iso.18013.5.1.mDL"]))
            .await
            .expect("Could not fetch unique mdocs by doctypes");

        // One matching `Mdoc` should be returned and it should be a different copy than the fist one.
        assert_eq!(fetched_unique_doctype.len(), 1);
        let mdoc_copy2 = fetched_unique_doctype.first().unwrap();
        assert_eq!(&mdoc_copy2.mdoc, mdoc_copies.first());
        assert_ne!(mdoc_copy1.mdoc_copy_id, mdoc_copy2.mdoc_copy_id);

        // Increment the usage count for this mdoc.
        storage
            .increment_mdoc_copies_usage_count(vec![mdoc_copy2.mdoc_copy_id])
            .await
            .expect("Could not increment usage count for mdoc copy");

        // Fetch unique mdocs twice, which should result in exactly the same
        // copy, since it is the last one that has a `usage_count` of 0.
        let fetched_unique_remaining1 = storage
            .fetch_unique_mdocs()
            .await
            .expect("Could not fetch unique mdocs");
        let fetched_unique_remaining2 = storage
            .fetch_unique_mdocs()
            .await
            .expect("Could not fetch unique mdocs");

        // Test that the copy identifiers are the same and that they
        // are different from the other two mdoc copy identifiers.
        assert_eq!(fetched_unique_remaining1.len(), 1);
        let remaning_mdoc_copy_id1 = fetched_unique_remaining1.first().unwrap().mdoc_copy_id;
        assert_eq!(fetched_unique_remaining2.len(), 1);
        let remaning_mdoc_copy_id2 = fetched_unique_remaining2.first().unwrap().mdoc_copy_id;

        assert_eq!(remaning_mdoc_copy_id1, remaning_mdoc_copy_id2);
        assert_ne!(mdoc_copy1.mdoc_copy_id, remaning_mdoc_copy_id1);
        assert_ne!(mdoc_copy2.mdoc_copy_id, remaning_mdoc_copy_id1);

        // Fetch unique mdocs based on non-existent doctype
        let fetched_unique_doctype_mismatch = storage
            .fetch_unique_mdocs_by_doctypes(&HashSet::from(["foo", "bar"]))
            .await
            .unwrap();

        // No entries should be returned
        assert!(fetched_unique_doctype_mismatch.is_empty());
    }

    #[tokio::test]
    async fn test_event_log_storage_ordering() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));
        test_history_ordering(&mut storage).await;
    }

    #[tokio::test]
    async fn test_event_log_storage_by_doc_type() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        test_history_by_doc_type(&mut storage).await;
    }

    #[tokio::test]
    async fn test_storing_disclosure_cancel_event() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let disclosure_cancel = WalletEvent::disclosure_cancel(timestamp, READER_KEY.certificate().clone());

        // No data shared with RP
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());

        // Log cancel event
        storage.log_wallet_event(disclosure_cancel.clone()).await.unwrap();

        // Cancel event should exist
        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![disclosure_cancel.clone(),]
        );

        // Still no data shared with RP
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_storing_disclosure_error_event_without_data() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let disclosure_error = WalletEvent::disclosure_error(timestamp, READER_KEY.certificate().clone());

        // No data shared with RP
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());

        // Log error event
        storage.log_wallet_event(disclosure_error.clone()).await.unwrap();

        // Error event should exist
        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![disclosure_error.clone(),]
        );

        // Still no data shared with RP
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_storing_disclosure_error_event_with_data() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let disclosure_error = WalletEvent::disclosure_error_from_str(
            &[PID_DOCTYPE],
            timestamp,
            READER_KEY.certificate().clone(),
            ISSUER_KEY.certificate(),
        );

        // No data shared with RP
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());

        storage.log_wallet_event(disclosure_error.clone()).await.unwrap();

        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![disclosure_error.clone(),]
        );

        // Still no data has been shared with RP, because we only consider Successful events
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());
    }

    pub(crate) async fn test_history_ordering(storage: &mut impl Storage) {
        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let timestamp_older = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();
        let timestamp_even_older = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();

        let disclosure_at_timestamp = WalletEvent::disclosure_from_str(
            &[PID_DOCTYPE],
            timestamp,
            READER_KEY.certificate().clone(),
            ISSUER_KEY.certificate(),
        );
        let issuance_at_older_timestamp =
            WalletEvent::issuance_from_str(&[ADDRESS_DOCTYPE], timestamp_older, ISSUER_KEY.certificate());
        let issuance_at_even_older_timestamp =
            WalletEvent::issuance_from_str(&[PID_DOCTYPE], timestamp_even_older, ISSUER_KEY.certificate());

        // No data shared with RP
        assert!(!storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());

        // Insert events, in non-standard order, from new to old
        storage.log_wallet_event(disclosure_at_timestamp.clone()).await.unwrap();
        storage
            .log_wallet_event(issuance_at_older_timestamp.clone())
            .await
            .unwrap();
        storage
            .log_wallet_event(issuance_at_even_older_timestamp.clone())
            .await
            .unwrap();

        // Data has been shared with RP
        assert!(storage
            .did_share_data_with_relying_party(READER_KEY.certificate())
            .await
            .unwrap());

        // Fetch and verify events are sorted descending by timestamp
        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![
                disclosure_at_timestamp.clone(),
                issuance_at_older_timestamp.clone(),
                issuance_at_even_older_timestamp.clone()
            ]
        );
        // Fetch event by pid and verify events are sorted descending by timestamp
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(PID_DOCTYPE).await.unwrap(),
            vec![
                disclosure_at_timestamp.clone(),
                issuance_at_even_older_timestamp.clone()
            ]
        );
        // Fetch event by address
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(ADDRESS_DOCTYPE).await.unwrap(),
            vec![issuance_at_older_timestamp]
        );
        // Fetching for unknown-doc-type returns empty Vec
        assert_eq!(
            storage
                .fetch_wallet_events_by_doc_type("unknown-doc-type")
                .await
                .unwrap(),
            vec![]
        );
    }

    pub(crate) async fn test_history_by_doc_type(storage: &mut impl Storage) {
        let timestamp = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();
        let timestamp_newer = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();
        let timestamp_newest = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();

        // Log Issuance of pid and address cards
        let issuance =
            WalletEvent::issuance_from_str(&[PID_DOCTYPE, ADDRESS_DOCTYPE], timestamp, ISSUER_KEY.certificate());
        storage.log_wallet_event(issuance.clone()).await.unwrap();

        // Log Disclosure of pid and address cards
        let disclosure_pid_and_address = WalletEvent::disclosure_from_str(
            &[PID_DOCTYPE, ADDRESS_DOCTYPE],
            timestamp_newer,
            READER_KEY.certificate().clone(),
            ISSUER_KEY.certificate(),
        );
        storage
            .log_wallet_event(disclosure_pid_and_address.clone())
            .await
            .unwrap();

        // Log Disclosure of pid card only
        let disclosure_pid_only = WalletEvent::disclosure_from_str(
            &[PID_DOCTYPE],
            timestamp_newest,
            READER_KEY.certificate().clone(),
            ISSUER_KEY.certificate(),
        );
        storage.log_wallet_event(disclosure_pid_only.clone()).await.unwrap();

        // Fetch event by pid and verify events contain issuance of pid, and both full disclosure transactions with pid
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(PID_DOCTYPE).await.unwrap(),
            vec![
                disclosure_pid_only.clone(),
                disclosure_pid_and_address.clone(),
                issuance.clone(),
            ]
        );
        // Fetch event by address and verify events contain issuance of address, and one full disclosure transactions
        // with address
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(ADDRESS_DOCTYPE).await.unwrap(),
            vec![disclosure_pid_and_address, issuance,]
        );
    }
}
