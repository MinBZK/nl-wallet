use anyhow::Result;

use hsm::service::Pkcs11Hsm;
use openid4vc::server_state::SessionStore;
use openid4vc::verifier::DisclosureData;
use wallet_common::trust_anchor::BorrowingTrustAnchor;

use crate::settings::Settings;
use crate::verifier;

use super::*;

pub async fn serve<S>(settings: Settings, hsm: Option<Pkcs11Hsm>, disclosure_sessions: S) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

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
        Router::new().nest("/disclosure", wallet_disclosure_router),
        Router::new().nest("/disclosure", requester_router),
        log_requests,
    )
    .await
}
