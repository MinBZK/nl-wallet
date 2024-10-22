use anyhow::Result;

use wallet_server::{
    pid::attributes::BrpPidAttributeService,
    server::{self, wallet_server_main},
    settings::Settings,
    store::SessionStoreVariant,
};

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("wallet_server.toml", "wallet_server", main_impl).await
}

async fn main_impl(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let disclosure_sessions = SessionStoreVariant::new(storage_settings.url.clone(), storage_settings.into()).await?;
    // Clone from `disclosure_sessions` so that database connection pool is reused when using PostgreSQL.
    let issuance_sessions = disclosure_sessions.clone_into();

    // This will block until the server shuts down.
    server::wallet_server::serve(
        BrpPidAttributeService::try_from(&settings.issuer)?,
        settings,
        disclosure_sessions,
        issuance_sessions,
    )
    .await
}
