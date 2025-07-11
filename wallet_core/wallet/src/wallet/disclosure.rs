use std::collections::HashSet;
use std::iter;
use std::sync::Arc;

use chrono::Utc;
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
use entity::disclosure_event::EventStatus;
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
    pub(super) redirect_uri_purpose: RedirectUriPurpose,
    pub(super) disclosure_type: DisclosureType,
    pub(super) attestations: Option<VecNonEmpty<DisclosureAttestation>>,
    pub(super) protocol_state: DCS,
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
                // Get a reference from `requested_attestation_types` for `.chunk_by()`,
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

                // If none of the requested paths map to a 2-tuple, there can be no mdoc candidates.
                if mdoc_paths.is_empty() {
                    return None;
                }

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

        // At this point, determine the disclosure type and if data was ever shared with this RP before, as the UI
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
                        // TODO (PVW-4132): Mdoc attestations contained in the database should be assumed to be valid.
                        //                  Once this is expressed within the type system, these errors can be removed.
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
                            stored_mdoc.mdoc_id.to_string(),
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
        // 3. We then check above that none of the attestation types have multiple candidates, so the length of
        //    disclosure_attestations is the same as attestations_by_type, which is at least 1.
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

        let (reader_certificate, _) = session
            .protocol_state
            .verifier_certificate()
            .clone()
            .into_certificate_and_registration();

        let return_url = session.protocol_state.terminate().await?.map(BaseUrl::into_inner);

        self.store_disclosure_event(
            Utc::now(),
            attestations,
            reader_certificate,
            session.disclosure_type,
            EventStatus::Cancelled,
            DataDisclosureStatus::NotDisclosed,
        )
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

        let (reader_certificate, reader_registration) = session
            .protocol_state
            .verifier_certificate()
            .clone()
            .into_certificate_and_registration();

        if let Err(error) = result {
            if let Err(e) = self
                .store_disclosure_event(
                    Utc::now(),
                    Some(
                        attestations
                            .iter()
                            .map(|attestation| attestation.presentation.clone())
                            .collect_vec()
                            .try_into()
                            .unwrap(),
                    ),
                    reader_certificate,
                    session.disclosure_type,
                    EventStatus::Error,
                    DataDisclosureStatus::NotDisclosed,
                )
                .await
            {
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
                let disclosure_error =
                    DisclosureError::with_organization(error.error, reader_registration.organization);

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
                    if let Err(e) = self
                        .store_disclosure_event(
                            Utc::now(),
                            Some(
                                session
                                    .attestations
                                    .as_ref()
                                    .unwrap()
                                    .iter()
                                    .map(|attestation| attestation.presentation.clone())
                                    .collect_vec()
                                    .try_into()
                                    .unwrap(),
                            ),
                            reader_certificate,
                            session.disclosure_type,
                            EventStatus::Error,
                            data_status,
                        )
                        .await
                    {
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
        self.store_disclosure_event(
            Utc::now(),
            Some(
                session
                    .attestations
                    .unwrap()
                    .into_iter()
                    .map(|attestation| attestation.presentation)
                    .collect_vec()
                    .try_into()
                    .unwrap(),
            ),
            reader_certificate,
            session.disclosure_type,
            EventStatus::Success,
            DataDisclosureStatus::Disclosed,
        )
        .await
        .map_err(DisclosureError::EventStorage)?;

        Ok(return_url)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use itertools::Itertools;
    use mockall::predicate::always;
    use mockall::predicate::eq;
    use rstest::rstest;
    use serde::de::Error;
    use url::Url;
    use uuid::Uuid;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::auth::Organization;
    use attestation_data::disclosure_type::DisclosureType;
    use attestation_data::x509::generate::mock::generate_reader_mock;
    use attestation_types::attribute_paths::AttestationAttributePaths;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::BorrowingCertificateExtension;
    use http_utils::urls;
    use http_utils::urls::BaseUrl;
    use mdoc::utils::cose::CoseError;
    use openid4vc::disclosure_session;
    use openid4vc::disclosure_session::mock::MockDisclosureClient;
    use openid4vc::disclosure_session::mock::MockDisclosureSession;
    use openid4vc::disclosure_session::DisclosureUriSource;
    use openid4vc::disclosure_session::VerifierCertificate;
    use openid4vc::disclosure_session::VpClientError;
    use openid4vc::disclosure_session::VpMessageClientError;
    use openid4vc::disclosure_session::VpSessionError;
    use openid4vc::disclosure_session::VpVerifierError;
    use openid4vc::errors::DisclosureErrorResponse;
    use openid4vc::errors::ErrorResponse;
    use openid4vc::errors::GetRequestErrorCode;
    use openid4vc::issuance_session::IssuedCredential;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::verifier::SessionType;
    use openid4vc::PostAuthResponseErrorCode;
    use update_policy_model::update_policy::VersionState;
    use utils::vec_at_least::VecNonEmpty;

    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationIdentity;
    use crate::attestation::AttestationPresentation;
    use crate::config::UNIVERSAL_LINK_BASE_URL;
    use crate::errors::InstructionError;
    use crate::errors::RemoteEcdsaKeyError;
    use crate::issuance::MockDigidSession;
    use crate::issuance::PID_DOCTYPE;
    use crate::storage::DisclosureStatus;
    use crate::storage::WalletEvent;
    use crate::wallet::disclosure::DisclosureAttestation;
    use crate::wallet::test::setup_mock_recent_history_callback;

    use super::super::test::create_example_pid_mdoc_credential;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::super::Session;
    use super::DisclosureError;
    use super::DisclosureProposalPresentation;
    use super::RedirectUriPurpose;
    use super::WalletDisclosureSession;

    static DISCLOSURE_URI: LazyLock<Url> =
        LazyLock::new(|| urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).join("Zm9vYmFy?foo=bar"));
    const PIN: &str = "051097";
    static RETURN_URL: LazyLock<BaseUrl> =
        LazyLock::new(|| BaseUrl::from_str("https://example.com/return/here").unwrap());
    static DEFAULT_REQUESTED_PID_PATH: LazyLock<VecNonEmpty<String>> = LazyLock::new(|| {
        vec![PID_DOCTYPE.to_string(), "age_over_18".to_string()]
            .try_into()
            .unwrap()
    });

    // Set up properties for a `MockDisclosureSession`.
    fn setup_disclosure_session_verifier_certificate(
        verifier_certificate: VerifierCertificate,
        requested_pid_path: VecNonEmpty<String>,
    ) -> MockDisclosureSession {
        let requested_attribute_paths = AttestationAttributePaths::try_new(HashMap::from([(
            PID_DOCTYPE.to_string(),
            HashSet::from([requested_pid_path]),
        )]))
        .unwrap();

        let mut disclosure_session = MockDisclosureSession::new();
        disclosure_session
            .expect_session_type()
            .return_const(SessionType::CrossDevice);
        disclosure_session
            .expect_verifier_certificate()
            .return_const(verifier_certificate);
        disclosure_session
            .expect_requested_attribute_paths()
            .return_const(requested_attribute_paths);

        disclosure_session
    }

    // Set up properties for a `MockDisclosureSession`.
    fn setup_disclosure_session(
        requested_pid_path: VecNonEmpty<String>,
    ) -> (MockDisclosureSession, VerifierCertificate) {
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let reader_registration = ReaderRegistration::new_mock();
        let key_pair = generate_reader_mock(&ca, Some(reader_registration)).unwrap();
        let verifier_certificate = VerifierCertificate::try_new(key_pair.into()).unwrap().unwrap();

        let disclosure_session =
            setup_disclosure_session_verifier_certificate(verifier_certificate.clone(), requested_pid_path);

        (disclosure_session, verifier_certificate)
    }

    /// Set up the expected response of `MockDisclosureClient` when starting a new `MockDisclosureSession`.
    fn setup_disclosure_client_start(
        disclosure_client: &mut MockDisclosureClient,
        requested_pid_path: VecNonEmpty<String>,
    ) -> VerifierCertificate {
        let (disclosure_session, verifier_certificate) = setup_disclosure_session(requested_pid_path);

        disclosure_client
            .expect_start()
            .times(1)
            .with(eq("foo=bar"), eq(DisclosureUriSource::QrCode), always())
            .return_once(|_request_uri_query, _uri_source, _trust_anchors| Ok(disclosure_session));

        verifier_certificate
    }

    fn setup_wallet_disclosure_session_missing_attributes() -> (
        Session<MockDigidSession, MockIssuanceSession, MockDisclosureSession>,
        VerifierCertificate,
    ) {
        let (disclosure_session, verifier_certificate) = setup_disclosure_session(DEFAULT_REQUESTED_PID_PATH.clone());

        let session = Session::Disclosure(WalletDisclosureSession::new_missing_attributes(
            RedirectUriPurpose::Browser,
            DisclosureType::Regular,
            disclosure_session,
        ));

        (session, verifier_certificate)
    }

    fn setup_wallet_disclosure_session() -> (
        Session<MockDigidSession, MockIssuanceSession, MockDisclosureSession>,
        VerifierCertificate,
    ) {
        let (disclosure_session, verifier_certificate) = setup_disclosure_session(DEFAULT_REQUESTED_PID_PATH.clone());

        let mdoc_credential = create_example_pid_mdoc_credential();
        let metadata = mdoc_credential.metadata_documents.to_normalized().unwrap();
        let IssuedCredential::MsoMdoc(mdoc) = mdoc_credential.copies.into_inner().into_first() else {
            unreachable!();
        };
        let issuer_registration = IssuerRegistration::from_certificate(&mdoc.issuer_certificate().unwrap())
            .unwrap()
            .unwrap();
        let attributes = mdoc.clone().issuer_signed.into_entries_by_namespace();
        let presentation = AttestationPresentation::create_for_disclosure(
            String::from("id123"),
            metadata,
            issuer_registration.organization,
            attributes,
        )
        .unwrap();

        let session = Session::Disclosure(WalletDisclosureSession::new_proposal(
            RedirectUriPurpose::Browser,
            DisclosureType::Regular,
            vec![DisclosureAttestation {
                copy_id: Uuid::new_v4(),
                mdoc: *mdoc,
                presentation,
            }]
            .try_into()
            .unwrap(),
            disclosure_session,
        ));

        (session, verifier_certificate)
    }

    // TODO (PVW-3829): Add tests with more elaborate candidation selection, e.g. requests spanning multiple attestation
    //                  types and multiple attestation type instances with optional attributes.

    /// This tests the full happy path for disclosure, calling both
    /// `Wallet::start_disclosure()` and `Wallet::accept_disclosure()`.
    #[tokio::test]
    async fn test_wallet_disclosure() {
        // Populate a registered wallet with an example PID.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let mdoc_credential = create_example_pid_mdoc_credential();
        wallet
            .mut_storage()
            .issued_credential_copies
            .insert(mdoc_credential.attestation_type.clone(), vec![mdoc_credential]);

        // Set up the relevant mocks.
        let verifier_certificate =
            setup_disclosure_client_start(&mut wallet.disclosure_client, DEFAULT_REQUESTED_PID_PATH.clone());

        // Starting disclosure should not fail.
        let proposal = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect("starting disclosure should succeed");

        // Test that the returned `DisclosureProposalPresentation` contains the processed data we set up earlier.
        assert_matches!(
            proposal,
            DisclosureProposalPresentation {
                reader_registration,
                shared_data_with_relying_party_before,
                session_type: SessionType::CrossDevice,
                disclosure_type: DisclosureType::Regular,
                purpose: RedirectUriPurpose::Browser,
                ..
            } if reader_registration == *verifier_certificate.registration() && !shared_data_with_relying_party_before
        );
        assert_eq!(proposal.attestations.len(), 1);

        let presentation = proposal.attestations.first().unwrap();

        assert_matches!(presentation.identity, AttestationIdentity::Fixed { .. });
        assert_eq!(presentation.attestation_type, PID_DOCTYPE);
        assert_eq!(presentation.attributes.len(), 1);

        let attribute = presentation.attributes.first().unwrap();

        assert_eq!(attribute.key, vec!["age_over_18"]);
        assert_matches!(
            attribute.value,
            AttestationAttributeValue::Basic(AttributeValue::Bool(true))
        );

        // Starting disclosure should not cause mdoc copy usage counts to be incremented.
        wallet.mut_storage().attestation_copies_usage_counts.is_empty();

        // Test that the `Wallet` now contains a `DisclosureSession`.
        let Some(Session::Disclosure(session)) = wallet.session.as_mut() else {
            panic!("wallet should contain disclosure session");
        };
        assert_eq!(session.redirect_uri_purpose, RedirectUriPurpose::Browser);

        // Starting disclosure should not have caused events to be recorded yet.
        assert!(events.lock().last().unwrap().is_empty());

        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .withf(|mdocs| mdocs.len().get() == 1 && mdocs.first().mso.doc_type == PID_DOCTYPE)
            .return_once(|_mdocs| Ok(Some(RETURN_URL.clone())));

        let return_url = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect("accepting disclosure should succeed");

        assert_eq!(return_url.as_ref(), Some(RETURN_URL.as_ref()));

        // Check that the disclosure session is no longer present on the wallet.
        assert!(wallet.session.is_none());

        // Verify that a single disclosure success event is logged that contains the revelant information.
        let recent_events = events.lock();
        let event = recent_events
            .last()
            .unwrap()
            .iter()
            .exactly_one()
            .expect("disclosure should have resulted in a single event");

        assert_matches!(
            event,
            WalletEvent::Disclosure {
                attestations,
                reader_certificate,
                reader_registration,
                status: DisclosureStatus::Success,
                ..
            } if attestations.len() == 1 &&
                attestations.first().unwrap().attestation_type == PID_DOCTYPE &&
                reader_certificate.as_ref() == verifier_certificate.certificate() &&
                reader_registration.as_ref() == verifier_certificate.registration()
        );

        // Test that the attestation usage count got incremented in the database.
        let usage_count = wallet
            .mut_storage()
            .attestation_copies_usage_counts
            .values()
            .copied()
            .exactly_one()
            .expect("the database should contain a single usage count");
        assert_eq!(usage_count, 1);
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_blocked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.update_policy_repository.state = VersionState::Block;

        // Starting disclosure on a blocked wallet should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::VersionBlocked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Starting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Starting disclosure on a locked wallet should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::Locked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_session_state() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Start an active disclosure session.
        wallet.session = Some(Session::Disclosure(WalletDisclosureSession::new_missing_attributes(
            RedirectUriPurpose::Browser,
            DisclosureType::Regular,
            MockDisclosureSession::new(),
        )));

        // Starting disclosure on a wallet with an active disclosure should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_disclosure_uri() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Starting disclosure on a wallet with an unknown disclosure URI should result in an error.
        let disclosure_url = Url::parse("http://example.com?invalid").unwrap();
        let error = wallet
            .start_disclosure(&disclosure_url, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(&error, DisclosureError::DisclosureUri(url) if url == &disclosure_url);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_disclosure_uri_query() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Starting disclosure on a wallet with a disclosure URI with no query parameters should result in an error.
        let error = wallet
            .start_disclosure(
                &urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).join("Zm9vYmFy"),
                DisclosureUriSource::Link,
            )
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::DisclosureUriQuery(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_vp_client() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up `DisclosureSession` start to return the following error.
        wallet
            .disclosure_client
            .expect_start()
            .times(1)
            .return_once(|_, _, _| Err(VpClientError::RequestUri(serde::de::Error::custom("error")).into()));

        // Starting disclosure which returns an error should forward that error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::VpClient(VpClientError::RequestUri(_)));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_vp_client_return_url() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up an `MdocDisclosureSession` start to return the following error.
        let start_return_url = RETURN_URL.clone();
        wallet.disclosure_client.expect_start().times(1).return_once(|_, _, _| {
            Err(VpClientError::Request(
                DisclosureErrorResponse {
                    error_response: ErrorResponse {
                        error: GetRequestErrorCode::ServerError,
                        error_description: None,
                        error_uri: None,
                    },
                    redirect_uri: Some(start_return_url),
                }
                .into(),
            )
            .into())
        });

        // Starting disclosure where the verifier returns responds with a HTTP error body containing
        // a redirect URI should result in that URI being available on the returned error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::VpClient(VpClientError::Request(_)));
        assert_eq!(error.return_url(), Some(RETURN_URL.as_ref()));
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_vp_verifier_server() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up `DisclosureSession` start to return the following error.
        wallet
            .disclosure_client
            .expect_start()
            .times(1)
            .return_once(|_, _, _| Err(VpVerifierError::MissingSessionType.into()));

        // Starting disclosure which returns an error should forward that error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(
            error,
            DisclosureError::VpVerifierServer {
                error: VpVerifierError::MissingSessionType,
                organization: None,
            }
        );
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_attestation_retrieval() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.mut_storage().has_query_error = true;
        let _verifier_certificate =
            setup_disclosure_client_start(&mut wallet.disclosure_client, DEFAULT_REQUESTED_PID_PATH.clone());

        // Starting disclosure on a wallet that has a faulty database should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::AttestationRetrieval(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    // TODO (PVW-1879): Add test for `DisclosureError::HistoryRetrieval`.

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_attributes_not_available() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let verifier_certificate =
            setup_disclosure_client_start(&mut wallet.disclosure_client, DEFAULT_REQUESTED_PID_PATH.clone());

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        let expected_attributes = HashSet::from([format!("{PID_DOCTYPE}/{PID_DOCTYPE}/age_over_18")]);
        assert_matches!(
            error,
            DisclosureError::AttributesNotAvailable {
                reader_registration,
                requested_attributes,
                shared_data_with_relying_party_before,
                session_type: SessionType::CrossDevice,
            } if reader_registration.as_ref() == verifier_certificate.registration() &&
                requested_attributes == expected_attributes &&
                !shared_data_with_relying_party_before
        );
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_attributes_not_available_non_mdoc() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set the requested attribute path to something that will not match the mdoc 2-tuple
        // of namespace and attribute, which should lead to no candidates being available.
        let verifier_certificate = setup_disclosure_client_start(
            &mut wallet.disclosure_client,
            vec!["long".to_string(), "path".to_string(), "age_over_18".to_string()]
                .try_into()
                .unwrap(),
        );

        let mdoc_credential = create_example_pid_mdoc_credential();
        wallet
            .mut_storage()
            .issued_credential_copies
            .insert(mdoc_credential.attestation_type.clone(), vec![mdoc_credential]);

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        let expected_attributes = HashSet::from([format!("{PID_DOCTYPE}/long/path/age_over_18")]);
        assert_matches!(
            error,
            DisclosureError::AttributesNotAvailable {
                reader_registration,
                requested_attributes,
                shared_data_with_relying_party_before,
                session_type: SessionType::CrossDevice,
            } if reader_registration.as_ref() == verifier_certificate.registration() &&
                requested_attributes == expected_attributes &&
                !shared_data_with_relying_party_before
        );
        assert!(wallet.session.is_some());
    }

    // TODO (PVW-4132): Attestations that are stored in the database should be assumed to contain valid certificates and
    //                  metadata. Codifying these assumptions in to the type system should be implemented in PVW-4132.
    //                  This comment should be removed once that is implemented.
    //
    // For the above reason tests are not included for the following error cases:
    // * DisclosureError::MdocCertificate
    // * DisclosureError::IssuerRegistration
    // * DisclosureError::MissingIssuerRegistration
    // * DisclosureError::AttestationAttributes

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_multiple_candidates() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let mdoc_credential = create_example_pid_mdoc_credential();
        wallet.mut_storage().issued_credential_copies.insert(
            mdoc_credential.attestation_type.clone(),
            vec![mdoc_credential.clone(), mdoc_credential],
        );

        let _verifier_certificate =
            setup_disclosure_client_start(&mut wallet.disclosure_client, DEFAULT_REQUESTED_PID_PATH.clone());

        // Starting disclosure on a wallet that contains multiple instances
        // of the same attestation type should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(
            &error,
            DisclosureError::MultipleCandidates(attestation_types)
                if *attestation_types == vec![PID_DOCTYPE.to_string()]
        );
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_cancel_disclosure(#[values(false, true)] has_missing_attributes: bool) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, verifier_certificate) = if has_missing_attributes {
            setup_wallet_disclosure_session_missing_attributes()
        } else {
            setup_wallet_disclosure_session()
        };
        wallet.session = Some(session);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Set up the `terminate()` method to return the following.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        let terminate_return_url = RETURN_URL.clone();
        session
            .protocol_state
            .expect_terminate()
            .times(1)
            .return_once(|| Ok(Some(terminate_return_url)));

        // Cancelling disclosure should result in a `Wallet` without a disclosure session.
        let cancel_return_url = wallet
            .cancel_disclosure()
            .await
            .expect("cancelling disclosure should succeed");

        assert_eq!(cancel_return_url.as_ref(), Some(RETURN_URL.as_ref()));
        assert!(wallet.session.is_none());

        // Verify that a disclosure cancel event has been recorded.
        let recent_events = events.lock();
        let event = recent_events
            .last()
            .unwrap()
            .iter()
            .exactly_one()
            .expect("disclosure should have resulted in a single event");

        assert_matches!(
            event,
            WalletEvent::Disclosure {
                attestations,
                reader_certificate,
                reader_registration,
                status: DisclosureStatus::Cancelled,
                ..
            } if attestations.is_empty() &&
                reader_certificate.as_ref() == verifier_certificate.certificate() &&
                reader_registration.as_ref() == verifier_certificate.registration()
        );
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_blocked() {
        // Prepare a registered and unlocked wallet with an active disclosure session that is blocked.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        wallet.update_policy_repository.state = VersionState::Block;

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Cancelling disclosure on a blocked wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::VersionBlocked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Cancelling disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_locked() {
        // Prepare a registered and locked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        wallet.lock();

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Cancelling disclosure on a locked wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::Locked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Cancelling disclosure on a wallet without an active disclosure session should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_vp_client_return_url() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Cancelling disclosure where the verifier returns responds with a HTTP error body containing
        // a redirect URI should result in that URI being available on the returned error.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        let terminate_return_url = RETURN_URL.clone();
        session.protocol_state.expect_terminate().times(1).return_once(|| {
            Err(VpClientError::Request(
                DisclosureErrorResponse {
                    error_response: ErrorResponse {
                        error: GetRequestErrorCode::ServerError,
                        error_description: None,
                        error_uri: None,
                    },
                    redirect_uri: Some(terminate_return_url),
                }
                .into(),
            )
            .into())
        });

        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::VpClient(VpClientError::Request(_)));
        assert_eq!(error.return_url(), Some(RETURN_URL.as_ref()));
        assert!(wallet.session.is_none());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_event_storage() {
        // Prepare a registered and unlocked wallet with an active disclosure session and a faulty database.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        wallet.mut_storage().has_query_error = true;

        // Cancelling disclosure on a wallet with a faulty database should result
        // in an error, while the disclosure session should be removed.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        session
            .protocol_state
            .expect_terminate()
            .times(1)
            .return_once(|| Ok(None));

        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::EventStorage(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    /// This contains a lightweight test of `accept_disclosure()`.
    /// For a more thorough test see `test_wallet_disclosure()`
    #[tokio::test]
    async fn test_wallet_accept_disclosure_abridged() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Set up the `disclose()` method to return the following.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        let disclose_return_url = RETURN_URL.clone();
        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .return_once(|_mdocs| Ok(Some(disclose_return_url)));

        let accept_return_url = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect("accepting disclosure should succeed");

        // Accepting disclosure should result in a `Wallet` without a disclosure session.
        assert_eq!(accept_return_url.as_ref(), Some(RETURN_URL.as_ref()));
        assert!(wallet.session.is_none());
        assert_eq!(events.lock().pop().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_blocked() {
        // Prepare a registered and unlocked wallet with an active disclosure session that is blocked.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        wallet.update_policy_repository.state = VersionState::Block;

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Accepting disclosure on a blocked wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::VersionBlocked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Accepting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_locked() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        wallet.lock();

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Accepting disclosure on a locked wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::Locked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Accepting disclosure on a wallet without an active disclosure session should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_unexpected_redirect_uri_purpose() {
        // Prepare a registered and unlocked wallet with an active disclosure based issuance session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        session.redirect_uri_purpose = RedirectUriPurpose::Issuance;

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Accepting disclosure on a wallet that has a disclosure based issuance session should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(
            error,
            DisclosureError::UnexpectedRedirectUriPurpose {
                expected: RedirectUriPurpose::Issuance,
                found: RedirectUriPurpose::Browser
            }
        );
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state_missing_attributes() {
        // Prepare a registered and unlocked wallet with an active disclosure session that has missing attributes.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        session.attestations = None;

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Accepting disclosure on a wallet without an active disclosure session should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
        assert!(events.lock().pop().unwrap().is_empty());
    }

    // TODO (PVW-3844): Add tests for continuing a PIN change when accepting disclosure.

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_increment_usage_count() {
        // Prepare a registered and unlocked wallet with an active disclosure session and a faulty database.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, _verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        wallet.mut_storage().has_query_error = true;

        // Accepting disclosure on a wallet with a faulty database should result
        // in an error, the disclosure session should not be removed.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::IncrementUsageCount(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());

        // TODO (PVW-1879): If incrementing the usage count fails, a disclosure error event should be recorded.
        //                  However, we cannot test that here because of the limitations of `MockStorage`.
        //                  Once this mock is based on `mockall`, checking the event should be added to this test.
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ClientErrorType {
        VpClient,
        VpVerifier,
    }

    impl ClientErrorType {
        fn check_error(self, error: &DisclosureError, expected_organization: &Organization) {
            match self {
                ClientErrorType::VpClient => {
                    assert_matches!(error, DisclosureError::VpClient(VpClientError::Request(_)));
                }
                ClientErrorType::VpVerifier => {
                    assert_matches!(
                        error,
                        DisclosureError::VpVerifierServer {
                            organization,
                            error: VpVerifierError::Request(_)
                        } if organization.as_deref() == Some(expected_organization)
                    );
                }
            }
        }
    }

    #[rstest]
    #[case(
        || DisclosureErrorResponse {
            error_response: ErrorResponse {
                error: PostAuthResponseErrorCode::InvalidRequest,
                error_description: None,
                error_uri: None,
            },
            redirect_uri: Some(RETURN_URL.clone()),
        },
        ClientErrorType::VpClient,
        true,
    )]
    #[case(
        || DisclosureErrorResponse {
            error_response: ErrorResponse {
                error: PostAuthResponseErrorCode::InvalidRequest,
                error_description: None,
                error_uri: None,
            },
            redirect_uri: None,
        },
        ClientErrorType::VpClient,
        false,
    )]
    #[case(
        || serde_json::Error::custom("error"),
        ClientErrorType::VpVerifier,
        false,
    )]
    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_vp_client_verifier<F, E>(
        #[case] error_factory: F,
        #[case] expected_error_type: ClientErrorType,
        #[case] expect_return_url: bool,
        #[values(true, false)] data_shared: bool,
    ) where
        F: Fn() -> E,
        E: Into<VpMessageClientError>,
    {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        let copy_ids = session
            .attestations
            .as_ref()
            .unwrap()
            .iter()
            .map(|attestation| attestation.copy_id)
            .collect_vec();

        let disclose_verifier_certificate = verifier_certificate.clone();
        let mut disclosure_error = disclosure_session::DisclosureError::from(error_factory().into());
        // Fudge the `data_shared` property, since we cannot emulate an error that will make it be `false`.
        disclosure_error.data_shared = data_shared;
        session.protocol_state.expect_disclose().times(1).return_once(|_mdocs| {
            Err((
                setup_disclosure_session_verifier_certificate(
                    disclose_verifier_certificate,
                    DEFAULT_REQUESTED_PID_PATH.clone(),
                ),
                disclosure_error,
            ))
        });

        // Accepting disclosure when the verifier responds with an invalid request error should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        // Check the error type and its return URL and check if the wallet still has an active disclosure session.
        expected_error_type.check_error(&error, &verifier_certificate.registration().organization);
        if expect_return_url {
            assert_eq!(error.return_url(), Some(RETURN_URL.as_ref()));
        } else {
            assert!(error.return_url().is_none());
        }
        assert!(wallet.session.is_some());

        // Verify that a disclosure error event has been recorded, with attestations if the data was shared.
        {
            let recent_events = events.lock();
            let event = recent_events
                .last()
                .unwrap()
                .iter()
                .exactly_one()
                .expect("disclosure should have resulted in a single event");

            assert_matches!(
                event,
                WalletEvent::Disclosure {
                    attestations,
                    reader_certificate,
                    reader_registration,
                    status: DisclosureStatus::Error,
                    ..
                } if attestations.len() == if data_shared {1} else {0} &&
                    reader_certificate.as_ref() == verifier_certificate.certificate() &&
                    reader_registration.as_ref() == verifier_certificate.registration()
            );
        }

        // Check that the usage count got incremented for all of the attestation copy ids.
        for copy_id in &copy_ids {
            assert_eq!(
                wallet
                    .mut_storage()
                    .attestation_copies_usage_counts
                    .get(copy_id)
                    .copied()
                    .unwrap_or_default(),
                1
            );
        }

        // Repeating the disclosure with exactly the same error should result in an
        // increment in usage count and exactly the same disclosure error event.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        let disclose_verifier_certificate = verifier_certificate.clone();
        let mut disclosure_error = disclosure_session::DisclosureError::from(error_factory().into());
        disclosure_error.data_shared = data_shared;
        session.protocol_state.expect_disclose().times(1).return_once(|_mdocs| {
            Err((
                setup_disclosure_session_verifier_certificate(
                    disclose_verifier_certificate,
                    DEFAULT_REQUESTED_PID_PATH.clone(),
                ),
                disclosure_error,
            ))
        });

        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        expected_error_type.check_error(&error, &verifier_certificate.registration().organization);
        if expect_return_url {
            assert_eq!(error.return_url(), Some(RETURN_URL.as_ref()));
        } else {
            assert!(error.return_url().is_none());
        }
        assert!(wallet.session.is_some());

        for copy_id in &copy_ids {
            assert_eq!(
                wallet
                    .mut_storage()
                    .attestation_copies_usage_counts
                    .get(copy_id)
                    .copied()
                    .unwrap_or_default(),
                2
            );
        }

        let recent_events = events.lock();
        let (first_event, second_event) = recent_events
            .last()
            .unwrap()
            .iter()
            .collect_tuple()
            .expect("disclosure should have resulted in two events");

        assert_matches!(
            (first_event, second_event),
            (WalletEvent::Disclosure {
                attestations: first_attestations,
                    status: DisclosureStatus::Error,
                ..
            }, WalletEvent::Disclosure {
                attestations: second_attestations,
                reader_certificate,
                    reader_registration,
                    status: DisclosureStatus::Error,
                ..
            }) if first_attestations == second_attestations &&
                reader_certificate.as_ref() == verifier_certificate.certificate() &&
                reader_registration.as_ref() == verifier_certificate.registration()
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum InstructionExpectation {
        Retry,
        RetryWithEvent,
        Termination,
    }

    #[rstest]
    #[case(
        InstructionError::IncorrectPin { attempts_left_in_round: 1, is_final_round: false },
        InstructionExpectation::Retry
    )]
    #[case(InstructionError::InstructionValidation, InstructionExpectation::RetryWithEvent)]
    #[case(InstructionError::Timeout { timeout_millis: 10_000 }, InstructionExpectation::Termination)]
    #[case(InstructionError::Blocked, InstructionExpectation::Termination)]
    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_instruction(
        #[case] instruction_error: InstructionError,
        #[case] instruction_expectation: InstructionExpectation,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);
        let (session, verifier_certificate) = setup_wallet_disclosure_session();
        wallet.session = Some(session);

        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        let copy_ids = session
            .attestations
            .as_ref()
            .unwrap()
            .iter()
            .map(|attestation| attestation.copy_id)
            .collect_vec();

        let disclose_verifier_certificate = verifier_certificate.clone();
        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .return_once(move |_mdocs| {
                let mut session = setup_disclosure_session_verifier_certificate(
                    disclose_verifier_certificate,
                    DEFAULT_REQUESTED_PID_PATH.clone(),
                );

                if instruction_expectation == InstructionExpectation::Termination {
                    session.expect_terminate().times(1).return_once(|| Ok(None));
                }

                Err((
                    session,
                    disclosure_session::DisclosureError::before_sharing(VpSessionError::Client(
                        VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(Box::new(
                            RemoteEcdsaKeyError::Instruction(instruction_error),
                        )))),
                    )),
                ))
            });

        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::Instruction(_));

        if instruction_expectation == InstructionExpectation::Termination {
            // If the disclosure session is expected to be terminated, the wallet should
            // no long have an active disclosure session and should be locked.
            assert!(wallet.session.is_none());
            assert!(wallet.is_locked());
        } else {
            // Otherwise, the session should still be present and the wallet unlocked.
            assert!(wallet.session.is_some());
            assert!(!wallet.is_locked());
        }

        let recent_events = events.lock();
        match instruction_expectation {
            InstructionExpectation::Retry => {
                assert!(recent_events.last().unwrap().is_empty());
            }
            InstructionExpectation::RetryWithEvent => {
                // Verify that a disclosure error event is recorded.
                let event = recent_events
                    .last()
                    .unwrap()
                    .iter()
                    .exactly_one()
                    .expect("disclosure should have resulted in a single event");

                assert_matches!(
                    event,
                    WalletEvent::Disclosure {
                        attestations,
                        reader_certificate,
                        reader_registration,
                        status: DisclosureStatus::Error,
                        ..
                    } if attestations.is_empty() &&
                        reader_certificate.as_ref() == verifier_certificate.certificate() &&
                        reader_registration.as_ref() == verifier_certificate.registration()
                );
            }
            InstructionExpectation::Termination => {
                // Verify that both a disclosure cancellation and error event are recorded.
                let (first_event, second_event) = recent_events
                    .last()
                    .unwrap()
                    .iter()
                    .collect_tuple()
                    .expect("disclosure should have resulted in two events");

                assert_matches!(
                    first_event,
                    WalletEvent::Disclosure {
                        attestations,
                        reader_certificate,
                        reader_registration,
                        status: DisclosureStatus::Cancelled,
                        ..
                    } if attestations.is_empty() &&
                        reader_certificate.as_ref() == verifier_certificate.certificate() &&
                        reader_registration.as_ref() == verifier_certificate.registration()
                );
                assert_matches!(
                    second_event,
                    WalletEvent::Disclosure {
                        attestations,
                        reader_certificate,
                        reader_registration,
                        status: DisclosureStatus::Error,
                        ..
                    } if attestations.is_empty() &&
                        reader_certificate.as_ref() == verifier_certificate.certificate() &&
                        reader_registration.as_ref() == verifier_certificate.registration()
                );
            }
        }

        // Check that the usage count got incremented for all of the attestation copy ids.
        for copy_id in &copy_ids {
            assert_eq!(
                wallet
                    .mut_storage()
                    .attestation_copies_usage_counts
                    .get(copy_id)
                    .copied()
                    .unwrap_or_default(),
                1
            );
        }
    }
}
