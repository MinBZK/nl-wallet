use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use openid4vc::issuer::WiaConfig;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
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
        wia_issuer_pubkey: (&settings.wua_issuer_pubkey.into_inner()).into(),
    };
    let upstream_oauth_identifier = settings.digid.client_settings.oidc_identifier.clone();

    let pid_attr_service = BrpPidAttributeService::try_new(
        HttpBrpClient::new(settings.brp_server),
        &settings.digid.bsn_privkey,
        settings.digid.client_id,
        settings.digid.client_settings,
        SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?,
    )?;

    let (issuer, database_checkers, _, server_settings) = settings
        .issuer_settings
        .into_issuer(hsm, Some(wia_config), Some(upstream_oauth_identifier), pid_attr_service)
        .await?;

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(issuer, server_settings, serve_status_lists, health_checkers).await
}
