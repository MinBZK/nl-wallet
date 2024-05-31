use anyhow::Result;

use nl_wallet_mdoc::{server_state::SessionStore, verifier::DisclosureData};
use openid4vc::issuer::AttributeService;

use super::{disclosure::setup_disclosure, *};
use crate::settings::Settings;

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

    let wallet_socket = create_wallet_socket(&settings);
    let requester_socket = create_requester_socket(&settings);
    let (wallet_disclosure_router, requester_router) = setup_disclosure(settings.clone(), disclosure_sessions)?;

    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings, issuance_sessions, attr_service).await?;

    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/issuance/", wallet_issuance_router, log_requests).merge(decorate_router(
            "/disclosure/",
            wallet_disclosure_router,
            log_requests,
        )),
        decorate_router("/disclosure/sessions", requester_router, log_requests),
    )
    .await?;

    Ok(())
}
