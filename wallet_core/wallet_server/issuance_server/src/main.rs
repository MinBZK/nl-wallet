use std::sync::Arc;

use anyhow::Result;

use hsm::service::Pkcs11Hsm;
use issuance_server::disclosure::HttpAttributesFetcher;
use issuance_server::server;
use issuance_server::settings::IssuanceServerSettings;
use server_utils::server::wallet_server_main;
use server_utils::store::DatabaseConnection;
use server_utils::store::SessionStoreVariant;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("issuance_server.toml", "issuance_server", main_impl).await
}

async fn main_impl(settings: IssuanceServerSettings) -> Result<()> {
    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;

    let storage_settings = &settings.issuer_settings.server_settings.storage;
    let db_connection = DatabaseConnection::try_new(storage_settings.url.clone()).await?;

    let issuance_sessions = Arc::new(SessionStoreVariant::new(db_connection.clone(), storage_settings.into()));
    let disclosure_sessions = Arc::new(SessionStoreVariant::new(db_connection, storage_settings.into()));

    let attributes_fetcher = HttpAttributesFetcher::new(
        settings
            .disclosure_settings
            .iter()
            .map(|(id, s)| (id.clone(), s.attestation_url.clone()))
            .collect(),
    );

    // This will block until the server shuts down.
    server::serve(
        settings,
        hsm,
        issuance_sessions,
        disclosure_sessions,
        attributes_fetcher,
    )
    .await
}
