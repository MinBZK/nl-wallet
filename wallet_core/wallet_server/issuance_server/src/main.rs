use std::sync::Arc;

use anyhow::Result;
use anyhow::anyhow;

use health_checkers::boxed;
use health_checkers::hsm::HsmChecker;
use health_checkers::postgres::DatabaseChecker;
use hsm::service::Pkcs11Hsm;
use http_utils::health::create_health_router;
use http_utils::reqwest::default_reqwest_client_builder;
use issuance_server::disclosure::HttpAttributesFetcher;
use issuance_server::server;
use issuance_server::settings::IssuanceServerSettings;
use issuer_settings::settings::StatusListAttestationSettings;
use server_utils::server::wallet_server_main;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
use server_utils::store::postgres::new_connection;
use status_lists::postgres::PostgresStatusListServices;
use status_lists::serve::create_serve_router;
use token_status_list::verification::reqwest::HttpStatusListClient;

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
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let storage_settings = &settings.issuer_settings.server_settings.storage;
    let store_connection = StoreConnection::try_new(storage_settings.url.clone()).await?;
    let mut store_checker = match &store_connection {
        StoreConnection::Postgres(connection) => Some(DatabaseChecker::new("db-session", connection)),
        _ => None,
    };

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

    let (db_connection, status_list_checker) = match (store_connection, settings.status_lists.storage_url.as_ref()) {
        (_, Some(url)) => {
            let connection = new_connection(url.clone()).await.map_err(anyhow::Error::from)?;
            let checker = DatabaseChecker::new("db-status-list", &connection);
            Ok((connection, Some(checker)))
        }
        (StoreConnection::Postgres(db_connection), None) => {
            // Safe unwrap as store is Postgres
            store_checker.as_mut().unwrap().rename("db");
            Ok((db_connection, None))
        }
        _ => Err(anyhow!(
            "No database connection configured for status list in issuance server"
        )),
    }?;
    let status_list_configs = StatusListAttestationSettings::settings_into_configs(
        settings
            .issuer_settings
            .attestation_settings
            .as_ref()
            .iter()
            .map(|(id, settings)| (id.clone(), settings.status_list.clone())),
        &settings.status_lists,
        settings.issuer_settings.public_url.as_base_url(),
        hsm.clone(),
    )
    .await?;
    let status_list_services = PostgresStatusListServices::try_new(db_connection, status_list_configs).await?;
    status_list_services.initialize_lists().await?;
    status_list_services.start_refresh_jobs();
    let status_list_router = settings
        .status_lists
        .serve
        .then(|| {
            create_serve_router(
                (&settings.issuer_settings.attestation_settings)
                    .into_iter()
                    .map(|(_, settings)| {
                        (
                            settings.status_list.context_path.as_str(),
                            settings.status_list.publish_dir.clone(),
                        )
                    }),
                settings.status_lists.ttl(),
            )
        })
        .transpose()?;

    let status_list_client = HttpStatusListClient::new(default_reqwest_client_builder())?;

    let db_checkers = [store_checker, status_list_checker].into_iter().flat_map(boxed);
    let health_router = create_health_router(std::iter::once(hsm_checker).flat_map(boxed).chain(db_checkers));

    // This will block until the server shuts down.
    server::serve(
        settings,
        hsm,
        issuance_sessions,
        disclosure_sessions,
        attributes_fetcher,
        status_list_services,
        status_list_router,
        status_list_client,
        health_router,
    )
    .await
}
