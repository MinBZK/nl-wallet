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
use crypto::x509::CertificateError;
use entity::disclosure_event::EventStatus;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::client::TlsPinningConfig;
use jwt::nonce::Nonce;
use mdoc::DeviceRequest;
use mdoc::DeviceRequestParseError;
use mdoc::DeviceResponse;
use mdoc::DeviceResponseStatus;
use mdoc::SessionTranscript;
use mdoc::utils::cose::CoseError;
use mdoc::utils::serialization::CborError;
use mdoc::utils::serialization::cbor_serialize;
use openid4vc::disclosure_session::DataDisclosed;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::openid4vp::ClientId;
use openid4vc::verifier::SessionType;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureError as PlatformError;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
use rustls_pki_types::TrustAnchor;
use update_policy_model::update_policy::VersionState;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::WalletConfiguration;
use wscd::wscd::JwtPoaInput;

use crate::AttributesNotAvailable;
use crate::DisclosureProposalPresentation;
use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::errors::InstructionError;
use crate::errors::RemoteEcdsaKeyError;
use crate::errors::UpdatePolicyError;
use crate::instruction::RemoteEcdsaWscd;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::DisclosableAttestation;
use crate::storage::PartialAttestation;
use crate::storage::Storage;
use crate::wallet::DisclosureError;
use crate::wallet::Session;
use crate::wallet::disclosure::AttestedKeyAndRegistrationData;
use crate::wallet::disclosure::RedirectUriPurpose;
use crate::wallet::disclosure::WalletDisclosureAttestations;
use crate::wallet::disclosure::instruction_error_from_signing_error;
use crate::wallet::disclosure::requested_attribute_paths;

#[derive(Debug)]
pub enum CloseProximityDisclosureUpdate {
    Connecting,
    Connected,
    DeviceRequestReceived,
    Disconnected,
    Errored(CloseProximityDisclosureError),
}

type CloseProximityDisclosureCallbackFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub type CloseProximityDisclosureCallback =
    Box<dyn Fn(CloseProximityDisclosureUpdate) -> CloseProximityDisclosureCallbackFuture + Send + Sync>;

#[nutype(validate(predicate = |s| s.parse::<Url>().is_ok_and(|u| u.scheme() == "mdoc")), derive(Debug, Clone, TryFrom, FromStr, AsRef, Into, Display))]
pub struct MdocUri(String);

