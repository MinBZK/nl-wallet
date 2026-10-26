use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use attestation_data::disclosure_type::DisclosureType;
use attestation_data::organization::Organization;
use attestation_data::registration_certificate::REQUEST_INFO_WRPRC_KEY;
use attestation_data::registration_certificate::RegistrationCertificateAuthorizationError;
use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::RegistrationCertificateEnvelopeError;
use attestation_data::registration_certificate::RegistrationCertificateRequestParseError;
use attestation_data::registration_certificate::RegistrationCertificateStatusValidationError;
use attestation_data::registration_certificate::RegistrationCertificateValidationError;
use attestation_data::registration_certificate::SubjectType;
use attestation_data::registration_certificate::verify_registration_certificate_envelope;
use attestation_data::x509::RelyingParty;
use attestation_data::x509::RelyingPartyError;
use chrono::DateTime;
use chrono::Utc;
use ciborium::value::Value;
use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::x509::CertificateVerifyError;
use crypto::x509::DistinguishedNameError;
use crypto::x509::crl::CertificateCrlVerificationError;
use crypto::x509::crl::CertificateCrlVerifier;
use crypto::x509::crl::CrlFetcher;
use derive_more::IsVariant;
use entity::disclosure_event::EventStatus;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use futures::future::try_join;
use futures::future::try_join_all;
use http_utils::client::TlsPinningConfig;
use indexmap::IndexMap;
use itertools::Itertools;
use mdoc::DeviceRequest;
use mdoc::DeviceRequestParseError;
use mdoc::DeviceResponse;
use mdoc::DeviceResponseStatus;
use mdoc::SessionTranscript;
use mdoc::holder::disclosure::PartialMdoc;
use mdoc::utils::cose::CoseError;
use mdoc::utils::serialization::CborError;
use mdoc::utils::serialization::cbor_serialize;
use nutype::nutype;
use openid4vc::disclosure_session::DataDisclosed;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::DisclosureTrustAnchors;
use openid4vc::verifier::SessionType;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use parking_lot::Mutex;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureError as PlatformError;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationVerifier;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;
use tracing::instrument;
use update_policy_model::update_policy::VersionState;
use url::Url;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::AttributesNotAvailable;
use crate::DisclosureProposalPresentation;
use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::errors::InstructionError;
use crate::errors::RemoteEcdsaKeyError;
use crate::errors::UpdatePolicyError;
use crate::instruction::RemoteEcdsaWscd;
use crate::pin::key::Pin as WalletPin;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::DisclosableAttestation;
use crate::storage::PartialAttestation;
use crate::storage::Storage;
use crate::storage::WithKeyIdentifier;
use crate::wallet::DisclosureError;
use crate::wallet::Session;
use crate::wallet::disclosure::RedirectUriPurpose;
use crate::wallet::disclosure::VpDisclosableAttestation;
use crate::wallet::disclosure::WalletDisclosureAttestations;
use crate::wallet::disclosure::instruction_error_from_signing_error;
use crate::wallet::disclosure::requested_attribute_paths;
use crate::wallet::init::WalletCertificateCrlVerifier;
use crate::wallet::state::AttestedKeyRegistrationDataAndConfig;

const REQUEST_INFO_DOC_FORMAT_KEY: &str = "docFormat";
const MSO_MDOC_DOC_FORMAT: &str = "mso_mdoc";

#[derive(Debug)]
pub enum CloseProximityDisclosureUpdate {
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

type CloseProximityDisclosableAttestation = DisclosableAttestation<(PartialMdoc, String)>;
type CloseProximityDisclosureAttestations = WalletDisclosureAttestations<usize, (PartialMdoc, String)>;

impl TryFrom<VpDisclosableAttestation> for CloseProximityDisclosableAttestation {
    type Error = DisclosureError;

    fn try_from(attestation: VpDisclosableAttestation) -> Result<Self, Self::Error> {
        attestation.try_map_partial_attestation(
            |WithKeyIdentifier {
                 key_identifier,
                 data: attestation,
             }| {
                let PartialAttestation::MsoMdoc(partial_mdoc) = attestation else {
                    // `ItemsRequest` only requests mdocs, and storage is expected to filter candidates by that format.
                    // Receiving an SD-JWT here means that soft contract was violated. Handle this defensively so the
                    // programming error remains observable as a distinct Sentry error.
                    return Err(DisclosureError::UnexpectedAttestationFormat);
                };

                Ok((*partial_mdoc, key_identifier))
            },
        )
    }
}

fn close_proximity_disclosure_proposal(
    attestations: IndexMap<usize, VecNonEmpty<VpDisclosableAttestation>>,
) -> Result<CloseProximityDisclosureAttestations, DisclosureError> {
    let attestations = attestations
        .into_iter()
        .map(|(id, candidates)| {
            let candidates = candidates
                .into_nonempty_iter()
                .map(CloseProximityDisclosableAttestation::try_from)
                .collect::<Result<VecNonEmpty<_>, _>>()?;

            Ok((id, candidates))
        })
        .collect::<Result<IndexMap<_, _>, DisclosureError>>()?;

    Ok(WalletDisclosureAttestations::Proposal(attestations))
}

#[derive(Debug, Clone, IsVariant)]
enum CloseProximityDisclosureSessionState {
    Advertising,
    SessionEstablished {
        session_transcript: Vec<u8>,
        device_request: Vec<u8>,
    },
    DisclosureProposed {
        session_transcript: Box<SessionTranscript>,
        organization: Box<Organization>,
        attestations: CloseProximityDisclosureAttestations,
    },
    Errored {
        #[expect(
            dead_code,
            reason = "will be used when processing close proximity disclosure errors (PVW-5710)"
        )]
        error: PlatformError,
        organization: Option<Box<Organization>>,
    },
}

#[derive(Debug)]
pub struct CloseProximityDisclosureSession {
    wrprc_strict_mode: bool,
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
                    let organization = match &*current_state {
                        CloseProximityDisclosureSessionState::DisclosureProposed { organization, .. } => {
                            Some(organization.clone())
                        }
                        _ => None,
                    };

