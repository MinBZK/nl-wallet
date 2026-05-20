use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use issuer_common::par_store::IssuerParStore;
use issuer_common::pkce_store::IssuerPkceStore;
use openid4vc::issuer::WiaConfig;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
use pid_issuer::pid::digid::DigidAuthorizationAdapter;
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
    let upstream_authorization_adapter =
        DigidAuthorizationAdapter::new(Arc::clone(&digid_metadata_cache), settings.digid.client_id.clone());

    let pid_attr_service = BrpPidAttributeService::try_new(
        HttpBrpClient::new(settings.brp_server),
        &settings.digid.bsn_privkey,
        settings.digid.client_id,
        digid_metadata_cache,
        SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?,
    )?;

    let (issuer, database_checkers, _, server_settings) = settings
        .issuer_settings
        .into_authorizing_issuer(
            hsm,
            Some(wia_config),
            pid_attr_service,
            IssuerParStore::new,
            IssuerPkceStore::new,
            upstream_authorization_adapter,
        )
        .await?;

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(issuer, server_settings, serve_status_lists, health_checkers).await
}
