use std::sync::Arc;

use anyhow::Result;
use anyhow::anyhow;

use hsm::service::Pkcs11Hsm;
use http_utils::health::create_health_router;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
use pid_issuer::server;
use pid_issuer::settings::PidIssuerSettings;
use server_utils::checkers::DatabaseChecker;
use server_utils::checkers::HsmChecker;
use server_utils::checkers::boxed;
use server_utils::keys::SecretKeyVariant;
use server_utils::server::wallet_server_main;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
use server_utils::store::postgres::new_connection;
use status_lists::postgres::PostgresStatusListServices;
use status_lists::serve::create_serve_router;
use status_lists::settings::StatusListAttestationSettings;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pid_issuer.toml", "pid_issuer", main_impl).await
}

async fn main_impl(settings: PidIssuerSettings) -> Result<()> {
    let issuer_settings = settings.issuer_settings;
    let hsm = issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let storage_settings = &issuer_settings.server_settings.storage;
    let store_connection = StoreConnection::try_new(storage_settings.url.clone()).await?;
    let mut store_checker = match &store_connection {
        StoreConnection::Postgres(connection) => Some(DatabaseChecker::new("db-session", connection)),
        _ => None,
    };

    let sessions = Arc::new(SessionStoreVariant::new(
        store_connection.clone(),
        storage_settings.into(),
    ));

    let pid_attr_service = BrpPidAttributeService::try_new(
        HttpBrpClient::new(settings.brp_server),
        &settings.digid.bsn_privkey,
        settings.digid.http_config,
        SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?,
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
            "No database connection configured for status list in pid issuer"
        )),
    }?;
    let status_list_configs = StatusListAttestationSettings::settings_into_configs(
        issuer_settings
            .attestation_settings
            .as_ref()
            .iter()
            .map(|(id, settings)| (id.clone(), settings.status_list.clone())),
        &settings.status_lists,
        &issuer_settings.server_settings.public_url,
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
                (&issuer_settings.attestation_settings)
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

    let db_checkers = [store_checker, status_list_checker].into_iter().flat_map(boxed);
    let health_router = create_health_router(std::iter::once(hsm_checker).flat_map(boxed).chain(db_checkers));

    // This will block until the server shuts down.
    server::serve(
        pid_attr_service,
        issuer_settings,
        hsm,
        sessions,
        settings.wua_issuer_pubkey.into_inner(),
        status_list_services,
        status_list_router,
        health_router,
    )
    .await
}
