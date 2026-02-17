use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use std::thread::available_parallelism;
use std::time::Duration;

use async_dropper::AsyncDrop;
use async_dropper::AsyncDropper;
use async_trait::async_trait;
use bollard::errors::Error as BollardError;
use rand::Rng;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::sqlx::ConnectOptions;
use sea_orm::sqlx::Connection;
use sea_orm::sqlx::Encode;
use sea_orm::sqlx::Executor;
use sea_orm::sqlx::PgConnection;
use sea_orm::sqlx::Postgres;
use sea_orm::sqlx::QueryBuilder;
use sea_orm::sqlx::Row;
use sea_orm::sqlx::Type;
use sea_orm::sqlx::postgres::PgConnectOptions;
use sea_orm::sqlx::query;
use sea_orm_migration::MigratorTrait;
use strum::AsRefStr;
use strum::Display;
use strum::EnumString;
use strum::VariantArray;
use testcontainers::ImageExt;
use testcontainers::ReuseDirective;
use testcontainers::TestcontainersError;
use testcontainers::core::error::ClientError;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres;
use tracing::log::LevelFilter;
use url::Url;

const DB_PORT: u16 = 5432;
const DB_USER: &str = "postgres";
const DB_PASSWORD: &str = "postgres";
const DB_DEFAULT_DATABASE: &str = "postgres";

const DB_TESTCONTAINER_IMAGE: &str = "docker.io/library/postgres";
const DB_TESTCONTAINER_IMAGE_TAG: &str = "18.2-trixie";
const DB_TESTCONTAINER_NAME: &str = "wallet-postgres-test";
const DB_TESTCONTAINER_TRIES: u8 = 5;
const DB_TESTCONTAINER_RETRY_DELAY: Duration = Duration::from_secs(11);

#[derive(Default)]
struct AsyncDropPgConnection(Option<PgConnection>);

impl From<PgConnection> for AsyncDropPgConnection {
    fn from(value: PgConnection) -> Self {
        Self(Some(value))
    }
}

impl AsyncDropPgConnection {
    fn as_mut_ref(&mut self) -> &mut PgConnection {
        self.0.as_mut().unwrap()
    }

    async fn try_advisory_lock(&mut self, n: u32) -> bool {
        let row = self
            .as_mut_ref()
            .fetch_one(query("SELECT pg_try_advisory_lock($1)").bind(i64::from(n)))
            .await
            .expect("Try advisory lock failed");
        row.get(0)
    }

    async fn advisory_lock(&mut self, n: u32) -> () {
        self.as_mut_ref()
            .execute(query("SELECT pg_advisory_lock($1)").bind(i64::from(n)))
            .await
            .expect("Advisory lock failed");
    }

    async fn advisory_unlock(&mut self, n: u32) -> () {
        self.as_mut_ref()
            .execute(query("SELECT pg_advisory_unlock($1)").bind(i64::from(n)))
            .await
            .expect("Advisory unlock failed");
    }
}

#[async_trait]
impl AsyncDrop for AsyncDropPgConnection {
    async fn async_drop(&mut self) {
        if let Some(connection) = self.0.take() {
            _ = connection.close().await;
        }
    }
}

/// DbSetup is a setup of an exclusive set of databases for a test
///
/// DbSetup uses advisory locks to coordinate which tests has access to which set of databases.
/// When this struct is dropped it releases the advisory locks.
pub struct DbSetup {
    _connection: AsyncDropper<AsyncDropPgConnection>,
    connect_options: PgConnectOptions,
    index: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRefStr, Display, EnumString, VariantArray)]
#[strum(serialize_all = "snake_case")]
enum DbName {
    StatusLists,
    WalletProviderAuditLog,
}

impl DbName {
    fn template_url(self, connect_options: PgConnectOptions) -> Url {
        connect_options.database(self.as_ref()).to_url_lossy()
    }

    fn with_index(self, index: u32) -> String {
        format!("{self}_{index}")
    }

    fn url(self, connect_options: PgConnectOptions, index: u32) -> Url {
        connect_options.database(&self.with_index(index)).to_url_lossy()
    }
}