#[derive(Debug, Clone, IsVariant)]
enum CloseProximityDisclosureSessionState {
    Advertising,
    SessionEstablished {
        session_transcript: Vec<u8>,
        device_request: Vec<u8>,
    },
    DisclosureProposed {
        session_transcript: Box<SessionTranscript>,
        verifier_certificate: Box<VerifierCertificate>,
        attestations: WalletDisclosureAttestations<usize>,
    },
    Errored {
        #[expect(
            dead_code,
            reason = "will be used when processing close proximity disclosure errors (PVW-5710)"
        )]
        error: PlatformError,
        verifier_certificate: Option<Box<VerifierCertificate>>,
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
                PlatformUpdate::Error { error } => {
                    // Platform support already translated the transport/session failure to the
                    // reader-facing status code before reporting it here; core keeps the errored
                    // state so the app can surface the failure.
                    error!("Received PlatformUpdate::Error: {error:?}");
                    let mut current_state = session_state.lock();
                    let verifier_certificate = match &*current_state {
                        CloseProximityDisclosureSessionState::DisclosureProposed {
                            verifier_certificate, ..
                        } => Some(verifier_certificate.clone()),
                        _ => None,
                    };

                    *current_state = CloseProximityDisclosureSessionState::Errored {
                        error: error.clone(),
                        verifier_certificate,
                    };
                    CloseProximityDisclosureUpdate::Errored(error.into())
                }
            };

            info!("Close proximity disclosure update: {wallet_update:?}");
            callback(wallet_update).await;
        }
    })
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum CloseProximityDisclosureError {
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
    MalformedDeviceRequest(#[from] CborError),

    #[error("Received invalid Device Request structure from reader: {0}")]
    #[category(critical)]
    InvalidDeviceRequest(#[source] CborError),

    #[error("Could not encode error DeviceResponse: {0}")]
    #[category(critical)]
    ErrorDeviceResponseEncoding(#[source] CborError),

    #[error("Received error from native close proximity bridge: {0}")]
    #[category(critical)]
    PlatformError(#[from] PlatformError),

    #[error("Failed creating device response: {0}")]
    #[category(pd)]
    DeviceResponse(#[source] mdoc::Error),

    #[error("Could not extract SAN DNS name from certificate: {0}")]
    #[category(critical)]
    InvalidCertificate(#[source] CertificateError),

    #[error("No SAN DNS name found in certificate")]
    #[category(critical)]
    MissingSanDnsName,
}

fn parse_device_request(bytes: &[u8]) -> Result<DeviceRequest, CloseProximityDisclosureError> {
    DeviceRequest::try_from_bytes(bytes).map_err(|error| match error {
        DeviceRequestParseError::MalformedCbor(error) => CloseProximityDisclosureError::MalformedDeviceRequest(error),
        DeviceRequestParseError::InvalidStructure(error) => CloseProximityDisclosureError::InvalidDeviceRequest(error),
    })
}

fn error_device_response_status(error: &CloseProximityDisclosureError) -> Option<DeviceResponseStatus> {
    match error {
        CloseProximityDisclosureError::MalformedDeviceRequest(_) => Some(DeviceResponseStatus::CborDecodingError),
        CloseProximityDisclosureError::InvalidDeviceRequest(_) => Some(DeviceResponseStatus::InvalidRequest),
        CloseProximityDisclosureError::MissingReaderAuth
        | CloseProximityDisclosureError::InconsistentReaderAuths
        | CloseProximityDisclosureError::InvalidDocRequest(_)
        | CloseProximityDisclosureError::MissingReaderRegistration
        | CloseProximityDisclosureError::InvalidCertificateType(_)
        | CloseProximityDisclosureError::RequestedUnregisteredAttributes(_)
        | CloseProximityDisclosureError::InvalidCertificate(_)
        | CloseProximityDisclosureError::MissingSanDnsName => Some(DeviceResponseStatus::GeneralError),
        // These are either internal wallet errors or failures already handled by platform support,
        // so we do not expect to send a protocol-level error DeviceResponse for them.
        CloseProximityDisclosureError::ErrorDeviceResponseEncoding(_)
        | CloseProximityDisclosureError::PlatformError(_)
        | CloseProximityDisclosureError::DeviceResponse(_) => None,
    }
}

fn encode_error_device_response(status: DeviceResponseStatus) -> Result<Vec<u8>, CloseProximityDisclosureError> {
    // Defensive: with the current fixed DeviceResponse shape we do not expect CBOR serialization to fail in practice.
    cbor_serialize(&DeviceResponse::error(status)).map_err(CloseProximityDisclosureError::ErrorDeviceResponseEncoding)
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
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

    async fn send_close_proximity_error_response_and_stop(
        &mut self,
        error: &CloseProximityDisclosureError,
    ) -> Result<(), DisclosureError> {
        let Some(status) = error_device_response_status(error) else {
            return Ok(());
        };

        // After sending the error DeviceResponse, always stop BLE and clear the
        // close proximity session so the verifier is not left waiting on a dead session.
        let response = encode_error_device_response(status)?;
        let send_result = CPC::send_device_response(response).await;

        match self.session.take() {
            Some(Session::CloseProximityDisclosure(session)) => {
                session.listener.abort();
            }
            other => {
                self.session = other;
            }
        }

        let stop_result = CPC::stop_ble_server().await;

        send_result?;
        stop_result?;

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_close_proximity_disclosure(
        &mut self,
    ) -> Result<DisclosureProposalPresentation, DisclosureError> {
        info!("Continuing close proximity disclosure");

        self.check_disclosure_preconditions()?;

        info!("Checking if there is an active close proximity session");
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

        let device_request = match parse_device_request(&device_request) {
            Ok(device_request) => device_request,
            Err(error) => {
                self.send_close_proximity_error_response_and_stop(&error).await?;
                return Err(error.into());
            }
        };
        // the session transcript is made by our own native code, so deserialization errors are programmer errors
        let session_transcript = SessionTranscript::try_from_bytes(&session_transcript).unwrap();

        let wallet_config = self.config_repository.get();
        let verifier_certificate = match verify_device_request(
            &device_request,
            &session_transcript,
            &TimeGenerator,
            &wallet_config.disclosure.rp_trust_anchors(),
        ) {
            Ok(verifier_certificate) => verifier_certificate,
            Err(error) => {
                self.send_close_proximity_error_response_and_stop(&error).await?;
                return Err(error.into());
            }
        };

        let (candidate_attestations, shared_data_with_relying_party_before) = self
            .prepare_disclosure(
                &device_request.items_requests().collect_vec(),
                &wallet_config.pid_attributes,
                &verifier_certificate,
            )
            .await?;

        let candidate_attestations = candidate_attestations
            .into_iter()
            .flatten() // remove entries for which no suitable candidates were found
            .collect::<Vec<_>>();

        let reader_registration = verifier_certificate.registration().to_owned();
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
                reader_registration.clone(),
                shared_data_with_relying_party_before,
                session_type,
                disclosure_type,
                purpose,
            );

            *session_state.lock() = CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(session_transcript),
                verifier_certificate: Box::new(verifier_certificate),
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
            verifier_certificate: Box::new(verifier_certificate),
            attestations: WalletDisclosureAttestations::Missing,
        };

        Err(DisclosureError::AttributesNotAvailable(AttributesNotAvailable {
            reader_registration: Box::new(reader_registration),
            requested_attributes,
            shared_data_with_relying_party_before,
            session_type,
        }))
    }

    #[instrument(skip_all)]
    pub(super) async fn perform_close_proximity_disclosure(
        &mut self,
        close_proximity_session: CloseProximityDisclosureSession,
        selected_indices: &[usize],
        pin: String,
        attested_key_and_registration_data: AttestedKeyAndRegistrationData<AKH>,
    ) -> Result<(), DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
    {
        // If we do not have a proposal, this method should not have been called, so return an error.
        let CloseProximityDisclosureSessionState::DisclosureProposed {
            attestations,
            verifier_certificate,
            session_transcript,
            ..
        } = close_proximity_session.session_state.lock().to_owned()
        else {
            self.session
                .replace(Session::CloseProximityDisclosure(close_proximity_session));
            return Err(DisclosureError::SessionState);
        };

        let reader_certificate = verifier_certificate.certificate().clone();

        // Prepare the `RemoteEcdsaWscd` for signing using the provided PIN.
        let remote_wscd = match self.prepare_remote_wscd(pin, attested_key_and_registration_data).await {
            Ok(ok) => ok,
            Err(e) => {
                self.session
                    .replace(Session::CloseProximityDisclosure(close_proximity_session));
                return Err(e);
            }
        };

        // Note that this will panic if any of the indices are out of bounds.
        let attestations = attestations.select_proposal(selected_indices);

        // There is guaranteed to be at least one attestation because of the logic in `start_disclosure()`.
        let attestation_values = VecNonEmpty::try_from(attestations.values().copied().collect_vec()).unwrap();

        // NOTE: If the disclosure fails and is retried, the disclosure count will jump by
        //       more than 1, since the same copies are shared with the verifier again.
        //       It is necessary to increment the disclosure count before sending the attestations
        //       to the verifier, as we do not know if disclosure fails before or after the
        //       verifier has received the attributes.
        let (attestation_presentations, result) = self
            .increment_usage_count_and_collect_presentations(attestation_values)
            .await;

        let disclosure_type = DisclosureType::Regular;

        if let Err(error) = result {
            // If storing the event results in an error, log it but do nothing else.
            let _ = self
                .store_disclosure_event(
                    Utc::now(),
                    Some(attestation_presentations),
                    reader_certificate.clone(),
                    disclosure_type,
                    EventStatus::Error,
                    DataDisclosed::NotDisclosed,
                )
                .await
                .inspect_err(|e| {
                    error!("Could not store error in history: {e}");
                });

            // Put back the session for a later attempt
            self.session
                .replace(Session::CloseProximityDisclosure(close_proximity_session));
            return Err(DisclosureError::IncrementUsageCount(error));
        }

        let device_response = match Self::create_close_proximity_device_response(
            attestations.values().copied(),
            session_transcript.as_ref(),
            verifier_certificate.as_ref(),
            &remote_wscd,
        )
        .await
        {
            Ok(device_response) => device_response,
            Err(disclosure_error) => {
                // IncorrectPin is a functional error and does not need to be recorded.
                //
                // If storing the event results in an error, log it but do nothing else.
                if !matches!(
                    disclosure_error,
                    DisclosureError::Instruction(InstructionError::IncorrectPin { .. })
                ) && let Err(error) = self
                    .store_disclosure_event(
                        Utc::now(),
                        Some(attestation_presentations),
                        reader_certificate,
                        disclosure_type,
                        EventStatus::Error,
                        DataDisclosed::NotDisclosed,
                    )
                    .await
                {
                    error!("Could not store error in history: {error}");
                }

                match disclosure_error {
                    DisclosureError::Instruction(InstructionError::Timeout { .. } | InstructionError::Blocked) => {
                        // On a PIN timeout we should proactively terminate the disclosure session
                        // and lock the wallet, as the user is probably not the owner of the wallet.
                        // The UI should catch this specific error and close the disclosure screens.
                        //
                        // If terminating the session results in an error, log it but do nothing else.
                        close_proximity_session.listener.abort();

                        let _ = CPC::stop_ble_server().await.inspect_err(|terminate_error| {
                            error!(
                                "Error while terminating disclosure session on PIN timeout: {}",
                                terminate_error
                            );
                        });

                        self.lock.lock();
                    }
                    DisclosureError::Instruction(InstructionError::AccountRevoked(data)) => {
                        self.handle_wallet_revocation(data).await;
                    }
                    _ => {
                        // If we did not just terminate the close proximity disclosure session, place it back in the
                        // wallet state so that the user may retry disclosure.
                        self.session
                            .replace(Session::CloseProximityDisclosure(close_proximity_session));
                    }
                }

                return Err(disclosure_error);
            }
        };

        // Actually perform the disclosure by sending the device response to the reader
        // if serialization fails, there's a bug in the code
        CPC::send_device_response(cbor_serialize(&device_response).unwrap()).await?;

        // Disclosure is now successful. Any errors that occur after this point will result in the `Wallet` not having
        // an active disclosure session anymore.
        self.store_disclosure_event(
            Utc::now(),
            Some(attestation_presentations),
            reader_certificate,
            disclosure_type,
            EventStatus::Success,
            DataDisclosed::Disclosed,
        )
        .await
        .map_err(DisclosureError::EventStorage)?;

        Ok(())
    }

    pub async fn terminate_close_proximity_disclosure_session(
        &mut self,
        session: CloseProximityDisclosureSession,
    ) -> Result<(), DisclosureError> {
        // First abort the listener, s.t. no more events are passed along
        session.listener.abort();

        let state = session.session_state.lock().to_owned();
        let (event, send_result) = match state {
            CloseProximityDisclosureSessionState::DisclosureProposed {
                verifier_certificate, ..
            } => (
                Some((verifier_certificate, EventStatus::Cancelled)),
                CPC::send_session_termination().await,
            ),
            CloseProximityDisclosureSessionState::Errored {
                verifier_certificate: Some(verifier_certificate),
                ..
            } => (Some((verifier_certificate, EventStatus::Error)), Ok(())),
            _ => (None, Ok(())),
        };

        let stop_result = CPC::stop_ble_server().await;
        send_result?;
        stop_result?;

        // Only store the event if the session is past SessionEstablished state (i.e. DisclosureProposed or later)
        if let Some((certificate, status)) = event {
            self.store_disclosure_event(
                Utc::now(),
                // TODO (PVW-5078): Store credential requests in disclosure event.
                None,
                (*certificate).into_certificate(),
                DisclosureType::Regular,
                status,
                DataDisclosed::NotDisclosed,
            )
            .await?;
        }

        Ok(())
    }

    async fn create_close_proximity_device_response<'a>(
        attestations: impl IntoIterator<Item = &'a DisclosableAttestation>,
        session_transcript: &SessionTranscript,
        verifier_certificate: &VerifierCertificate,
        remote_wscd: &RemoteEcdsaWscd<S, AKH::AppleKey, AKH::GoogleKey, APC>,
    ) -> Result<DeviceResponse, DisclosureError>
    where
        APC: AccountProviderClient,
    {
        // Gather all partial mdocs presentations by cloning the attestations held in the session, as disclosing
        // attestations needs to be retryable.
        let partial_mdocs = attestations
            .into_iter()
            .map(|attestation| {
                let PartialAttestation::MsoMdoc { partial_mdoc } = attestation.partial_attestation() else {
                    panic!("SD-JWT attestations are not supported in close proximity disclosure")
                };

                *partial_mdoc.clone()
            })
            .collect_vec()
            .try_into()
            .unwrap();

        // if this fails, there's a bug in the code
        let nonce = Nonce::from(hex::encode(cbor_serialize(session_transcript).unwrap()));
        // use the same aud as for SD-JWT
        let aud = ClientId::x509_san_dns(
            verifier_certificate
                .certificate()
                .san_dns_name()
                .map_err(CloseProximityDisclosureError::InvalidCertificate)?
                .ok_or(CloseProximityDisclosureError::MissingSanDnsName)?,
        )
        .to_string();
        let poa_input = JwtPoaInput::new(Some(nonce), aud);

        // Create the device response, casting any `InstructionError` that occurs during signing
        // to `RemoteEcdsaKeyError::Instruction`.
        DeviceResponse::sign_from_partial_mdocs(partial_mdocs, session_transcript, remote_wscd, poa_input)
            .await
            .map_err(|error| match error {
                mdoc::Error::Cose(CoseError::Signing(signing_error))
                    if matches!(
                        signing_error.downcast_ref::<RemoteEcdsaKeyError>(),
                        Some(RemoteEcdsaKeyError::Instruction(_))
                    ) =>
                {
                    DisclosureError::Instruction(instruction_error_from_signing_error(signing_error))
                }
                other => DisclosureError::CloseProximityDisclosureSessionError(
                    CloseProximityDisclosureError::DeviceResponse(other),
                ),
            })
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
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Signer;
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
    use crypto::p256_der::DerSignature;
    use crypto::server_keys::generate::Ca;
    use dcql::CredentialFormat;
    use dcql::normalized::NormalizedCredentialRequests;
    use entity::disclosure_event::EventStatus;
    use mdoc::DeviceEngagement;
    use mdoc::DeviceRequest;
    use mdoc::DeviceResponse;
    use mdoc::Handover;
    use mdoc::ItemsRequest;
    use mdoc::SessionTranscript;
    use mdoc::SessionTranscriptKeyed;
    use mdoc::holder::disclosure::create_doc_request;
    use mdoc::utils::cose::CoseKey;
    use mdoc::utils::serialization::CborSeq;
    use mdoc::utils::serialization::cbor_deserialize;
    use mdoc::utils::serialization::cbor_serialize;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannel;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannelImpl;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
    use platform_support::close_proximity_disclosure::MockCloseProximityDisclosureClient;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wallet_account::messages::errors::AccountError;
    use wallet_account::messages::errors::IncorrectPinData;
    use wallet_account::messages::errors::PinTimeoutData;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::instructions::Sign;
    use wallet_account::messages::instructions::SignResult;

    use crate::DisclosureAttestationOptions;
    use crate::DisclosureProposalPresentation;
    use crate::account_provider::AccountProviderError;
    use crate::account_provider::AccountProviderResponseError;
    use crate::attestation::mock::EmptyPresentationConfig;
    use crate::errors::StorageError;
    use crate::storage::ChangePinData;
    use crate::storage::DisclosableAttestation;
    use crate::storage::InstructionData;
    use crate::wallet::DisclosureError;
    use crate::wallet::Session;
    use crate::wallet::disclosure::WalletDisclosureAttestations;
    use crate::wallet::test::READER_CA;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_wp_result;
    use crate::wallet::test::example_pid_stored_attestation_copy;
    use crate::wallet::test::example_stored_attestation_copy;

    use super::CloseProximityDisclosureError;
    use super::CloseProximityDisclosureSession;
    use super::CloseProximityDisclosureSessionState;
    use super::CloseProximityDisclosureUpdate;
    use super::DeviceResponseStatus;
    use super::PlatformError;
    use super::verify_device_request;

    fn pid_given_name_items_request() -> ItemsRequest {
        ItemsRequest {
            doc_type: PID_ATTESTATION_TYPE.to_owned(),
            name_spaces: IndexMap::from_iter(vec![(
                PID_ATTESTATION_TYPE.to_owned(),
                IndexMap::from_iter(vec![(PID_GIVEN_NAME.to_owned(), true)])
                    .try_into()
                    .unwrap(),
            )])
            .try_into()
            .unwrap(),
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
    async fn test_terminate_close_proximity_disclosure_session_disclosure_proposed_missing_sends_session_termination() {
        let send_context = MockCloseProximityDisclosureClient::send_session_termination_context();
        send_context.expect().once().returning(|| Ok(()));

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

        let verifier_certificate = VerifierCertificate::try_new(reader_certificate).unwrap().unwrap();
        let session = CloseProximityDisclosureSession {
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(CborSeq(SessionTranscriptKeyed {
                    device_engagement_bytes: None,
                    e_reader_key_bytes: None,
                    handover: Handover::QrHandover,
                })),
                verifier_certificate: Box::new(verifier_certificate),
                attestations: WalletDisclosureAttestations::Missing,
            })),
        };

        wallet
            .terminate_close_proximity_disclosure_session(session)
            .await
            .expect("terminating close proximity disclosure session should succeed");
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_terminate_close_proximity_disclosure_session_disclosure_proposed_proposal_sends_session_termination()
    {
        let send_context = MockCloseProximityDisclosureClient::send_session_termination_context();
        send_context.expect().once().returning(|| Ok(()));

        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);
        let reader_certificate = verifier_certificate.certificate().clone();

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate),
                eq(EventStatus::Cancelled),
                eq(DisclosureType::Regular),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let Some(Session::CloseProximityDisclosure(session)) = wallet.session.take() else {
            panic!("expected a close proximity disclosure session");
        };

        wallet
            .terminate_close_proximity_disclosure_session(session)
            .await
            .expect("terminating close proximity disclosure session should succeed");
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_terminate_close_proximity_disclosure_session_send_session_termination_error_still_stops_ble_server() {
        let send_context = MockCloseProximityDisclosureClient::send_session_termination_context();
        send_context.expect().once().returning(|| {
            Err(PlatformError::PlatformError {
                reason: "send failed".to_string(),
            })
        });

        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let _verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        wallet.mut_storage().expect_log_disclosure_event().never();

        let Some(Session::CloseProximityDisclosure(session)) = wallet.session.take() else {
            panic!("expected a close proximity disclosure session");
        };

        let result = wallet.terminate_close_proximity_disclosure_session(session).await;

        assert_matches!(
            result,
            Err(DisclosureError::PlatformCloseProximityDisclosureSessionError(
                PlatformError::PlatformError { reason }
            )) if reason == "send failed"
        );
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

    fn install_session_established_close_proximity_session(
        wallet: &mut TestWalletMockStorage,
        session_transcript: Vec<u8>,
        device_request: Vec<u8>,
    ) {
        wallet.session = Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            listener: tokio::task::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::SessionEstablished {
                session_transcript,
                device_request,
            })),
        }));
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
            .once()
            .return_once(move |_, _, _| Ok(vec![pid1, pid2.clone(), pid3]));

        // The wallet will check in the database if data was shared with the RP before.
        wallet
            .mut_storage()
            .expect_did_share_data_with_relying_party()
            .once()
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

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_invalid_device_request_cbor() {
        let send_context = MockCloseProximityDisclosureClient::send_device_response_context();
        send_context
            .expect()
            .once()
            .withf(|response| {
                let device_response: DeviceResponse = cbor_deserialize(response.as_slice()).unwrap();
                device_response.documents.is_none()
                    && device_response.document_errors.is_none()
                    && device_response.status == DeviceResponseStatus::CborDecodingError
            })
            .returning(|_| Ok(()));

        let stop_context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        stop_context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        install_session_established_close_proximity_session(
            &mut wallet,
            cbor_serialize(&qr_session_transcript(None)).unwrap(),
            vec![0xff],
        );

        let result = wallet.continue_close_proximity_disclosure().await;

        assert_matches!(
            result,
            Err(DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::MalformedDeviceRequest(_)
            ))
        );
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_invalid_device_request_structure() {
        let send_context = MockCloseProximityDisclosureClient::send_device_response_context();
        send_context
            .expect()
            .once()
            .withf(|response| {
                let device_response: DeviceResponse = cbor_deserialize(response.as_slice()).unwrap();
                device_response.documents.is_none()
                    && device_response.document_errors.is_none()
                    && device_response.status == DeviceResponseStatus::InvalidRequest
            })
            .returning(|_| Ok(()));

        let stop_context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        stop_context.expect().once().returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        install_session_established_close_proximity_session(
            &mut wallet,
            cbor_serialize(&qr_session_transcript(None)).unwrap(),
            cbor_serialize(&42u8).unwrap(),
        );

        let result = wallet.continue_close_proximity_disclosure().await;

        assert_matches!(
            result,
            Err(DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::InvalidDeviceRequest(_)
            ))
        );
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_reader_auth_failure() {
        let send_context = MockCloseProximityDisclosureClient::send_device_response_context();
        send_context
            .expect()
            .once()
            .withf(|response| {
                let device_response: DeviceResponse = cbor_deserialize(response.as_slice()).unwrap();
                device_response.documents.is_none()
                    && device_response.document_errors.is_none()
                    && device_response.status == DeviceResponseStatus::GeneralError
            })
            .returning(|_| Ok(()));

        let stop_context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        stop_context.expect().once().returning(|| Ok(()));

        let items_request = pid_given_name_items_request();
        let (mut device_request, session_transcript, _trust_anchors) =
            setup_device_request(vec![items_request], None).await;
        device_request
            .doc_requests
            .iter_mut()
            .for_each(|doc_request| doc_request.reader_auth = None);

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        install_session_established_close_proximity_session(
            &mut wallet,
            cbor_serialize(&session_transcript).unwrap(),
            cbor_serialize(&device_request).unwrap(),
        );

        let result = wallet.continue_close_proximity_disclosure().await;

        assert_matches!(
            result,
            Err(DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::MissingReaderAuth
            ))
        );
        assert!(wallet.session.is_none());
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
                IndexMap::from_iter(vec![(PID_FAMILY_NAME.to_owned(), true)])
                    .try_into()
                    .unwrap(),
            )])
            .try_into()
            .unwrap(),
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

    // The PIN used in accept_disclosure tests.
    const PIN: &str = "051097";

    /// Creates a `CloseProximityDisclosureSession` in the `DisclosureProposed` state and installs
    /// it as `wallet.session`. Returns the `VerifierCertificate` stored inside the session so that
    /// callers can set up event-log expectations against its certificate.
    fn setup_close_proximity_disclosure_proposed_session(wallet: &mut TestWalletMockStorage) -> VerifierCertificate {
        let items_request = pid_given_name_items_request();
        let mut reader_registration = ReaderRegistration::new_mock();
        let (doc_type, claims) = items_request.clone().into_doctype_and_claims();
        reader_registration.authorized_attributes = HashMap::from_iter([(doc_type, claims.collect_vec())]);

        let key_pair = generate_reader_mock_with_registration(&READER_CA, reader_registration).unwrap();

        let verifier_certificate = VerifierCertificate::try_new(key_pair.certificate().to_owned())
            .unwrap()
            .unwrap();

        let pid_credential_payload = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());
        let pid = example_stored_attestation_copy(
            CredentialFormat::MsoMdoc,
            pid_credential_payload.clone(),
            NormalizedTypeMetadata::nl_pid_example(),
        );

        let credential_requests = NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(PID_ATTESTATION_TYPE, &[&[PID_ATTESTATION_TYPE, PID_GIVEN_NAME]])],
            None,
        );
        let disclosable_attestation = DisclosableAttestation::try_new(
            pid,
            credential_requests.as_ref().first().unwrap().claim_paths(),
            &EmptyPresentationConfig,
        )
        .unwrap();

        let ephemeral_key = SigningKey::random(&mut OsRng);
        let cose_key: CoseKey = ephemeral_key.verifying_key().try_into().unwrap();
        let session_transcript = SessionTranscript::new_qr(cose_key, None);

        wallet.session = Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(session_transcript),
                verifier_certificate: Box::new(verifier_certificate.clone()),
                attestations: WalletDisclosureAttestations::Proposal(IndexMap::from([(
                    0,
                    vec_nonempty![disclosable_attestation],
                )])),
            })),
        }));

        verifier_certificate
    }

    fn setup_mock_sign_instruction(wallet: &mut TestWalletMockStorage) {
        wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| Ok(None));
        wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .with(always(), always())
            .returning(|_, _| Ok(vec![0u8; 32]));

        // Sign a dummy payload with a throwaway key to get a well-formed DerSignature.
        let signing_key = SigningKey::random(&mut OsRng);
        let signature: Signature = signing_key.sign(b"");
        let der_sig = DerSignature::from(signature);

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .with(always(), always())
            .return_once(move |_, _: Instruction<Sign>| {
                Ok(create_wp_result(SignResult {
                    signatures: vec![vec![der_sig]],
                    poa: None,
                }))
            });
    }

    /// Sets up the mock account provider to return `account_error` when the Sign instruction is
    /// sent, after a successful instruction challenge.
    fn setup_mock_sign_instruction_error(wallet: &mut TestWalletMockStorage, account_error: AccountError) {
        wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| Ok(None));
        wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .with(always(), always())
            .returning(|_, _| Ok(vec![0u8; 32]));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .with(always(), always())
            .returning(move |_, _: Instruction<Sign>| {
                Err(AccountProviderError::Response(AccountProviderResponseError::Account(
                    account_error,
                    None,
                )))
            });
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction(&mut wallet);

        let context = MockCloseProximityDisclosureClient::send_device_response_context();
        context
            .expect()
            .once()
            .withf(|device_response| {
                let device_response: DeviceResponse = cbor_deserialize(device_response.as_slice()).unwrap();
                // device response has documents and success status
                device_response.documents.is_some() && device_response.status == DeviceResponseStatus::Ok
            })
            .returning(|_| Ok(()));

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Ok(()));

        let reader_certificate = verifier_certificate.certificate().to_owned();
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                always(),
                eq(reader_certificate),
                eq(EventStatus::Success),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let result = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect("accepting close proximity disclosure should succeed");

        // Close proximity disclosure has no redirect URI.
        assert_eq!(result, None);
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_error_increment_usage_count() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction(&mut wallet);

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Err(StorageError::NotOpened));

        let reader_certificate = verifier_certificate.certificate().to_owned();
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate.clone()),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting close proximity disclosure should not succeed");

        assert_matches!(error, DisclosureError::IncrementUsageCount(_));
        // The session must be preserved so that the user may retry.
        assert!(wallet.session.is_some());

        // And we can actually retry
        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .times(1)
            .return_once(|_| Ok(()));

        let context = MockCloseProximityDisclosureClient::send_device_response_context();
        context
            .expect()
            .once()
            .withf(|device_response| {
                let device_response: DeviceResponse = cbor_deserialize(device_response.as_slice()).unwrap();
                // device response has documents and success status
                device_response.documents.is_some() && device_response.status == DeviceResponseStatus::Ok
            })
            .returning(|_| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                always(),
                eq(reader_certificate),
                eq(EventStatus::Success),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        assert!(
            wallet
                .accept_disclosure(&[0], PIN.to_string())
                .await
                .expect("accepting close proximity disclosure should succeed")
                .is_none()
        );
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_error_instruction_incorrect_pin() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction_error(
            &mut wallet,
            AccountError::IncorrectPin(IncorrectPinData {
                attempts_left_in_round: 2,
                is_final_round: false,
            }),
        );
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Ok(()));

        // No event should be recorded.
        wallet.mut_storage().expect_log_disclosure_event().never();

        let error = wallet
            .accept_disclosure(&[0], "whatever".to_string())
            .await
            .expect_err("accepting close proximity disclosure should not succeed");

        assert_matches!(error, DisclosureError::Instruction(_));

        // The session must be preserved so that the user may retry.
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_error_instruction_validation() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction_error(&mut wallet, AccountError::InstructionValidation);
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Ok(()));

        let reader_certificate = verifier_certificate.certificate().clone();
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting close proximity disclosure should not succeed");

        assert_matches!(error, DisclosureError::Instruction(_));

        // The session must be preserved so that the user may retry.
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_error_instruction_pin_timeout() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction_error(
            &mut wallet,
            AccountError::PinTimeout(PinTimeoutData {
                time_left_in_ms: 10_000,
            }),
        );
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Ok(()));

        let reader_certificate = verifier_certificate.certificate().clone();

        // On Timeout the session is terminated via `stop_ble_server`
        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting close proximity disclosure should not succeed");

        assert_matches!(error, DisclosureError::Instruction(_));
        assert!(wallet.session.is_none());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_error_instruction_account_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let verifier_certificate = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction_error(&mut wallet, AccountError::AccountBlocked);
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Ok(()));

        let reader_certificate = verifier_certificate.certificate().clone();

        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting close proximity disclosure should not succeed");

        assert_matches!(error, DisclosureError::Instruction(_));
        assert!(wallet.session.is_none());
        assert!(wallet.is_locked());
    }
}
