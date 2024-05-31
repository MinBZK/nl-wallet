use anyhow::Result;
use axum::Router;

use nl_wallet_mdoc::{server_state::SessionStore, verifier::DisclosureData};

use super::*;

pub(crate) fn setup_disclosure<S>(settings: Settings, disclosure_sessions: S) -> Result<(Router, Router)>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let (wallet_disclosure_router, mut requester_router) =
        crate::verifier::create_routers(settings.clone(), disclosure_sessions)?;

    requester_router = secure_router(&settings, requester_router);

    Ok((wallet_disclosure_router, requester_router))
}
