use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use issuer_common::par_store::IssuerParStore;
use issuer_common::pkce_store::IssuerPkceStore;
use openid4vc::issuer::WiaConfig;
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
    let serve_status_lists = settings.issuer_settings.status_lists.serve;

    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let wia_config = WiaConfig {
        wia_trust_anchors: settings.wia_trust_anchors,
    };

    let digid_metadata_cache = Arc::new(DigidMetadataCache::try_new(settings.digid.client_settings)?);
    let brp_client = HttpBrpClient::new(settings.brp_server);
    let recovery_code_secret_key = SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?;
    let digid_client_id = settings.digid.client_id;
    let bsn_privkey = settings.digid.bsn_privkey;

    let (issuer, database_checkers, _, server_settings) = settings
        .issuer_settings
        .into_authorizing_issuer(hsm, Some(wia_config), IssuerParStore::new, |store_connection| {
            UpstreamOidcAuthorizationCodeFlow::try_new(
                brp_client,
                &bsn_privkey,
                digid_client_id,
                digid_metadata_cache,
                recovery_code_secret_key,
                Arc::new(IssuerPkceStore::new(store_connection)),
            )
        })
        .await?;

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(issuer, server_settings, serve_status_lists, health_checkers).await
}
