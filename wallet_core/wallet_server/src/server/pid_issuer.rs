use anyhow::Result;

use hsm::service::Pkcs11Hsm;
use openid4vc::issuer::AttributeService;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::WteTracker;
use openid4vc_server::issuer::create_issuance_router;
use openid4vc_server::issuer::IssuerKeyRing;

use crate::settings::Settings;
use crate::settings::TryFromKeySettings;

use super::*;

pub async fn serve<A, IS, W>(
    attr_service: A,
    settings: Settings,
    hsm: Option<Pkcs11Hsm>,
    issuance_sessions: IS,
    wte_tracker: W,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
    W: WteTracker + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let private_keys = IssuerKeyRing::try_from_key_settings(settings.issuer.private_keys, hsm).await?;
    let wallet_issuance_router = create_issuance_router(
        &settings.urls,
        private_keys,
        issuance_sessions,
        attr_service,
        settings.issuer.wallet_client_ids,
        settings.issuer.wte_issuer_pubkey.into_inner(),
        wte_tracker,
    );

    listen_wallet_only(
        settings.wallet_server,
        Router::new().nest("/issuance", wallet_issuance_router),
        log_requests,
    )
    .await
}
