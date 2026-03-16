use std::marker::PhantomData;
use std::sync::Arc;

use nutype::nutype;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tracing::info;
use url::Url;

use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::oidc::OidcClient;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureError;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::DisclosureProposalPresentation;
use crate::Wallet;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::wallet::DisclosureError;
use crate::wallet::Session;

#[derive(Debug)]
pub enum CloseProximityDisclosureUpdate {
    Connecting,
    Connected,
    DeviceRequestReceived,
    Disconnected,
}

pub type CloseProximityDisclosureCallback = Box<dyn Fn(CloseProximityDisclosureUpdate) + Send + Sync>;

#[nutype(validate(predicate = |s| s.parse::<Url>().is_ok_and(|u| u.scheme() == "mdoc")), derive(Debug, Clone, TryFrom, FromStr, AsRef, Into, Display))]
pub struct MdocUri(String);

#[derive(Debug)]
pub struct CloseProximityDisclosureSessionData {
    pub session_transcript: Vec<u8>,
    pub device_request: Vec<u8>,
}

pub struct CloseProximityDisclosureManager<CPC> {
    callback: Arc<Mutex<Option<CloseProximityDisclosureCallback>>>,
    session_data: Arc<Mutex<Option<CloseProximityDisclosureSessionData>>>,
    _phantom: PhantomData<CPC>,
}

impl<CPC> CloseProximityDisclosureManager<CPC> {
    pub fn init() -> Self {
        Self {
            callback: Arc::default(),
            session_data: Arc::default(),
            _phantom: PhantomData,
        }
    }

    pub fn set_callback(&self, callback: CloseProximityDisclosureCallback) -> Option<CloseProximityDisclosureCallback> {
        self.callback.lock().replace(callback)
    }

    pub fn clear_callback(&self) -> Option<CloseProximityDisclosureCallback> {
        self.callback.lock().take()
    }
}

impl<CPC: CloseProximityDisclosureClient> CloseProximityDisclosureManager<CPC> {
    pub async fn start_session(&mut self) -> Result<String, CloseProximityDisclosureError> {
        let (qr, receiver) = CPC::start_qr_handover().await?;

        // Reset session data
        self.session_data = Arc::default();
        spawn_listener(receiver, Arc::clone(&self.session_data), Arc::clone(&self.callback));

        Ok(qr)
    }
}

fn spawn_listener(
    mut receiver: mpsc::Receiver<PlatformUpdate>,
    session_data: Arc<Mutex<Option<CloseProximityDisclosureSessionData>>>,
    callback: Arc<Mutex<Option<CloseProximityDisclosureCallback>>>,
) {
    tokio::spawn(async move {
        while let Some(update) = receiver.recv().await {
            let wallet_update = match update {
                PlatformUpdate::Connecting => CloseProximityDisclosureUpdate::Connecting,
                PlatformUpdate::Connected => CloseProximityDisclosureUpdate::Connected,
                PlatformUpdate::SessionEstablished {
                    session_transcript,
                    device_request,
                } => {
                    session_data.lock().replace(CloseProximityDisclosureSessionData {
                        session_transcript,
                        device_request,
                    });

                    CloseProximityDisclosureUpdate::DeviceRequestReceived
                }
                PlatformUpdate::Closed => CloseProximityDisclosureUpdate::Disconnected,
                // TODO process error (PVW-5710)
                PlatformUpdate::Error { .. } => CloseProximityDisclosureUpdate::Disconnected,
            };

            info!("Close proximity disclosure update: {wallet_update:?}");
            if let Some(callback) = callback.lock().as_ref() {
                callback(wallet_update);
            }
        }
    });
}

// ── Wallet methods ────────────────────────────────────────────────────────────

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

        let qr = self.close_proximity_disclosure.start_session().await?;
        self.session.replace(Session::CloseProximityDisclosure);

        let uri = format!("mdoc:{qr}").parse().expect("should always parse as an MdocUri");

        Ok(uri)
    }

    pub async fn continue_close_proximity_disclosure(
        &mut self,
    ) -> Result<DisclosureProposalPresentation, DisclosureError> {
        unimplemented!()
    }
}

impl<CR, UR, S, AKH, APC, OC, IS, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, OC, IS, DCC, CPC, SLC>
where
    AKH: AttestedKeyHolder,
    OC: OidcClient,
    DCC: DisclosureClient,
{
    pub fn set_close_proximity_disclosure_callback(
        &mut self,
        callback: CloseProximityDisclosureCallback,
    ) -> Option<CloseProximityDisclosureCallback> {
        self.close_proximity_disclosure.set_callback(callback)
    }

    pub fn clear_close_proximity_disclosure_callback(&mut self) {
        self.close_proximity_disclosure.clear_callback();
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::wallet::Session;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;

    use super::CloseProximityDisclosureUpdate;

    #[tokio::test]
    async fn test_wallet_start_close_proximity_disclosure() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let qr = wallet
            .start_close_proximity_disclosure()
            .await
            .expect("starting proximity disclosure should succeed");

        assert_eq!(qr.as_ref(), "mdoc:some_qr_code");
        assert!(matches!(wallet.session.take(), Some(Session::CloseProximityDisclosure)));
    }

    #[tokio::test]
    async fn test_wallet_close_proximity_disclosure_callback_updates() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<CloseProximityDisclosureUpdate>();
        wallet.set_close_proximity_disclosure_callback(Box::new(move |update| {
            let _ = tx.send(update);
        }));

        wallet
            .start_close_proximity_disclosure()
            .await
            .expect("starting proximity disclosure should succeed");

        // Matching the mock close proximity disclosure updates.
        let update = rx.recv().await.expect("should receive Connecting update");
        assert_matches!(update, CloseProximityDisclosureUpdate::Connecting);

        let update = rx.recv().await.expect("should receive Connected update");
        assert_matches!(update, CloseProximityDisclosureUpdate::Connected);

        let update = rx.recv().await.expect("should receive DeviceRequestReceived update");
        assert_matches!(update, CloseProximityDisclosureUpdate::DeviceRequestReceived);

        let data = wallet.close_proximity_disclosure.session_data.lock();
        assert!(data.is_some());

        let update = rx.recv().await.expect("should receive Disconnected update");
        assert_matches!(update, CloseProximityDisclosureUpdate::Disconnected);
    }
}
