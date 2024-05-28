use anyhow::Result;
use tracing::debug;

use nl_wallet_mdoc::server_state::SessionStore;
use openid4vc::issuer::AttributeService;
use wallet_server::{
    issuer,
    pid::{attributes::BrpPidAttributeService, brp::client::HttpBrpClient},
    server,
    settings::Settings,
    store::SessionStores,
};

pub async fn serve_pid_issuer<A, IS>(attr_service: A, settings: Settings, issuance_sessions: IS) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_socket = server::create_wallet_socket(&settings);
    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings, issuance_sessions, attr_service).await?;
    let wallet_router = server::decorate_router("/issuance/", wallet_issuance_router, log_requests);

    debug!("listening for wallet and requester on {}", wallet_socket);
    axum::Server::bind(&wallet_socket)
        .serve(wallet_router.into_make_service())
        .await
        .expect("wallet server should be started");

    Ok(())
}

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new_custom("pid_issuer.toml", "pid_issuer")?;

    // Retain [`ClientInitGuard`]
    let _guard = settings
        .sentry
        .as_ref()
        .map(|sentry| sentry.init(sentry::release_name!()));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { async_main(settings).await })
}

async fn async_main(settings: Settings) -> Result<()> {
    let storage_settings = &settings.storage;
    let sessions = SessionStores::init(storage_settings.url.clone(), storage_settings.into()).await?;

    // This will block until the server shuts down.
    serve_pid_issuer(
        BrpPidAttributeService::new(
            HttpBrpClient::new(settings.issuer.brp_server.clone()),
            settings.issuer.digid.issuer_url.clone(),
            settings.issuer.digid.bsn_privkey.clone(),
            settings.issuer.digid.trust_anchors.clone(),
            settings.issuer.certificates(),
        )?,
        settings,
        sessions.issuance,
    )
    .await?;

    Ok(())
}
