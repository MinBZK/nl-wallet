use anyhow::Result;

use nl_wallet_mdoc::server_state::SessionStore;
use openid4vc::{issuer::AttributeService, verifier::DisclosureData};

use super::*;
use crate::{issuer::create_issuance_router, settings::Settings, verifier};

pub async fn serve<A, DS, IS>(
    attr_service: A,
    settings: Settings,
    disclosure_sessions: DS,
    issuance_sessions: IS,
) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    DS: SessionStore<DisclosureData> + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_issuance_router =
        create_issuance_router(&settings.urls, settings.issuer, issuance_sessions, attr_service)?;
    let (wallet_disclosure_router, requester_router) =
        verifier::create_routers(settings.urls, settings.verifier, disclosure_sessions)?;

    listen(
        settings.wallet_server,
        settings.requester_server,
        Router::new()
            .nest("/issuance", wallet_issuance_router)
            .nest("/disclosure", wallet_disclosure_router),
        Router::new().nest("/disclosure/sessions", requester_router),
        log_requests,
    )
    .await
}
