use std::sync::Arc;

use anyhow::Result;
use anyhow::anyhow;
use hsm::service::Pkcs11Hsm;
use itertools::Itertools;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
use pid_issuer::server;
use pid_issuer::settings::PidIssuerSettings;
use server_utils::keys::SecretKeyVariant;
use server_utils::server::wallet_server_main;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
use server_utils::store::postgres::new_connection;
use status_lists::postgres::PostgresStatusListService;

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

    let storage_settings = &issuer_settings.server_settings.storage;
    let store_connection = StoreConnection::try_new(storage_settings.url.clone()).await?;

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

    let db_connection = match (store_connection, settings.status_lists.storage_url.as_ref()) {
        (_, Some(url)) => new_connection(url.clone()).await.map_err(anyhow::Error::from),
        (StoreConnection::Postgres(db_connection), None) => Ok(db_connection),
        _ => Err(anyhow!(
            "No database connection configured for status list in pid issuer"
        )),
    }?;
    let status_list_service = PostgresStatusListService::try_new(
        db_connection,
        settings.status_lists,
        &issuer_settings
            .attestation_settings
            .as_ref()
            .keys()
            .cloned()
            .collect_vec(),
    )
    .await?;
    status_list_service.initialize_lists().await?;

    // This will block until the server shuts down.
    server::serve(
        pid_attr_service,
        issuer_settings,
        hsm,
        sessions,
        settings.wua_issuer_pubkey.into_inner(),
        status_list_service,
    )
    .await
}