impl DbSetup {
    /// Create a database setup
    pub async fn create() -> Self {
        Self::create_with_clean(false).await
    }

    /// Create a clean database setup
    ///
    /// This involves dropping the database and recreating from the templates.
    pub async fn create_clean() -> Self {
        Self::create_with_clean(true).await
    }

    async fn create_with_clean(clean: bool) -> Self {
        let ci = std::env::var("CI").is_ok();

        // Start testcontainer when not on CI
        let (host, port) = if ci {
            (
                std::env::var("DB_HOST").expect("DB_HOST should be set").to_string(),
                std::env::var("DB_PORT")
                    .map(|text| text.parse::<u16>().expect("DB_PORT is not a u16"))
                    .unwrap_or(DB_PORT),
            )
        } else {
            start_testcontainer().await
        };

        // Connect to database
        let connect_options = PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(DB_USER)
            .password(DB_PASSWORD)
            .database(DB_DEFAULT_DATABASE);
        let connection = PgConnection::connect_with(&connect_options)
            .await
            .expect("Connecting to database failed");
        // Wrap in dropper to ensure connection is closed on dropped and advisory locks are released
        let mut connection = AsyncDropper::new(connection.into());

        // Create templates
        setup_templates(&mut connection, &connect_options).await;

        // Find free set of databases
        let index = find_free_set_of_databases(&mut connection).await;

        // Create databases
        setup_databases(&mut connection, index, clean).await;

        Self {
            _connection: connection,
            connect_options,
            index,
        }
    }

    /// Returns the connect options to the database server with the default database
    pub fn connect_options(&self) -> &PgConnectOptions {
        &self.connect_options
    }

    /// Returns a lossy url to the database server with the default database
    pub fn connect_url(&self) -> Url {
        self.connect_options.to_url_lossy()
    }

    pub fn audit_log_url(&self) -> Url {
        DbName::WalletProviderAuditLog.url(self.connect_options.clone(), self.index)
    }

    pub fn status_lists_url(&self) -> Url {
        DbName::StatusLists.url(self.connect_options.clone(), self.index)
    }
}

async fn start_testcontainer() -> (String, u16) {
    let mut n = 0;
    let container = loop {
        let result = postgres::Postgres::default()
            .with_name(DB_TESTCONTAINER_IMAGE)
            .with_tag(DB_TESTCONTAINER_IMAGE_TAG)
            .with_container_name(DB_TESTCONTAINER_NAME)
            .with_reuse(ReuseDirective::Always)
            .start()
            .await;
        match result {
            // When the container is not yet started, conflicts are returned, retry after a delay
            Err(TestcontainersError::Client(ClientError::CreateContainer(
                BollardError::DockerResponseServerError { status_code: 409, .. },
            ))) => {
                tokio::time::sleep(DB_TESTCONTAINER_RETRY_DELAY).await;
            }
            result => break result,
        }
        n += 1;
        if n == DB_TESTCONTAINER_TRIES {
            panic!("Could not start testcontainer in {n} tries");
        }
    }
    .expect("Could not start testcontainer");
    (
        container
            .get_host()
            .await
            .expect("Could not get testcontainer host")
            .to_string(),
        container
            .get_host_port_ipv4(DB_PORT)
            .await
            .expect("Could not get testcontainer port"),
    )
}

/// Find a free set of databases
///
/// This uses the [1,n] advisory locks from the database server. It randomly
/// starts and tries to acquire a lock. `n` is equal to the available
/// parallelism, which should be equal to the maximum number of tests.
async fn find_free_set_of_databases(connection: &mut AsyncDropPgConnection) -> u32 {
    let max_parallel = available_parallelism().expect("cannot get parallelism").get();
    let max_parallel = max_parallel.try_into().unwrap();

    let mut n = rand::thread_rng().gen_range(1..=max_parallel);
    for _ in 1..=max_parallel {
        if connection.try_advisory_lock(n).await {
            return n;
        }

        if n == max_parallel {
            n = 0;
        }
        n += 1;
    }
    panic!("Too much contention!")
}

