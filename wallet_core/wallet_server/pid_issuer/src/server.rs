use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use p256::ecdsa::VerifyingKey;
use tokio::net::TcpListener;

use hsm::service::Pkcs11Hsm;
use issuer_settings::settings::IssuerSettings;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::Issuer;
use openid4vc::issuer::WuaConfig;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::WuaTracker;
use openid4vc_server::issuer::create_issuance_router;
use server_utils::server::create_wallet_listener;
use server_utils::server::listen;

pub async fn serve<A, IS, W>(
    attr_service: A,
    settings: IssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    wua_issuer_pubkey: VerifyingKey,
    wua_tracker: W,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
    W: WuaTracker + Send + Sync + 'static,
{
    let listener = create_wallet_listener(&settings.server_settings.wallet_server).await?;
    serve_with_listener(
        listener,
        attr_service,
        settings,
        hsm,
        issuance_sessions,
        wua_issuer_pubkey,
        wua_tracker,
    )
    .await
}

#[expect(clippy::too_many_arguments, reason = "Setup function")]
pub async fn serve_with_listener<A, IS, W>(
    listener: TcpListener,
    attr_service: A,
    settings: IssuerSettings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: Arc<IS>,
    wua_issuer_pubkey: VerifyingKey,
    wua_tracker: W,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
    W: WuaTracker + Send + Sync + 'static,
{
    let log_requests = settings.server_settings.log_requests;

    let attestation_config = settings.attestation_settings.parse(&hsm, &settings.metadata).await?;

    let wallet_issuance_router = create_issuance_router(Arc::new(Issuer::new(
        issuance_sessions,
        attr_service,
        attestation_config,
        &settings.server_settings.public_url,
        settings.wallet_client_ids,
        Some(WuaConfig {
            wua_issuer_pubkey: (&wua_issuer_pubkey).into(),
            wua_tracker: Arc::new(wua_tracker),
        }),
    )));

    listen(
        listener,
        Router::new().nest("/issuance", wallet_issuance_router),
        log_requests,
    )
    .await
}
