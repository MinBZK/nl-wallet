use std::sync::Arc;

use anyhow::Result;

use hsm::service::Pkcs11Hsm;
use server_utils::server::wallet_server_main;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
use verification_server::server;
use verification_server::settings::VerifierSettings;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("verification_server.toml", "verification_server", main_impl).await
}

async fn main_impl(settings: VerifierSettings) -> Result<()> {
    let hsm = settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;

    let storage_settings = &settings.server_settings.storage;
    let sessions = Arc::new(SessionStoreVariant::new(
        StoreConnection::try_new(storage_settings.url.clone()).await?,
        storage_settings.into(),
    ));

    // This will block until the server shuts down.
    server::serve(settings, hsm, sessions).await
}
