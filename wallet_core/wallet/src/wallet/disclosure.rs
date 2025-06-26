use std::collections::HashSet;
use std::iter;
use std::sync::Arc;

use derive_more::Constructor;
use itertools::Either;
use itertools::Itertools;
use tracing::error;
use tracing::info;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

pub use openid4vc::disclosure_session::DisclosureUriSource;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::auth::Organization;
use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use mdoc::holder::disclosure::attribute_paths_to_mdoc_paths;
use mdoc::holder::Mdoc;
use mdoc::utils::cose::CoseError;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpSessionError;
use openid4vc::disclosure_session::VpVerifierError;
use openid4vc::verifier::SessionType;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::vec_at_least::VecNonEmpty;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::AttestationError;
use crate::attestation::AttestationPresentation;
use crate::errors::ChangePinError;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionError;
use crate::instruction::RemoteEcdsaKeyError;
use crate::instruction::RemoteEcdsaKeyFactory;
use crate::issuance::BSN_ATTR_NAME;
use crate::issuance::PID_DOCTYPE;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::DataDisclosureStatus;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::WalletEvent;
use crate::wallet::Session;

use super::uri::identify_uri;
use super::UriType;
use super::Wallet;

#[derive(Debug, Clone)]
pub struct DisclosureProposalPresentation {
    pub attestations: Vec<AttestationPresentation>,
    pub reader_registration: ReaderRegistration,
    pub shared_data_with_relying_party_before: bool,
    pub session_type: SessionType,
    pub disclosure_type: DisclosureType,
    pub purpose: RedirectUriPurpose,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DisclosureError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,
    #[error("disclosure session is not in the correct state")]
    #[category(expected)]
    SessionState,
    #[error("did not recognize disclosure URI: {0}")]
    #[category(pd)]
    DisclosureUri(Url),
    #[error("disclosure URI is missing query parameter(s): {0}")]
    #[category(pd)]
    DisclosureUriQuery(Url),
    #[error("could not create HTTP client: {0}")]
    #[category(critical)]
    HttpClient(#[source] reqwest::Error),
    #[error("error in OpenID4VP disclosure session: {0}")]
    VpClient(#[source] VpClientError),
    #[error("error in OpenID4VP disclosure session: {error}")]
    VpVerifierServer {
        organization: Option<Box<Organization>>,
        #[defer]
        #[source]
        error: VpVerifierError,
    },
    #[error("could not fetch if attributes were shared before: {0}")]
    HistoryRetrieval(#[source] StorageError),
    #[error("could not fetch candidate attestations from database: {0}")]
    AttestationRetrieval(#[source] StorageError),
    #[error("multiple candidates found for attestation type(s): {}", .0.join(", "))]
    // We do not want to leak information about the attestation types in the wallet.
    #[category(pd)]
    MultipleCandidates(Vec<String>),
    #[error("not all requested attributes are available, requested: {requested_attributes:?}")]
    #[category(pd)] // Might reveal information about what attributes are stored in the Wallet
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        requested_attributes: HashSet<String>,
        shared_data_with_relying_party_before: bool,
        session_type: SessionType,
    },
    #[error("could not extract issuer certificate from stored mdoc: {0}")]
    MdocCertificate(#[source] CoseError),
    #[error("could not extract issuer registration from stored attestation certificate: {0}")]
    IssuerRegistration(#[source] CertificateError),
    #[error("stored mdoc certificate does not contain issuer registration")]
    #[category(critical)]
    MissingIssuerRegistration,
    #[error("could not interpret attestation attributes: {0}")]
    AttestationAttributes(#[source] AttestationError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[source] InstructionError),
    #[error("could not increment usage count of mdoc copies in database: {0}")]
    IncrementUsageCount(#[source] StorageError),
    #[error("could not store event in history database: {0}")]
    EventStorage(#[source] StorageError),
    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),
    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),
    #[error("unexpected redirect URI purpose: expected {expected:?}, found {found:?}")]
    #[category(critical)]
    UnexpectedRedirectUriPurpose {
        expected: RedirectUriPurpose,
        found: RedirectUriPurpose,
    },
}

impl DisclosureError {
    fn with_organization(error: VpSessionError, organization: Organization) -> Self {
        match error {
            VpSessionError::Verifier(error) => Self::VpVerifierServer {
                organization: Some(Box::new(organization)),
                error,
            },
            error => error.into(),
        }
    }

    pub fn return_url(&self) -> Option<&Url> {
        match self {
            Self::VpVerifierServer {
                error: VpVerifierError::Request(error),
                ..
            }
            | Self::VpClient(VpClientError::Request(error)) => error.redirect_uri().map(AsRef::as_ref),
            _ => None,
        }
    }
}

impl From<VpSessionError> for DisclosureError {
    fn from(value: VpSessionError) -> Self {
        match value {
            // Upgrade any signing errors that are caused an instruction error to `DisclosureError::Instruction`.
            VpSessionError::Client(VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(
                signing_error,
            )))) if matches!(
                signing_error.downcast_ref::<RemoteEcdsaKeyError>(),
                Some(RemoteEcdsaKeyError::Instruction(_))
            ) =>
            {
                // Note that this statement is safe, as checking is performed within the guard statements above.
                if let Ok(RemoteEcdsaKeyError::Instruction(instruction_error)) =
                    signing_error.downcast::<RemoteEcdsaKeyError>().map(|error| *error)
                {
                    DisclosureError::Instruction(instruction_error)
                } else {
                    unreachable!()
                }
            }
            // Any other error should result in its generic top-level error variant.
            VpSessionError::Client(client_error) => DisclosureError::VpClient(client_error),
            VpSessionError::Verifier(verifier_error) => DisclosureError::VpVerifierServer {
                organization: None,
                error: verifier_error,
            },
        }
    }
}

/// Encodes what the user can do with the redirect URI that the wallet (maybe) receives at the end of the
/// disclosure session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectUriPurpose {
    /// The redirect URI is an ordinary https URI and can be opened in the browser.
    Browser,
    /// The redirect URI contains an OpenID4VCI Credential Offer and can be used to start an issuance session.
    Issuance,
}

#[derive(Debug, Clone)]
pub struct WalletDisclosureSession<DCS> {
    redirect_uri_purpose: RedirectUriPurpose,
    disclosure_type: DisclosureType,
    attestations: Option<VecNonEmpty<DisclosureAttestation>>,
    protocol_state: DCS,
}

impl<DCS> WalletDisclosureSession<DCS> {
    pub fn new_proposal(
        redirect_uri_purpose: RedirectUriPurpose,
        disclosure_type: DisclosureType,
        attestations: VecNonEmpty<DisclosureAttestation>,
        protocol_state: DCS,
    ) -> Self {
        Self {
            redirect_uri_purpose,
            disclosure_type,
            attestations: Some(attestations),
            protocol_state,
        }
    }

    pub fn new_missing_attributes(
        redirect_uri_purpose: RedirectUriPurpose,
        disclosure_type: DisclosureType,
        protocol_state: DCS,
    ) -> Self {
        Self {
            redirect_uri_purpose,
            disclosure_type,
            attestations: None,
            protocol_state,
        }
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct DisclosureAttestation {
    copy_id: Uuid,
    mdoc: Mdoc,
    presentation: AttestationPresentation,
}

impl<DCS> WalletDisclosureSession<DCS> {
    pub(super) fn protocol_state(&self) -> &DCS {
        &self.protocol_state
    }
}

impl RedirectUriPurpose {
    fn from_uri(uri: &Url) -> Result<Self, DisclosureError> {
        let purpose = identify_uri(uri)
            .and_then(|uri_type| match uri_type {
                UriType::PidIssuance => None,
                UriType::Disclosure => Some(Self::Browser),
                UriType::DisclosureBasedIssuance => Some(Self::Issuance),
            })
            .ok_or_else(|| DisclosureError::DisclosureUri(uri.clone()))?;

        Ok(purpose)
    }
}

impl<CR, UR, S, AKH, APC, DS, IS, DC, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, DC, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    DC: DisclosureClient,
    S: Storage,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn start_disclosure(
        &mut self,
        uri: &Url,
        source: DisclosureUriSource,
    ) -> Result<DisclosureProposalPresentation, DisclosureError> {
        info!("Performing disclosure based on received URI: {}", uri);

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

        let config = &self.config_repository.get().disclosure;

        let purpose = RedirectUriPurpose::from_uri(uri)?;
        let disclosure_uri_query = uri
            .query()
            .ok_or_else(|| DisclosureError::DisclosureUriQuery(uri.clone()))?;

        // Start the disclosure session based on the parsed disclosure URI.
        let session = self
            .disclosure_client
            .start(disclosure_uri_query, source, &config.rp_trust_anchors())
            .await?;

        // Retrieve all attestation with the requested attestation types from
        // the database, which are returned in original insertion order.
        let requested_attestation_types = session
            .requested_attribute_paths()
            .as_ref()
            .keys()
            .map(String::as_str)
            .collect();
        let stored_mdocs = self
            .storage
            .read()
            .await
            .fetch_unique_mdocs_by_doctypes(&requested_attestation_types)
            .await
            .map_err(DisclosureError::AttestationRetrieval)?;

        // Group `StoredMdocCopy` values by attestation type in a `Vec<VecNonEmpty<_>>`.
        // The order of this reflects the original database order, at least within one attestation type.
        let stored_mdocs_by_type = stored_mdocs
            .into_iter()
            .filter_map(|stored_mdoc| {
                // Get a reference from `requested_attestation_types`` for  `.chunk_by()`,
                // filtering out any attestations with types that were not requested,
                // even though the database should never return this.
                requested_attestation_types
                    .get(stored_mdoc.mdoc.mso.doc_type.as_str())
                    .map(|doc_type| (*doc_type, stored_mdoc))
            })
            .chunk_by(|(attestation_type, _)| *attestation_type)
            .into_iter()
            .filter_map(|(attestation_type, stored_mdoc_iter)| {
                // Get the requested paths for this attestation type.
                let mdoc_paths = attribute_paths_to_mdoc_paths(session.requested_attribute_paths(), attestation_type);

                let candidate_attestations = stored_mdoc_iter
                    .into_iter()
                    .filter_map(|(_, stored_mdoc)| {
                        // Only select those attestations that contain all of the requested attributes.
                        // TODO (PVW-4537): Have this be part of the database query using some index.
                        stored_mdoc
                            .mdoc
                            .issuer_signed
                            .matches_attribute_paths(&mdoc_paths)
                            .then_some(stored_mdoc)
                    })
                    .collect_vec();

                // Filter out any attestation type that has no candidates.
                VecNonEmpty::try_from(candidate_attestations).ok()
            })
            .collect_vec();

        // At this point, determine the disclosure type and if data was every shared with this RP before, as the UI
        // needs this context both for when all requested attributes are present and for when attributes are missing.
        let disclosure_type = DisclosureType::from_request_attribute_paths(
            session.requested_attribute_paths(),
            PID_DOCTYPE,
            (PID_DOCTYPE, BSN_ATTR_NAME),
        );

        let verifier_certificate = session.verifier_certificate();
        let shared_data_with_relying_party_before = self
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(verifier_certificate.certificate())
            .await
            .map_err(DisclosureError::HistoryRetrieval)?;

        // If no suitable candidates were found for at least one of the
        // requested attestation types, report this as an error to the UI.
        if stored_mdocs_by_type.len() < requested_attestation_types.len() {
            info!("At least one attribute is missing in order to satisfy the disclosure request");

            let reader_registration = verifier_certificate.registration().clone();
            // For now we simply represent the requested attribute paths by joining all elements with a slash.
            // TODO (PVW-3813): Attempt to translate the requested attributes using the TAS cache.
            let requested_attributes = session
                .requested_attribute_paths()
                .as_ref()
                .iter()
                .flat_map(|(attestation_type, paths)| {
                    paths
                        .iter()
                        .map(|path| iter::once(attestation_type).chain(path.iter()).join("/"))
                        .collect_vec()
                })
                .collect();
            let session_type = session.session_type();

            // Store the session so that it will only be terminated on user interaction.
            // This prevents gleaning of missing attributes by a verifier.
            self.session
                .replace(Session::Disclosure(WalletDisclosureSession::new_missing_attributes(
                    purpose,
                    disclosure_type,
                    session,
                )));

            return Err(DisclosureError::AttributesNotAvailable {
                reader_registration: Box::new(reader_registration),
                requested_attributes,
                shared_data_with_relying_party_before,
                session_type,
            });
        }

        // Now that we know that we have at least one candidate per attestation type, start attempting to convert the
        // stored attestations to `AttestationPresentation` values, placing these in `DisclosureAttestation` along with
        // the original attestation.
        let attestations_by_type = stored_mdocs_by_type
            .into_iter()
            .map(|stored_mdocs| {
                let mdoc_paths = attribute_paths_to_mdoc_paths(
                    session.requested_attribute_paths(),
                    &stored_mdocs.first().mdoc.mso.doc_type,
                );

                let attestations = stored_mdocs
                    .into_iter()
                    .map(|stored_mdoc| {
                        let mdoc_certificate = stored_mdoc
                            .mdoc
                            .issuer_certificate()
                            .map_err(DisclosureError::MdocCertificate)?;
                        let issuer_registration = IssuerRegistration::from_certificate(&mdoc_certificate)
                            .map_err(DisclosureError::IssuerRegistration)?
                            .ok_or(DisclosureError::MissingIssuerRegistration)?;

                        // Remove any attributes that were not requested from the presentation attributes.
                        let issuer_signed = stored_mdoc
                            .mdoc
                            .issuer_signed
                            .clone()
                            .into_attribute_subset(&mdoc_paths);

                        let attestation_presentation = AttestationPresentation::create_for_disclosure(
                            stored_mdoc.normalized_metadata.clone(),
                            issuer_registration.organization,
                            issuer_signed.into_entries_by_namespace(),
                        )
                        .map_err(DisclosureError::AttestationAttributes)?;

                        let attestation = DisclosureAttestation::new(
                            stored_mdoc.mdoc_copy_id,
                            stored_mdoc.mdoc,
                            attestation_presentation,
                        );

                        Ok(attestation)
                    })
                    .collect::<Result<Vec<_>, DisclosureError>>()?
                    .try_into()
                    // Safe, as the source of the iterator is `VecNonEmpty`.
                    .unwrap();

                Ok(attestations)
            })
            .collect::<Result<Vec<VecNonEmpty<_>>, DisclosureError>>()?;

        // For now, return an error if multiple attestations are found for a requested attestation type.
        // TODO (PVW-3829): Allow the user to select amongst multiple disclosure candidates.

        let (disclosure_attestations, duplicate_attestation_types): (Vec<_>, Vec<_>) = attestations_by_type
            .into_iter()
            .partition_map(|candidate_attestations| {
                if candidate_attestations.len().get() == 1 {
                    Either::Left(candidate_attestations.into_first())
                } else {
                    Either::Right(candidate_attestations.into_first().presentation.attestation_type)
                }
            });

        if !duplicate_attestation_types.is_empty() {
            info!("At least one attestation type has multiple disclosure candidates");

            return Err(DisclosureError::MultipleCandidates(duplicate_attestation_types));
        }

        // This unwrap is guaranteed to succeed as:
        // 1. The `RequestedAttributePaths` type inherently guarantees that it contains at least one attestation type.
        // 2. We check above if there is at least one candidate for every attestation type.
        // 3. We then check above that none of the attestation types have multiple candidates, so the
        //    length of disclosure_attestations is the same as attestations_by_type, which is at least 1.
        let disclosure_attestations = VecNonEmpty::try_from(disclosure_attestations).unwrap();

        info!("All attributes in the disclosure request are present in the database, return a proposal to the user");

        // Place the propopsed attestations in a `DisclosureProposalPresentation`,
        // along with a copy of the `ReaderRegistration`.
        let proposal = DisclosureProposalPresentation {
            attestations: disclosure_attestations
                .iter()
                .map(|attestation| attestation.presentation.clone())
                .collect(),
            reader_registration: verifier_certificate.registration().clone(),
            shared_data_with_relying_party_before,
            session_type: session.session_type(),
            disclosure_type,
            purpose,
        };

        // Retain the session as `Wallet` state.
        self.session
            .replace(Session::Disclosure(WalletDisclosureSession::new_proposal(
                purpose,
                disclosure_type,
                disclosure_attestations,
                session,
            )));

        Ok(proposal)
    }

    async fn terminate_disclosure_session(
        &mut self,
        session: WalletDisclosureSession<DC::Session>,
    ) -> Result<Option<Url>, DisclosureError> {
        let attestations = session.attestations.map(|attestations| {
            attestations
                .into_iter()
                .map(|attestation| attestation.presentation)
                .collect_vec()
                .try_into()
                // Safe, as the source of the iterator is `VecNonEmpty`.
                .unwrap()
        });

        let event = WalletEvent::new_disclosure_cancel(
            attestations,
            session.protocol_state.verifier_certificate().clone(),
            session.disclosure_type,
        );

        let return_url = session.protocol_state.terminate().await?.map(BaseUrl::into_inner);

        self.store_history_event(event)
            .await
            .map_err(DisclosureError::EventStorage)?;

        Ok(return_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn has_active_disclosure_session(&self) -> Result<bool, DisclosureError> {
        info!("Checking for active disclosure session");

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

        let has_active_session = matches!(self.session, Some(Session::Disclosure(..)));

        Ok(has_active_session)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_disclosure(&mut self) -> Result<Option<Url>, DisclosureError> {
        info!("Cancelling disclosure");

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

        info!("Checking if a disclosure session is present");
        if !matches!(self.session, Some(Session::Disclosure(..))) {
            return Err(DisclosureError::SessionState);
        }

        let Session::Disclosure(session) = self.session.take().unwrap() else {
            panic!()
        };

        self.terminate_disclosure_session(session).await
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_disclosure(&mut self, pin: String) -> Result<Option<Url>, DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
        WIC: Default,
    {
        self.perform_disclosure(pin, RedirectUriPurpose::Browser, self.config_repository.get().as_ref())
            .await
    }

    #[instrument(skip_all)]
    pub(super) async fn perform_disclosure(
        &mut self,
        pin: String,
        redirect_uri_purpose: RedirectUriPurpose,
        config: &WalletConfiguration,
    ) -> Result<Option<Url>, DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
        WIC: Default,
    {
        info!("Accepting disclosure");
        info!("Fetching update policy");
        self.update_policy_repository
            .fetch(&config.update_policy_server.http_config)
            .await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(DisclosureError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| DisclosureError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DisclosureError::Locked);
        }

        info!("Checking if a disclosure session is present");
        let Some(Session::Disclosure(session)) = &self.session else {
            return Err(DisclosureError::SessionState);
        };

        if session.redirect_uri_purpose != redirect_uri_purpose {
            return Err(DisclosureError::UnexpectedRedirectUriPurpose {
                expected: session.redirect_uri_purpose,
                found: redirect_uri_purpose,
            });
        }

        let attestations = session.attestations.as_ref().ok_or(DisclosureError::SessionState)?;

        // Prepare the `RemoteEcdsaKeyFactory` for signing using the provided PIN.
        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let remote_instruction = self
            .new_instruction_client(
                pin,
                Arc::clone(attested_key),
                registration_data.clone(),
                config.account_server.http_config.clone(),
                instruction_result_public_key,
            )
            .await?;

        let remote_key_factory = RemoteEcdsaKeyFactory::new(remote_instruction);

        // Increment the disclosure counts of the attestation copies referenced in the proposal,
        // so that for the next disclosure different copies are used.

        // NOTE: If the disclosure fails and is retried, the disclosure count will jump by
        //       more than 1, since the same copies are shared with the verifier again.
        //       It is necessary to increment the disclosure count before sending the attestations
        //       to the verifier, as we do not know if disclosure fails before or after the
        //       verifier has received the attributes.

        let result = self
            .storage
            .write()
            .await
            .increment_attestation_copies_usage_count(
                attestations.iter().map(|attestation| attestation.copy_id).collect(),
            )
            .await;

        if let Err(error) = result {
            let event = WalletEvent::new_disclosure_error(
                attestations
                    .iter()
                    .map(|attestation| attestation.presentation.clone())
                    .collect_vec()
                    .try_into()
                    .unwrap(),
                session.protocol_state().verifier_certificate().clone(),
                session.disclosure_type,
                DataDisclosureStatus::NotDisclosed,
            );

            if let Err(e) = self.store_history_event(event).await {
                error!("Could not store error in history: {e}");
            }

            return Err(DisclosureError::IncrementUsageCount(error));
        }

        // Clone some values from `WalletDisclosureSession`, before we have to give away ownership of it.
        let mdocs = attestations
            .iter()
            .map(|attestation| attestation.mdoc.clone())
            .collect_vec()
            .try_into()
            // Safe, as the source of the iterator is `VecNonEmpty`.
            .unwrap();
        let verifier_certificate = session.protocol_state.verifier_certificate().clone();

        // Take ownership of the disclosure session and actually perform disclosure, casting any
        // `InstructionError` that occurs during signing to `RemoteEcdsaKeyError::Instruction`.
        let Some(Session::Disclosure(mut session)) = self.session.take() else {
            // This not possible, as we took a reference to this value before.
            unreachable!();
        };
        let result = session.protocol_state.disclose(mdocs, &remote_key_factory).await;
        let return_url = match result {
            Ok(return_url) => return_url.map(BaseUrl::into_inner),
            Err((protocol_state, error)) => {
                let organization = verifier_certificate.registration().organization.clone();
                let disclosure_error = DisclosureError::with_organization(error.error, organization);

                // IncorrectPin is a functional error and does not need to be recorded.
                if !matches!(
                    disclosure_error,
                    DisclosureError::Instruction(InstructionError::IncorrectPin { .. })
                ) {
                    let data_status = if error.data_shared {
                        DataDisclosureStatus::Disclosed
                    } else {
                        DataDisclosureStatus::NotDisclosed
                    };
                    let event = WalletEvent::new_disclosure_error(
                        // These unwraps are safe, as session.attestations was checked to be
                        // present above and the source of the iterator is also `VecNonEmpty`.
                        session
                            .attestations
                            .as_ref()
                            .unwrap()
                            .iter()
                            .map(|attestation| attestation.presentation.clone())
                            .collect_vec()
                            .try_into()
                            .unwrap(),
                        verifier_certificate,
                        session.disclosure_type,
                        data_status,
                    );

                    if let Err(e) = self.store_history_event(event).await {
                        error!("Could not store error in history: {e}");
                    }
                }

                // At this point place the `DisclosureSession` back so that `WalletDisclosureSession` is whole again.
                session.protocol_state = protocol_state;

                if matches!(
                    disclosure_error,
                    DisclosureError::Instruction(InstructionError::Timeout { .. } | InstructionError::Blocked)
                ) {
                    // On a PIN timeout we should proactively terminate the disclosure session
                    // and lock the wallet, as the user is probably not the owner of the wallet.
                    // The UI should catch this specific error and close the disclosure screens.

                    if let Err(terminate_error) = self.terminate_disclosure_session(session).await {
                        // Log the error, but do not return it from this method.
                        error!(
                            "Error while terminating disclosure session on PIN timeout: {}",
                            terminate_error
                        );
                    }

                    self.lock.lock();
                } else {
                    // If we did not just give away ownership of the disclosure session by terminating it,
                    // place it back in the wallet state so that the user may retry disclosure.
                    self.session.replace(Session::Disclosure(session));
                }

                return Err(disclosure_error);
            }
        };

        // Disclosure is now successful. Any errors that occur after this point will result in the `Wallet` not having
        // an active disclosure session anymore. Note that these unwraps are safe, as session.attestations was checked
        // to be present above and the source of the iterator is also `VecNonEmpty`.
        let event = WalletEvent::new_disclosure_success(
            session
                .attestations
                .unwrap()
                .into_iter()
                .map(|attestation| attestation.presentation)
                .collect_vec()
                .try_into()
                .unwrap(),
            verifier_certificate,
            session.disclosure_type,
        );

        self.store_history_event(event)
            .await
            .map_err(DisclosureError::EventStorage)?;

        Ok(return_url)
    }
}

// TODO: Re-enable and rewrite these tests.

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;
//     use std::sync::atomic::Ordering;
//     use std::sync::Arc;
//     use std::sync::LazyLock;

//     use assert_matches::assert_matches;
//     use itertools::Itertools;
//     use mockall::predicate::*;
//     use parking_lot::Mutex;
//     use rstest::rstest;
//     use serial_test::serial;
//     use uuid::uuid;

//     use attestation_data::attributes::AttributeValue;
//     use attestation_data::attributes::AttributesError;
//     use http_utils::urls;
//     use mdoc::holder::Mdoc;
//     use mdoc::holder::ProposedAttributes;
//     use mdoc::holder::ProposedDocumentAttributes;
//     use mdoc::test::data::PID;
//     use mdoc::DataElementValue;
//     use mdoc::Entry;
//     use openid4vc::disclosure_session::VpMessageClientError;
//     use openid4vc::issuance_session::CredentialWithMetadata;
//     use openid4vc::issuance_session::IssuedCredential;
//     use openid4vc::issuance_session::IssuedCredentialCopies;
//     use openid4vc::DisclosureErrorResponse;
//     use openid4vc::ErrorResponse;
//     use openid4vc::GetRequestErrorCode;
//     use openid4vc::PostAuthResponseErrorCode;
//     use sd_jwt_vc_metadata::examples::VCT_EXAMPLE_CREDENTIAL;
//     use sd_jwt_vc_metadata::JsonSchemaPropertyType;
//     use sd_jwt_vc_metadata::NormalizedTypeMetadata;
//     use sd_jwt_vc_metadata::UncheckedTypeMetadata;
//     use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;

//     use crate::attestation::AttestationAttributeValue;
//     use crate::attestation::AttestationError;
//     use crate::config::UNIVERSAL_LINK_BASE_URL;
//     use crate::disclosure::mock::MockDisclosureMissingAttributes;
//     use crate::disclosure::mock::MockDisclosureProposal;
//     use crate::disclosure::mock::MockDisclosureSession;
//     use crate::storage::DisclosureStatus;
//     use crate::AttestationAttribute;

//     use super::super::test;
//     use super::super::test::WalletDeviceVendor;
//     use super::super::test::WalletWithMocks;
//     use super::super::test::ISSUER_KEY;
//     use super::*;

//     static DISCLOSURE_URI: LazyLock<Url> =
//         LazyLock::<Url>::new(|| urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).join("Zm9vYmFy?foo=bar"));
//     const PROPOSED_ID: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

//     impl<DCS> WalletDisclosureSession<DCS> {
//         pub(crate) fn new_browser(protocol_state: DCS) -> Self {
//             WalletDisclosureSession {
//                 redirect_uri_purpose: RedirectUriPurpose::Browser,
//                 protocol_state,
//             }
//         }
//     }

//     fn setup_proposed_attributes(attrs: &[(&str, DataElementValue)]) -> ProposedAttributes {
//         let metadata_props = attrs
//             .iter()
//             .map(|(name, value)| {
//                 (
//                     *name,
//                     match value {
//                         DataElementValue::Text(_) => JsonSchemaPropertyType::String,
//                         DataElementValue::Bool(_) => JsonSchemaPropertyType::Boolean,
//                         DataElementValue::Integer(_) => JsonSchemaPropertyType::Integer,
//                         DataElementValue::Float(_) => JsonSchemaPropertyType::Number,
//                         DataElementValue::Null => JsonSchemaPropertyType::Null,
//                         _ => unimplemented!(),
//                     },
//                     None,
//                 )
//             })
//             .collect::<Vec<_>>();

//         IndexMap::from([(
//             "com.example.pid".to_string(),
//             ProposedDocumentAttributes {
//                 type_metadata: NormalizedTypeMetadata::from_single_example(
//                     UncheckedTypeMetadata::example_with_claim_names("com.example.pid", &metadata_props),
//                 ),
//                 attributes: IndexMap::from([(
//                     "com.example.pid".to_string(),
//                     attrs
//                         .iter()
//                         .map(|(name, value)| Entry {
//                             name: String::from(*name),
//                             value: value.clone(),
//                         })
//                         .collect::<Vec<_>>(),
//                 )]),
//                 issuer: ISSUER_KEY.issuance_key.certificate().clone(),
//             },
//         )])
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_start_disclosure() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Set up an `MdocDisclosureSession` to be returned with the following values.
//         let reader_registration = ReaderRegistration::new_mock();
//         let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
//         let proposal_session = MockDisclosureProposal {
//             proposed_source_identifiers: vec![PROPOSED_ID],
//             proposed_attributes,
//             ..Default::default()
//         };

//         MockDisclosureSession::next_fields(
//             reader_registration,
//             DisclosureSessionState::Proposal(proposal_session),
//             None,
//         );

//         // Starting disclosure should not fail.
//         let proposal = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
//             .await
//             .expect("Could not start disclosure");

//         // Test that the `Wallet` now contains a `DisclosureSession`.
//         assert!(matches!(
//             wallet.session,
//             Some(Session::Disclosure(session))
//                 if session.protocol_state.uri_source == DisclosureUriSource::QrCode
//         ));

//         // Test that the returned `DisclosureProposal` contains the
//         // `ReaderRegistration` we set up earlier, as well as the
//         // proposed attributes converted to a `ProposedDisclosureDocument`.
//         assert_eq!(proposal.attestations.len(), 1);
//         let document = proposal.attestations.first().unwrap();
//         assert_eq!(document.attestation_type, "com.example.pid");
//         assert_eq!(document.attributes.len(), 1);
//         assert_matches!(
//             document.attributes.first().unwrap(),
//             AttestationAttribute {
//                 value: AttestationAttributeValue::Basic(AttributeValue::Bool(true)),
//                 ..
//             }
//         );

//         // Starting disclosure should not cause mdoc copy usage counts to be incremented.
//         assert!(wallet.storage.read().await.attestation_copies_usage_counts.is_empty());
//     }

//     #[tokio::test]
//     async fn test_wallet_start_disclosure_error_locked() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         wallet.lock();

//         // Starting disclosure on a locked wallet should result in an error.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::Locked);
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     async fn test_wallet_start_disclosure_error_unregistered() {
//         // Prepare an unregistered wallet.
//         let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

//         // Starting disclosure on an unregistered wallet should result in an error.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::NotRegistered);
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     async fn test_wallet_start_disclosure_error_session_state() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Start an active disclosure session.
//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             MockDisclosureSession::default(),
//         )));

//         // Starting disclosure on a wallet with an active disclosure should result in an error.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::SessionState);
//         assert!(wallet.session.is_some());
//     }

//     #[tokio::test]
//     async fn test_wallet_start_disclosure_error_disclosure_uri() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Starting disclosure on a wallet with an unknown disclosure URI should result in an error.
//         let error = wallet
//             .start_disclosure(
//                 &Url::parse("http://example.com?invalid").unwrap(),
//                 DisclosureUriSource::Link,
//             )
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::DisclosureUri(_));
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     async fn test_wallet_start_disclosure_error_disclosure_uri_query() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Starting disclosure on a wallet with a disclosure URI with no query parameters should result in an error.
//         let error = wallet
//             .start_disclosure(
//                 &urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).join("Zm9vYmFy"),
//                 DisclosureUriSource::Link,
//             )
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::DisclosureUriQuery(_));
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_start_disclosure_error_disclosure_session() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Set up an `MdocDisclosureSession` start to return the following error.
//         MockDisclosureSession::next_start_error(VpVerifierError::MissingSessionType.into());

//         // Starting disclosure which returns an error should forward that error.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::VpVerifierServer {
//                 error: VpVerifierError::MissingSessionType,
//                 ..
//             }
//         );
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_start_disclosure_error_return_url() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Set up an `MdocDisclosureSession` start to return the following error.
//         let return_url = Url::parse("https://example.com/return/here").unwrap();
//         MockDisclosureSession::next_start_error(
//             VpClientError::Request(
//                 DisclosureErrorResponse {
//                     error_response: ErrorResponse {
//                         error: GetRequestErrorCode::ServerError,
//                         error_description: None,
//                         error_uri: None,
//                     },
//                     redirect_uri: Some(return_url.clone().try_into().unwrap()),
//                 }
//                 .into(),
//             )
//             .into(),
//         );

//         // Starting disclosure where the verifier returns responds with a HTTP error body containing
//         // a redirect URI should result in that URI being available on the returned error.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::VpClient(_));
//         assert_eq!(error.return_url(), Some(&return_url));
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_start_disclosure_error_attributes_not_available() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Set up an `MdocDisclosureSession` start to return that attributes are not available.
//         let missing_attributes = vec!["com.example.pid/com.example.pid/age_over_18".parse().unwrap()];
//         let mut missing_attr_session = MockDisclosureMissingAttributes::default();
//         missing_attr_session
//             .expect_missing_attributes()
//             .return_const(missing_attributes);

//         MockDisclosureSession::next_fields(
//             ReaderRegistration::new_mock(),
//             DisclosureSessionState::MissingAttributes(missing_attr_session),
//             None,
//         );

//         // Starting disclosure where an unavailable attribute is requested should result in an error.
//         // As an exception, this error should leave the `Wallet` with an active disclosure session.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::AttributesNotAvailable {
//                 reader_registration: _,
//                 missing_attributes,
//                 shared_data_with_relying_party_before,
//                 session_type: SessionType::SameDevice,
//             } if !shared_data_with_relying_party_before &&
//                 missing_attributes == vec!["com.example.pid/com.example.pid/age_over_18"]
//         );
//         assert!(wallet.session.is_some());
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_start_disclosure_error_mdoc_attributes() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Set up an `MdocDisclosureSession` to be returned with the following values.
//         let mut proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);

//         proposed_attributes
//             .get_mut("com.example.pid")
//             .unwrap()
//             .attributes
//             .get_mut("com.example.pid")
//             .unwrap()
//             .push(Entry {
//                 name: "foo".to_string(),
//                 value: DataElementValue::Text("bar".to_string()),
//             });

//         let proposal_session = MockDisclosureProposal {
//             proposed_attributes,
//             ..Default::default()
//         };

//         MockDisclosureSession::next_fields(
//             ReaderRegistration::new_mock(),
//             DisclosureSessionState::Proposal(proposal_session),
//             None,
//         );

//         // Starting disclosure where unknown attributes are requested should result in an error.
//         let error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::AttestationAttributes(
//                 AttestationError::Attributes(AttributesError::SomeAttributesNotProcessed(claims)))
//                 if claims == IndexMap::from([
//                     (String::from("com.example.pid"),
//                     vec![Entry {
//                         name: String::from("foo"),
//                         value: ciborium::value::Value::Text(String::from("bar"))
//                     }]
//                 )]
//             )
//         );

//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_cancel_disclosure() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

//         // Set up an `MdocDisclosureSession` to be returned with the following values.
//         let reader_registration = ReaderRegistration::new_mock();
//         let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
//         let proposal_session = MockDisclosureProposal {
//             proposed_source_identifiers: vec![PROPOSED_ID],
//             proposed_attributes,
//             ..Default::default()
//         };

//         let return_url = BaseUrl::from_str("https://example.com/return/here").unwrap();

//         MockDisclosureSession::next_fields(
//             reader_registration,
//             DisclosureSessionState::Proposal(proposal_session),
//             Some(return_url.clone()),
//         );

//         // Start a disclosure session, to ensure a proper session exists that can be cancelled.
//         let _ = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect("Could not start disclosure");

//         // Verify disclosure session is not yet terminated
//         let Some(Session::Disclosure(session)) = wallet.session.as_ref() else {
//             panic!("wallet in unexpected state")
//         };
//         let was_terminated = Arc::clone(&session.protocol_state.was_terminated);
//         assert!(!was_terminated.load(Ordering::Relaxed));

//         // Get latest emitted recent_history events
//         let latest_events = events.lock().pop().unwrap();
//         // Verify no history events are yet logged
//         assert!(latest_events.is_empty());

//         // Cancelling disclosure should result in a `Wallet` without a disclosure
//         // session, while the session that was there should be terminated.
//         let cancel_return_url = wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

//         assert_eq!(cancel_return_url.as_ref(), Some(return_url.as_ref()));

//         // Verify disclosure session is terminated
//         assert!(wallet.session.is_none());
//         assert!(was_terminated.load(Ordering::Relaxed));

//         // Get latest emitted recent_history events
//         let events = events.lock().pop().unwrap();
//         // Verify a Disclosure Cancel event is logged
//         assert_eq!(events.len(), 1);
//         assert_matches!(
//             &events[0],
//             WalletEvent::Disclosure {
//                 status: DisclosureStatus::Cancelled,
//                 ..
//             }
//         );

//         // Cancelling disclosure should not cause mdoc copy usage counts to be incremented.
//         assert!(wallet.storage.read().await.attestation_copies_usage_counts.is_empty());
//     }

//     #[tokio::test]
//     #[serial(MockMdocDisclosureSession)]
//     async fn test_wallet_cancel_disclosure_missing_attributes() {
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

//         // Set up an `MdocDisclosureSession` start to return that attributes are not available.
//         let missing_attributes = vec![
//             "com.example.pid/com.example.pid/bsn".parse().unwrap(),
//             "com.example.pid/com.example.pid/age_over_18".parse().unwrap(),
//         ];
//         let mut missing_attr_session = MockDisclosureMissingAttributes::default();
//         missing_attr_session
//             .expect_missing_attributes()
//             .return_const(missing_attributes);

//         let return_url = BaseUrl::from_str("https://example.com/return/here").unwrap();

//         MockDisclosureSession::next_fields(
//             ReaderRegistration::new_mock(),
//             DisclosureSessionState::MissingAttributes(missing_attr_session),
//             Some(return_url.clone()),
//         );

//         // Starting disclosure where an unavailable attribute is requested should result in an error.
//         // As an exception, this error should leave the `Wallet` with an active disclosure session.
//         let _error = wallet
//             .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
//             .await
//             .expect_err("Starting disclosure should have resulted in an error");
//         assert!(wallet.session.is_some());

//         // Verify disclosure session is not yet terminated
//         let Some(Session::Disclosure(session)) = wallet.session.as_ref() else {
//             panic!("wallet in unexpected state")
//         };
//         let was_terminated = Arc::clone(&session.protocol_state.was_terminated);
//         assert!(!was_terminated.load(Ordering::Relaxed));

//         // Get latest emitted recent_history events
//         let latest_events = events.lock().pop().unwrap();
//         // Verify no history events are yet logged
//         assert!(latest_events.is_empty());

//         // Cancelling disclosure should result in a `Wallet` without a disclosure
//         // session, while the session that was there should be terminated.
//         let cancel_return_url = wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

//         assert_eq!(cancel_return_url.as_ref(), Some(return_url.as_ref()));

//         // Verify disclosure session is terminated
//         assert!(wallet.session.is_none());
//         assert!(was_terminated.load(Ordering::Relaxed));

//         // Get latest emitted recent_history events
//         let events = events.lock().pop().unwrap();
//         // Verify a single Disclosure Error event is logged
//         assert_eq!(events.len(), 1);
//         assert_matches!(
//             &events[0],
//             WalletEvent::Disclosure {
//                 status: DisclosureStatus::Cancelled,
//                 ..
//             }
//         );
//     }

//     #[tokio::test]
//     async fn test_wallet_cancel_disclosure_error_locked() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             MockDisclosureSession::default(),
//         )));

//         wallet.lock();

//         // Cancelling disclosure on a locked wallet should result in an error.
//         let error = wallet
//             .cancel_disclosure()
//             .await
//             .expect_err("Cancelling disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::Locked);
//         assert!(wallet.session.is_some());
//     }

//     #[tokio::test]
//     async fn test_wallet_cancel_disclosure_error_unregistered() {
//         // Prepare an unregistered wallet.
//         let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

//         // Cancelling disclosure on an unregistered wallet should result in an error.
//         let error = wallet
//             .cancel_disclosure()
//             .await
//             .expect_err("Cancelling disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::NotRegistered);
//         assert!(wallet.session.is_none());
//     }

//     #[tokio::test]
//     async fn test_wallet_cancel_disclosure_error_session_state() {
//         // Prepare a registered and unlocked wallet.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Cancelling disclosure on a wallet without an active
//         // disclosure session should result in an error.
//         let error = wallet
//             .cancel_disclosure()
//             .await
//             .expect_err("Cancelling disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::SessionState);
//         assert!(wallet.session.is_none());
//     }

//     const PIN: &str = "051097";

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

//         let return_url = BaseUrl::from_str("https://example.com/return/here").unwrap();

//         let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
//         let disclosure_session = MockDisclosureProposal {
//             disclose_return_url: Some(return_url.clone()),
//             proposed_source_identifiers: vec![PROPOSED_ID],
//             proposed_attributes,
//             ..Default::default()
//         };

//         // Create a `MockMdocDisclosureSession` with the return URL and the `MockMdocDisclosureProposal`,
//         // copy the disclosure count and check that it is 0.
//         let disclosure_session = MockDisclosureSession {
//             session_state: DisclosureSessionState::Proposal(disclosure_session),
//             ..Default::default()
//         };
//         let disclosure_count = match disclosure_session.session_state {
//             DisclosureSessionState::Proposal(ref proposal) => Arc::clone(&proposal.disclosure_count),
//             _ => unreachable!(),
//         };
//         assert_eq!(disclosure_count.load(Ordering::Relaxed), 0);

//         let reader_certificate = disclosure_session.certificate.clone();

//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             disclosure_session,
//         )));

//         // Accepting disclosure should succeed and give us the return URL.
//         let accept_result = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect("Could not accept disclosure");

//         assert_matches!(accept_result, Some(result_return_url) if &result_return_url == return_url.as_ref());

//         // Check that the disclosure session is no longer
//         // present and that the disclosure count is 1.
//         assert!(wallet.session.is_none());
//         assert!(!wallet.is_locked());
//         assert_eq!(disclosure_count.load(Ordering::Relaxed), 1);

//         // Get latest emitted recent_history events
//         let events = events.lock().pop().unwrap();

//         // Verify a single Disclosure Success event is logged, and documents are shared
//         assert_eq!(events.len(), 1);
//         assert_matches!(
//             &events[0],
//             WalletEvent::Disclosure {
//                 status: DisclosureStatus::Success,
//                 attestations,
//                 ..
//             } if !attestations.is_empty()
//         );
//         // Verify that `did_share_data_with_relying_party()` now returns `true`
//         assert!(wallet
//             .storage
//             .read()
//             .await
//             .did_share_data_with_relying_party(&reader_certificate)
//             .await
//             .unwrap());

//         // Test that the usage count got incremented for the proposed mdoc copy id.
//         let mdoc_copies_usage_counts = &wallet.storage.read().await.attestation_copies_usage_counts;
//         assert_eq!(mdoc_copies_usage_counts.len(), 1);
//         assert_eq!(
//             mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
//             1
//         );
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_locked() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             MockDisclosureSession::default(),
//         )));

//         wallet.lock();

//         // Accepting disclosure on a locked wallet should result in an error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::Locked);
//         assert!(wallet.session.is_some());
//         assert!(wallet.is_locked());

//         let Some(Session::Disclosure(session)) = &wallet.session else {
//             panic!("wallet in unexpected state")
//         };
//         let DisclosureSessionState::Proposal(proposal) = &session.protocol_state.session_state else {
//             panic!("wallet in unexpected state")
//         };
//         assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);

//         // The mdoc copy usage counts should not be incremented.
//         assert!(wallet.storage.read().await.attestation_copies_usage_counts.is_empty());

//         // Verify no Disclosure events are logged
//         assert!(wallet
//             .storage
//             .read()
//             .await
//             .fetch_wallet_events()
//             .await
//             .unwrap()
//             .is_empty());
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_unregistered() {
//         // Prepare an unregistered wallet.
//         let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

//         // Accepting disclosure on an unregistered wallet should result in an error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::NotRegistered);
//         assert!(wallet.session.is_none());
//         assert!(wallet.is_locked());

//         // Verify no Disclosure events are logged
//         assert!(wallet
//             .storage
//             .read()
//             .await
//             .fetch_wallet_events()
//             .await
//             .unwrap()
//             .is_empty());
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_session_state_no_session() {
//         // Prepare a registered and unlocked wallet.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         // Accepting disclosure on a wallet without an active
//         // disclosure session should result in an error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::SessionState);
//         assert!(wallet.session.is_none());
//         assert!(!wallet.is_locked());
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_session_state_missing_attributes() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let disclosure_session = MockDisclosureSession {
//             session_state: DisclosureSessionState::MissingAttributes(Default::default()),
//             ..Default::default()
//         };
//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             disclosure_session,
//         )));

//         // Accepting disclosure on a wallet that has a disclosure session
//         // with missing attributes should result in an error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::SessionState);
//         assert!(wallet.session.is_some());
//         assert!(!wallet.is_locked());

//         // The mdoc copy usage counts should not be incremented.
//         assert!(wallet.storage.read().await.attestation_copies_usage_counts.is_empty());

//         // Verify no Disclosure events are logged
//         assert!(wallet
//             .storage
//             .read()
//             .await
//             .fetch_wallet_events()
//             .await
//             .unwrap()
//             .is_empty());
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_disclosure_session() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

//         let response = DisclosureErrorResponse {
//             error_response: ErrorResponse {
//                 error: PostAuthResponseErrorCode::InvalidRequest,
//                 error_description: None,
//                 error_uri: None,
//             },
//             redirect_uri: None,
//         };
//         let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
//         let disclosure_session = MockDisclosureSession {
//             session_state: DisclosureSessionState::Proposal(MockDisclosureProposal {
//                 proposed_source_identifiers: vec![PROPOSED_ID],
//                 proposed_attributes,
//                 next_error: Mutex::new(Some(VpSessionError::Client(VpClientError::Request(response.into())))),
//                 ..Default::default()
//             }),
//             ..Default::default()
//         };
//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             disclosure_session,
//         )));

//         // Accepting disclosure when the verifier responds with
//         // an invalid request error should result in an error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::VpClient(VpClientError::Request(VpMessageClientError::AuthPostResponse(_)))
//         );
//         assert!(wallet.session.is_some());
//         assert!(!wallet.is_locked());
//         let Some(Session::Disclosure(session)) = &wallet.session else {
//             panic!("wallet in unexpected state")
//         };
//         let DisclosureSessionState::Proposal(proposal) = &session.protocol_state.session_state else {
//             panic!("wallet in unexpected state")
//         };
//         assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);

//         // Test that the usage count got incremented for the proposed mdoc copy id.
//         assert_eq!(wallet.storage.read().await.attestation_copies_usage_counts.len(), 1);
//         assert_eq!(
//             wallet
//                 .storage
//                 .read()
//                 .await
//                 .attestation_copies_usage_counts
//                 .get(&PROPOSED_ID)
//                 .copied()
//                 .unwrap_or_default(),
//             1
//         );

//         // Get latest emitted recent_history events
//         let events = events.lock().pop().unwrap();
//         // Verify a Disclosure error event is logged, with no documents shared
//         assert_eq!(events.len(), 1);
//         assert_matches!(
//             &events[0],
//             WalletEvent::Disclosure {
//                 status: DisclosureStatus::Error,
//                 attestations,
//                 ..
//             } if attestations.is_empty()
//         );

//         // Set up the disclosure session to return a different error.
//         match session.protocol_state.session_state {
//             DisclosureSessionState::Proposal(ref proposal) => proposal.next_error.lock().replace(
//                 VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(
//                     RemoteEcdsaKeyError::KeyNotFound("foobar".to_string()).into(),
//                 )))
//                 .into(),
//             ),
//             _ => unreachable!(),
//         };

//         // Accepting disclosure when the wallet provider responds that key with
//         // a particular identifier is not present should result in an error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::VpClient(VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(_))))
//         );
//         assert!(wallet.session.is_some());
//         assert!(!wallet.is_locked());
//         let Some(Session::Disclosure(session)) = &wallet.session else {
//             panic!("wallet in unexpected state")
//         };
//         let DisclosureSessionState::Proposal(proposal) = &session.protocol_state.session_state else {
//             panic!("wallet in unexpected state")
//         };
//         assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);

//         // Test that the usage count got incremented again for the proposed mdoc copy id.
//         let mdoc_copies_usage_counts = &wallet.storage.read().await.attestation_copies_usage_counts;
//         assert_eq!(mdoc_copies_usage_counts.len(), 1);
//         assert_eq!(
//             mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
//             2
//         );

//         // Verify another Disclosure error event is logged, with no documents shared
//         let events = wallet.storage.read().await.fetch_wallet_events().await.unwrap();
//         assert_eq!(events.len(), 2);
//         assert_matches!(
//             &events[1],
//             WalletEvent::Disclosure {
//                 status: DisclosureStatus::Error,
//                 attestations,
//                 ..
//             } if attestations.is_empty()
//         );
//     }

//     #[rstest]
//     #[case(InstructionError::IncorrectPin{ attempts_left_in_round: 1, is_final_round: false }, false, false)]
//     #[case(InstructionError::Timeout{ timeout_millis: 10_000 }, true, true)]
//     #[case(InstructionError::Blocked, true, true)]
//     #[case(InstructionError::InstructionValidation, false, true)]
//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_instruction(
//         #[case] instruction_error: InstructionError,
//         #[case] expect_termination: bool,
//         #[case] expect_history_log: bool,
//     ) {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

//         let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
//         let disclosure_session = MockDisclosureSession {
//             session_state: DisclosureSessionState::Proposal(MockDisclosureProposal {
//                 proposed_source_identifiers: vec![PROPOSED_ID],
//                 proposed_attributes,
//                 next_error: Mutex::new(Some(VpSessionError::Client(VpClientError::DeviceResponse(
//                     mdoc::Error::Cose(CoseError::Signing(
//                         RemoteEcdsaKeyError::Instruction(instruction_error).into(),
//                     )),
//                 )))),
//                 ..Default::default()
//             }),
//             ..Default::default()
//         };

//         let was_terminated = Arc::clone(&disclosure_session.was_terminated);
//         assert!(!was_terminated.load(Ordering::Relaxed));

//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             disclosure_session,
//         )));

//         // Accepting disclosure when the verifier responds with an `InstructionError` indicating
//         // that the account is blocked should result in a `DisclosureError::Instruction` error.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(error, DisclosureError::Instruction(_));

//         if expect_termination {
//             // If the disclosure session should be terminated, there
//             // should be no session, the wallet should be locked and
//             // the session should be terminated at the remote end.
//             assert!(wallet.session.is_none());
//             assert!(wallet.is_locked());
//             assert!(was_terminated.load(Ordering::Relaxed));
//         } else {
//             // Otherwise, the session should still be present, the wallet
//             // unlocked and the session should not be terminated.
//             assert!(wallet.session.is_some());
//             assert!(!wallet.is_locked());
//             assert!(!was_terminated.load(Ordering::Relaxed));
//         }

//         // Test that the usage count got incremented for the proposed mdoc copy id.
//         let mdoc_copies_usage_counts = &wallet.storage.read().await.attestation_copies_usage_counts;
//         assert_eq!(mdoc_copies_usage_counts.len(), 1);
//         assert_eq!(
//             mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
//             1
//         );

//         // Get latest emitted recent_history events
//         let events = events.lock().pop().unwrap();

//         match (expect_termination, expect_history_log) {
//             (true, true) => {
//                 // Verify both a disclosure cancellation and error event are logged
//                 assert_eq!(events.len(), 2);
//                 assert_matches!(
//                     &events[0],
//                     WalletEvent::Disclosure {
//                         status: DisclosureStatus::Cancelled,
//                         ..
//                     }
//                 );
//                 assert_matches!(
//                     &events[1],
//                     WalletEvent::Disclosure {
//                         status: DisclosureStatus::Error,
//                         attestations,
//                         ..
//                     } if attestations.is_empty()
//                 );
//             }
//             (false, true) => {
//                 // Verify a disclosure error event is logged
//                 assert_eq!(events.len(), 1);
//                 assert_matches!(
//                     &events[0],
//                     WalletEvent::Disclosure {
//                         status: DisclosureStatus::Error,
//                         attestations,
//                         ..
//                     } if attestations.is_empty()
//                 );
//             }
//             (false, false) => {
//                 assert_eq!(events.len(), 0);
//             }
//             (true, false) => {
//                 panic!(
//                     "In practice this cannot happen, as the InstructionError cannot be both (Timeout or Blocked) and \
//                      IncorrectPin"
//                 );
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_holder_attributes_are_shared() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

//         let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
//         let disclosure_session = MockDisclosureSession {
//             session_state: DisclosureSessionState::Proposal(MockDisclosureProposal {
//                 proposed_source_identifiers: vec![PROPOSED_ID],
//                 proposed_attributes,
//                 next_error: Mutex::new(Some(VpVerifierError::MissingReaderRegistration.into())),
//                 attributes_shared: true,
//                 ..Default::default()
//             }),
//             ..Default::default()
//         };

//         let reader_certificate = disclosure_session.certificate.clone();

//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_browser(
//             disclosure_session,
//         )));

//         // Accepting disclosure when the verifier responds with an error indicating that
//         // attributes were shared should result in a disclosure event being recorded.
//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::VpVerifierServer {
//                 error: VpVerifierError::MissingReaderRegistration,
//                 ..
//             }
//         );
//         assert!(wallet.session.is_some());
//         assert!(!wallet.is_locked());
//         let Some(Session::Disclosure(session)) = &wallet.session else {
//             panic!("wallet in unexpected state")
//         };
//         let DisclosureSessionState::Proposal(proposal) = &session.protocol_state.session_state else {
//             panic!("wallet in unexpected state")
//         };
//         assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);

//         // Test that the usage count got incremented for the proposed mdoc copy id.
//         let mdoc_copies_usage_counts = &wallet.storage.read().await.attestation_copies_usage_counts;
//         assert_eq!(mdoc_copies_usage_counts.len(), 1);
//         assert_eq!(
//             mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
//             1
//         );

//         // Get latest emitted recent_history events
//         let events = events.lock().pop().unwrap();
//         // Verify a Disclosure error event is logged, and documents are shared
//         assert_eq!(events.len(), 1);
//         assert_matches!(
//             &events[0],
//             WalletEvent::Disclosure {
//                 status: DisclosureStatus::Error,
//                 attestations,
//                 ..
//             } if !attestations.is_empty()
//         );
//         assert!(wallet
//             .storage
//             .read()
//             .await
//             .did_share_data_with_relying_party(&reader_certificate)
//             .await
//             .unwrap());
//     }

//     #[tokio::test]
//     async fn test_wallet_accept_disclosure_error_wrong_redirect_uri_purpose() {
//         // Prepare a registered and unlocked wallet with an active disclosure session.
//         let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

//         let disclosure_session = MockDisclosureSession::default();
//         wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new(
//             RedirectUriPurpose::Issuance,
//             disclosure_session,
//         )));

//         let error = wallet
//             .accept_disclosure(PIN.to_owned())
//             .await
//             .expect_err("Accepting disclosure should have resulted in an error");

//         assert_matches!(
//             error,
//             DisclosureError::UnexpectedRedirectUriPurpose {
//                 expected: RedirectUriPurpose::Issuance,
//                 found: RedirectUriPurpose::Browser,
//             }
//         );
//     }

//     #[tokio::test]
//     async fn test_mdoc_by_doc_types() {
//         // Prepare a wallet in initial state.
//         let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

//         // Create some fake `Mdoc` entries to place into wallet storage.
//         let mdoc1 = Mdoc::new_mock().await;
//         let mdoc2 = Mdoc::new_mock_with_doctype("com.example.doc_type").await;

//         let credential1 = IssuedCredential::MsoMdoc(Box::new(mdoc1.clone()));
//         let credential2 = IssuedCredential::MsoMdoc(Box::new(mdoc2.clone()));

//         // Place 3 copies of each `Mdoc` into `MockStorage`.
//         wallet
//             .storage
//             .write()
//             .await
//             .insert_credentials(vec![
//                 CredentialWithMetadata::new(
//                     IssuedCredentialCopies::new_or_panic(
//                         vec![credential1.clone(), credential1.clone(), credential1.clone()]
//                             .try_into()
//                             .unwrap(),
//                     ),
//                     String::from(VCT_EXAMPLE_CREDENTIAL),
//                     VerifiedTypeMetadataDocuments::nl_pid_example(),
//                 ),
//                 CredentialWithMetadata::new(
//                     IssuedCredentialCopies::new_or_panic(
//                         vec![credential2.clone(), credential2.clone(), credential2.clone()]
//                             .try_into()
//                             .unwrap(),
//                     ),
//                     String::from("com.example.doc_type"),
//                     // Note that the attestation type of this metadata does not match the mdoc doc_type,
//                     // which is not relevant for this particular test.
//                     VerifiedTypeMetadataDocuments::nl_pid_example(),
//                 ),
//             ])
//             .await
//             .unwrap();

//         // Call the `MdocDataSource.mdoc_by_doc_types()` method on the `Wallet`.
//         let mdoc_by_doc_types = wallet
//             .mdoc_by_doc_types(&["com.example.doc_type", PID].into())
//             .await
//             .expect("Could not get mdocs by doc types from wallet");

//         // The result should be one copy of each distinct `Mdoc`,
//         // while retaining the original insertion order.
//         assert_eq!(mdoc_by_doc_types.len(), 2);
//         assert_eq!(mdoc_by_doc_types[0].len(), 1);
//         assert_eq!(mdoc_by_doc_types[1].len(), 1);

//         assert_matches!(&mdoc_by_doc_types[0][0], StoredMdoc { mdoc, .. } if *mdoc == mdoc1);
//         assert_matches!(&mdoc_by_doc_types[1][0], StoredMdoc { mdoc, .. } if *mdoc == mdoc2);

//         let unique_ids = mdoc_by_doc_types
//             .into_iter()
//             .flat_map(|stored_mdocs| stored_mdocs.into_iter().map(|stored_mdoc| stored_mdoc.id))
//             .unique()
//             .collect::<Vec<_>>();

//         assert_eq!(unique_ids.len(), 2);
//     }

//     #[tokio::test]
//     async fn test_mdoc_by_doc_types_empty() {
//         // Prepare a wallet in initial state.
//         let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

//         // Calling the `MdocDataSource.mdoc_by_doc_types()` method
//         // on the `Wallet` should return an empty result.
//         let mdoc_by_doc_types = wallet
//             .mdoc_by_doc_types(&Default::default())
//             .await
//             .expect("Could not get mdocs by doc types from wallet");

//         assert!(mdoc_by_doc_types.is_empty());
//     }

//     #[tokio::test]
//     async fn test_mdoc_by_doc_types_error() {
//         // Prepare a wallet in initial state.
//         let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

//         // Set up `MockStorage` to return an error when performing a query.
//         wallet.storage.write().await.has_query_error = true;

//         // Calling the `MdocDataSource.mdoc_by_doc_types()` method
//         // on the `Wallet` should forward the `StorageError`.
//         let error = wallet
//             .mdoc_by_doc_types(&Default::default())
//             .await
//             .expect_err("Getting mdocs by doc types from wallet should result in an error");

//         assert_matches!(error, StorageError::Database(_));
//     }
// }
