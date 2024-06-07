use anyhow::Result;
use tracing::debug;

use nl_wallet_mdoc::server_state::SessionStore;
use openid4vc::issuer::AttributeService;

use super::*;
use crate::settings::Settings;

pub async fn serve<A, IS>(attr_service: A, settings: Settings, issuance_sessions: IS) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_socket = create_wallet_socket(&settings);
    let requester_socket = create_requester_socket(&settings);

    let mut requester_router = Router::new();
    requester_router = secure_router(&settings, requester_router);

    let wallet_issuance_router =
        crate::issuer::create_issuance_router(settings, issuance_sessions, attr_service).await?;

    debug!("listening for wallet and requester on {}", wallet_socket);
    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/issuance/", wallet_issuance_router, log_requests),
        decorate_router("/requester/", requester_router, log_requests), // currently unused
    )
    .await
}
