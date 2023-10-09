use tracing::info;
use url::Url;

use crate::digid::DigidSession;

use super::Wallet;

pub enum RedirectUriType {
    PidIssuance,
    Unknown,
}

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P>
where
    D: DigidSession,
{
    pub fn identify_redirect_uri(&self, redirect_uri: &Url) -> RedirectUriType {
        info!("Idetifying type of URI: {}", redirect_uri);

        if self
            .digid_session
            .as_ref()
            .map(|session| session.matches_received_redirect_uri(redirect_uri))
            .unwrap_or_default()
        {
            return RedirectUriType::PidIssuance;
        }

        RedirectUriType::Unknown
    }
}
