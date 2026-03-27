use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::Utc;
use derive_more::IsVariant;
use nutype::nutype;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;
use url::Url;

use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_event::EventStatus;
use mdoc::DeviceRequest;
use openid4vc::disclosure_session::DataDisclosed;
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

#[derive(Debug, Clone, Copy)]
pub enum CloseProximityDisclosureUpdate {
    Connecting,
    Connected,
    DeviceRequestReceived,
    Disconnected,
}

type CloseProximityDisclosureCallbackFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub type CloseProximityDisclosureCallback =
    Box<dyn Fn(CloseProximityDisclosureUpdate) -> CloseProximityDisclosureCallbackFuture + Send + Sync>;

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
        reader_certificate: Box<BorrowingCertificate>,
        attestations: WalletDisclosureAttestations,
    },
}

#[derive(Debug)]
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
                    let mut current_state = session_state.lock();
                    if matches!(*current_state, CloseProximityDisclosureSessionState::Advertising) {
                        *current_state = CloseProximityDisclosureSessionState::SessionEstablished {
                            session_transcript,
                            device_request,
                        };
                        CloseProximityDisclosureUpdate::DeviceRequestReceived
                    } else {
                        // we only support a single SessionEstablished update
                        error!("Received SessionEstablished update while not in Advertising state");
                        continue;
                    }
                }
                PlatformUpdate::Closed => CloseProximityDisclosureUpdate::Disconnected,
                // TODO process error (PVW-5710)
                PlatformUpdate::Error { .. } => CloseProximityDisclosureUpdate::Disconnected,
            };

            info!("Close proximity disclosure update: {wallet_update:?}");
            callback(wallet_update).await;
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

    pub async fn terminate_close_proximity_disclosure_session(
        &mut self,
        session: CloseProximityDisclosureSession,
    ) -> Result<(), DisclosureError> {
        // First abort the listener, s.t. no more events are passed along
        session.listener.abort();

        CPC::stop_ble_server().await?;

        let state = session.session_state.lock().to_owned();
        // Only store the event if the session is past SessionEstablished state (i.e. DisclosureProposed or later)
        if let CloseProximityDisclosureSessionState::DisclosureProposed { reader_certificate, .. } = state {
            self.store_disclosure_event(
                Utc::now(),
                // TODO (PVW-5078): Store credential requests in disclosure event.
                None,
                *reader_certificate,
                DisclosureType::Regular,
                EventStatus::Cancelled,
                DataDisclosed::NotDisclosed,
            )
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use mockall::predicate::always;
    use mockall::predicate::eq;
    use parking_lot::Mutex;
    use serial_test::serial;

    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::disclosure_type::DisclosureType;
    use attestation_data::x509::generate::mock::generate_reader_mock_with_registration;
    use crypto::server_keys::generate::Ca;
    use entity::disclosure_event::EventStatus;
    use mdoc::DeviceRequest;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannel;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannelImpl;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
    use platform_support::close_proximity_disclosure::MockCloseProximityDisclosureClient;

    use crate::wallet::Session;
    use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureSession;
    use crate::wallet::disclosure::WalletDisclosureAttestations;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;

    use super::CloseProximityDisclosureSessionState;
    use super::CloseProximityDisclosureUpdate;

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_start_close_proximity_disclosure() {
        let context = MockCloseProximityDisclosureClient::start_qr_handover_context();
        context.expect().once().returning(|| {
            let (_channel, receiver) = CloseProximityDisclosureChannelImpl::new();
            Ok(("some_qr_code".to_owned(), receiver))
        });

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let qr = wallet
            .start_close_proximity_disclosure(Box::new(|_| Box::pin(async {})))
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
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_close_proximity_disclosure_callback_updates() {
        let context = MockCloseProximityDisclosureClient::start_qr_handover_context();
        context.expect().once().returning(|| {
            let (channel, receiver) = CloseProximityDisclosureChannelImpl::new();
            tokio::spawn(async move {
                let _ = channel
                    .send_update(PlatformUpdate::SessionEstablished {
                        session_transcript: vec![0x01, 0x02, 0x03],
                        device_request: vec![0x04, 0x05, 0x06],
                    })
                    .await;
            });

            Ok(("some_qr_code".to_owned(), receiver))
        });

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<CloseProximityDisclosureUpdate>();
        wallet
            .start_close_proximity_disclosure(Box::new(move |update| {
                let _ = tx.send(update);
                Box::pin(async {})
            }))
            .await
            .expect("starting proximity disclosure should succeed");

        // Matching the mock close proximity disclosure updates.
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
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_terminate_close_proximity_disclosure_session_advertising() {
        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // When the session is still in the Advertising state (i.e. no reader has connected yet),
        // no disclosure event should be stored.
        wallet.mut_storage().expect_log_disclosure_event().never();

        let session = CloseProximityDisclosureSession {
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::Advertising)),
        };

        wallet
            .terminate_close_proximity_disclosure_session(session)
            .await
            .expect("terminating close proximity disclosure session should succeed");
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_terminate_close_proximity_disclosure_session_session_established() {
        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // When the session is in the SessionEstablished state (a reader connected but disclosure
        // was not yet proposed), no disclosure event should be stored.
        wallet.mut_storage().expect_log_disclosure_event().never();

        let session = CloseProximityDisclosureSession {
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::SessionEstablished {
                session_transcript: vec![0x01, 0x02, 0x03],
                device_request: vec![0x04, 0x05, 0x06],
            })),
        };

        wallet
            .terminate_close_proximity_disclosure_session(session)
            .await
            .expect("terminating close proximity disclosure session should succeed");
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_terminate_close_proximity_disclosure_session_disclosure_proposed() {
        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let ca = Ca::generate_reader_mock_ca().unwrap();
        let key_pair = generate_reader_mock_with_registration(&ca, ReaderRegistration::new_mock()).unwrap();
        let reader_certificate = key_pair.certificate().clone();

        // When the session is in the DisclosureProposed state (the reader sent a device request),
        // a Cancelled event should be stored with the reader certificate and no disclosed data.
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate.clone()),
                eq(EventStatus::Cancelled),
                eq(DisclosureType::Regular),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let session = CloseProximityDisclosureSession {
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: vec![0x01, 0x02, 0x03],
                device_request: DeviceRequest::default(),
                reader_certificate: Box::new(reader_certificate),
                attestations: WalletDisclosureAttestations::Missing,
            })),
        };

        wallet
            .terminate_close_proximity_disclosure_session(session)
            .await
            .expect("terminating close proximity disclosure session should succeed");
    }
}
