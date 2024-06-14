use anyhow::Result;

use nl_wallet_mdoc::server_state::SessionStore;
use openid4vc::issuer::AttributeService;

use super::*;
use crate::{issuer::create_issuance_router, settings::Settings};

pub async fn serve<A, IS>(attr_service: A, settings: Settings, issuance_sessions: IS) -> Result<()>
where
    A: AttributeService + Send + Sync + 'static,
    IS: SessionStore<openid4vc::issuer::IssuanceData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_issuance_router =
        create_issuance_router(&settings.urls, settings.issuer, issuance_sessions, attr_service)?;

    listen(
        settings.wallet_server,
        None,
        decorate_router("/issuance/", wallet_issuance_router, log_requests),
        None,
    )
    .await
}
