use anyhow::Result;

use openid4vc_server::store::DatabaseConnection;
use openid4vc_server::store::SessionStoreVariant;
use openid4vc_server::store::WteTrackerVariant;
use wallet_server::pid::attributes::BrpPidAttributeService;
use wallet_server::server;
use wallet_server::server::wallet_server_main;
use wallet_server::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("wallet_server.toml", "wallet_server", main_impl).await
}

async fn main_impl(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let db_connection = DatabaseConnection::try_new(storage_settings.url.clone()).await?;

    let disclosure_sessions = SessionStoreVariant::new(db_connection.clone(), storage_settings.into());
    let issuance_sessions = SessionStoreVariant::new(db_connection.clone(), storage_settings.into());
    let wte_tracker = WteTrackerVariant::new(db_connection);

    // This will block until the server shuts down.
    server::wallet_server::serve(
        BrpPidAttributeService::try_from(&settings.issuer)?,
        settings,
        disclosure_sessions,
        issuance_sessions,
        wte_tracker,
    )
    .await
}
