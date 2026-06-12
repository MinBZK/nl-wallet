use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use pid_issuer::pid::auth_code_flow::UpstreamOidcAuthorizationCodeFlow;
use pid_issuer::pid::brp::client::HttpBrpClient;
use pid_issuer::pid::digid::DigidMetadataCache;
use pid_issuer::server;
use pid_issuer::settings::PidIssuerSettings;
use server_utils::keys::SecretKeyVariant;
use server_utils::server::wallet_server_main;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pid_issuer.toml", "pid_issuer", main_impl).await
}

async fn main_impl(settings: PidIssuerSettings) -> Result<()> {
    let serve_status_lists = settings.authorizing_issuer_settings.issuer_settings.status_lists.serve;

    let hsm = settings
        .authorizing_issuer_settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let digid_metadata_cache = DigidMetadataCache::try_new(settings.digid.client_settings)?;
    let brp_client = HttpBrpClient::new(settings.brp_server);
    let recovery_code_secret_key = SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?;
    let digid_client_id = settings.digid.client_id;
    let bsn_privkey = settings.digid.bsn_privkey;

    let callback_base_url = settings
        .authorizing_issuer_settings
        .issuer_settings
        .public_url
        .as_base_url()
        .clone();

    let (issuer, database_checkers, _, server_settings) = settings
        .authorizing_issuer_settings
        .into_authorizing_issuer(hsm, |store_connection| {
            UpstreamOidcAuthorizationCodeFlow::try_new(
                brp_client,
                &bsn_privkey,
                digid_client_id,
                digid_metadata_cache,
                recovery_code_secret_key,
                store_connection,
                &callback_base_url,
            )
        })
        .await?;

    let authorizing_issuer = Arc::new(issuer);

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(authorizing_issuer, server_settings, serve_status_lists, health_checkers).await
}
