use std::sync::Arc;

use nutype::nutype;
use tracing::info;
use url::Url;

use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::oidc::OidcClient;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::DisclosureProposalPresentation;
use crate::Wallet;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::wallet::DisclosureError;
use crate::wallet::Session;

pub enum CloseProximityDisclosureUpdate {
    Connecting,
    Connected,
    DeviceRequestReceived,
    Disconnected,
}

pub type CloseProximityDisclosureCallback = Box<dyn Fn(CloseProximityDisclosureUpdate) + Send + Sync>;

#[nutype(validate(predicate = |s| s.parse::<Url>().is_ok_and(|u| u.scheme() == "mdoc")), derive(Debug, Clone, TryFrom, FromStr, AsRef, Into, Display))]
pub struct MdocUri(String);

impl<CR, UR, S, AKH, APC, OC, IS, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, OC, IS, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    OC: OidcClient,
    DCC: DisclosureClient,
    CPC: CloseProximityDisclosureClient,
    S: Storage,
{
    pub async fn start_close_proximity_disclosure(&mut self) -> Result<MdocUri, DisclosureError> {
        info!("Starting close proximity disclosure");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(DisclosureError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(DisclosureError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DisclosureError::Locked);
        }

        info!("Checking if there is already an active session");
        if self.session.is_some() {
            return Err(DisclosureError::SessionState);
        }

        let (qr, _receiver) = CPC::start_qr_handover().await?;

        // TODO actually listen on receiver (PVW-5624)
        self.session.replace(Session::CloseProximityDisclosure);

        let uri = format!("mdoc:{}", qr)
            .parse()
            .expect("should always parse as an MdocUri");

        Ok(uri)
    }

    pub async fn continue_close_proximity_disclosure(
        &mut self,
    ) -> Result<DisclosureProposalPresentation, DisclosureError> {
        unimplemented!()
    }

    pub fn set_close_proximity_disclosure_callback(
        &mut self,
        callback: CloseProximityDisclosureCallback,
    ) -> Option<CloseProximityDisclosureCallback> {
        self.close_proximity_disclosure_callback.replace(callback)
    }

    pub fn clear_close_proximity_disclosure_callback(&mut self) {
        self.close_proximity_disclosure_callback.take();
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::Session;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;

    #[tokio::test]
    async fn test_wallet_start_close_proximity_disclosure() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Starting disclosure should return a QR code.
        let qr = wallet
            .start_close_proximity_disclosure()
            .await
            .expect("starting proximity disclosure should succeed");

        assert_eq!(qr.as_ref(), "mdoc:some_qr_code");
        assert!(matches!(wallet.session.take(), Some(Session::CloseProximityDisclosure)))
    }
}
