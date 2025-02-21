use anyhow::Result;

use openid4vc_server::store::DatabaseConnection;
use openid4vc_server::store::SessionStoreVariant;
use wallet_server::server;
use wallet_server::server::wallet_server_main;
use wallet_server::settings::VerifierSettings;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("verification_server.toml", "verification_server", main_impl).await
}

async fn main_impl(settings: VerifierSettings) -> Result<()> {
    let storage_settings = &settings.server_settings.storage;
    let sessions = SessionStoreVariant::new(
        DatabaseConnection::try_new(storage_settings.url.clone()).await?,
        storage_settings.into(),
    );

    // This will block until the server shuts down.
    server::verification_server::serve(settings, sessions).await
}
