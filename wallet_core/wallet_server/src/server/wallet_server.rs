use anyhow::Result;

use hsm::service::Pkcs11Hsm;
use openid4vc::issuer::AttributeService;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::WteTracker;
use openid4vc::verifier::DisclosureData;
use wallet_common::trust_anchor::BorrowingTrustAnchor;

use crate::issuer::create_issuance_router;
use crate::issuer::IssuerKeyRing;
use crate::settings::Settings;
use crate::settings::TryFromKeySettings;
use crate::verifier;

use super::*;

pub async fn serve<A, DS, IS, W>(
    attr_service: A,
    settings: Settings,
    hsm: Option<Pkcs11Hsm>,
    disclosure_sessions: DS,
    issuance_sessions: IS,
    wte_tracker: W,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
    W: WteTracker + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let private_keys = IssuerKeyRing::try_from_key_settings(settings.issuer.private_keys, hsm.clone()).await?;
    let wallet_issuance_router = create_issuance_router(
        &settings.urls,
        private_keys,
        issuance_sessions,
        attr_service,
        settings.issuer.wallet_client_ids,
        settings.issuer.wte_issuer_pubkey.into_inner(),
        wte_tracker,
    )?;
    let (wallet_disclosure_router, requester_router) = verifier::create_routers(
        settings.urls,
        settings.verifier,
        settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect(),
        disclosure_sessions,
        hsm,
    )
    .await?;

    listen(
        settings.wallet_server,
        settings.requester_server,
        Router::new()
            .nest("/issuance", wallet_issuance_router)
            .nest("/disclosure", wallet_disclosure_router),
        Router::new().nest("/disclosure", requester_router),
        log_requests,
    )
    .await
}
