use anyhow::Result;

use openid4vc::{server_state::SessionStore, verifier::DisclosureData};

use super::*;
use crate::{settings::Settings, verifier};

pub async fn serve<S>(settings: Settings, disclosure_sessions: S) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let (wallet_disclosure_router, requester_router) =
        verifier::create_routers(settings.urls, settings.verifier, disclosure_sessions)?;

    listen(
        settings.wallet_server,
        settings.requester_server,
        Router::new().nest("/disclosure", wallet_disclosure_router),
        Router::new().nest("/disclosure", requester_router),
        log_requests,
    )
    .await
}
