use anyhow::Result;

use wallet_server::pid::attributes::BrpPidAttributeService;
use wallet_server::server::wallet_server_main;
use wallet_server::server::{self};
use wallet_server::settings::Settings;
use wallet_server::store::DatabaseConnection;
use wallet_server::store::SessionStoreVariant;
use wallet_server::store::WteTrackerVariant;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pid_issuer.toml", "pid_issuer", main_impl).await
}

async fn main_impl(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let db_connection = DatabaseConnection::try_new(storage_settings.url.clone()).await?;

    let sessions = SessionStoreVariant::new(db_connection.clone(), storage_settings.into());
    let wte_tracker = WteTrackerVariant::new(db_connection);

    // This will block until the server shuts down.
    server::pid_issuer::serve(
        BrpPidAttributeService::try_from(&settings.issuer)?,
        settings,
        sessions,
        wte_tracker,
    )
    .await
}
