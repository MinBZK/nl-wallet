use anyhow::Result;
use wallet_server::{
    pid::attributes::BrpPidAttributeService,
    server::{self, wallet_server_main},
    settings::Settings,
    store::SessionStores,
};

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<()> {
    wallet_server_main("wallet_server.toml", "wallet_server", async_main)
}

async fn async_main(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let sessions = SessionStores::init(storage_settings.url.clone(), storage_settings.into()).await?;

    // This will block until the server shuts down.
    server::wallet_server::serve(
        BrpPidAttributeService::try_from(&settings)?,
        settings,
        sessions.disclosure,
        sessions.issuance,
    )
    .await
}
