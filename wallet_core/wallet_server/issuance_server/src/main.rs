use std::sync::Arc;

use anyhow::Result;
use anyhow::anyhow;

use hsm::service::Pkcs11Hsm;
use issuance_server::disclosure::HttpAttributesFetcher;
use issuance_server::server;
use issuance_server::settings::IssuanceServerSettings;
use server_utils::server::wallet_server_main;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
use server_utils::store::postgres::new_connection;
use status_lists::config::StatusListConfigs;
use status_lists::postgres::PostgresStatusListServices;

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
    let store_connection = StoreConnection::try_new(storage_settings.url.clone()).await?;

    let issuance_sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));
    let disclosure_sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));

    let attributes_fetcher = HttpAttributesFetcher::try_new(
        settings
            .disclosure_settings
            .iter()
            .map(|(id, s)| (id.clone(), s.attestation_url_config.clone()))
            .collect(),
    )?;

    let db_connection = match (store_connection, settings.status_lists.storage_url.as_ref()) {
        (_, Some(url)) => new_connection(url.clone()).await.map_err(anyhow::Error::from),
        (StoreConnection::Postgres(db_connection), None) => Ok(db_connection),
        _ => Err(anyhow!(
            "No database connection configured for status list in issuance server"
        )),
    }?;
    let status_list_configs = StatusListConfigs::from_settings(
        &settings.status_lists,
        (&settings.issuer_settings.attestation_settings)
            .into_iter()
            .map(|(id, settings)| (id.to_owned(), settings.status_list.clone())),
        &hsm,
    )
    .await?;
    let status_list_service = PostgresStatusListServices::try_new(db_connection, status_list_configs).await?;
    status_list_service.initialize_lists().await?;

    // This will block until the server shuts down.
    server::serve(
        settings,
        hsm,
        issuance_sessions,
        disclosure_sessions,
        attributes_fetcher,
        status_list_service,
    )
    .await
}
