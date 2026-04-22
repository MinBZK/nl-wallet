use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKeyHolder;

use super::Session;
use super::Wallet;
use super::issuance::WalletIssuanceSession;
use super::pin_recovery::PinRecoverySession;

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    pub fn current_oauth_state(&self) -> Option<&str> {
        match &self.session {
            Some(Session::Issuance(WalletIssuanceSession::OAuth {
                authorization_session, ..
            })) => Some(authorization_session.state()),
            Some(Session::PinRecovery(PinRecoverySession::OAuth { authorization_session })) => {
                Some(authorization_session.state())
            }
            _ => None,
        }
    }
}
