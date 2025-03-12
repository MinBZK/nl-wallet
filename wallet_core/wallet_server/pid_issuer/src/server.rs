use anyhow::Result;
use axum::Router;
use tracing::info;

use hsm::service::Pkcs11Hsm;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::WalletSettings;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::WteTracker;
use openid4vc_server::issuer::create_issuance_router;
use server_utils::server::create_wallet_listener;
use server_utils::server::decorate_router;
use server_utils::settings::Server;
use wallet_common::built_info::version_string;

use crate::settings::IssuerSettings;

pub async fn serve<A, IS, W>(
    attr_service: A,
    settings: IssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: IS,
    wte_tracker: W,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
    W: WteTracker + Send + Sync + 'static,
{
    let log_requests = settings.server_settings.log_requests;

    let type_metadata = settings.metadata();
    let attestation_config = settings.attestation_settings.parse(&hsm, &type_metadata).await?;

    let wallet_issuance_router = create_issuance_router(
        &settings.server_settings.public_url,
        attestation_config,
        issuance_sessions,
        attr_service,
        WalletSettings {
            wallet_client_ids: settings.wallet_client_ids,
            wte_issuer_pubkey: settings.wte_issuer_pubkey.into_inner(),
            wte_tracker,
        },
    );

    listen(
        settings.server_settings.wallet_server,
        Router::new().nest("/issuance", wallet_issuance_router),
        log_requests,
    )
    .await
}

async fn listen(wallet_server: Server, mut wallet_router: Router, log_requests: bool) -> Result<()> {
    wallet_router = decorate_router(wallet_router, log_requests);

    let wallet_listener = create_wallet_listener(&wallet_server).await?;

    info!("{}", version_string());

    info!("listening for wallet on {}", wallet_listener.local_addr()?);
    axum::serve(wallet_listener, wallet_router)
        .await
        .expect("wallet server should be started");

    Ok(())
}
