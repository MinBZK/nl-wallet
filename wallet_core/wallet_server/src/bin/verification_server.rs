use anyhow::Result;

use wallet_server::{
    server::{self, wallet_server_main},
    settings::Settings,
    store::SessionStores,
};

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<()> {
    wallet_server_main("verification_server.toml", "verification_server", async_main)
}

async fn async_main(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let sessions = SessionStores::init(storage_settings.url.clone(), storage_settings.into()).await?;

    // This will block until the server shuts down.
    server::verification_server::serve(settings, sessions.disclosure).await
}