async fn fetch_existing_database<'a, T>(
    connection: &mut AsyncDropPgConnection,
    databases: impl IntoIterator<Item = impl Encode<'a, Postgres> + Type<Postgres> + 'a>,
) -> HashSet<T>
where
    T: FromStr + PartialEq + Eq + Hash,
    T::Err: Debug,
{
    let mut builder = QueryBuilder::new("SELECT datname FROM pg_catalog.pg_database WHERE datname IN");
    let query = builder
        .push_tuples(databases.into_iter(), |mut b, name| {
            b.push_bind(name);
        })
        .build();
    connection
        .as_mut_ref()
        .fetch_all(query)
        .await
        .expect("Could not query for databases")
        .into_iter()
        .map(|row| T::from_str(row.get::<&str, _>(0)))
        .collect::<Result<HashSet<_>, _>>()
        .expect("Could not fetch databases")
}

async fn setup_templates(connection: &mut AsyncDropPgConnection, connect_options: &PgConnectOptions) {
    // Fetch all created databases
    let existing = fetch_existing_database::<DbName>(connection, DbName::VARIANTS.iter().map(DbName::as_ref)).await;

    // Early exit if all databases exists
    if existing.len() == DbName::VARIANTS.len() {
        return;
    }

    // Get exclusive lock to prevent double migration
    connection.advisory_lock(0).await;

    // Update after exclusive lock
    let existing = fetch_existing_database::<DbName>(connection, DbName::VARIANTS.iter().map(DbName::as_ref)).await;

    // Create non-existing databases and migrate
    for name in DbName::VARIANTS {
        if !existing.contains(name) {
            connection
                .as_mut_ref()
                .execute(query(&format!(r#"CREATE DATABASE "{name}""#)))
                .await
                .unwrap_or_else(|e| panic!("Could not create database {}: {}", name, e));

            // Run migrations on all newly created databases. Although we could run migrations on
            // all databases, it does not happen often, and it is easier to just stop and remove the
            // container.
            migrate(*name, connect_options.clone()).await
        }
    }

    // Unlock
    connection.advisory_unlock(0).await;
}

async fn setup_databases(connection: &mut AsyncDropPgConnection, index: u32, clean: bool) {
    // Fetch all created databases
    let existing =
        fetch_existing_database::<String>(connection, DbName::VARIANTS.iter().map(|name| name.with_index(index))).await;

    // Create tables from template that do not exist
    for name in DbName::VARIANTS {
        let indexed_name = name.with_index(index);

        // Drop with force if clean table is requested
        let exist = match (existing.contains(&indexed_name), clean) {
            (true, true) => {
                connection
                    .as_mut_ref()
                    .execute(query(&format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, indexed_name)))
                    .await
                    .unwrap_or_else(|e| panic!("Could not drop database {}: {}", indexed_name, e));
                false
            }
            (exist, _) => exist,
        };

        if !exist {
            connection
                .as_mut_ref()
                .execute(query(&format!(r#"CREATE DATABASE "{indexed_name}" TEMPLATE "{name}""#)))
                .await
                .unwrap_or_else(|e| panic!("Could not create database {indexed_name}: {e}"));
        }
    }
}

async fn migrate(name: DbName, connect_options: PgConnectOptions) {
    let url = name.template_url(connect_options);
    let pool = connection_from_url(url.clone()).await;
    match name {
        DbName::StatusLists => status_lists_migrations::Migrator::up(&pool, None).await,
        DbName::WalletProviderAuditLog => audit_log_migrations::Migrator::up(&pool, None).await,
    }
    .unwrap_or_else(|e| panic!("Could not migrate database {}: {}", name, e));
    _ = pool.close().await;
}

pub async fn connection_from_url(url: Url) -> DatabaseConnection {
    let mut connection_options = sea_orm::ConnectOptions::new(url);
    default_connection_options(&mut connection_options);
    Database::connect(connection_options)
        .await
        .expect("cannot connect to database")
}

pub fn default_connection_options(options: &mut sea_orm::ConnectOptions) {
    options
        .connect_timeout(Duration::from_secs(3))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);
}
