use anyhow::Result;

use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::server;
use pid_issuer::settings::IssuerSettings;
use pid_issuer::wte_tracker::WteTrackerVariant;
use server_utils::server::wallet_server_main;
use server_utils::store::DatabaseConnection;
use server_utils::store::SessionStoreVariant;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pid_issuer.toml", "pid_issuer", main_impl).await
}

async fn main_impl(settings: IssuerSettings) -> Result<()> {
    let storage_settings = &settings.server_settings.storage;
    let db_connection = DatabaseConnection::try_new(storage_settings.url.clone()).await?;

    let sessions = SessionStoreVariant::new(db_connection.clone(), storage_settings.into());
    let wte_tracker = WteTrackerVariant::new(db_connection);

    // This will block until the server shuts down.
    server::serve(
        BrpPidAttributeService::try_from(&settings)?,
        settings,
        sessions,
        wte_tracker,
    )
    .await
}
