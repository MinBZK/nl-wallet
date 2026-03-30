use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use derive_more::IsVariant;
use itertools::Itertools;
use nutype::nutype;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;
use tracing::instrument;
use url::Url;

use attestation_data::auth::reader_auth::ValidationError;
use attestation_data::disclosure_type::DisclosureType;
use attestation_data::verifier_certificate::VerifierCertificate;
use attestation_data::x509::CertificateTypeError;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_event::EventStatus;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
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
    #[instrument(skip_all)]
    #[sentry_capture_error]
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

    #[instrument(skip_all)]
    #[sentry_capture_error]
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

        let verifier_certificate = verify_device_request(
            &device_request,
            &session_transcript,
            &TimeGenerator,
            &wallet_config.disclosure.rp_trust_anchors(),
        )?;

        let (candidate_attestations, shared_data_with_relying_party_before) = self
            .prepare_disclosure(
                &device_request.items_requests().collect_vec(),
                &wallet_config.pid_attributes,
                verifier_certificate.certificate(),
            )
            .await?;

        let candidate_attestations = candidate_attestations
            .into_iter()
            .flatten() // remove entries for which no suitable candidates were found
            .collect::<Vec<_>>();

        let (reader_certificate, reader_registration) = verifier_certificate.into_certificate_and_registration();
        let session_type = SessionType::CrossDevice; // all close proximity disclosure sessions are cross-device
        let disclosure_type = DisclosureType::Regular; // all close proximity disclosure sessions are regular
        let purpose = RedirectUriPurpose::Browser; // irrelevant for close proximity disclosure sessions

        if let Ok(candidate_attestations) = VecNonEmpty::try_from(candidate_attestations)
            && candidate_attestations.len() == device_request.doc_requests.len()
        {
            info!(
                "All attributes in the disclosure request are present in the database, return a proposal to the user"
            );

            let proposal = DisclosureProposalPresentation::from_candidates(
                candidate_attestations.clone(),
                reader_registration,
                shared_data_with_relying_party_before,
                session_type,
                disclosure_type,
                purpose,
            );

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

        // If no suitable candidates were found for at least one of the requests, report this as an error to the UI.
        info!("At least one attribute from one attestation is missing in order to satisfy the disclosure request");

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
    let verifier_certificate =
        VerifierCertificate::try_new(certificate)?.ok_or(CloseProximityDisclosureError::MissingReaderRegistration)?;

    // Verify that the requested attributes are included in the reader authentication.
    verifier_certificate
        .registration()
        .verify_requested_attributes(device_request.items_requests())?;

    Ok(verifier_certificate)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use futures::future::join_all;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use mockall::predicate::always;
    use mockall::predicate::eq;
    use p256::ecdsa::SigningKey;
    use parking_lot::Mutex;
    use rand_core::OsRng;
    use rustls_pki_types::TrustAnchor;
    use serial_test::serial;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::disclosure_type::DisclosureType;
    use attestation_data::verifier_certificate::VerifierCertificate;
    use attestation_data::x509::generate::mock::generate_reader_mock_with_registration;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_FAMILY_NAME;
    use attestation_types::pid_constants::PID_GIVEN_NAME;
    use crypto::WithVerifyingKey;
    use crypto::server_keys::generate::Ca;
    use dcql::CredentialFormat;
    use entity::disclosure_event::EventStatus;
    use mdoc::DeviceEngagement;
    use mdoc::DeviceRequest;
    use mdoc::Handover;
    use mdoc::ItemsRequest;
    use mdoc::SessionTranscript;
    use mdoc::SessionTranscriptKeyed;
    use mdoc::examples::Example;
    use mdoc::holder::disclosure::create_doc_request;
    use mdoc::utils::cose::CoseKey;
    use mdoc::utils::serialization::CborSeq;
    use mdoc::utils::serialization::cbor_serialize;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannel;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannelImpl;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
    use platform_support::close_proximity_disclosure::MockCloseProximityDisclosureClient;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;

    use crate::DisclosureAttestationOptions;
    use crate::DisclosureProposalPresentation;
    use crate::wallet::Session;
    use crate::wallet::disclosure::WalletDisclosureAttestations;
    use crate::wallet::test::READER_CA;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::example_pid_stored_attestation_copy;
    use crate::wallet::test::example_stored_attestation_copy;

    use super::CloseProximityDisclosureError;
    use super::CloseProximityDisclosureSession;
    use super::CloseProximityDisclosureSessionState;
    use super::CloseProximityDisclosureUpdate;
    use super::verify_device_request;

    fn pid_given_name_items_request() -> ItemsRequest {
        ItemsRequest {
            doc_type: PID_ATTESTATION_TYPE.to_owned(),
            name_spaces: IndexMap::from_iter(vec![(
                PID_ATTESTATION_TYPE.to_owned(),
                IndexMap::from_iter(vec![(PID_GIVEN_NAME.to_owned(), true)]),
            )]),
            request_info: None,
        }
    }

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
                    e_reader_key_bytes: None,
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

    // Set up properties for a close proximity disclosure session.
    async fn setup_close_proximity_disclosure_session(
        wallet: &mut TestWalletMockStorage,
        items_request: ItemsRequest,
    ) -> VerifierCertificate {
        let mut reader_registration = ReaderRegistration::new_mock();
        let (doc_type, claims) = items_request.clone().into_doctype_and_claims();
        reader_registration.authorized_attributes = HashMap::from_iter([(doc_type, claims.collect_vec())]);

        let key_pair = generate_reader_mock_with_registration(&READER_CA, reader_registration).unwrap();

        let verifier_certificate = VerifierCertificate::try_new(key_pair.certificate().to_owned())
            .unwrap()
            .unwrap();

        let cose_key: CoseKey = (&key_pair.verifying_key().await.unwrap()).try_into().unwrap();
        let session_transcript = SessionTranscript::new_qr(cose_key, None);

        let doc_request = create_doc_request(items_request, &session_transcript, &key_pair).await;

        wallet.session = Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            listener: tokio::task::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::SessionEstablished {
                session_transcript: cbor_serialize(&session_transcript).unwrap(),
                device_request: cbor_serialize(&DeviceRequest::from_doc_requests(vec_nonempty![doc_request])).unwrap(),
            })),
        }));

        verifier_certificate
    }

    /// This tests `Wallet::continue_close_proximity_disclosure()` from a session that has already been established.
    #[tokio::test]
    async fn test_wallet_continue_close_proximity_disclosure() {
        // Populate a registered wallet with an example PID.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let items_request = pid_given_name_items_request();

        let verifier_certificate = setup_close_proximity_disclosure_session(&mut wallet, items_request).await;

        // Create three PID attestations.
        let mut pid_credential_payload = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());
        let mut attributes_root = pid_credential_payload.previewable_payload.attributes.into_inner();
        *attributes_root.get_mut(PID_GIVEN_NAME).unwrap() = Attribute::Single(AttributeValue::Text("Jane".to_string()));
        pid_credential_payload.previewable_payload.attributes = attributes_root.into();
        let pid1 = example_stored_attestation_copy(
            CredentialFormat::MsoMdoc,
            pid_credential_payload.clone(),
            NormalizedTypeMetadata::nl_pid_example(),
        );

        let pid2 = example_pid_stored_attestation_copy(CredentialFormat::MsoMdoc);

        let mut attributes_root = pid_credential_payload.previewable_payload.attributes.into_inner();
        *attributes_root.get_mut(PID_GIVEN_NAME).unwrap() = Attribute::Single(AttributeValue::Text("John".to_string()));
        pid_credential_payload.previewable_payload.attributes = attributes_root.into();
        let pid3 = example_stored_attestation_copy(
            CredentialFormat::MsoMdoc,
            pid_credential_payload,
            NormalizedTypeMetadata::nl_pid_example(),
        );

        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_types_and_format()
            .withf(move |attestation_types, format, _| {
                *attestation_types == HashSet::from([PID_ATTESTATION_TYPE.to_owned()])
                    && *format == CredentialFormat::MsoMdoc
            })
            .times(1)
            .return_once(move |_, _, _| Ok(vec![pid1, pid2.clone(), pid3]));

        // The wallet will check in the database if data was shared with the RP before.
        wallet
            .mut_storage()
            .expect_did_share_data_with_relying_party()
            .times(1)
            .returning(|_| Ok(false));

        // Starting disclosure should not cause attestation copy usage counts to be incremented.
        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .never();

        // Starting disclosure should not cause a disclosure event to be recorded yet.
        wallet.mut_storage().expect_log_disclosure_event().never();

        // Starting disclosure should not fail.
        let proposal = wallet
            .continue_close_proximity_disclosure()
            .await
            .expect("starting disclosure should succeed");

        wallet.mut_storage().checkpoint();

        // Test that the returned `DisclosureProposalPresentation` contains the processed data we set up earlier.
        assert_matches!(
            proposal,
            DisclosureProposalPresentation {
                reader_registration,
                shared_data_with_relying_party_before,
                ..
            } if reader_registration == *verifier_certificate.registration() && !shared_data_with_relying_party_before
        );
        assert_eq!(proposal.attestation_options.len().get(), 1);
        assert!(matches!(
            proposal.attestation_options.first(),
            DisclosureAttestationOptions::Multiple(options) if options.len().get() == 3
        ));
    }

    fn qr_session_transcript(device_engagement: Option<DeviceEngagement>) -> SessionTranscript {
        let ephemeral_key_pair = SigningKey::random(&mut OsRng);
        let cose_key: CoseKey = ephemeral_key_pair.verifying_key().try_into().unwrap();
        SessionTranscript::new_qr(cose_key, device_engagement)
    }

    async fn setup_device_request<'a>(
        items_requests: Vec<ItemsRequest>,
        device_engagement: Option<DeviceEngagement>,
    ) -> (DeviceRequest, SessionTranscript, Vec<TrustAnchor<'a>>) {
        let mut reader_registration = ReaderRegistration::new_mock();
        items_requests.clone().into_iter().for_each(|items_request| {
            let (doc_type, claims) = items_request.into_doctype_and_claims();
            reader_registration
                .authorized_attributes
                .insert(doc_type, claims.collect_vec());
        });

        let session_transcript = qr_session_transcript(device_engagement);

        let key_pair = generate_reader_mock_with_registration(&READER_CA, reader_registration).unwrap();
        let doc_requests = join_all(
            items_requests
                .into_iter()
                .map(async |items_request| create_doc_request(items_request, &session_transcript, &key_pair).await),
        )
        .await
        .try_into()
        .unwrap();

        let device_request = DeviceRequest::from_doc_requests(doc_requests);
        let trust_anchors = vec![READER_CA.to_trust_anchor().to_owned()];

        (device_request, session_transcript, trust_anchors)
    }

    #[tokio::test]
    async fn test_verify_device_request_success() {
        let items_request = pid_given_name_items_request();

        let (device_request, session_transcript, trust_anchors) = setup_device_request(vec![items_request], None).await;

        let result = verify_device_request(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            &trust_anchors,
        );

        assert!(result.is_ok());
    }

    fn empty_items_request() -> ItemsRequest {
        ItemsRequest {
            doc_type: PID_ATTESTATION_TYPE.to_owned(),
            name_spaces: IndexMap::new(),
            request_info: None,
        }
    }

    #[tokio::test]
    async fn test_verify_device_request_no_attributes() {
        let (device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![empty_items_request()], None).await;

        let result = verify_device_request(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            &trust_anchors,
        );

        assert_matches!(result, Err(CloseProximityDisclosureError::NoAttributesRequested));
    }

    #[tokio::test]
    async fn test_verify_device_request_missing_reader_auth() {
        let items_request = pid_given_name_items_request();

        let (mut device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![items_request], None).await;

        device_request
            .doc_requests
            .iter_mut()
            .for_each(|doc_request| doc_request.reader_auth = None);

        let result = verify_device_request(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            &trust_anchors,
        );

        assert_matches!(result, Err(CloseProximityDisclosureError::MissingReaderAuth));
    }

    #[tokio::test]
    async fn test_verify_device_request_inconsistent_reader_auths() {
        let items_request1 = pid_given_name_items_request();
        let items_request2 = ItemsRequest {
            doc_type: PID_ATTESTATION_TYPE.to_owned(),
            name_spaces: IndexMap::from_iter(vec![(
                PID_ATTESTATION_TYPE.to_owned(),
                IndexMap::from_iter(vec![(PID_FAMILY_NAME.to_owned(), true)]),
            )]),
            request_info: None,
        };

        let mut reader_registration = ReaderRegistration::new_mock();
        [items_request1.clone(), items_request2.clone()]
            .into_iter()
            .for_each(|items_request| {
                let (doc_type, claims) = items_request.into_doctype_and_claims();
                reader_registration
                    .authorized_attributes
                    .insert(doc_type, claims.collect_vec());
            });

        // Create two different key pairs from the same CA, so that the resulting certificates differ.
        let key_pair1 = generate_reader_mock_with_registration(&READER_CA, reader_registration.clone()).unwrap();
        let key_pair2 = generate_reader_mock_with_registration(&READER_CA, reader_registration).unwrap();

        let session_transcript = qr_session_transcript(None);

        let doc_request1 = create_doc_request(items_request1, &session_transcript, &key_pair1).await;
        let doc_request2 = create_doc_request(items_request2, &session_transcript, &key_pair2).await;

        let device_request = DeviceRequest::from_doc_requests(vec_nonempty![doc_request1, doc_request2]);
        let trust_anchors = [READER_CA.to_trust_anchor()];

        let result = verify_device_request(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            &trust_anchors,
        );

        assert_matches!(result, Err(CloseProximityDisclosureError::InconsistentReaderAuths));
    }

    #[tokio::test]
    async fn test_verify_device_request_unregistered_attributes() {
        let items_request = pid_given_name_items_request();

        let reader_registration = ReaderRegistration::new_mock();
        // do not add any authorized attributes to the registration

        let key_pair = generate_reader_mock_with_registration(&READER_CA, reader_registration).unwrap();
        let cose_key: CoseKey = (&key_pair.verifying_key().await.unwrap()).try_into().unwrap();
        let session_transcript = SessionTranscript::new_qr(cose_key, None);

        let doc_requests = create_doc_request(items_request, &session_transcript, &key_pair).await;

        let device_request = DeviceRequest::from_doc_requests(vec_nonempty![doc_requests]);
        let trust_anchors = vec![READER_CA.to_trust_anchor().to_owned()];

        let result = verify_device_request(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            &trust_anchors,
        );

        assert_matches!(
            result,
            Err(CloseProximityDisclosureError::RequestedUnregisteredAttributes(_))
        );
    }
}
