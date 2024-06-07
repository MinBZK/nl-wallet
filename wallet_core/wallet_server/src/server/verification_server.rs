use anyhow::Result;

use nl_wallet_mdoc::{server_state::SessionStore, verifier::DisclosureData};

use super::{disclosure::setup_disclosure, *};
use crate::settings::Settings;

pub async fn serve<S>(settings: Settings, disclosure_sessions: S) -> Result<()>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let log_requests = settings.log_requests;

    let wallet_socket = create_wallet_socket(&settings);
    let requester_socket = create_requester_socket(&settings);
    let (wallet_disclosure_router, requester_router) = setup_disclosure(&settings, disclosure_sessions)?;

    listen(
        wallet_socket,
        requester_socket,
        decorate_router("/disclosure/", wallet_disclosure_router, log_requests),
        decorate_router("/disclosure/sessions", requester_router, log_requests),
    )
    .await?;

    Ok(())
}
