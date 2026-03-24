use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use derive_more::IsVariant;
use futures::future::try_join_all;
use nutype::nutype;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;
use url::Url;

use attestation_data::auth::reader_auth::ValidationError;
use attestation_data::disclosure_type::DisclosureType;
use attestation_data::verifier_certificate::VerifierCertificate;
use attestation_data::x509::CertificateTypeError;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_event::EventStatus;
use error_category::ErrorCategory;
use mdoc::DeviceRequest;
use mdoc::SessionTranscript;
use mdoc::utils::serialization::CborError;
use openid4vc::disclosure_session::DataDisclosed;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::oidc::OidcClient;
use openid4vc::verifier::SessionType;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
use rustls_pki_types::TrustAnchor;
use update_policy_model::update_policy::VersionState;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::AttributesNotAvailable;
use crate::DisclosureProposalPresentation;
use crate::Wallet;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::wallet::DisclosureError;
use crate::wallet::Session;
use crate::wallet::disclosure::RedirectUriPurpose;
use crate::wallet::disclosure::WalletDisclosureAttestations;
use crate::wallet::disclosure::candidates_to_attestation_options;
use crate::wallet::disclosure::is_request_for_recovery_code;
use crate::wallet::disclosure::requested_attribute_paths;

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
        session_transcript: Box<SessionTranscript>,
        device_request: DeviceRequest,
        reader_certificate: Box<BorrowingCertificate>,
        attestations: WalletDisclosureAttestations<usize>,
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

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum CloseProximityDisclosureError {
    #[error("Device Request does not have any doc requests")]
    #[category(critical)]
    EmptyRequest,

    #[error("Device Request has no attributes")]
    #[category(critical)]
    NoAttributesRequested,

    #[error("Device Request has no ReaderAuth")]
    #[category(critical)]
    MissingReaderAuth,

    #[error("ReaderAuths are not all identical")]
    #[category(critical)]
    InconsistentReaderAuths,

    #[error("Invalid DocRequest")]
    InvalidDocRequest(#[from] mdoc::Error),

    #[error("Missing ReaderRegistration in certificate")]
    #[category(critical)]
    MissingReaderRegistration,

    #[error("Invalid certificate type: {0}")]
    InvalidCertificateType(#[from] CertificateTypeError),

    #[error("Requested unregistered attributes: {0}")]
    #[category(pd)]
    RequestedUnregisteredAttributes(#[from] ValidationError),

    #[error("Received invalid CBOR from reader: {0}")]
    #[category(critical)]
    InvalidCbor(#[from] CborError),
}

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

        self.check_disclosure_preconditions()?;

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

    pub async fn continue_close_proximity_disclosure(
        &mut self,
    ) -> Result<DisclosureProposalPresentation, DisclosureError> {
        info!("Continuing close proximity disclosure");

        self.check_disclosure_preconditions()?;

        info!("Checking if there is an active close prosession");
        let Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession { session_state, .. })) =
            self.session.as_ref()
        else {
            return Err(DisclosureError::SessionState);
        };

        let CloseProximityDisclosureSessionState::SessionEstablished {
            device_request,
            session_transcript,
        } = session_state.lock().to_owned()
        else {
            return Err(DisclosureError::SessionState);
        };

        let wallet_config = &self.config_repository.get();

        // TODO send error to reader (PVW-5710)
        let device_request =
            DeviceRequest::try_from_bytes(&device_request).map_err(CloseProximityDisclosureError::InvalidCbor)?;
        let session_transcript = SessionTranscript::try_from_bytes(&session_transcript)
            .map_err(CloseProximityDisclosureError::InvalidCbor)?;

        let verifier_certificate = Self::verify_device_request(
            &device_request,
            &session_transcript,
            &TimeGenerator,
            &wallet_config.disclosure.rp_trust_anchors(),
        )?;

        // Check for recovery code request
        if device_request
            .items_requests()
            .any(|request| is_request_for_recovery_code(request.clone(), &wallet_config.pid_attributes))
        {
            return Err(DisclosureError::RecoveryCodeRequested);
        }

        // For each disclosure request, fetch the candidates from the database and convert
        // each of them to an `AttestationPresentation` that can be shown to the user.
        let storage = self.storage.read().await;
        let candidate_attestations = try_join_all(
            device_request
                .items_requests()
                .map(|request| Self::fetch_candidate_attestations(&*storage, request, &wallet_config.pid_attributes)),
        )
        .await
        .map_err(DisclosureError::AttestationRetrieval)?
        .into_iter()
        .flatten() // remove entries for which no suitable candidates were found
        .collect::<Vec<_>>();

        let shared_data_with_relying_party_before = self
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(verifier_certificate.certificate())
            .await
            .map_err(DisclosureError::HistoryRetrieval)?;

        let (reader_certificate, reader_registration) = verifier_certificate.into_certificate_and_registration();
        let session_type = SessionType::CrossDevice; // all close proximity disclosure sessions are cross-device
        let disclosure_type = DisclosureType::Regular; // all close proximity disclosure sessions are regular
        let purpose = RedirectUriPurpose::Browser; // irrelevant for close proximity disclosure sessions

        // If no suitable candidates were found for at least one of the requests, report this as an error to the UI.
        if let Ok(candidate_attestations) = VecNonEmpty::try_from(candidate_attestations)
            && candidate_attestations.len() == device_request.doc_requests.len()
        {
            info!(
                "All attributes in the disclosure request are present in the database, return a proposal to the user"
            );

            // Place the proposed attestations in a `DisclosureProposalPresentation`,
            // along with a copy of the `ReaderRegistration`.
            let attestation_options = candidate_attestations
                .nonempty_iter()
                .map(candidates_to_attestation_options)
                .collect();

            let proposal = DisclosureProposalPresentation {
                attestation_options,
                reader_registration,
                shared_data_with_relying_party_before,
                session_type,
                disclosure_type,
                purpose,
            };

            *session_state.lock() = CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(session_transcript),
                reader_certificate: Box::new(reader_certificate),
                device_request,
                attestations: WalletDisclosureAttestations::Proposal(
                    candidate_attestations.into_iter().enumerate().collect(),
                ),
            };

            return Ok(proposal);
        }

        info!("At least one attribute from one attestation is missing in order to satisfy the disclosure request");

        // For now we simply represent the requested attribute paths by joining all elements with a slash.
        // TODO (PVW-3813): Attempt to translate the requested attributes using the TAS cache.
        let requested_attributes = requested_attribute_paths(device_request.items_requests()).collect();

        // Store the session so that it will only be terminated on user interaction.
        // This prevents gleaning of missing attributes by a verifier.
        *session_state.lock() = CloseProximityDisclosureSessionState::DisclosureProposed {
            session_transcript: Box::new(session_transcript),
            device_request,
            reader_certificate: Box::new(reader_certificate),
            attestations: WalletDisclosureAttestations::Missing,
        };

        Err(DisclosureError::AttributesNotAvailable(AttributesNotAvailable {
            reader_registration: Box::new(reader_registration),
            requested_attributes,
            shared_data_with_relying_party_before,
            session_type,
        }))
    }

    /// Verify device request and reader authentication.
    /// Note that since each DocRequest carries its own reader authentication, the spec allows the DocRequests to be
    /// signed by distinct readers. For now, this function requires all of the DocRequests to be signed by the same
    /// reader.
    pub fn verify_device_request(
        device_request: &DeviceRequest,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<VerifierCertificate, CloseProximityDisclosureError> {
        // A device request without any attributes is useless, so return an error.
        if !device_request.has_attributes() {
            return Err(CloseProximityDisclosureError::NoAttributesRequested);
        }

        // Verify all `DocRequest` entries and make sure the resulting certificates are all exactly equal.
        let certificate = device_request
            .doc_requests
            .iter()
            .try_fold(None, {
                |result_cert, doc_request| -> Result<_, _> {
                    // `DocRequest::verify()` will return `None` if `reader_auth` is absent
                    let doc_request_cert = doc_request
                        .verify(session_transcript, time, trust_anchors)?
                        .ok_or(CloseProximityDisclosureError::MissingReaderAuth)?;

                    // If there is a certificate from a previous iteration, compare our certificate to that.
                    if let Some(result_cert) = result_cert
                        && doc_request_cert != result_cert
                    {
                        return Err(CloseProximityDisclosureError::InconsistentReaderAuths);
                    }

                    Ok(doc_request_cert.into())
                }
            })?
            .unwrap(); // the try_fold either returns an error or return Some(certificate)

        // Extract `ReaderRegistration` from the certificate.
        let verifier_certificate = VerifierCertificate::try_new(certificate)?
            .ok_or(CloseProximityDisclosureError::MissingReaderRegistration)?;

        // Verify that the requested attributes are included in the reader authentication.
        verifier_certificate
            .registration()
            .verify_requested_attributes(device_request.items_requests())?;

        Ok(verifier_certificate)
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
    use mdoc::Handover;
    use mdoc::SessionTranscriptKeyed;
    use mdoc::examples::Example;
    use mdoc::utils::serialization::CborSeq;
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
                session_transcript: Box::new(CborSeq(SessionTranscriptKeyed {
                    device_engagement_bytes: None,
                    ereader_key_bytes: None,
                    handover: Handover::QrHandover,
                })),
                device_request: DeviceRequest::example(),
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
