use anyhow::Result;

use wallet_server::{
    server::{self, wallet_server_main},
    settings::Settings,
    store::SessionStoreVariant,
};

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<()> {
    wallet_server_main("wallet_server.toml", "wallet_server", async_main)
}

async fn async_main(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let disclosure_sessions = SessionStoreVariant::new(storage_settings.url.clone(), storage_settings.into()).await?;
    // Clone from `disclosure_sessions` so that database connection pool is reused when using PostgreSQL.
    let issuance_sessions = disclosure_sessions.clone_into();

    // This will block until the server shuts down.
    server::wallet_server::serve(settings, disclosure_sessions, issuance_sessions).await
}
