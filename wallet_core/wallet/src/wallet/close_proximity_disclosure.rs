use std::sync::Arc;

use derive_more::IsVariant;
use nutype::nutype;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::info;
use url::Url;

use mdoc::DeviceRequest;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::oidc::OidcClient;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::DisclosureProposalPresentation;
use crate::Wallet;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::wallet::DisclosureError;
use crate::wallet::Session;
use crate::wallet::disclosure::WalletDisclosureAttestations;

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

#[derive(Debug, Clone, IsVariant)]
#[expect(
    unused,
    reason = "will be used when continue_close_proximity_disclosure is implemented"
)]
enum CloseProximityDisclosureSessionState {
    Advertising,
    SessionEstablished {
        session_transcript: Vec<u8>,
        device_request: Vec<u8>,
    },
    DisclosureProposed {
        session_transcript: Vec<u8>,
        device_request: DeviceRequest,
        attestations: WalletDisclosureAttestations,
    },
}

#[derive(Debug)]
#[expect(
    unused,
    reason = "will be used when continue_close_proximity_disclosure is implemented"
)]
pub struct CloseProximityDisclosureSession {
    listener: JoinHandle<()>,
    session_state: Arc<Mutex<CloseProximityDisclosureSessionState>>,
}

fn spawn_listener(
    mut receiver: mpsc::Receiver<PlatformUpdate>,
    session_state: Arc<Mutex<CloseProximityDisclosureSessionState>>,
    callback: CloseProximityDisclosureCallback,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(update) = receiver.recv().await {
            let wallet_update = match update {
                PlatformUpdate::Connecting => CloseProximityDisclosureUpdate::Connecting,
                PlatformUpdate::Connected => CloseProximityDisclosureUpdate::Connected,
                PlatformUpdate::SessionEstablished {
                    session_transcript,
                    device_request,
                } => {
                    // TODO only do this if the current state is Advertising
                    *session_state.lock() = CloseProximityDisclosureSessionState::SessionEstablished {
                        session_transcript,
                        device_request,
                    };

                    CloseProximityDisclosureUpdate::DeviceRequestReceived
                }
                PlatformUpdate::Closed => CloseProximityDisclosureUpdate::Disconnected,
                // TODO process error (PVW-5710)
                PlatformUpdate::Error { .. } => CloseProximityDisclosureUpdate::Disconnected,
            };

            info!("Close proximity disclosure update: {wallet_update:?}");
            callback(wallet_update);
        }
    })
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
    pub async fn start_close_proximity_disclosure(
        &mut self,
        callback: CloseProximityDisclosureCallback,
    ) -> Result<MdocUri, DisclosureError> {
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

        let (qr, receiver) = CPC::start_qr_handover().await?;

        let session_state = Arc::new(Mutex::new(CloseProximityDisclosureSessionState::Advertising));

        let listener = spawn_listener(receiver, Arc::clone(&session_state), callback);
        self.session
            .replace(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
                listener,
                session_state,
            }));

        let uri = format!("mdoc:{qr}").parse().expect("should always parse as an MdocUri");

        Ok(uri)
    }

    pub fn continue_close_proximity_disclosure(&mut self) -> Result<DisclosureProposalPresentation, DisclosureError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::wallet::Session;
    use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureSession;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;

    use super::CloseProximityDisclosureUpdate;

    #[tokio::test]
    async fn test_wallet_start_close_proximity_disclosure() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let qr = wallet
            .start_close_proximity_disclosure(Box::new(|_| {}))
            .await
            .expect("starting proximity disclosure should succeed");

        assert_eq!(qr.as_ref(), "mdoc:some_qr_code");
        assert!(matches!(
            wallet.session.take(),
            Some(Session::CloseProximityDisclosure(
                CloseProximityDisclosureSession { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn test_wallet_close_proximity_disclosure_callback_updates() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<CloseProximityDisclosureUpdate>();
        wallet
            .start_close_proximity_disclosure(Box::new(move |update| {
                let _ = tx.send(update);
            }))
            .await
            .expect("starting proximity disclosure should succeed");

        // Matching the mock close proximity disclosure updates.
        let update = rx.recv().await.expect("should receive Connecting update");
        assert_matches!(update, CloseProximityDisclosureUpdate::Connecting);

        let update = rx.recv().await.expect("should receive Connected update");
        assert_matches!(update, CloseProximityDisclosureUpdate::Connected);

        let update = rx.recv().await.expect("should receive DeviceRequestReceived update");
        assert_matches!(update, CloseProximityDisclosureUpdate::DeviceRequestReceived);

        let data = wallet.session.as_ref().and_then(|s| {
            if let Session::CloseProximityDisclosure(session) = s {
                Some(session.session_state.lock())
            } else {
                None
            }
        });
        assert!(data.is_some());

        let update = rx.recv().await.expect("should receive Disconnected update");
        assert_matches!(update, CloseProximityDisclosureUpdate::Disconnected);
    }
}
