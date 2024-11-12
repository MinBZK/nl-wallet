use anyhow::Result;

use wallet_server::{
    server::{self, wallet_server_main},
    settings::Settings,
    store::{DatabaseConnection, SessionStoreVariant},
};

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("verification_server.toml", "verification_server", main_impl).await
}

async fn main_impl(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let sessions = SessionStoreVariant::new(
        DatabaseConnection::try_new(storage_settings.url.clone()).await?,
        storage_settings.into(),
    );

    // This will block until the server shuts down.
    server::verification_server::serve(settings, sessions).await
}
