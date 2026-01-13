use std::path::Path;
use std::time::Duration;

use config::Config;
use config::File;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tracing::log::LevelFilter;
use url::Url;

use utils::path::prefix_local_path;

#[derive(Debug, Clone, Deserialize)]
pub struct TestSettings {
    pub storage_url: Url,
}

pub fn test_settings() -> TestSettings {
    Config::builder()
        .add_source(File::from(prefix_local_path(Path::new("test_settings.toml")).as_ref()).required(true))
        .build()
        .expect("cannot build config")
        .try_deserialize()
        .expect("cannot read test settings")
}

pub fn default_connection_options(options: &mut ConnectOptions) {
    options
        .connect_timeout(Duration::from_secs(3))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);
}

pub async fn connection_from_settings() -> DatabaseConnection {
    let mut connection_options = ConnectOptions::new(test_settings().storage_url);
    default_connection_options(&mut connection_options);
    Database::connect(connection_options)
        .await
        .expect("cannot connect to database")
}
