use std::path::Path;

use config::Config;
use config::File;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use url::Url;

use utils::path::prefix_local_path;

#[derive(Debug, Clone, Deserialize)]
struct TestSettings {
    storage_url: Url,
}

pub async fn connection_from_settings() -> anyhow::Result<DatabaseConnection> {
    let settings: TestSettings = Config::builder()
        .add_source(File::from(prefix_local_path(Path::new("test_settings.toml")).as_ref()).required(true))
        .build()?
        .try_deserialize()?;
    let connection = crate::store::postgres::new_connection(settings.storage_url).await?;
    Ok(connection)
}