                    *current_state = CloseProximityDisclosureSessionState::Errored {
                        error: error.clone(),
                        organization,
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
    #[error("not all ReaderAuths are identical")]
    #[category(critical)]
    InconsistentReaderAuths,

    #[error("invalid DocRequest")]
    InvalidDocRequest(#[from] mdoc::Error),

    #[error("reader access certificate validation failed")]
    InvalidReaderCertificate(#[source] CertificateVerifyError),

    #[error("reader access certificate CRL verification failed")]
    ReaderCertificateCrlVerification(#[source] CertificateCrlVerificationError),

    #[error("could not parse reader access certificate subject")]
    #[category(pd)]
    ReaderCertificateSubject(#[source] DistinguishedNameError),

    #[error("reader access certificate does not contain a relying-party subject")]
    #[category(pd)]
    ReaderCertificateRelyingParty(#[source] RelyingPartyError),

    #[error("registration certificate parsing failed")]
    #[category(pd)]
    RegistrationCertificateParsing(#[source] RegistrationCertificateRequestParseError),

    #[error("registration certificate verification failed")]
    #[category(pd)]
    RegistrationCertificateVerification(#[source] RegistrationCertificateEnvelopeError),

    #[error("registration certificate has invalid WRPAC binding")]
    #[category(pd)]
    RegistrationCertificateBinding(#[source] RegistrationCertificateValidationError),

    #[error("registration certificate status validation failed")]
    #[category(pd)]
    RegistrationCertificateStatus(#[source] RegistrationCertificateStatusValidationError),

    #[error("registration certificate request authorization failed")]
    #[category(pd)]
    RegistrationCertificateAuthorization(#[source] RegistrationCertificateAuthorizationError),

    #[error("unsupported requested document format: {doc_format}")]
    #[category(critical)]
    UnsupportedDocFormat { doc_format: String },

    #[error("received invalid CBOR from reader: {0}")]
    #[category(critical)]
    MalformedDeviceRequest(#[source] CborError),

    #[error("received invalid Device Request structure from reader: {0}")]
    #[category(critical)]
    InvalidDeviceRequest(#[source] CborError),

    #[error("could not encode error DeviceResponse: {0}")]
    #[category(critical)]
    DeviceResponseEncoding(#[source] CborError),

    #[error("received error from native close proximity bridge: {0}")]
    #[category(critical)]
    PlatformError(#[from] PlatformError),

    #[error("close proximity disclosure session disconnected")]
    #[category(expected)]
    Disconnected,

    #[error("failed creating device response: {0}")]
    #[category(pd)]
    DeviceResponse(#[source] mdoc::Error),
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
        CloseProximityDisclosureError::InvalidDeviceRequest(_)
        | CloseProximityDisclosureError::UnsupportedDocFormat { .. }
        | CloseProximityDisclosureError::RegistrationCertificateParsing(_) => {
            Some(DeviceResponseStatus::InvalidRequest)
        }
        CloseProximityDisclosureError::InconsistentReaderAuths
        | CloseProximityDisclosureError::InvalidDocRequest(_)
        | CloseProximityDisclosureError::InvalidReaderCertificate(_)
        | CloseProximityDisclosureError::ReaderCertificateCrlVerification(_)
        | CloseProximityDisclosureError::ReaderCertificateSubject(_)
        | CloseProximityDisclosureError::ReaderCertificateRelyingParty(_)
        | CloseProximityDisclosureError::RegistrationCertificateVerification(_)
        | CloseProximityDisclosureError::RegistrationCertificateBinding(_)
        | CloseProximityDisclosureError::RegistrationCertificateStatus(_)
        | CloseProximityDisclosureError::RegistrationCertificateAuthorization(_) => {
            Some(DeviceResponseStatus::GeneralError)
        }
        // These are either internal wallet errors or failures already handled by platform support,
        // so we do not expect to send a protocol-level error DeviceResponse for them.
        CloseProximityDisclosureError::DeviceResponseEncoding(_)
        | CloseProximityDisclosureError::PlatformError(_)
        | CloseProximityDisclosureError::Disconnected
        | CloseProximityDisclosureError::DeviceResponse(_) => None,
    }
}

fn encode_error_device_response(status: DeviceResponseStatus) -> Result<Vec<u8>, CloseProximityDisclosureError> {
    // Defensive: with the current fixed DeviceResponse shape we do not expect CBOR serialization to fail in practice.
    cbor_serialize(&DeviceResponse::error(status)).map_err(CloseProximityDisclosureError::DeviceResponseEncoding)
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
        wrprc_strict_mode: bool,
        callback: CloseProximityDisclosureCallback,
    ) -> Result<MdocUri, DisclosureError> {
        info!("Starting close proximity disclosure");

        self.check_session_preconditions()?;

        info!("Checking if there is already an active session");
        if self.session.is_some() {
            return Err(DisclosureError::SessionState);
        }

        let (qr, receiver) = CPC::start_qr_handover().await?;

        let session_state = Arc::new(Mutex::new(CloseProximityDisclosureSessionState::Advertising));

        let listener = spawn_listener(receiver, Arc::clone(&session_state), callback);
        self.session
            .replace(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
                wrprc_strict_mode,
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
        match self.session.take() {
            Some(Session::CloseProximityDisclosure(session)) => {
                session.listener.abort();
            }
            other => {
                self.session = other;
            }
        }
        let send_result = match error_device_response_status(error) {
            Some(status) => {
                let response = encode_error_device_response(status)?;
                Some(CPC::send_device_response(response).await)
            }
            None => None,
        };

        let should_stop_ble_server = match &send_result {
            Some(Ok(())) => false, // response sent successfully, the BLE server is stopped by platform_support
            Some(Err(_)) |         // attempted send failed, the BLE server may not have been stopped
            None => true,          // no response was sent, server should be stopped by us
        };

        let stop_result = if should_stop_ble_server {
            CPC::stop_ble_server().await
        } else {
            Ok(())
        };

        // A failed send should still trigger BLE cleanup, but if both operations fail we return
        // the send error because that is the primary action in this helper.
        send_result.transpose()?;
        stop_result?;
        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_close_proximity_disclosure(
        &mut self,
    ) -> Result<DisclosureProposalPresentation, DisclosureError>
    where
        SLC: StatusListClient,
    {
        info!("Continuing close proximity disclosure");

        self.check_session_preconditions()?;

        info!("Checking if there is an active close proximity session");
        let Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            wrprc_strict_mode,
            session_state,
            ..
        })) = self.session.as_ref()
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
        let verification_result = match &self.crl_verifier {
            WalletCertificateCrlVerifier::Http(verifier) => {
                verify_device_request_with_crl(
                    &device_request,
                    &session_transcript,
                    &TimeGenerator,
                    DisclosureTrustAnchors {
                        wrpac: wallet_config.wrpac_trust_anchors(),
                        wrprc: wallet_config.wrprc_trust_anchors(),
                    },
                    *wrprc_strict_mode,
                    verifier,
                    &self.status_list_client,
                )
                .await
            }
            #[cfg(any(test, feature = "test"))]
            WalletCertificateCrlVerifier::Mock(verifier) => {
                verify_device_request_with_crl(
                    &device_request,
                    &session_transcript,
                    &TimeGenerator,
                    DisclosureTrustAnchors {
                        wrpac: wallet_config.wrpac_trust_anchors(),
                        wrprc: wallet_config.wrprc_trust_anchors(),
                    },
                    *wrprc_strict_mode,
                    verifier,
                    &self.status_list_client,
                )
                .await
            }
        };
        let organization = match verification_result {
            Ok(organization) => organization,
            Err(error) => {
                self.send_close_proximity_error_response_and_stop(&error).await?;
                return Err(error.into());
            }
        };

        let (candidate_attestations, shared_data_with_relying_party_before) = self
            .prepare_disclosure(
                &device_request.items_requests().collect_vec(),
                &wallet_config.pid_attributes,
                &organization,
            )
            .await?;

        let candidate_attestations = candidate_attestations
            .into_iter()
            .flatten() // remove entries for which no suitable candidates were found
            .collect::<Vec<_>>();

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
                organization.clone(),
                shared_data_with_relying_party_before,
                session_type,
                disclosure_type,
                purpose,
            );

            *session_state.lock() = CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(session_transcript),
                organization: Box::new(organization),
                attestations: close_proximity_disclosure_proposal(
                    candidate_attestations.into_iter().enumerate().collect(),
                )?,
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
            organization: Box::new(organization.clone()),
            attestations: CloseProximityDisclosureAttestations::Missing,
        };

        Err(DisclosureError::AttributesNotAvailable(AttributesNotAvailable {
            organization: Box::new(organization),
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
        pin: WalletPin,
        attested_key_registration_data_and_config: AttestedKeyRegistrationDataAndConfig<AKH>,
    ) -> Result<(), DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
    {
        // If we do not have a proposal, this method should not have been called, so return an error.
        let CloseProximityDisclosureSessionState::DisclosureProposed {
            attestations,
            organization,
            session_transcript,
            ..
        } = close_proximity_session.session_state.lock().to_owned()
        else {
            self.session
                .replace(Session::CloseProximityDisclosure(close_proximity_session));
            return Err(DisclosureError::SessionState);
        };

        // Prepare the `RemoteEcdsaWscd` for signing using the provided PIN.
        let remote_wscd = match self
            .prepare_remote_instruction_client(pin, attested_key_registration_data_and_config)
            .await
        {
            Ok(remote_instruction_client) => RemoteEcdsaWscd::new(remote_instruction_client),
            Err(e) => {
                self.session
                    .replace(Session::CloseProximityDisclosure(close_proximity_session));
                return Err(e.into());
            }
        };

        // Note that this will panic if any of the indices are out of bounds.
        let attestations = attestations.select_proposal(selected_indices);

        // There is guaranteed to be at least one attestation because of the logic in `start_disclosure()`.
        let attestation_values = VecNonEmpty::try_from(attestations.into_values().collect_vec()).unwrap();

        // NOTE: If the disclosure fails and is retried, the disclosure count will jump by
        //       more than 1, since the same copies are shared with the verifier again.
        //       It is necessary to increment the disclosure count before sending the attestations
        //       to the verifier, as we do not know if disclosure fails before or after the
        //       verifier has received the attributes.
        let (attestation_presentations, result) = self
            .increment_usage_count_and_collect_presentations(attestation_values.clone())
            .await;

        let disclosure_type = DisclosureType::Regular;

        if let Err(error) = result {
            // If storing the event results in an error, log it but do nothing else.
            let _ = self
                .store_disclosure_event(
                    Utc::now(),
                    Some(attestation_presentations),
                    &organization,
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
            attestation_values,
            session_transcript.as_ref(),
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
                        &organization,
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

        // Actually perform the disclosure by sending the device response to the reader. If the
        // transport is already gone, surface the domain-specific disconnected state instead of a
        // generic platform error.
        CPC::send_device_response(cbor_serialize(&device_response).unwrap())
            .await
            .map_err(|error| {
                error!("Error while sending close proximity device response: {error}");
                DisclosureError::from(CloseProximityDisclosureError::Disconnected)
            })?;

        // Disclosure is now successful. Any errors that occur after this point will result in the `Wallet` not having
        // an active disclosure session anymore.
        self.store_disclosure_event(
            Utc::now(),
            Some(attestation_presentations),
            &organization,
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

        let (event, send_result, sent_termination) = match state {
            CloseProximityDisclosureSessionState::DisclosureProposed { organization, .. } => (
                Some((organization, EventStatus::Cancelled)),
                CPC::send_session_termination().await,
                true,
            ),

            CloseProximityDisclosureSessionState::Errored {
                organization: Some(organization),
                ..
            } => (Some((organization, EventStatus::Error)), Ok(()), false),

            _ => (None, Ok(()), false),
        };

        // Keep the cleanup decision separate from the call itself: if sending the termination
        // status failed, we still need to stop BLE before returning that send error.
        let should_stop_ble_server = !sent_termination || send_result.is_err();

        let stop_result = if should_stop_ble_server {
            CPC::stop_ble_server().await
        } else {
            Ok(())
        };

        send_result?;
        stop_result?;

        // Only store the event if the session is past SessionEstablished state (i.e. DisclosureProposed or later)
        if let Some((organization, status)) = event {
            self.store_disclosure_event(
                Utc::now(),
                // TODO (PVW-5078): Store credential requests in disclosure event.
                None,
                &organization,
                DisclosureType::Regular,
                status,
                DataDisclosed::NotDisclosed,
            )
            .await?;
        }

        Ok(())
    }

    async fn create_close_proximity_device_response<'a, K, W>(
        attestations: impl IntoNonEmptyIterator<Item = &'a CloseProximityDisclosableAttestation>,
        session_transcript: &SessionTranscript,
        wscd: &W,
    ) -> Result<DeviceResponse, DisclosureError>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K>,
    {
        // Gather all partial mdocs presentations by cloning the attestations held in the session, as disclosing
        // attestations needs to be retryable.
        let partial_mdocs = attestations
            .into_nonempty_iter()
            .map(|attestation| attestation.partial_attestation().clone())
            .collect();

        // Create the device response, casting any `InstructionError` that occurs during signing
        // to `RemoteEcdsaKeyError::Instruction`.
        DeviceResponse::sign_from_partial_mdocs(partial_mdocs, session_transcript, wscd)
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

// Temporary fallback for readers without WRPRC support. Keep the required organization fields and its stored
// representation compatible with the strict implementation so that reverting this exception preserves history.
// Keep the support placeholder in sync with the Flutter organization detail screen.
const WRPRC_SUPPORT_UNAVAILABLE_URI: &str = "https://example.invalid/wrprc-support-unavailable";

fn organization_without_wrprc(subject: RelyingParty) -> Organization {
    let (display_name, country_code, legal_name, identifier, person_type) = match subject {
        RelyingParty::LegalPerson {
            common_name,
            country_name,
            organization_name,
            organization_identifier,
        } => (
            common_name,
            country_name,
            organization_name,
            organization_identifier,
            SubjectType::LegalPerson,
        ),
        RelyingParty::NaturalPerson {
            common_name,
            country_name,
            serial_number,
            surname,
            given_name,
        } => (
            common_name,
            country_name,
            format!("{given_name} {surname}"),
            serial_number,
            SubjectType::NaturalPerson,
        ),
    };

    Organization {
        display_name,
        legal_name,
        description: vec![],
        web_url: None,
        identifier,
        country_code,
        privacy_policy_url: None,
        purpose: None,
        support_uri: WRPRC_SUPPORT_UNAVAILABLE_URI.to_owned(),
        public_body: None,
        person_type,
    }
}

/// Verify reader authentication, WRPRC envelopes, binding, expiry, status and request authorization, requiring a valid
/// CRL for the complete WRPAC chain.
#[expect(
    clippy::too_many_arguments,
    reason = "Pass the temporary WRPRC strict-mode flag explicitly so it is easy to remove"
)]
async fn verify_device_request_with_crl(
    device_request: &DeviceRequest,
    session_transcript: &SessionTranscript,
    time: &(impl Generator<DateTime<Utc>> + Clone + Send + Sync + 'static),
    trust_anchors: DisclosureTrustAnchors<'_>,
    wrprc_strict_mode: bool,
    crl_verifier: &CertificateCrlVerifier<impl CrlFetcher>,
    status_list_client: &Arc<impl StatusListClient>,
) -> Result<Organization, CloseProximityDisclosureError> {
    // Verify all `DocRequest` entries in parallel and make sure the resulting certificates are all exactly equal.
    let certificate = try_join_all(device_request.doc_requests.iter().map(
        async |doc_request| -> Result<_, CloseProximityDisclosureError> {
            doc_request
                .verify_with_crl(session_transcript, time, trust_anchors.wrpac, crl_verifier)
                .await
                .map_err(|error| match error {
                    mdoc::Error::Cose(CoseError::CertificateVerify(error)) => {
                        CloseProximityDisclosureError::InvalidReaderCertificate(error)
                    }
                    mdoc::Error::Cose(CoseError::CertificateCrl(error)) => {
                        CloseProximityDisclosureError::ReaderCertificateCrlVerification(error)
                    }
                    error => CloseProximityDisclosureError::InvalidDocRequest(error),
                })
        },
    ))
    .await?
    .into_iter()
    // There is always at least one certificate, as `device_request.doc_requests` is non-empty and verification
    // either returns a certificate or an error.
    .unique()
    .exactly_one()
    .map_err(|_| CloseProximityDisclosureError::InconsistentReaderAuths)?;

    verify_requested_doc_formats(device_request)?;

    let access_certificate_subject = certificate
        .to_distinguished_name()
        .map_err(CloseProximityDisclosureError::ReaderCertificateSubject)?;
    let access_subject = RelyingParty::try_from(access_certificate_subject)
        .map_err(CloseProximityDisclosureError::ReaderCertificateRelyingParty)?;

    // Temporary interoperability exception: only an entirely absent WRPRC set is allowed. Mixed presence and
    // malformed values still go through the strict checks below. Reader authentication and CRLs were verified above.
    if !wrprc_strict_mode
        && device_request.items_requests().all(|request| {
            request
                .request_info
                .as_ref()
                .is_none_or(|info| !info.contains_key(REQUEST_INFO_WRPRC_KEY))
        })
    {
        return Ok(organization_without_wrprc(access_subject));
    }

    let revocation_verifier = RevocationVerifier::new_with_defaults(Arc::clone(status_list_client), time.clone());
    let registration_certificates = device_request.doc_requests.nonempty_iter().map(
        async |doc_request| -> Result<_, CloseProximityDisclosureError> {
            let items_request = &doc_request.items_request.0;
            let registration_certificate = RegistrationCertificateEnvelope::from_request_info(items_request)
                .map_err(CloseProximityDisclosureError::RegistrationCertificateParsing)?;
            let verified_registration_certificate =
                verify_registration_certificate_envelope(&registration_certificate, trust_anchors.wrprc, time)
                    .map_err(CloseProximityDisclosureError::RegistrationCertificateVerification)?;
            let payload = verified_registration_certificate.into_payload();
            let bound_registration_certificate = payload
                .validate_binding(&access_subject)
                .map_err(CloseProximityDisclosureError::RegistrationCertificateBinding)?;

            let registration_certificate = bound_registration_certificate
                .validate_status(&revocation_verifier, trust_anchors.wrprc, time)
                .await
                .map_err(CloseProximityDisclosureError::RegistrationCertificateStatus)?;

            registration_certificate
                .validate_mdoc_items_request_authorization(items_request)
                .map_err(CloseProximityDisclosureError::RegistrationCertificateAuthorization)?;

            Ok(registration_certificate)
        },
    );

    let (first, remaining) = registration_certificates.next();
    let (first, _) = try_join(first, try_join_all(remaining)).await?;

    // TODO (PVW-6407): Pending specification clarification for multiple WRPRCs, use the first DocRequest's
    // organization and purpose for the whole disclosure. Every WRPRC must still pass all checks above.
    Ok(Organization::from(&first))
}

fn verify_requested_doc_formats(device_request: &DeviceRequest) -> Result<(), CloseProximityDisclosureError> {
    for items_request in device_request.items_requests() {
        let Some(requested_doc_format) = items_request
            .request_info
            .as_ref()
            .and_then(|request_info| request_info.get(REQUEST_INFO_DOC_FORMAT_KEY))
        else {
            continue;
        };

        match requested_doc_format {
            Value::Text(format) if format == MSO_MDOC_DOC_FORMAT => {}
            Value::Text(format) => {
                return Err(CloseProximityDisclosureError::UnsupportedDocFormat {
                    doc_format: format.to_owned(),
                });
            }
            other => {
                return Err(CloseProximityDisclosureError::UnsupportedDocFormat {
                    doc_format: format!("{other:?}"),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use attestation_data::attributes::Attribute;
    use attestation_data::disclosure_type::DisclosureType;
    use attestation_data::organization::Organization;
    use attestation_data::registration_certificate::ParsedRegistrationCertificate;
    use attestation_data::registration_certificate::REQUEST_INFO_WRPRC_KEY;
    use attestation_data::registration_certificate::RegistrationCertificateAuthorizationError;
    use attestation_data::registration_certificate::RegistrationCertificateEnvelopeError;
    use attestation_data::registration_certificate::RegistrationCertificateRequestParseError;
    use attestation_data::registration_certificate::RegistrationCertificateStatusValidationError;
    use attestation_data::registration_certificate::RegistrationCertificateValidationError;
    use attestation_data::registration_certificate::SubjectType;
    use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
    use attestation_data::registration_certificate::mock::RegistrationCertificateFixture;
    use attestation_data::registration_certificate::mock::STATUS_LIST_URI;
    use attestation_data::registration_certificate::mock::verifier_registration_certificate_payload;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_FAMILY_NAME;
    use attestation_types::pid_constants::PID_GIVEN_NAME;
    use attestation_types::pid_constants::PID_MDOC_DOCTYPE;
    use chrono::Utc;
    use ciborium::value::Value;
    use crypto::WithVerifyingKey;
    use crypto::mock_remote::DisclosureMockRemoteWscd;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::p256_der::DerSignature;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::crl::CertificateCrlVerificationError;
    use crypto::x509::crl::mock::MockCertificateCrlVerifier;
    use dcql::Query;
    use dcql::normalized::NormalizedCredentialRequests;
    use entity::disclosure_event::EventStatus;
    use futures::future::join_all;
    use indexmap::IndexMap;
    use jwt::error::JwtParseError;
    use jwt::error::JwtVerifyError;
    use jwt::error::JwtX5cVerifyError;
    use jwt::error::ValidationErrorKind;
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
    use mockall::predicate::always;
    use mockall::predicate::eq;
    use openid4vc::disclosure_session::DisclosureTrustAnchors;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Signer;
    use p256::elliptic_curve::Generate;
    use parking_lot::Mutex;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannel;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureChannelImpl;
    use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate as PlatformUpdate;
    use platform_support::close_proximity_disclosure::MockCloseProximityDisclosureClient;
    use rstest::rstest;
    use serde_json::json;
    use serial_test::serial;
    use token_status_list::status_list::StatusList;
    use token_status_list::status_list::StatusType;
    use token_status_list::status_list_token::StatusListToken;
    use token_status_list::verification::client::StatusListClientError;
    use token_status_list::verification::client::mock::MockStatusListClient;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wallet_account::messages::errors::AccountError;
    use wallet_account::messages::errors::IncorrectPinData;
    use wallet_account::messages::errors::PinTimeoutData;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::instructions::Sign;
    use wallet_account::messages::instructions::SignResult;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use super::CloseProximityDisclosableAttestation;
    use super::CloseProximityDisclosureAttestations;
    use super::CloseProximityDisclosureError;
    use super::CloseProximityDisclosureSession;
    use super::CloseProximityDisclosureSessionState;
    use super::CloseProximityDisclosureUpdate;
    use super::DeviceResponseStatus;
    use super::PlatformError;
    use super::WRPRC_SUPPORT_UNAVAILABLE_URI;
    use super::close_proximity_disclosure_proposal;
    use super::verify_device_request_with_crl;
    use crate::DisclosureAttestationOptions;
    use crate::DisclosureProposalPresentation;
    use crate::Pin;
    use crate::account_provider::AccountProviderError;
    use crate::account_provider::AccountProviderResponseError;
    use crate::attestation::mock::EmptyPresentationConfig;
    use crate::config::LocalConfigurationRepository;
    use crate::errors::StorageError;
    use crate::storage::ChangePinData;
    use crate::storage::DisclosableAttestation;
    use crate::storage::InstructionData;
    use crate::storage::StoredAttestationCopy;
    use crate::wallet::DisclosureError;
    use crate::wallet::Session;
    use crate::wallet::disclosure::VpDisclosableAttestation;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WRPAC_CA;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_wallet_configuration;
    use crate::wallet::test::create_wp_result;
    use crate::wallet::test::example_pid_payload_and_metadata;
    use crate::wallet::test::example_pid_stored_attestation_copy;
    use crate::wallet::test::example_stored_attestation_copy;

    const ISO_MDL_DOCTYPE: &str = "org.iso.18013.5.1.mDL";
    const ISO_MDL_NAMESPACE: &str = "org.iso.18013.5.1";
    const EUDI_PID_DOCTYPE: &str = "eu.europa.ec.eudi.pid.1";
    const PHOTO_ID_DOCTYPE_LOWERCASE: &str = "org.iso.23220.photoid.1";
    const PHOTO_ID_BASE_NAMESPACE: &str = "org.iso.23220.1";
    const PHOTO_ID_NAMESPACE: &str = "org.iso.23220.photoid.1";

    static REGISTRATION_CERTIFICATE_AUTHORITY: LazyLock<MockRegistrationCertificateAuthority> =
        LazyLock::new(MockRegistrationCertificateAuthority::new);
    static REGISTRATION_CERTIFICATE_REQUEST_INFO: LazyLock<IndexMap<String, Value>> = LazyLock::new(|| {
        let certificate = REGISTRATION_CERTIFICATE_AUTHORITY.sign_cwt(&registration_certificate_payload());
        IndexMap::from_iter([(REQUEST_INFO_WRPRC_KEY.to_string(), Value::Bytes(certificate))])
    });

    #[derive(Debug, Clone, Copy)]
    enum EnvelopeFormat {
        Jwt,
        Cwt,
    }

    impl EnvelopeFormat {
        fn sign(
            self,
            authority: &MockRegistrationCertificateAuthority,
            payload: &RegistrationCertificateFixture,
        ) -> Vec<u8> {
            match self {
                Self::Jwt => authority.sign_jwt(payload),
                Self::Cwt => authority.sign_cwt(payload),
            }
        }
    }

    fn close_proximity_wallet_configuration() -> WalletConfiguration {
        WalletConfiguration {
            wrprc_trust_anchors: REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors.clone(),
            ..create_wallet_configuration()
        }
    }

    fn registration_certificate_payload() -> RegistrationCertificateFixture {
        let reader = WRPAC_CA.generate_wrpac_verifier_mock().unwrap();
        verifier_registration_certificate_payload(reader.certificate(), Query::new_mock_mdoc_pid_example())
    }

    fn registration_certificate_organization() -> Organization {
        let certificate: ParsedRegistrationCertificate =
            serde_json::from_value(registration_certificate_payload().0).unwrap();
        Organization::from(&certificate)
    }

    fn registration_certificate_status_list_client() -> Arc<MockStatusListClient> {
        let mut client = MockStatusListClient::new();
        client
            .expect_fetch()
            .with(eq(STATUS_LIST_URI.parse::<Url>().unwrap()))
            .returning(|_| Ok(REGISTRATION_CERTIFICATE_AUTHORITY.status_list_client.0.clone()));
        Arc::new(client)
    }

    fn add_matching_registration_certificate(
        items_request: &mut ItemsRequest,
        mut payload: RegistrationCertificateFixture,
    ) {
        payload.0["credentials"] = json!([{
            "format": "mso_mdoc",
            "meta": { "doctype_value": items_request.doc_type },
            "claim": items_request.claims().map(|path| json!({ "path": path })).collect::<Vec<_>>()
        }]);
        let certificate = REGISTRATION_CERTIFICATE_AUTHORITY.sign_cwt(&payload);
        items_request
            .request_info
            .get_or_insert_default()
            .insert(REQUEST_INFO_WRPRC_KEY.to_string(), Value::Bytes(certificate));
    }

    fn pid_given_name_items_request() -> ItemsRequest {
        interop_items_request(
            PID_MDOC_DOCTYPE,
            &[(PID_MDOC_DOCTYPE, &[PID_GIVEN_NAME][..])],
            Some(REGISTRATION_CERTIFICATE_REQUEST_INFO.clone()),
        )
    }

    fn interop_items_request(
        doc_type: &str,
        name_spaces: &[(&str, &[&str])],
        request_info: Option<IndexMap<String, Value>>,
    ) -> ItemsRequest {
        ItemsRequest {
            doc_type: doc_type.to_owned(),
            name_spaces: name_spaces
                .iter()
                .map(|(name_space, attributes)| {
                    (
                        (*name_space).to_owned(),
                        attributes
                            .iter()
                            .map(|attribute| ((*attribute).to_owned(), true))
                            .collect::<IndexMap<_, _>>()
                            .try_into()
                            .unwrap(),
                    )
                })
                .collect::<IndexMap<_, _>>()
                .try_into()
                .unwrap(),
            request_info,
        }
    }

    #[rstest]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_start_close_proximity_disclosure(#[values(true, false)] wrprc_strict_mode: bool) {
        let context = MockCloseProximityDisclosureClient::start_qr_handover_context();
        context.expect().once().returning(|| {
            let (_channel, receiver) = CloseProximityDisclosureChannelImpl::new();
            Ok(("some_qr_code".to_owned(), receiver))
        });

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let qr = wallet
            .start_close_proximity_disclosure(wrprc_strict_mode, Box::new(|_| Box::pin(async {})))
            .await
            .expect("starting proximity disclosure should succeed");

        assert_eq!(qr.as_ref(), "mdoc:some_qr_code");
        assert!(matches!(
            wallet.session.take(),
            Some(Session::CloseProximityDisclosure(
                CloseProximityDisclosureSession { wrprc_strict_mode: captured, .. }
            )) if captured == wrprc_strict_mode
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
            .start_close_proximity_disclosure(
                true,
                Box::new(move |update| {
                    let _ = tx.send(update);
                    Box::pin(async {})
                }),
            )
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
            wrprc_strict_mode: true,
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
            wrprc_strict_mode: true,
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
        context.expect().never();

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let organization = registration_certificate_organization();

        // When the session is in the DisclosureProposed state (the reader sent a device request),
        // a Cancelled event should be stored with the WRPRC organization and no disclosed data.
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(organization.clone()),
                eq(EventStatus::Cancelled),
                eq(DisclosureType::Regular),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let session = CloseProximityDisclosureSession {
            wrprc_strict_mode: true,
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(CborSeq(SessionTranscriptKeyed {
                    device_engagement_bytes: None,
                    e_reader_key_bytes: None,
                    handover: Handover::QrHandover,
                })),
                organization: Box::new(organization),
                attestations: CloseProximityDisclosureAttestations::Missing,
            })),
        };

        wallet
            .terminate_close_proximity_disclosure_session(session)
            .await
            .expect("terminating close proximity disclosure session should succeed");
    }

    #[rstest]
    #[case::cancelled(false)]
    #[case::platform_error(true)]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_terminate_close_proximity_disclosure_session_logs_organization(#[case] platform_error: bool) {
        let send_context = MockCloseProximityDisclosureClient::send_session_termination_context();
        send_context
            .expect()
            .times(usize::from(!platform_error))
            .returning(|| Ok(()));

        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().times(usize::from(platform_error)).returning(|| Ok(()));

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (organization, _) = setup_close_proximity_disclosure_proposed_session(&mut wallet);

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(organization),
                eq(if platform_error {
                    EventStatus::Error
                } else {
                    EventStatus::Cancelled
                }),
                eq(DisclosureType::Regular),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let Some(Session::CloseProximityDisclosure(mut session)) = wallet.session.take() else {
            panic!("expected a close proximity disclosure session");
        };

        if platform_error {
            let (channel, receiver) = CloseProximityDisclosureChannelImpl::new();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            session.listener.abort();
            session.listener = super::spawn_listener(
                receiver,
                Arc::clone(&session.session_state),
                Box::new(move |update| {
                    tx.send(update).unwrap();
                    Box::pin(async {})
                }),
            );
            channel
                .send_update(PlatformUpdate::Error {
                    error: PlatformError::PlatformError {
                        reason: "transport failed".to_string(),
                    },
                })
                .await
                .unwrap();
            assert_matches!(rx.recv().await, Some(CloseProximityDisclosureUpdate::Errored(_)));
        }

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
        setup_close_proximity_disclosure_proposed_session(&mut wallet);

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
    async fn setup_close_proximity_disclosure_session(wallet: &mut TestWalletMockStorage, items_request: ItemsRequest) {
        let key_pair = WRPAC_CA.generate_wrpac_verifier_mock_with_crl().unwrap();

        let cose_key: CoseKey = (&key_pair.verifying_key().await.unwrap()).try_into().unwrap();
        let session_transcript = SessionTranscript::new_qr(cose_key, None);

        let doc_request = create_doc_request(items_request, &session_transcript, &key_pair).await;

        wallet.session = Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            wrprc_strict_mode: true,
            listener: tokio::task::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::SessionEstablished {
                session_transcript: cbor_serialize(&session_transcript).unwrap(),
                device_request: cbor_serialize(&DeviceRequest::from_doc_requests(vec_nonempty![doc_request])).unwrap(),
            })),
        }));
    }

    fn install_session_established_close_proximity_session(
        wallet: &mut TestWalletMockStorage,
        session_transcript: Vec<u8>,
        device_request: Vec<u8>,
    ) {
        wallet.session = Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            wrprc_strict_mode: true,
            listener: tokio::task::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::SessionEstablished {
                session_transcript,
                device_request,
            })),
        }));
    }

    async fn continue_close_proximity_disclosure_with_attributes(
        attributes_available: bool,
    ) -> (Result<DisclosureProposalPresentation, DisclosureError>, Organization) {
        // Populate a registered wallet with an example PID.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked_with_config(
            WalletDeviceVendor::Apple,
            close_proximity_wallet_configuration(),
        )
        .await;
        wallet.status_list_client = registration_certificate_status_list_client();

        let items_request = pid_given_name_items_request();

        setup_close_proximity_disclosure_session(&mut wallet, items_request).await;
        let expected_organization = registration_certificate_organization();

        // Create three PID attestations, which differ only in their given name.
        let (mut pid_credential_payload, pid_metadata, _) = example_pid_payload_and_metadata(Format::MsoMdoc);
        let attributes = &mut pid_credential_payload.previewable_payload.attributes;
        attributes.overwrite([PID_MDOC_DOCTYPE, PID_GIVEN_NAME], Attribute::Text("Jane".to_string()));
        let pid1 = example_stored_attestation_copy(
            Format::MsoMdoc,
            pid_credential_payload.clone(),
            pid_metadata.clone(),
            &SigningKey::generate(),
        );

        let (pid2, _) = example_pid_stored_attestation_copy(Format::MsoMdoc);

        let attributes = &mut pid_credential_payload.previewable_payload.attributes;
        attributes.overwrite([PID_MDOC_DOCTYPE, PID_GIVEN_NAME], Attribute::Text("John".to_string()));
        let pid3 = example_stored_attestation_copy(
            Format::MsoMdoc,
            pid_credential_payload,
            pid_metadata,
            &SigningKey::generate(),
        );

        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_credential_kinds()
            .withf(move |credential_kinds, _| {
                *credential_kinds == HashSet::from([CredentialKind::new(Format::MsoMdoc, PID_MDOC_DOCTYPE.to_owned())])
            })
            .once()
            .return_once(move |_, _| {
                Ok(if attributes_available {
                    vec![pid1, pid2, pid3]
                } else {
                    vec![]
                })
            });

        // The wallet will check in the database if data was shared with the RP before.
        wallet
            .mut_storage()
            .expect_did_share_data_with_relying_party()
            .with(eq(expected_organization.clone()))
            .once()
            .returning(|_| Ok(false));

        // Starting disclosure should not cause attestation copy usage counts to be incremented.
        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .never();

        // Starting disclosure should not cause a disclosure event to be recorded yet.
        wallet.mut_storage().expect_log_disclosure_event().never();

        let result = wallet.continue_close_proximity_disclosure().await;

        wallet.mut_storage().checkpoint();

        let Some(Session::CloseProximityDisclosure(session)) = wallet.session.as_ref() else {
            panic!("expected a close proximity disclosure session");
        };
        assert_matches!(
            &*session.session_state.lock(),
            CloseProximityDisclosureSessionState::DisclosureProposed { organization, .. }
                if **organization == expected_organization
        );

        (result, expected_organization)
    }

    /// Continue disclosure from an established session with the requested attributes available.
    #[tokio::test]
    async fn test_wallet_continue_close_proximity_disclosure() {
        let (result, expected_organization) = continue_close_proximity_disclosure_with_attributes(true).await;

        let proposal = result.expect("starting disclosure should succeed");
        assert_eq!(proposal.organization, expected_organization);
        assert!(!proposal.shared_data_with_relying_party_before);
        assert_eq!(proposal.attestation_options.len().get(), 1);
        assert!(matches!(
            proposal.attestation_options.first(),
            DisclosureAttestationOptions::Multiple(options) if options.len().get() == 3
        ));
    }

    #[tokio::test]
    async fn test_wallet_continue_close_proximity_disclosure_missing_attributes() {
        let (result, expected_organization) = continue_close_proximity_disclosure_with_attributes(false).await;

        assert_matches!(
            result,
            Err(DisclosureError::AttributesNotAvailable(missing))
                if *missing.organization == expected_organization
                    && !missing.shared_data_with_relying_party_before
        );
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
        stop_context.expect().never();

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
        stop_context.expect().never();

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

    #[rstest]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_reader_auth_failure(
        #[values(true, false)] wrprc_strict_mode: bool,
    ) {
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
        stop_context.expect().never();

        let items_request = pid_given_name_items_request();
        let (mut device_request, session_transcript, _trust_anchors) =
            setup_device_request(vec![items_request], None, true).await;
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

        let Some(Session::CloseProximityDisclosure(session)) = wallet.session.as_mut() else {
            panic!("expected a close proximity disclosure session");
        };
        session.wrprc_strict_mode = wrprc_strict_mode;
        let result = wallet.continue_close_proximity_disclosure().await;

        assert_matches!(
            result,
            Err(DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::InvalidDocRequest(mdoc::Error::MissingReaderAuthentication)
            ))
        );
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_unsupported_doc_format() {
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
        stop_context.expect().never();

        let items_request = interop_items_request(
            EUDI_PID_DOCTYPE,
            &[(EUDI_PID_DOCTYPE, &["family_name", "given_name"][..])],
            Some(IndexMap::from_iter([(
                "docFormat".to_string(),
                Value::Text("sd-jwt+kb".to_string()),
            )])),
        );
        let (device_request, session_transcript, _trust_anchors) =
            setup_device_request(vec![items_request], None, true).await;

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
                CloseProximityDisclosureError::UnsupportedDocFormat { doc_format }
            )) if doc_format == "sd-jwt+kb"
        );
        assert!(wallet.session.is_none());
    }

    async fn assert_registration_certificate_rejected(
        registration_certificate: Option<Value>,
        expected_status: DeviceResponseStatus,
        status_list_result: Option<Result<StatusListToken, StatusListClientError>>,
        wrprc_strict_mode: bool,
    ) -> DisclosureError {
        let send_context = MockCloseProximityDisclosureClient::send_device_response_context();
        send_context
            .expect()
            .once()
            .withf(move |response| {
                let device_response: DeviceResponse = cbor_deserialize(response.as_slice()).unwrap();
                device_response.documents.is_none()
                    && device_response.document_errors.is_none()
                    && device_response.status == expected_status
            })
            .returning(|_| Ok(()));

        let stop_context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        stop_context.expect().never();

        let valid_request = pid_given_name_items_request();
        let mut invalid_request = pid_given_name_items_request();
        invalid_request.request_info =
            registration_certificate.map(|value| IndexMap::from_iter([(REQUEST_INFO_WRPRC_KEY.to_string(), value)]));
        // Sign the request after altering the WRPRC so that reader authentication remains valid.
        let (device_request, session_transcript, _) =
            setup_device_request(vec![valid_request, invalid_request], None, true).await;

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked_with_config(
            WalletDeviceVendor::Apple,
            close_proximity_wallet_configuration(),
        )
        .await;
        let mut status_list_client = MockStatusListClient::new();
        let status_list_result =
            status_list_result.unwrap_or_else(|| Ok(REGISTRATION_CERTIFICATE_AUTHORITY.status_list_client.0.clone()));
        status_list_client
            .expect_fetch()
            .with(eq(STATUS_LIST_URI.parse::<Url>().unwrap()))
            .once()
            .return_once(move |_| status_list_result);
        wallet.status_list_client = Arc::new(status_list_client);
        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_credential_kinds()
            .never();
        install_session_established_close_proximity_session(
            &mut wallet,
            cbor_serialize(&session_transcript).unwrap(),
            cbor_serialize(&device_request).unwrap(),
        );

        let Some(Session::CloseProximityDisclosure(session)) = wallet.session.as_mut() else {
            panic!("expected a close proximity disclosure session");
        };
        session.wrprc_strict_mode = wrprc_strict_mode;
        let error = wallet.continue_close_proximity_disclosure().await.unwrap_err();
        assert!(wallet.session.is_none());

        error
    }

    #[rstest]
    #[case::missing(None)]
    #[case::wrong_type(Some(Value::Text("not a byte string".to_string())))]
    #[case::malformed_cwt(Some(Value::Bytes(vec![0xff])))]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_registration_certificate_parse_error(
        #[case] registration_certificate: Option<Value>,
        #[values(true, false)] wrprc_strict_mode: bool,
    ) {
        let error = assert_registration_certificate_rejected(
            registration_certificate,
            DeviceResponseStatus::InvalidRequest,
            None,
            wrprc_strict_mode,
        )
        .await;

        assert_matches!(
            error,
            DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::RegistrationCertificateParsing(_)
            )
        );
    }

    #[rstest]
    #[case::untrusted_chain(&MockRegistrationCertificateAuthority::new(), false)]
    #[case::tampered_signature(&REGISTRATION_CERTIFICATE_AUTHORITY, true)]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_registration_certificate_verification_error(
        #[case] authority: &MockRegistrationCertificateAuthority,
        #[case] tamper_signature: bool,
        #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    ) {
        let mut certificate = format.sign(authority, &registration_certificate_payload());
        if tamper_signature {
            match format {
                EnvelopeFormat::Jwt => {
                    // Change the first base64url signature character, preserving the JWT structure and encoding.
                    let signature_start = certificate.iter().rposition(|byte| *byte == b'.').unwrap() + 1;
                    certificate[signature_start] = if certificate[signature_start] == b'A' {
                        b'B'
                    } else {
                        b'A'
                    };
                }
                EnvelopeFormat::Cwt => {
                    // COSE_Sign1 ends with the signature byte string; preserve the envelope structure.
                    *certificate.last_mut().unwrap() ^= 1;
                }
            }
        }
        let error = assert_registration_certificate_rejected(
            Some(Value::Bytes(certificate)),
            DeviceResponseStatus::GeneralError,
            None,
            true,
        )
        .await;

        assert_matches!(
            error,
            DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::RegistrationCertificateVerification(ref error)
            ) if matches!(
                (format, error),
                (EnvelopeFormat::Jwt, RegistrationCertificateEnvelopeError::Jwt(_))
                    | (EnvelopeFormat::Cwt, RegistrationCertificateEnvelopeError::Cwt(_))
            )
        );
    }

    #[rstest]
    #[case::mismatching_identifier(
        |payload: &mut RegistrationCertificateFixture| payload.0["sub"] = json!("other-relying-party"),
        |error: &CloseProximityDisclosureError| {
            matches!(error, CloseProximityDisclosureError::RegistrationCertificateBinding(
                RegistrationCertificateValidationError::SubjectIdentifierMismatch {
                    field: "organizationIdentifier"
                }
            ))
        }
    )]
    #[case::mismatching_subject_type(
        |payload: &mut RegistrationCertificateFixture| {
            payload.0["sub_ln"] = json!(null);
            payload.0["sub_gn"] = json!("Test");
            payload.0["sub_fn"] = json!("Person");
        },
        |error: &CloseProximityDisclosureError| {
            matches!(error, CloseProximityDisclosureError::RegistrationCertificateBinding(
                RegistrationCertificateValidationError::AccessCertificateSubjectTypeMismatch {
                    registration: SubjectType::NaturalPerson,
                    access: SubjectType::LegalPerson,
                }
            ))
        }
    )]
    #[case::expired(
        |payload: &mut RegistrationCertificateFixture| {
            let now = Utc::now().timestamp();
            payload.0["iat"] = json!(now - 180);
            // Stay beyond the JWT validator's 60-second leeway, regardless of when verification runs.
            payload.0["exp"] = json!(now - 120);
        },
        |error: &CloseProximityDisclosureError| {
            matches!(error, CloseProximityDisclosureError::RegistrationCertificateVerification(
                RegistrationCertificateEnvelopeError::Validity(RegistrationCertificateValidationError::Expired { .. })
            )) || matches!(error, CloseProximityDisclosureError::RegistrationCertificateVerification(
                RegistrationCertificateEnvelopeError::Jwt(JwtX5cVerifyError::JwtVerify(
                    JwtVerifyError::Validation(error)
                ))
            ) if *error.kind() == ValidationErrorKind::ExpiredSignature)
        }
    )]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_registration_certificate_validation_error(
        #[case] modify_payload: fn(&mut RegistrationCertificateFixture),
        #[case] matches_error: fn(&CloseProximityDisclosureError) -> bool,
        #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    ) {
        let mut payload = registration_certificate_payload();
        modify_payload(&mut payload);
        let certificate = format.sign(&REGISTRATION_CERTIFICATE_AUTHORITY, &payload);
        let error = assert_registration_certificate_rejected(
            Some(Value::Bytes(certificate)),
            DeviceResponseStatus::GeneralError,
            None,
            true,
        )
        .await;

        assert_matches!(
            error,
            DisclosureError::CloseProximityDisclosureSessionError(error) if matches_error(&error)
        );
    }

    #[rstest]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_registration_certificate_authorization_error(
        #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    ) {
        let mut payload = registration_certificate_payload();
        payload.0["credentials"][0]["claim"] = json!([{ "path": [PID_MDOC_DOCTYPE, PID_FAMILY_NAME] }]);
        let certificate = format.sign(&REGISTRATION_CERTIFICATE_AUTHORITY, &payload);
        let error = assert_registration_certificate_rejected(
            Some(Value::Bytes(certificate)),
            DeviceResponseStatus::GeneralError,
            Some(Ok(REGISTRATION_CERTIFICATE_AUTHORITY.status_list_client.0.clone())),
            true,
        )
        .await;

        assert_matches!(
            error,
            DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::RegistrationCertificateAuthorization(
                    RegistrationCertificateAuthorizationError::UnauthorizedItemsRequest(doc_type)
                )
            ) if doc_type == PID_MDOC_DOCTYPE
        );
    }

    #[derive(Debug, Clone, Copy)]
    enum RegistrationCertificateStatusFailure {
        Revoked,
        Suspended,
        Unavailable,
        UntrustedList,
        OutOfBounds,
    }

    #[rstest]
    #[case::revoked(RegistrationCertificateStatusFailure::Revoked)]
    #[case::suspended(RegistrationCertificateStatusFailure::Suspended)]
    #[case::unavailable(RegistrationCertificateStatusFailure::Unavailable)]
    #[case::untrusted_list(RegistrationCertificateStatusFailure::UntrustedList)]
    #[case::out_of_bounds(RegistrationCertificateStatusFailure::OutOfBounds)]
    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_reports_registration_certificate_status_error(
        #[case] failure: RegistrationCertificateStatusFailure,
        #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    ) {
        let mut payload = registration_certificate_payload();
        payload.0["status"]["idx"] = json!(match failure {
            // The ten-entry fixture is padded to sixteen entries when packed.
            RegistrationCertificateStatusFailure::OutOfBounds => "16",
            _ => "1",
        });
        let certificate = format.sign(&REGISTRATION_CERTIFICATE_AUTHORITY, &payload);
        let status_list_result = match failure {
            RegistrationCertificateStatusFailure::Revoked | RegistrationCertificateStatusFailure::Suspended => {
                let mut status_list = StatusList::new(10);
                let status = match failure {
                    RegistrationCertificateStatusFailure::Revoked => StatusType::Invalid,
                    _ => StatusType::Suspended,
                };
                status_list.insert(1, status);
                Ok(REGISTRATION_CERTIFICATE_AUTHORITY.sign_status_list(status_list))
            }
            RegistrationCertificateStatusFailure::Unavailable => Err(StatusListClientError::JwtParsing(
                JwtParseError::UnexpectedNumberOfParts(0),
            )),
            RegistrationCertificateStatusFailure::UntrustedList => {
                Ok(MockRegistrationCertificateAuthority::new().status_list_client.0)
            }
            RegistrationCertificateStatusFailure::OutOfBounds => {
                Ok(REGISTRATION_CERTIFICATE_AUTHORITY.status_list_client.0.clone())
            }
        };
        let error = assert_registration_certificate_rejected(
            Some(Value::Bytes(certificate)),
            DeviceResponseStatus::GeneralError,
            Some(status_list_result),
            true,
        )
        .await;

        let DisclosureError::CloseProximityDisclosureSessionError(
            CloseProximityDisclosureError::RegistrationCertificateStatus(error),
        ) = error
        else {
            panic!("expected a registration certificate status error");
        };
        assert_matches!(
            (failure, error),
            (
                RegistrationCertificateStatusFailure::Revoked | RegistrationCertificateStatusFailure::Suspended,
                RegistrationCertificateStatusValidationError::NotValid,
            ) | (
                RegistrationCertificateStatusFailure::Unavailable,
                RegistrationCertificateStatusValidationError::Undetermined,
            ) | (
                RegistrationCertificateStatusFailure::UntrustedList | RegistrationCertificateStatusFailure::OutOfBounds,
                RegistrationCertificateStatusValidationError::InvalidStatusListReference,
            )
        );
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_continue_close_proximity_disclosure_send_error_response_failure_still_stops_ble_server() {
        let send_context = MockCloseProximityDisclosureClient::send_device_response_context();
        send_context.expect().once().returning(|_| {
            Err(PlatformError::PlatformError {
                reason: "send failed".to_string(),
            })
        });

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
            Err(DisclosureError::PlatformCloseProximityDisclosureSessionError(
                PlatformError::PlatformError { reason }
            )) if reason == "send failed"
        );
        assert!(wallet.session.is_none());
    }

    fn qr_session_transcript(device_engagement: Option<DeviceEngagement>) -> SessionTranscript {
        let ephemeral_key_pair = SigningKey::generate();
        let cose_key: CoseKey = ephemeral_key_pair.verifying_key().try_into().unwrap();
        SessionTranscript::new_qr(cose_key, device_engagement)
    }

    async fn setup_device_request(
        items_requests: Vec<ItemsRequest>,
        device_engagement: Option<DeviceEngagement>,
        with_crl: bool,
    ) -> (DeviceRequest, SessionTranscript, TrustAnchors) {
        let session_transcript = qr_session_transcript(device_engagement);

        let key_pair = if with_crl {
            WRPAC_CA.generate_wrpac_verifier_mock_with_crl().unwrap()
        } else {
            WRPAC_CA.generate_wrpac_verifier_mock().unwrap()
        };
        let doc_requests = join_all(
            items_requests
                .into_iter()
                .map(async |items_request| create_doc_request(items_request, &session_transcript, &key_pair).await),
        )
        .await
        .try_into()
        .unwrap();

        let device_request = DeviceRequest::from_doc_requests(doc_requests);
        let trust_anchors = WRPAC_CA.to_trust_anchors();

        (device_request, session_transcript, trust_anchors)
    }

    #[rstest]
    #[tokio::test]
    async fn test_verify_device_request_without_wrprc(#[values(true, false)] wrprc_strict_mode: bool) {
        let mut request = pid_given_name_items_request();
        request.request_info = None;
        let (device_request, session_transcript, trust_anchors) = setup_device_request(vec![request], None, true).await;
        // Any status-list fetch would fail this test: absent WRPRCs have no status to check.
        let status_list_client = Arc::new(MockStatusListClient::new());
        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            wrprc_strict_mode,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &status_list_client,
        )
        .await;
        if wrprc_strict_mode {
            assert_matches!(
                result,
                Err(CloseProximityDisclosureError::RegistrationCertificateParsing(
                    RegistrationCertificateRequestParseError::Missing
                ))
            );
        } else {
            let organization = result.unwrap();
            let reader = WRPAC_CA.generate_wrpac_verifier_mock().unwrap();
            let subject = reader.certificate().to_distinguished_name().unwrap();
            assert_eq!(organization.display_name, subject.common_name);
            assert_eq!(Some(organization.legal_name), subject.organization_name);
            assert_eq!(Some(organization.identifier), subject.organization_identifier);
            assert_eq!(organization.support_uri, WRPRC_SUPPORT_UNAVAILABLE_URI);
        }
    }

    #[rstest]
    #[case::without_expiry(None)]
    #[case::unexpired(Some(60))]
    #[tokio::test]
    async fn test_verify_device_request_success(
        #[case] expires_in_seconds: Option<i64>,
        #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    ) {
        let mut payload = registration_certificate_payload();
        if let Some(expires_in_seconds) = expires_in_seconds {
            payload.0["exp"] = json!(Utc::now().timestamp() + expires_in_seconds);
        }
        let certificate = format.sign(&REGISTRATION_CERTIFICATE_AUTHORITY, &payload);
        let mut items_request = pid_given_name_items_request();
        items_request.request_info = Some(IndexMap::from_iter([(
            REQUEST_INFO_WRPRC_KEY.to_string(),
            Value::Bytes(certificate),
        )]));

        let (device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![items_request], None, true).await;

        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &registration_certificate_status_list_client(),
        )
        .await;

        assert_eq!(result.unwrap(), registration_certificate_organization());
    }

    #[rstest]
    #[tokio::test]
    async fn test_verify_device_request_with_crl_fails_without_distribution_point(
        #[values(true, false)] wrprc_strict_mode: bool,
    ) {
        let mut items_request = pid_given_name_items_request();
        items_request.request_info = None;

        let (device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![items_request], None, false).await;

        let verifier = MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA);
        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            wrprc_strict_mode,
            &verifier,
            &registration_certificate_status_list_client(),
        )
        .await;

        assert_matches!(
            result,
            Err(CloseProximityDisclosureError::ReaderCertificateCrlVerification(
                CertificateCrlVerificationError::NoCrlDistributionPoint
            ))
        );
    }

    #[rstest]
    #[case::same_details(false)]
    #[case::different_details(true)]
    #[tokio::test]
    async fn test_verify_device_request_accepts_same_reader_multi_document_request(
        #[case] different_registration_details: bool,
    ) {
        let mut pid_items_request = interop_items_request(
            EUDI_PID_DOCTYPE,
            &[(
                EUDI_PID_DOCTYPE,
                &["family_name", "given_name", "birth_date", "age_over_18"][..],
            )],
            None,
        );
        add_matching_registration_certificate(&mut pid_items_request, registration_certificate_payload());
        let mut mdl_items_request = interop_items_request(
            ISO_MDL_DOCTYPE,
            &[(
                ISO_MDL_NAMESPACE,
                &["family_name", "given_name", "birth_date", "document_number"][..],
            )],
            None,
        );
        let mut mdl_registration = registration_certificate_payload();
        if different_registration_details {
            mdl_registration.0["name"] = json!("Another service of the same RP");
            mdl_registration.0["purpose"] = json!([{ "lang": "en", "value": "Rent a vehicle." }]);
        }
        add_matching_registration_certificate(&mut mdl_items_request, mdl_registration);

        let (device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![pid_items_request, mdl_items_request], None, true).await;

        let mut status_list_client = MockStatusListClient::new();
        status_list_client
            .expect_fetch()
            .with(eq(STATUS_LIST_URI.parse::<Url>().unwrap()))
            .once()
            .returning(|_| Ok(REGISTRATION_CERTIFICATE_AUTHORITY.status_list_client.0.clone()));

        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &Arc::new(status_list_client),
        )
        .await;

        // Presentation details come from the first DocRequest's WRPRC, even if a later one differs.
        assert_eq!(result.unwrap(), registration_certificate_organization());
    }

    #[tokio::test]
    async fn test_verify_device_request_accepts_multi_namespace_items_request() {
        let mut photo_id_items_request = interop_items_request(
            PHOTO_ID_DOCTYPE_LOWERCASE,
            &[
                (
                    PHOTO_ID_BASE_NAMESPACE,
                    &["family_name", "given_name", "birth_date"][..],
                ),
                (PHOTO_ID_NAMESPACE, &["portrait", "document_number"][..]),
            ],
            None,
        );
        add_matching_registration_certificate(&mut photo_id_items_request, registration_certificate_payload());

        let (device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![photo_id_items_request], None, true).await;

        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &registration_certificate_status_list_client(),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_device_request_rejects_sd_jwt_kb_doc_format() {
        let items_request = interop_items_request(
            EUDI_PID_DOCTYPE,
            &[(EUDI_PID_DOCTYPE, &["family_name", "given_name"][..])],
            Some(IndexMap::from_iter([(
                "docFormat".to_string(),
                Value::Text("sd-jwt+kb".to_string()),
            )])),
        );

        let (device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![items_request], None, true).await;

        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &registration_certificate_status_list_client(),
        )
        .await;

        assert_matches!(
            result,
            Err(CloseProximityDisclosureError::UnsupportedDocFormat { doc_format }) if doc_format == "sd-jwt+kb"
        );
    }

    #[tokio::test]
    async fn test_verify_device_request_rejects_reader_auth_for_different_session_transcript() {
        let items_request = pid_given_name_items_request();

        let (device_request, _session_transcript, trust_anchors) =
            setup_device_request(vec![items_request], None, true).await;
        let other_session_transcript = qr_session_transcript(None);

        let result = verify_device_request_with_crl(
            &device_request,
            &other_session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &registration_certificate_status_list_client(),
        )
        .await;

        assert_matches!(result, Err(CloseProximityDisclosureError::InvalidDocRequest(_)));
    }

    #[tokio::test]
    async fn test_verify_device_request_missing_reader_auth() {
        let items_request = pid_given_name_items_request();

        let (mut device_request, session_transcript, trust_anchors) =
            setup_device_request(vec![items_request], None, true).await;

        device_request
            .doc_requests
            .iter_mut()
            .for_each(|doc_request| doc_request.reader_auth = None);

        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &registration_certificate_status_list_client(),
        )
        .await;

        assert_matches!(
            result,
            Err(CloseProximityDisclosureError::InvalidDocRequest(
                mdoc::Error::MissingReaderAuthentication
            ))
        );
    }

    #[tokio::test]
    async fn test_verify_device_request_inconsistent_reader_auths() {
        let items_request1 = pid_given_name_items_request();
        let items_request2 = ItemsRequest {
            doc_type: PID_MDOC_DOCTYPE.to_owned(),
            name_spaces: IndexMap::from_iter(vec![(
                PID_MDOC_DOCTYPE.to_owned(),
                IndexMap::from_iter(vec![(PID_FAMILY_NAME.to_owned(), true)])
                    .try_into()
                    .unwrap(),
            )])
            .try_into()
            .unwrap(),
            request_info: None,
        };

        // Create two different key pairs from the same CA, so that the resulting certificates differ.
        let key_pair1 = WRPAC_CA.generate_wrpac_verifier_mock_with_crl().unwrap();
        let key_pair2 = WRPAC_CA.generate_wrpac_verifier_mock_with_crl().unwrap();

        let session_transcript = qr_session_transcript(None);

        let doc_request1 = create_doc_request(items_request1, &session_transcript, &key_pair1).await;
        let doc_request2 = create_doc_request(items_request2, &session_transcript, &key_pair2).await;

        let device_request = DeviceRequest::from_doc_requests(vec_nonempty![doc_request1, doc_request2]);
        let trust_anchors = WRPAC_CA.to_trust_anchors();

        let result = verify_device_request_with_crl(
            &device_request,
            &session_transcript,
            &MockTimeGenerator::default(),
            DisclosureTrustAnchors {
                wrpac: &trust_anchors,
                wrprc: &REGISTRATION_CERTIFICATE_AUTHORITY.trust_anchors,
            },
            true,
            &MockCertificateCrlVerifier::new_for_ca(&WRPAC_CA),
            &registration_certificate_status_list_client(),
        )
        .await;

        assert_matches!(result, Err(CloseProximityDisclosureError::InconsistentReaderAuths));
    }

    // The PIN used in accept_disclosure tests.
    static PIN: LazyLock<Pin> = LazyLock::new(|| "051097".into());

    /// Creates a `CloseProximityDisclosureSession` in the `DisclosureProposed` state and installs
    /// it as `wallet.session`. Returns the WRPRC organization stored inside the session so that
    /// callers can set up event-log expectations against it.
    fn setup_close_proximity_disclosure_proposed_session(
        wallet: &mut TestWalletMockStorage,
    ) -> (Organization, SessionTranscript) {
        let organization = registration_certificate_organization();

        let (pid_credential_payload, pid_metadata, holder_key) = example_pid_payload_and_metadata(Format::MsoMdoc);
        let pid = example_stored_attestation_copy(
            Format::MsoMdoc,
            pid_credential_payload.clone(),
            pid_metadata,
            &holder_key,
        );

        let credential_requests = NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(PID_MDOC_DOCTYPE, &[&[PID_MDOC_DOCTYPE, PID_GIVEN_NAME]])],
            None,
        );
        let disclosable_attestation = DisclosableAttestation::try_new(
            pid,
            credential_requests.as_ref().first().unwrap().claim_paths(),
            &EmptyPresentationConfig,
        )
        .unwrap();

        let session_transcript = qr_session_transcript(None);

        wallet.session = Some(Session::CloseProximityDisclosure(CloseProximityDisclosureSession {
            wrprc_strict_mode: true,
            listener: tokio::spawn(async {}),
            session_state: Arc::new(Mutex::new(CloseProximityDisclosureSessionState::DisclosureProposed {
                session_transcript: Box::new(session_transcript.clone()),
                organization: Box::new(organization.clone()),
                attestations: close_proximity_disclosure_proposal(IndexMap::from([(
                    0,
                    vec_nonempty![disclosable_attestation],
                )]))
                .unwrap(),
            })),
        }));

        (organization, session_transcript)
    }

    fn setup_mock_sign_instruction(wallet: &mut TestWalletMockStorage) {
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));
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
        let signing_key = SigningKey::generate();
        let signature: Signature = signing_key.sign(b"");
        let der_sig = DerSignature::from(signature);

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .with(always(), always())
            .return_once(move |_, _: Instruction<Sign>| {
                Ok(create_wp_result(SignResult {
                    signatures: vec![vec![der_sig]],
                }))
            });
    }

    fn pid_given_name_disclosable_attestation() -> (VpDisclosableAttestation, SigningKey) {
        let credential_requests = NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(PID_MDOC_DOCTYPE, &[&[PID_MDOC_DOCTYPE, PID_GIVEN_NAME]])],
            None,
        );

        let (stored_attestation_copy, holder_key) = example_pid_stored_attestation_copy(Format::MsoMdoc);

        (
            disclosable_attestation_from_credential_requests(stored_attestation_copy, &credential_requests),
            holder_key,
        )
    }

    fn disclosable_attestation_from_credential_requests(
        stored_attestation_copy: StoredAttestationCopy,
        credential_requests: &NormalizedCredentialRequests,
    ) -> VpDisclosableAttestation {
        DisclosableAttestation::try_new(
            stored_attestation_copy,
            credential_requests.as_ref().first().unwrap().claim_paths(),
            &EmptyPresentationConfig,
        )
        .unwrap()
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
    async fn test_create_close_proximity_device_response_multiple_attestations() {
        let session_transcript = qr_session_transcript(None);

        let (pid1, key1) = pid_given_name_disclosable_attestation();
        let (pid2, key2) = example_pid_stored_attestation_copy(Format::MsoMdoc);

        let key_id1 = pid1.partial_attestation().private_key_id().to_owned();
        let key_id2 = pid2.private_key_id().to_owned();

        let disclosable_attestations = vec_nonempty![
            CloseProximityDisclosableAttestation::try_from(pid1).unwrap(),
            CloseProximityDisclosableAttestation::try_from(disclosable_attestation_from_credential_requests(
                pid2,
                &NormalizedCredentialRequests::new_mock_mdoc_from_slices(
                    &[(PID_MDOC_DOCTYPE, &[&[PID_MDOC_DOCTYPE, PID_FAMILY_NAME]])],
                    None,
                ),
            ))
            .unwrap(),
        ];

        let remote_key1 = MockRemoteEcdsaKey::new(key_id1, key1);
        let remote_key2 = MockRemoteEcdsaKey::new(key_id2, key2);
        let wscd = DisclosureMockRemoteWscd::new(vec![remote_key1, remote_key2]);

        let device_response =
            TestWalletMockStorage::<LocalConfigurationRepository>::create_close_proximity_device_response(
                disclosable_attestations.nonempty_iter(),
                &session_transcript,
                &wscd,
            )
            .await
            .expect("device response creation should succeed");

        assert_eq!(device_response.status, DeviceResponseStatus::Ok);
        assert_eq!(
            device_response.documents.expect("should contain documents").len().get(),
            2
        );
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (organization, _) = setup_close_proximity_disclosure_proposed_session(&mut wallet);

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

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                always(),
                eq(organization),
                eq(EventStatus::Success),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let result = wallet
            .accept_disclosure(&[0], PIN.clone())
            .await
            .expect("accepting close proximity disclosure should succeed");

        // Close proximity disclosure has no redirect URI.
        assert_eq!(result, None);
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_send_device_response_error_reports_disconnected() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        setup_close_proximity_disclosure_proposed_session(&mut wallet);

        setup_mock_sign_instruction(&mut wallet);

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .once()
            .returning(|_| Ok(()));

        wallet.mut_storage().expect_log_disclosure_event().never();

        let context = MockCloseProximityDisclosureClient::send_device_response_context();
        context.expect().once().returning(|_| {
            Err(PlatformError::PlatformError {
                reason: "transport closed".to_string(),
            })
        });

        let error = wallet
            .accept_disclosure(&[0], PIN.clone())
            .await
            .expect_err("accepting close proximity disclosure should report the disconnect");

        assert_matches!(
            error,
            DisclosureError::CloseProximityDisclosureSessionError(CloseProximityDisclosureError::Disconnected)
        );
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    #[serial(MockCloseProximityDisclosureClient)]
    async fn test_wallet_accept_close_proximity_disclosure_error_increment_usage_count() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (organization, _) = setup_close_proximity_disclosure_proposed_session(&mut wallet);

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

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(organization.clone()),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.clone())
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
                eq(organization),
                eq(EventStatus::Success),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        assert!(
            wallet
                .accept_disclosure(&[0], PIN.clone())
                .await
                .expect("accepting close proximity disclosure should succeed")
                .is_none()
        );
    }

    #[test]
    fn test_close_proximity_disclosure_proposal_error_unexpected_attestation_format() {
        let credential_requests = NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[PID_ATTESTATION_TYPE],
            &[&[PID_GIVEN_NAME]],
        )]);
        let (sd_jwt_attestation, _) = example_pid_stored_attestation_copy(Format::SdJwt);
        let sd_jwt_disclosable_attestation =
            disclosable_attestation_from_credential_requests(sd_jwt_attestation, &credential_requests);

        let error =
            close_proximity_disclosure_proposal(IndexMap::from([(0, vec_nonempty![sd_jwt_disclosable_attestation])]))
                .expect_err("a close proximity disclosure proposal should not contain SD-JWT attestations");

        assert_matches!(error, DisclosureError::UnexpectedAttestationFormat);
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
            .accept_disclosure(&[0], "whatever".into())
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
        let (organization, _) = setup_close_proximity_disclosure_proposed_session(&mut wallet);

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

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(organization),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.clone())
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
        let (organization, _) = setup_close_proximity_disclosure_proposed_session(&mut wallet);

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

        // On Timeout the session is terminated via `stop_ble_server`
        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(organization),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.clone())
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
        let (organization, _) = setup_close_proximity_disclosure_proposed_session(&mut wallet);

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

        let context = MockCloseProximityDisclosureClient::stop_ble_server_context();
        context.expect().once().returning(|| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(organization),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .once()
            .returning(|_, _, _, _, _| Ok(()));

        let error = wallet
            .accept_disclosure(&[0], PIN.clone())
            .await
            .expect_err("accepting close proximity disclosure should not succeed");

        assert_matches!(error, DisclosureError::Instruction(_));
        assert!(wallet.session.is_none());
        assert!(wallet.is_locked());
    }
}
