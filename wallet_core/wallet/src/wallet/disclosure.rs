use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;

use chrono::Utc;
use futures::future::try_join_all;
use indexmap::IndexMap;
use indexmap::IndexSet;
use itertools::Either;
use itertools::Itertools;
use tracing::error;
use tracing::info;
use tracing::instrument;
use url::Url;

use attestation_data::auth::Organization;
use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure::AttestationRequest;
use attestation_data::disclosure_type::DisclosureType;
use attestation_data::verifier_certificate::VerifierCertificate;
use attestation_types::claim_path::ClaimPath;
use dcql::CredentialFormat;
use dcql::CredentialQueryIdentifier;
use dcql::normalized::NormalizedCredentialRequest;
use entity::disclosure_event::EventStatus;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::client::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use jwt::error::JwtError;
use mdoc::utils::cose::CoseError;
use openid4vc::disclosure_session::DataDisclosed;
use openid4vc::disclosure_session::DisclosableAttestations;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpSessionError;
use openid4vc::disclosure_session::VpVerifierError;
use openid4vc::verifier::SessionType;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureClient;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureError as PlatformCloseProximityDisclosureError;
use sd_jwt::claims::NonSelectivelyDisclosableClaimsError;
use sd_jwt::error::SigningError;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use update_policy_model::update_policy::VersionState;
use utils::generator::TimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecAtLeastN;
use utils::vec_at_least::VecAtLeastTwo;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use wallet_configuration::wallet_config::PidAttributePaths;
use wallet_configuration::wallet_config::PidAttributesConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::AttestationPresentation;
use crate::attestation::AttestationPresentationConfig;
use crate::errors::ChangePinError;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionClientParameters;
use crate::instruction::InstructionError;
use crate::instruction::RemoteEcdsaKeyError;
use crate::instruction::RemoteEcdsaWscd;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::DisclosableAttestation;
use crate::storage::PartialAttestation;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::wallet::HistoryError;
use crate::wallet::Session;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureError;

use super::UriType;
use super::Wallet;
use super::uri::identify_uri;

pub use openid4vc::disclosure_session::DisclosureUriSource;

#[derive(Debug, Clone)]
pub struct DisclosureProposalPresentation {
    pub attestation_options: VecNonEmpty<DisclosureAttestationOptions>,
    pub reader_registration: ReaderRegistration,
    pub shared_data_with_relying_party_before: bool,
    pub session_type: SessionType,
    pub disclosure_type: DisclosureType,
    pub purpose: RedirectUriPurpose,
}

#[derive(Debug, Clone)]
pub enum DisclosureAttestationOptions {
    Single(Box<AttestationPresentation>),
    Multiple(VecAtLeastTwo<AttestationPresentation>),
}

#[derive(Debug, Clone)]
pub struct AttributesNotAvailable {
    pub reader_registration: Box<ReaderRegistration>,
    pub requested_attributes: HashSet<String>,
    pub shared_data_with_relying_party_before: bool,
    pub session_type: SessionType,
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

    #[error("not all requested attributes are available, requested: {:?}", .0.requested_attributes)]
    #[category(pd)] // Might reveal information about what attributes are stored in the Wallet
    AttributesNotAvailable(AttributesNotAvailable),

    #[error("cannot request recovery code")]
    #[category(critical)]
    RecoveryCodeRequested(Box<Organization>),

    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[source] InstructionError),

    #[error("could not increment usage count of mdoc copies in database: {0}")]
    IncrementUsageCount(#[source] StorageError),

    // TODO (PVW-5113): Have this specific error cause a warning screen instead of a generic error screen in Flutter.
    #[error("could not store event in history database: {0}")]
    EventStorage(#[from] HistoryError),

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

    #[error("non-selectively-disclosable claims: {1:?} not requested for requested vct values: {2:?}")]
    #[category(critical)]
    NonSelectivelyDisclosableClaimsNotRequested(Box<Organization>, Vec<VecNonEmpty<ClaimPath>>, Vec<String>),

    #[error("non-selectively-disclosable claim error: {1}")]
    #[category(critical)]
    NonSelectivelyDisclosableClaim(Box<Organization>, #[source] NonSelectivelyDisclosableClaimsError),

    #[error("Platform Close Proximity disclosure session error: {0}")]
    #[category(critical)]
    PlatformCloseProximityDisclosureSessionError(#[from] PlatformCloseProximityDisclosureError),

    #[error("Close Proximity disclosure session error: {0}")]
    #[category(defer)]
    CloseProximityDisclosureSessionError(#[from] CloseProximityDisclosureError),
}

impl DisclosureError {
    fn with_organization(error: VpSessionError, organization: Box<Organization>) -> Self {
        match error {
            VpSessionError::Verifier(error) => Self::VpVerifierServer {
                organization: Some(organization),
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
            ))))
            | VpSessionError::Client(VpClientError::SdJwtSigning(SigningError::Jwt(JwtError::Signing(
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
pub(super) enum WalletDisclosureAttestations<T> {
    Missing,
    Proposal(IndexMap<T, VecNonEmpty<DisclosableAttestation>>),
}

impl<T: Hash + Eq> WalletDisclosureAttestations<T> {
    /// Returns an [`IndexMap`] selecting one attestation per DCQL query from the proposal. Note that this panics when
    /// [`WalletDisclosureAttestations`] is not a propsal or any of the indices is out of bounds, as this is considered
    /// programmer error.
    pub fn select_proposal(&self, selected_indices: &[usize]) -> IndexMap<&T, &DisclosableAttestation> {
        match self {
            Self::Missing => panic!("disclosure proposal selected when missing attributes"),
            Self::Proposal(attestations) => {
                if selected_indices.len() != attestations.len() {
                    panic!(
                        "disclosure attestation count does not match query, expected {}, found {}",
                        attestations.len(),
                        selected_indices.len()
                    );
                }

                attestations
                    .iter()
                    .zip(selected_indices.iter().copied())
                    .enumerate()
                    .map(|(query_index, ((id, candidates), selected_index))| {
                        let Some(attestation) = candidates.as_ref().get(selected_index) else {
                            panic!(
                                "selected disclosure attestation out of bounds for query index {} with count {}: {}",
                                query_index,
                                candidates.len(),
                                selected_index,
                            );
                        };

                        (id, attestation)
                    })
                    .collect()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct WalletDisclosureSession<DCS> {
    pub redirect_uri_purpose: RedirectUriPurpose,
    pub disclosure_type: DisclosureType,
    pub attestations: WalletDisclosureAttestations<CredentialQueryIdentifier>,
    pub protocol_state: DCS,
}

impl<DCS> WalletDisclosureSession<DCS> {
    pub fn new_proposal(
        redirect_uri_purpose: RedirectUriPurpose,
        disclosure_type: DisclosureType,
        attestations: IndexMap<CredentialQueryIdentifier, VecNonEmpty<DisclosableAttestation>>,
        protocol_state: DCS,
    ) -> Self {
        Self {
            redirect_uri_purpose,
            disclosure_type,
            attestations: WalletDisclosureAttestations::Proposal(attestations),
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
            attestations: WalletDisclosureAttestations::Missing,
            protocol_state,
        }
    }
}

impl RedirectUriPurpose {
    fn from_uri(uri: &Url) -> Result<Self, DisclosureError> {
        let purpose = identify_uri(uri)
            .and_then(|uri_type| match uri_type {
                UriType::Disclosure => Some(Self::Browser),
                UriType::DisclosureBasedIssuance => Some(Self::Issuance),
                _ => None,
            })
            .ok_or_else(|| DisclosureError::DisclosureUri(uri.clone()))?;

        Ok(purpose)
    }
}

/// Check if the PID recovery code is part of a credential request.
pub(super) fn is_request_for_recovery_code(
    request: impl AttestationRequest,
    pid_attributes: &PidAttributesConfiguration,
) -> bool {
    let matches_recovery_code_paths = |pid_paths: &PidAttributePaths| {
        request
            .claim_paths()
            .any(|claim| ClaimPath::matches_key_path(&claim, pid_paths.recovery_code.iter().map(String::as_str)))
    };

    match request.format() {
        CredentialFormat::MsoMdoc => pid_attributes
            .mso_mdoc
            .get(request.credential_types().collect::<VecNonEmpty<_>>().first())
            .is_some_and(matches_recovery_code_paths),
        CredentialFormat::SdJwt => request
            .credential_types()
            .any(|vct| pid_attributes.sd_jwt.get(&vct).is_some_and(matches_recovery_code_paths)),
    }
}

impl DisclosureProposalPresentation {
    /// Converts a collection of candidate attestations into a [`DisclosureProposalPresentation`].
    pub(super) fn from_candidates(
        candidate_attestations: VecNonEmpty<VecNonEmpty<DisclosableAttestation>>,
        reader_registration: ReaderRegistration,
        shared_data_with_relying_party_before: bool,
        session_type: SessionType,
        disclosure_type: DisclosureType,
        purpose: RedirectUriPurpose,
    ) -> Self {
        // Place the proposed attestations in a `DisclosureProposalPresentation`,
        // along with a copy of the `ReaderRegistration`.
        let attestation_options = candidate_attestations
            .into_nonempty_iter()
            .map(|candidates| {
                let presentations = candidates
                    .into_nonempty_iter()
                    .map(|candidate| candidate.into_presentation())
                    .collect::<VecNonEmpty<_>>();

                match presentations.len() {
                    NonZeroUsize::MIN => DisclosureAttestationOptions::Single(Box::new(presentations.into_first())),
                    _ => DisclosureAttestationOptions::Multiple(presentations.into_inner().try_into().unwrap()),
                }
            })
            .collect();

        DisclosureProposalPresentation {
            attestation_options,
            reader_registration,
            shared_data_with_relying_party_before,
            session_type,
            disclosure_type,
            purpose,
        }
    }
}

/// Builds a list of requested attribute paths from disclosure requests, formatted as `"attestation_type/claim/path"`.
// TODO (PVW-3813): Attempt to translate the requested attributes using the TAS cache.
pub(super) fn requested_attribute_paths<'a, T: AttestationRequest + 'a>(
    requests: impl Iterator<Item = &'a T>,
) -> impl Iterator<Item = String> {
    requests.flat_map(|request| {
        request.credential_types().into_iter().flat_map(|attestation_type| {
            request
                .claim_paths()
                .map(move |claim_path| format!("{}/{}", attestation_type, claim_path.iter().join("/")))
        })
    })
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
    S: Storage,
{
    /// Checks the common preconditions for disclosure-related operations: version not blocked, wallet registered,
    /// and wallet not locked.
    pub(super) fn check_disclosure_preconditions(&self) -> Result<(), DisclosureError> {
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

        Ok(())
    }

    /// Helper method that fetches attestation from the database based on their attestation type, filters out any of
    /// them that do not match the request and convert the remaining ones to a [`DisclosableAttestation`], which
    /// contains an [`AttestationPresentation`] to show to the user.
    pub(super) async fn fetch_candidate_attestations(
        storage: &S,
        request: &impl AttestationRequest,
        presentation_config: &impl AttestationPresentationConfig,
    ) -> Result<Option<VecNonEmpty<DisclosableAttestation>>, StorageError> {
        let credential_types = request.credential_types().collect();

        let stored_attestations = storage
            .fetch_valid_unique_attestations_by_types_and_format(&credential_types, request.format(), TimeGenerator)
            .await?;

        let candidate_attestations = stored_attestations
            .into_iter()
            .filter_map(|attestation_copy| {
                // Only select those attestations that contain all of the requested attributes.
                // TODO (PVW-4537): Have this be part of the database query using some index.
                attestation_copy
                    .matches_requested_attributes(request.claim_paths().collect_vec().iter())
                    .then(|| {
                        // Create a disclosure proposal by removing any attributes that were not requested from the
                        // presentation attributes. Since the filtering above should remove any attestation in which the
                        // requested claim paths are not present and this is the only error condition, no error should
                        // occur.
                        DisclosableAttestation::try_new(
                            attestation_copy,
                            request.claim_paths().collect_vec().iter(),
                            presentation_config,
                        )
                        .expect("all claim paths should be present in attestation")
                    })
            })
            .collect_vec();

        // Return `None` if the list of candidates is empty.
        Ok(VecNonEmpty::try_from(candidate_attestations).ok())
    }

    pub(super) async fn prepare_disclosure(
        &self,
        attestation_requests: &[&impl AttestationRequest],
        pid_attributes: &PidAttributesConfiguration,
        verifier_certificate: &VerifierCertificate,
    ) -> Result<(Vec<Option<VecNonEmpty<DisclosableAttestation>>>, bool), DisclosureError> {
        // Check for recovery code request
        if attestation_requests
            .iter()
            .any(|request| is_request_for_recovery_code(request, pid_attributes))
        {
            return Err(DisclosureError::RecoveryCodeRequested(
                verifier_certificate.registration().organization.clone(),
            ));
        }

        // For each disclosure request, fetch the candidates from the database and convert
        // each of them to an `AttestationPresentation` that can be shown to the user.
        let storage = self.storage.read().await;
        let candidate_attestations = try_join_all(
            attestation_requests
                .iter()
                .map(|request| Self::fetch_candidate_attestations(&*storage, request, pid_attributes)),
        )
        .await
        .map_err(DisclosureError::AttestationRetrieval)?;

        let shared_data_with_relying_party_before = storage
            .did_share_data_with_relying_party(verifier_certificate.certificate())
            .await
            .map_err(DisclosureError::HistoryRetrieval)?;

        Ok((candidate_attestations, shared_data_with_relying_party_before))
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn start_disclosure(
        &mut self,
        uri: &Url,
        source: DisclosureUriSource,
    ) -> Result<DisclosureProposalPresentation, DisclosureError> {
        info!("Performing disclosure based on received URI: {}", uri);

        self.check_disclosure_preconditions()?;

        info!("Checking if there is already an active session");
        if self.session.is_some() {
            return Err(DisclosureError::SessionState);
        }

        let wallet_config = &self.config_repository.get();

        let purpose = RedirectUriPurpose::from_uri(uri)?;
        let disclosure_uri_query = uri
            .query()
            .ok_or_else(|| DisclosureError::DisclosureUriQuery(uri.clone()))?;

        // Start the disclosure session based on the parsed disclosure URI.
        let session = self
            .disclosure_client
            .start(
                disclosure_uri_query,
                source,
                &wallet_config.disclosure.rp_trust_anchors(),
            )
            .await?;

        let (candidate_attestations, shared_data_with_relying_party_before) = self
            .prepare_disclosure(
                &session.credential_requests().as_ref().iter().collect_vec(),
                &wallet_config.pid_attributes,
                session.verifier_certificate(),
            )
            .await?;

        let candidate_attestations = candidate_attestations
            .into_iter()
            .zip(session.credential_requests().as_ref());

        // Verify whether all non selectively disclosable claims are requested
        let reader_registration = session.verifier_certificate().registration().clone();
        Self::verify_non_selectively_disclosable_claims(
            candidate_attestations.clone(),
            &reader_registration.organization,
        )?;

        let candidate_attestations = candidate_attestations
            .flat_map(|(attestations, request)| attestations.map(|attestations| (request.id().clone(), attestations)))
            .collect::<IndexMap<_, _>>();

        // At this point, determine the disclosure type and if data was ever shared with this RP before, as the UI
        // needs this context both for when all requested attributes are present and for when attributes are missing.
        let disclosure_type = DisclosureType::from_credential_requests(
            session.credential_requests().as_ref(),
            &wallet_config.pid_attributes,
        );

        if let Ok(disclosable_attestations) =
            VecNonEmpty::try_from(candidate_attestations.values().cloned().collect_vec())
            && disclosable_attestations.len().get() == session.credential_requests().as_ref().len()
        {
            info!(
                "All attributes in the disclosure request are present in the database, return a proposal to the user"
            );

            let proposal = DisclosureProposalPresentation::from_candidates(
                disclosable_attestations,
                reader_registration,
                shared_data_with_relying_party_before,
                session.session_type(),
                disclosure_type,
                purpose,
            );

            // Retain the session as `Wallet` state.
            self.session
                .replace(Session::Disclosure(WalletDisclosureSession::new_proposal(
                    purpose,
                    disclosure_type,
                    candidate_attestations,
                    session,
                )));

            return Ok(proposal);
        }

        // If no suitable candidates were found for at least one of the requests, report this as an error to the UI.
        info!("At least one attribute from one attestation is missing in order to satisfy the disclosure request");

        let requested_attributes = requested_attribute_paths(session.credential_requests().as_ref().iter()).collect();
        let session_type = session.session_type();

        // Store the session so that it will only be terminated on user interaction.
        // This prevents gleaning of missing attributes by a verifier.
        self.session
            .replace(Session::Disclosure(WalletDisclosureSession::new_missing_attributes(
                purpose,
                disclosure_type,
                session,
            )));

        Err(DisclosureError::AttributesNotAvailable(AttributesNotAvailable {
            reader_registration: Box::new(reader_registration),
            requested_attributes,
            shared_data_with_relying_party_before,
            session_type,
        }))
    }

    fn verify_non_selectively_disclosable_claims<'a>(
        candidate_attestations: impl IntoIterator<
            Item = (
                Option<VecNonEmpty<DisclosableAttestation>>,
                &'a NormalizedCredentialRequest,
            ),
        >,
        organization: &Organization,
    ) -> Result<(), DisclosureError> {
        for (attestations, request) in candidate_attestations {
            for attestation in attestations.map(VecAtLeastN::into_inner).unwrap_or(Vec::new()) {
                if let PartialAttestation::SdJwt { sd_jwt, .. } = attestation.into_partial_attestation() {
                    Self::verify_sd_jwt_non_selectively_disclosable_claims(&sd_jwt, request, organization)?;
                }
            }
        }
        Ok(())
    }

    fn verify_sd_jwt_non_selectively_disclosable_claims(
        sd_jwt: &UnsignedSdJwtPresentation,
        request: &NormalizedCredentialRequest,
        organization: &Organization,
    ) -> Result<(), DisclosureError> {
        let non_selectively_disclosable_claims = sd_jwt
            .as_ref()
            .non_selectively_disclosable_claims()
            .map_err(|e| DisclosureError::NonSelectivelyDisclosableClaim(Box::new(organization.clone()), e))?;
        let requested_claims = request.claim_paths().cloned().collect::<IndexSet<_>>();
        let mut non_requested_claims = non_selectively_disclosable_claims
            .difference(&requested_claims)
            .peekable();
        if non_requested_claims.peek().is_some() {
            Err(DisclosureError::NonSelectivelyDisclosableClaimsNotRequested(
                Box::new(organization.clone()),
                non_requested_claims.cloned().collect(),
                request.credential_types().map(ToString::to_string).collect(),
            ))
        } else {
            Ok(())
        }
    }

    async fn terminate_disclosure_session(
        &mut self,
        session: WalletDisclosureSession<DCC::Session>,
    ) -> Result<Option<Url>, DisclosureError> {
        let (reader_certificate, _) = session
            .protocol_state
            .verifier_certificate()
            .clone()
            .into_certificate_and_registration();

        let return_url = session.protocol_state.terminate().await?.map(BaseUrl::into_inner);

        self.store_disclosure_event(
            Utc::now(),
            // TODO (PVW-5078): Store credential requests in disclosure event.
            None,
            reader_certificate,
            session.disclosure_type,
            EventStatus::Cancelled,
            DataDisclosed::NotDisclosed,
        )
        .await?;

        Ok(return_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_disclosure(&mut self) -> Result<Option<Url>, DisclosureError>
    where
        CPC: CloseProximityDisclosureClient,
    {
        info!("Cancelling disclosure");

        self.check_disclosure_preconditions()?;

        info!("Checking if a disclosure session is present");

        match self.session.take() {
            Some(Session::Disclosure(session)) => self.terminate_disclosure_session(session).await,
            Some(Session::CloseProximityDisclosure(session)) => {
                self.terminate_close_proximity_disclosure_session(session).await?;
                Ok(None)
            }
            other => {
                self.session = other;
                Err(DisclosureError::SessionState)
            }
        }
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_disclosure(
        &mut self,
        selected_indices: &[usize],
        pin: String,
    ) -> Result<Option<Url>, DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
    {
        self.perform_disclosure(
            selected_indices,
            pin,
            RedirectUriPurpose::Browser,
            self.config_repository.get().as_ref(),
        )
        .await
    }

    #[instrument(skip_all)]
    pub(super) async fn perform_disclosure(
        &mut self,
        selected_indices: &[usize],
        pin: String,
        redirect_uri_purpose: RedirectUriPurpose,
        config: &WalletConfiguration,
    ) -> Result<Option<Url>, DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
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
        let attested_key = Arc::clone(attested_key);

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DisclosureError::Locked);
        }

        info!("Checking if a disclosure session is present");
        let Some(Session::Disclosure(session)) = &self.session else {
            return Err(DisclosureError::SessionState);
        };

        // If we do not have a proposal, this method should not have been called, so return an error.
        if !matches!(session.attestations, WalletDisclosureAttestations::Proposal(_)) {
            return Err(DisclosureError::SessionState);
        }

        if session.redirect_uri_purpose != redirect_uri_purpose {
            return Err(DisclosureError::UnexpectedRedirectUriPurpose {
                expected: session.redirect_uri_purpose,
                found: redirect_uri_purpose,
            });
        }

        // Prepare the `RemoteEcdsaWscd` for signing using the provided PIN.
        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let remote_instruction = self
            .new_instruction_client(
                pin,
                attested_key,
                InstructionClientParameters::new(
                    registration_data.wallet_id.clone(),
                    registration_data.pin_salt.clone(),
                    registration_data.wallet_certificate.clone(),
                    config.account_server.http_config.clone(),
                    instruction_result_public_key,
                ),
            )
            .await?;

        // We have to take ownership of the disclosure session here, so that `attestations`
        // below doesn't borrow from self, as we also borrow mutably from self above.
        let Some(Session::Disclosure(mut session)) = self.session.take() else {
            unreachable!(); // This not possible, as we took a reference to this value before.
        };

        // Note that this will panic if any of the indices are out of bounds.
        let attestations = session.attestations.select_proposal(selected_indices);

        let remote_wscd = RemoteEcdsaWscd::new(remote_instruction);

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
                attestations
                    .values()
                    .map(|attestation| attestation.attestation_copy_id())
                    .unique()
                    .collect(),
            )
            .await;

        let (reader_certificate, reader_registration) = session
            .protocol_state
            .verifier_certificate()
            .clone()
            .into_certificate_and_registration();

        // Generate `AttestationPresentation`s of the disclosed attributes, to store in the disclosure event. Note
        // that there is guaranteed to be at least one attestation because of the logic in `start_disclosure()`.
        let attestation_presentations = attestations
            .values()
            .map(|attestation| attestation.presentation().clone())
            .collect_vec()
            .try_into()
            .unwrap();

        if let Err(error) = result {
            // If storing the event results in an error, log it but do nothing else.
            let _ = self
                .store_disclosure_event(
                    Utc::now(),
                    Some(attestation_presentations),
                    reader_certificate,
                    session.disclosure_type,
                    EventStatus::Error,
                    DataDisclosed::NotDisclosed,
                )
                .await
                .inspect_err(|e| {
                    error!("Could not store error in history: {e}");
                });

            // Put back the session for a later attempt
            self.session.replace(Session::Disclosure(session));
            return Err(DisclosureError::IncrementUsageCount(error));
        }

        // Gather both partial mdocs or SD-JWT presentations by cloning the attestations
        // held in the session, as disclosing attestations needs to be retryable.
        let (partial_mdocs, sd_jwt_presentations): (HashMap<_, _>, HashMap<_, _>) =
            attestations
                .iter()
                .partition_map(|(id, attestation)| match attestation.partial_attestation() {
                    PartialAttestation::MsoMdoc { partial_mdoc } => {
                        Either::Left(((*id).clone(), vec_nonempty![partial_mdoc.as_ref().clone()]))
                    }
                    PartialAttestation::SdJwt { key_identifier, sd_jwt } => {
                        Either::Right(((*id).clone(), vec_nonempty![(*sd_jwt.clone(), key_identifier.clone())]))
                    }
                });

        // This should result in either all partial mdocs or all SD-JWT presentations, which is guaranteed by the logic
        // in `VpDisclosureSession`, which rejects DCQL requests with a mix of formats. Additionally, there will be at
        // least one partial mdoc or SD-JWT presentation, which is guaranteed by `NormalizedCredentialRequests` and the
        // logic in `start_disclosure()`.
        let partial_mdocs_result = DisclosableAttestations::MsoMdoc(partial_mdocs).try_into();
        let sd_jwt_presentations_result = DisclosableAttestations::SdJwt(sd_jwt_presentations).try_into();
        let disclosable_attestations = match (partial_mdocs_result, sd_jwt_presentations_result) {
            (Ok(partial_mdocs), Err(_)) => partial_mdocs,
            (Err(_), Ok(sd_jwt_presentations)) => sd_jwt_presentations,
            (_, _) => panic!("VpDisclosureClient should not allow requesting a mix of formats"),
        };

        // Actually perform disclosure, casting any `InstructionError` that occurs during signing
        // to `RemoteEcdsaKeyError::Instruction`.
        let result = session
            .protocol_state
            .disclose(disclosable_attestations, &remote_wscd, &TimeGenerator)
            .await;
        let return_url = match result {
            Ok(return_url) => return_url.map(BaseUrl::into_inner),
            Err((protocol_state, error)) => {
                let disclosure_error =
                    DisclosureError::with_organization(error.error, reader_registration.organization);

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
                        session.disclosure_type,
                        EventStatus::Error,
                        error.data_shared,
                    )
                    .await
                {
                    error!("Could not store error in history: {error}");
                }

                // At this point place the `DisclosureSession` back so that `WalletDisclosureSession` is whole again.
                session.protocol_state = *protocol_state;

                match disclosure_error {
                    DisclosureError::Instruction(InstructionError::Timeout { .. } | InstructionError::Blocked) => {
                        // On a PIN timeout we should proactively terminate the disclosure session
                        // and lock the wallet, as the user is probably not the owner of the wallet.
                        // The UI should catch this specific error and close the disclosure screens.
                        //
                        // If terminating the session results in an error, log it but do nothing else.
                        let _ = self
                            .terminate_disclosure_session(session)
                            .await
                            .inspect_err(|terminate_error| {
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
                        // If we did not just give away ownership of the disclosure session by terminating it,
                        // place it back in the wallet state so that the user may retry disclosure.
                        self.session.replace(Session::Disclosure(session));
                    }
                }

                return Err(disclosure_error);
            }
        };

        // Disclosure is now successful. Any errors that occur after this point will result in the `Wallet` not having
        // an active disclosure session anymore. Note that these unwraps are safe, as session.attestations was checked
        // to be present above and the source of the iterator is also `VecNonEmpty`.
        self.store_disclosure_event(
            Utc::now(),
            Some(attestation_presentations),
            reader_certificate,
            session.disclosure_type,
            EventStatus::Success,
            DataDisclosed::Disclosed,
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
    use std::sync::Arc;
    use std::sync::LazyLock;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use mockall::predicate::always;
    use mockall::predicate::eq;
    use mockall::predicate::function;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serde::de::Error;
    use url::Url;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use attestation_data::auth::Organization;
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::disclosure_type::DisclosureType;
    use attestation_data::verifier_certificate::VerifierCertificate;
    use attestation_types::claim_path::ClaimPath;
    use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_ADDRESS_GROUP;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_FAMILY_NAME;
    use attestation_types::pid_constants::PID_GIVEN_NAME;
    use attestation_types::pid_constants::PID_RECOVERY_CODE;
    use attestation_types::pid_constants::PID_RESIDENT_HOUSE_NUMBER;
    use attestation_types::pid_constants::PID_RESIDENT_POSTAL_CODE;
    use dcql::CredentialFormat;
    use dcql::normalized::MdocAttributeRequest;
    use dcql::normalized::NormalizedCredentialRequest;
    use dcql::normalized::NormalizedCredentialRequests;
    use dcql::normalized::SdJwtAttributeRequest;
    use entity::disclosure_event::EventStatus;
    use http_utils::urls;
    use http_utils::urls::BaseUrl;
    use mdoc::iso::mdocs::Entry;
    use mdoc::utils::cose::CoseError;
    use openid4vc::PostAuthResponseErrorCode;
    use openid4vc::disclosure_session;
    use openid4vc::disclosure_session::DataDisclosed;
    use openid4vc::disclosure_session::DisclosableAttestations;
    use openid4vc::disclosure_session::DisclosureUriSource;
    use openid4vc::disclosure_session::VpClientError;
    use openid4vc::disclosure_session::VpMessageClientError;
    use openid4vc::disclosure_session::VpSessionError;
    use openid4vc::disclosure_session::VpVerifierError;
    use openid4vc::disclosure_session::mock::MockDisclosureClient;
    use openid4vc::disclosure_session::mock::MockDisclosureSession;
    use openid4vc::errors::DisclosureErrorResponse;
    use openid4vc::errors::ErrorResponse;
    use openid4vc::errors::GetRequestErrorCode;
    use openid4vc::verifier::SessionType;
    use openid4vc::wallet_issuance::mock::MockIssuanceDiscovery;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use update_policy_model::update_policy::VersionState;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;
    use wallet_account::messages::errors::AccountRevokedData;
    use wallet_account::messages::errors::RevocationReason;
    use wallet_configuration::wallet_config::PidAttributePaths;
    use wallet_configuration::wallet_config::PidAttributesConfiguration;

    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationIdentity;
    use crate::attestation::mock::EmptyPresentationConfig;
    use crate::config::UNIVERSAL_LINK_BASE_URL;
    use crate::errors::InstructionError;
    use crate::errors::RemoteEcdsaKeyError;
    use crate::errors::StorageError;
    use crate::storage::ChangePinData;
    use crate::storage::DisclosableAttestation;
    use crate::wallet::test::example_pid_stored_attestation_copy;
    use crate::wallet::test::example_stored_attestation_copy;
    use crate::wallet::test::mock_verifier_certificate;

    use super::super::Session;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::setup_mock_recent_history_callback;
    use super::AttributesNotAvailable;
    use super::DisclosureAttestationOptions;
    use super::DisclosureError;
    use super::DisclosureProposalPresentation;
    use super::RedirectUriPurpose;
    use super::WalletDisclosureAttestations;
    use super::WalletDisclosureSession;
    use super::is_request_for_recovery_code;

    static DISCLOSURE_URI: LazyLock<Url> =
        LazyLock::new(|| urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).join("Zm9vYmFy?foo=bar"));
    const PIN: &str = "051097";
    static RETURN_URL: LazyLock<BaseUrl> =
        LazyLock::new(|| BaseUrl::from_str("https://example.com/return/here").unwrap());
    static DEFAULT_MDOC_PID_CREDENTIAL_REQUESTS: LazyLock<NormalizedCredentialRequests> = LazyLock::new(|| {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(
            &[(PID_ATTESTATION_TYPE, &[&[PID_ATTESTATION_TYPE, PID_FAMILY_NAME]])],
            None,
        )
    });
    static DEFAULT_SD_JWT_PID_CREDENTIAL_REQUESTS: LazyLock<NormalizedCredentialRequests> = LazyLock::new(|| {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(&[PID_ATTESTATION_TYPE], &[&[PID_FAMILY_NAME]])])
    });

    fn default_pid_credential_requests(requested_format: CredentialFormat) -> NormalizedCredentialRequests {
        match requested_format {
            CredentialFormat::MsoMdoc => DEFAULT_MDOC_PID_CREDENTIAL_REQUESTS.clone(),
            CredentialFormat::SdJwt => DEFAULT_SD_JWT_PID_CREDENTIAL_REQUESTS.clone(),
        }
    }

    // Set up properties for a `MockDisclosureSession`.
    fn setup_disclosure_session_verifier_certificate(
        verifier_certificate: VerifierCertificate,
        credential_requests: NormalizedCredentialRequests,
    ) -> MockDisclosureSession {
        let mut disclosure_session = MockDisclosureSession::new();
        disclosure_session
            .expect_session_type()
            .return_const(SessionType::CrossDevice);
        disclosure_session
            .expect_verifier_certificate()
            .return_const(verifier_certificate);
        disclosure_session
            .expect_credential_requests()
            .return_const(credential_requests);

        disclosure_session
    }

    // Set up properties for a `MockDisclosureSession`.
    fn setup_disclosure_session(
        credential_requests: NormalizedCredentialRequests,
    ) -> (MockDisclosureSession, VerifierCertificate) {
        let verifier_certificate = mock_verifier_certificate();

        let disclosure_session =
            setup_disclosure_session_verifier_certificate(verifier_certificate.clone(), credential_requests);

        (disclosure_session, verifier_certificate)
    }

    /// Set up the expected response of `MockDisclosureClient` when starting a new `MockDisclosureSession`.
    fn setup_disclosure_client_start(
        disclosure_client: &mut MockDisclosureClient,
        credential_requests: NormalizedCredentialRequests,
    ) -> VerifierCertificate {
        let (disclosure_session, verifier_certificate) = setup_disclosure_session(credential_requests);

        disclosure_client
            .expect_start()
            .times(1)
            .with(eq("foo=bar"), eq(DisclosureUriSource::QrCode), always())
            .return_once(|_request_uri_query, _uri_source, _trust_anchors| Ok(disclosure_session));

        verifier_certificate
    }

    fn setup_wallet_disclosure_session_missing_attributes(
        requested_format: CredentialFormat,
    ) -> (
        Session<MockIssuanceDiscovery, MockDisclosureSession>,
        VerifierCertificate,
    ) {
        let (disclosure_session, verifier_certificate) =
            setup_disclosure_session(default_pid_credential_requests(requested_format));

        let session = Session::Disclosure(WalletDisclosureSession::new_missing_attributes(
            RedirectUriPurpose::Browser,
            DisclosureType::Regular,
            disclosure_session,
        ));

        (session, verifier_certificate)
    }

    fn setup_wallet_disclosure_session(
        requested_format: CredentialFormat,
    ) -> (
        Session<MockIssuanceDiscovery, MockDisclosureSession>,
        VerifierCertificate,
    ) {
        let credential_requests = default_pid_credential_requests(requested_format);

        // Remove any of the attributes not requested from the attestation.
        let stored_attestation = example_pid_stored_attestation_copy(requested_format);
        let disclosable_attestation = DisclosableAttestation::try_new(
            stored_attestation,
            credential_requests.as_ref().first().unwrap().claim_paths(),
            &EmptyPresentationConfig,
        )
        .unwrap();

        let (disclosure_session, verifier_certificate) = setup_disclosure_session(credential_requests);

        // Store that attestation and its `AttestationPresentation` in the session.
        let session = Session::Disclosure(WalletDisclosureSession::new_proposal(
            RedirectUriPurpose::Browser,
            DisclosureType::Regular,
            IndexMap::from([("id".try_into().unwrap(), vec_nonempty![disclosable_attestation])]),
            disclosure_session,
        ));

        (session, verifier_certificate)
    }

    async fn monitor_event_count(wallet: &mut TestWalletMockStorage) -> Arc<AtomicUsize> {
        wallet
            .mut_storage()
            .expect_fetch_recent_wallet_events()
            .returning(move || Ok(vec![]));

        let event_count = Arc::new(AtomicUsize::new(0));
        let callback_event_count = Arc::clone(&event_count);
        wallet
            .set_recent_history_callback(Box::new(move |_| {
                callback_event_count.fetch_add(1, Ordering::Relaxed);
            }))
            .await
            .unwrap();

        assert_eq!(event_count.load(Ordering::Relaxed), 1);

        event_count
    }

    /// This tests the full happy path for disclosure, calling both
    /// `Wallet::start_disclosure()` and `Wallet::accept_disclosure()`.
    #[rstest]
    #[tokio::test]
    async fn test_wallet_disclosure(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        // Populate a registered wallet with an example PID.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set up the relevant mocks.
        let credential_requests = match requested_format {
            CredentialFormat::MsoMdoc => NormalizedCredentialRequests::new_mock_mdoc_from_slices(
                &[
                    (PID_ATTESTATION_TYPE, &[&[PID_ATTESTATION_TYPE, PID_GIVEN_NAME]]),
                    (
                        ADDRESS_ATTESTATION_TYPE,
                        &[
                            &[
                                &format!("{ADDRESS_ATTESTATION_TYPE}.{PID_ADDRESS_GROUP}"),
                                PID_RESIDENT_POSTAL_CODE,
                            ],
                            &[
                                &format!("{ADDRESS_ATTESTATION_TYPE}.{PID_ADDRESS_GROUP}"),
                                PID_RESIDENT_HOUSE_NUMBER,
                            ],
                        ],
                    ),
                ],
                None,
            ),
            CredentialFormat::SdJwt => NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[
                (&[PID_ATTESTATION_TYPE], &[&[PID_GIVEN_NAME]]),
                (
                    &[ADDRESS_ATTESTATION_TYPE],
                    &[
                        &[PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
                        &[PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
                    ],
                ),
            ]),
        };

        let verifier_certificate = setup_disclosure_client_start(&mut wallet.disclosure_client, credential_requests);

        // Create three PID attestations.
        let mut pid_credential_payload = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());
        let mut attributes_root = pid_credential_payload.previewable_payload.attributes.into_inner();
        *attributes_root.get_mut(PID_GIVEN_NAME).unwrap() =
            Attribute::Single(AttributeValue::Text("Andere Naam".to_string()));
        pid_credential_payload.previewable_payload.attributes = attributes_root.into();
        let pid1 = example_stored_attestation_copy(
            requested_format,
            pid_credential_payload.clone(),
            NormalizedTypeMetadata::nl_pid_example(),
        );

        let pid2 = example_pid_stored_attestation_copy(requested_format);

        let mut attributes_root = pid_credential_payload.previewable_payload.attributes.into_inner();
        *attributes_root.get_mut(PID_GIVEN_NAME).unwrap() =
            Attribute::Single(AttributeValue::Text("Iemand Anders".to_string()));
        pid_credential_payload.previewable_payload.attributes = attributes_root.into();
        let pid3 = example_stored_attestation_copy(
            requested_format,
            pid_credential_payload,
            NormalizedTypeMetadata::nl_pid_example(),
        );

        // Create two address attestations.
        let mut address_credential_payload = CredentialPayload::nl_pid_address_example(&MockTimeGenerator::default());
        let address1 = example_stored_attestation_copy(
            requested_format,
            address_credential_payload.clone(),
            NormalizedTypeMetadata::nl_address_example(),
        );

        let mut attributes_root = address_credential_payload.previewable_payload.attributes.into_inner();
        let Attribute::Nested(address_group) = attributes_root.get_mut(PID_ADDRESS_GROUP).unwrap() else {
            panic!("");
        };
        *address_group.get_mut(PID_RESIDENT_HOUSE_NUMBER).unwrap() =
            Attribute::Single(AttributeValue::Text("68".to_string()));
        *address_group.get_mut(PID_RESIDENT_POSTAL_CODE).unwrap() =
            Attribute::Single(AttributeValue::Text("2514 GL".to_string()));
        address_credential_payload.previewable_payload.attributes = attributes_root.into();
        let address2 = example_stored_attestation_copy(
            requested_format,
            address_credential_payload,
            NormalizedTypeMetadata::nl_address_example(),
        );

        // The wallet will query the database for both attestation types, mock returning them.
        for (attestation_type, attestations) in [
            (PID_ATTESTATION_TYPE, vec![pid1, pid2.clone(), pid3]),
            (ADDRESS_ATTESTATION_TYPE, vec![address1.clone(), address2]),
        ] {
            wallet
                .mut_storage()
                .expect_fetch_valid_unique_attestations_by_types_and_format()
                .withf(move |attestation_types, format, _| {
                    *attestation_types == HashSet::from([attestation_type.to_owned()]) && *format == requested_format
                })
                .times(1)
                .return_once(move |_, _, _| Ok(attestations));
        }

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
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect("starting disclosure should succeed");

        wallet.mut_storage().checkpoint();

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
        assert_eq!(proposal.attestation_options.len().get(), 2);

        let DisclosureAttestationOptions::Multiple(pid_presentations) = &proposal.attestation_options[0] else {
            panic!("multiple proposal attestations expected");
        };

        for (presentation, expected_name) in
            pid_presentations
                .iter()
                .zip_eq(["Andere Naam", "Willeke Liselotte", "Iemand Anders"])
        {
            assert_matches!(presentation.identity, AttestationIdentity::Fixed { .. });
            assert_eq!(presentation.attestation_type, PID_ATTESTATION_TYPE);
            assert_eq!(presentation.attributes.len(), 1);

            let attribute = &presentation.attributes[0];

            assert!(attribute.key.iter().eq([PID_GIVEN_NAME]));
            assert_matches!(
                &attribute.value,
                AttestationAttributeValue::Basic(AttributeValue::Text(given_name)) if given_name == expected_name
            );
        }

        let DisclosureAttestationOptions::Multiple(address_presentations) = &proposal.attestation_options[1] else {
            panic!("multiple proposal attestations expected");
        };

        for (presentation, (expected_house_number, expected_postal_code)) in address_presentations
            .iter()
            .zip_eq([("147", "2511 DP"), ("68", "2514 GL")])
        {
            assert_matches!(presentation.identity, AttestationIdentity::Fixed { .. });
            assert_eq!(presentation.attestation_type, ADDRESS_ATTESTATION_TYPE);
            assert_eq!(presentation.attributes.len(), 2);

            let attribute = &presentation.attributes[0];

            assert!(attribute.key.iter().eq([PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER]));
            assert_matches!(
                &attribute.value,
                AttestationAttributeValue::Basic(AttributeValue::Text(house_number)) if house_number == expected_house_number
            );

            let attribute = &presentation.attributes[1];

            assert!(attribute.key.iter().eq([PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE]));
            assert_matches!(
                &attribute.value,
                AttestationAttributeValue::Basic(AttributeValue::Text(postal_code)) if postal_code == expected_postal_code
            );
        }

        // Test that the `Wallet` now contains a `DisclosureSession`.
        let Some(Session::Disclosure(session)) = wallet.session.as_ref() else {
            panic!("wallet should contain disclosure session");
        };
        assert_eq!(session.redirect_uri_purpose, RedirectUriPurpose::Browser);

        // The wallet will check in the database if there is a PIN change in progress.
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .times(1)
            .returning(|| Ok(None));

        // The wallet will increment the attestation usage count in the database on disclosure.
        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .times(1)
            .returning(|_| Ok(()));

        // The wallet will use the OpenID4VP disclosure client to disclose the actual attributes.
        let Some(Session::Disclosure(session)) = wallet.session.as_mut() else {
            panic!("wallet should contain disclosure session");
        };
        session
            .protocol_state
            .expect_disclose()
            .withf(move |disclosable_attestations| {
                // Make sure that the correct set of attributes is disclosed.
                match (requested_format, disclosable_attestations.as_ref()) {
                    (CredentialFormat::MsoMdoc, DisclosableAttestations::MsoMdoc(partial_mdocs)) => {
                        let attributes = partial_mdocs
                            .iter()
                            .map(|(id, partial_mdocs)| {
                                let attributes = partial_mdocs
                                    .iter()
                                    .map(|partial_mdoc| {
                                        partial_mdoc.issuer_signed().clone().into_entries_by_namespace()
                                    })
                                    .collect_vec();

                                (id.as_ref(), attributes)
                            })
                            .collect::<HashMap<_, _>>();

                        let expected_attributes = HashMap::from([
                            (
                                "mdoc_0",
                                vec![IndexMap::from([(
                                    PID_ATTESTATION_TYPE.to_string(),
                                    vec![Entry {
                                        name: PID_GIVEN_NAME.to_string(),
                                        value: ciborium::Value::Text("Willeke Liselotte".to_string()),
                                    }],
                                )])],
                            ),
                            (
                                "mdoc_1",
                                vec![IndexMap::from([(
                                    format!("{ADDRESS_ATTESTATION_TYPE}.{PID_ADDRESS_GROUP}"),
                                    vec![
                                        Entry {
                                            name: PID_RESIDENT_HOUSE_NUMBER.to_string(),
                                            value: ciborium::Value::Text("147".to_string()),
                                        },
                                        Entry {
                                            name: PID_RESIDENT_POSTAL_CODE.to_string(),
                                            value: ciborium::Value::Text("2511 DP".to_string()),
                                        },
                                    ],
                                )])],
                            ),
                        ]);

                        attributes == expected_attributes
                    }
                    (CredentialFormat::SdJwt, DisclosableAttestations::SdJwt(sd_jwt_presentations)) => {
                        let attributes = sd_jwt_presentations
                            .iter()
                            .map(|(id, presentations)| {
                                let attributes = presentations
                                    .iter()
                                    .map(|(presentation, _)| {
                                        let attributes =
                                            Attributes::try_from(presentation.as_ref().decoded_claims().unwrap())
                                                .unwrap();

                                        attributes
                                            .flattened()
                                            .into_iter()
                                            .map(|(path, value)| {
                                                (path.into_iter().map(str::to_string).collect_vec(), value.to_string())
                                            })
                                            .collect::<HashMap<_, _>>()
                                    })
                                    .collect_vec();

                                (id.as_ref(), attributes)
                            })
                            .collect::<HashMap<_, _>>();

                        let expected_attributes = HashMap::from([
                            (
                                "sd_jwt_0",
                                vec![HashMap::from([(
                                    vec![PID_GIVEN_NAME.to_string()],
                                    "Willeke Liselotte".to_string(),
                                )])],
                            ),
                            (
                                "sd_jwt_1",
                                vec![HashMap::from([
                                    (
                                        vec![PID_ADDRESS_GROUP.to_string(), PID_RESIDENT_HOUSE_NUMBER.to_string()],
                                        "147".to_string(),
                                    ),
                                    (
                                        vec![PID_ADDRESS_GROUP.to_string(), PID_RESIDENT_POSTAL_CODE.to_string()],
                                        "2511 DP".to_string(),
                                    ),
                                ])],
                            ),
                        ]);

                        attributes == expected_attributes
                    }
                    _ => false,
                }
            })
            .times(1)
            .returning(|_disclosable_attestations| Ok(Some(RETURN_URL.clone())));

        // The wallet will log a single disclosure event, containing
        // `AttestationPresentation` values for those attributes disclosed.
        let reader_certificate = verifier_certificate.certificate().clone();
        let mut expected_pid_presentation = pid2.into_attestation_presentation(&EmptyPresentationConfig);
        expected_pid_presentation
            .attributes
            .retain(|attribute| attribute.key.iter().eq([PID_GIVEN_NAME]));
        let mut expected_address_presentation = address1.into_attestation_presentation(&EmptyPresentationConfig);
        expected_address_presentation.attributes.retain(|attribute| {
            attribute.key.iter().eq([PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER])
                || attribute.key.iter().eq([PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE])
        });
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![expected_pid_presentation, expected_address_presentation]),
                eq(reader_certificate),
                eq(EventStatus::Success),
                eq(DisclosureType::Regular),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let event_count = monitor_event_count(&mut wallet).await;

        // Accept the disclosure, selecting the contents of `pid2` and `address1`.
        let return_url = wallet
            .accept_disclosure(&[1, 0], PIN.to_string())
            .await
            .expect("accepting disclosure should succeed");

        assert_eq!(return_url.as_ref(), Some(RETURN_URL.as_ref()));

        // Check that the disclosure session is no longer present on the wallet.
        assert!(wallet.session.is_none());

        // Check that the event was emitted.
        assert_eq!(event_count.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set up an `DisclosureClient` start to return the following error.
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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let _verifier_certificate = setup_disclosure_client_start(
            &mut wallet.disclosure_client,
            default_pid_credential_requests(CredentialFormat::MsoMdoc),
        );

        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_types_and_format()
            .times(1)
            .returning(move |_, _, _| Err(StorageError::AlreadyOpened));

        // Starting disclosure on a wallet that has a faulty database should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::AttestationRetrieval(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_disclosure_history_retrieval_error() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        setup_disclosure_client_start(
            &mut wallet.disclosure_client,
            default_pid_credential_requests(CredentialFormat::MsoMdoc),
        );

        let stored_attestation_copy = example_pid_stored_attestation_copy(CredentialFormat::MsoMdoc);

        let expectation_attestation_copy = stored_attestation_copy.clone();
        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_types_and_format()
            .withf(move |attestation_types, format, _| {
                *attestation_types == HashSet::from([PID_ATTESTATION_TYPE.to_owned()])
                    && *format == CredentialFormat::MsoMdoc
            })
            .times(1)
            .return_once(move |_, _, _| Ok(vec![expectation_attestation_copy.clone()]));

        wallet
            .mut_storage()
            .expect_did_share_data_with_relying_party()
            .return_once(|_| Err(StorageError::AlreadyOpened));

        // Starting disclosure where retrieving whether data has been shared with the relying party fails, should result
        // in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::HistoryRetrieval(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_start_disclosure_error_attributes_not_available_not_present(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        use crate::wallet::disclosure::AttributesNotAvailable;

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let credential_requests = default_pid_credential_requests(requested_format);
        let verifier_certificate =
            setup_disclosure_client_start(&mut wallet.disclosure_client, credential_requests.clone());

        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_types_and_format()
            .times(1)
            .returning(move |_, _, _| Ok(vec![]));

        wallet
            .mut_storage()
            .expect_did_share_data_with_relying_party()
            .return_once(|_| Ok(false));

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        let expected_attributes = credential_requests
            .as_ref()
            .iter()
            .flat_map(|request| {
                request
                    .claim_paths()
                    .map(|path| format!("{}/{}", PID_ATTESTATION_TYPE, path.iter().join("/")))
            })
            .collect::<HashSet<_>>();

        assert_matches!(
            error,
            DisclosureError::AttributesNotAvailable(AttributesNotAvailable {
                reader_registration,
                requested_attributes,
                shared_data_with_relying_party_before,
                session_type: SessionType::CrossDevice,
            }) if reader_registration.as_ref() == verifier_certificate.registration() &&
                requested_attributes == expected_attributes &&
                !shared_data_with_relying_party_before
        );
        assert!(wallet.session.is_some());
    }

    #[rstest]
    #[case(CredentialFormat::MsoMdoc, &[PID_ATTESTATION_TYPE, "favourite_colour"])]
    #[case(CredentialFormat::MsoMdoc, &["family_name"])]
    #[case(CredentialFormat::MsoMdoc, &["long", "path", "family_name"])]
    #[case(CredentialFormat::SdJwt, &["favourite_colour"])]
    #[case(CredentialFormat::SdJwt, &[PID_ATTESTATION_TYPE, "family_name"])]
    #[case(CredentialFormat::SdJwt, &["long", "path", "family_name"])]
    #[tokio::test]
    async fn test_wallet_start_disclosure_error_attributes_not_available_non_matching(
        #[case] requested_format: CredentialFormat,
        #[case] path: &[&str],
    ) {
        let credential_requests = match requested_format {
            CredentialFormat::MsoMdoc => {
                NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(PID_ATTESTATION_TYPE, &[path])], None)
            }
            CredentialFormat::SdJwt => {
                NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(&[PID_ATTESTATION_TYPE], &[path])])
            }
        };

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set the requested attribute path to something that will not match the mdoc 2-tuple
        // of namespace and attribute, which should lead to no candidates being available.
        let verifier_certificate = setup_disclosure_client_start(&mut wallet.disclosure_client, credential_requests);

        let stored_attestation_copy = example_pid_stored_attestation_copy(requested_format);

        let expectation_attestation_copy = stored_attestation_copy.clone();
        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_types_and_format()
            .times(1)
            .returning(move |_, _, _| Ok(vec![expectation_attestation_copy.clone()]));

        wallet
            .mut_storage()
            .expect_did_share_data_with_relying_party()
            .return_once(|_| Ok(false));

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        let expected_attributes = HashSet::from([format!("{}/{}", PID_ATTESTATION_TYPE, path.join("/"))]);
        assert_matches!(
            error,
            DisclosureError::AttributesNotAvailable(AttributesNotAvailable {
                reader_registration,
                requested_attributes,
                shared_data_with_relying_party_before,
                session_type: SessionType::CrossDevice,
            }) if reader_registration.as_ref() == verifier_certificate.registration() &&
                requested_attributes == expected_attributes &&
                !shared_data_with_relying_party_before
        );
        assert!(wallet.session.is_some());
    }

    #[rstest]
    #[case(CredentialFormat::MsoMdoc, &[PID_ATTESTATION_TYPE, PID_RECOVERY_CODE])]
    #[case(CredentialFormat::SdJwt, &[PID_RECOVERY_CODE])]
    #[tokio::test]
    async fn test_wallet_start_disclosure_error_recovery_code_requested(
        #[case] requested_format: CredentialFormat,
        #[case] path: &[&str],
    ) {
        let credential_requests = match requested_format {
            CredentialFormat::MsoMdoc => {
                NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(PID_ATTESTATION_TYPE, &[path])], None)
            }
            CredentialFormat::SdJwt => {
                NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(&[PID_ATTESTATION_TYPE], &[path])])
            }
        };

        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Set the requested attribute path to the recovery code
        setup_disclosure_client_start(&mut wallet.disclosure_client, credential_requests);

        // Starting disclosure where the recovery code is requested should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should not succeed");

        assert_matches!(error, DisclosureError::RecoveryCodeRequested(_));
        assert!(wallet.session.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_cancel_disclosure(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
        #[values(false, true)] has_missing_attributes: bool,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = if has_missing_attributes {
            setup_wallet_disclosure_session_missing_attributes(requested_format)
        } else {
            setup_wallet_disclosure_session(requested_format)
        };
        wallet.session = Some(session);

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

        // Verify that a disclosure cancel event will be recorded.
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
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let event_count = monitor_event_count(&mut wallet).await;

        // Cancelling disclosure should result in a `Wallet` without a disclosure session.
        let cancel_return_url = wallet
            .cancel_disclosure()
            .await
            .expect("cancelling disclosure should succeed");

        assert_eq!(cancel_return_url.as_ref(), Some(RETURN_URL.as_ref()));
        assert!(wallet.session.is_none());

        assert_eq!(event_count.load(Ordering::Relaxed), 2);
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_blocked(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session that is blocked.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(requested_format);
        wallet.session = Some(session);

        wallet.update_policy_repository.state = VersionState::Block;

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Cancelling disclosure on a blocked wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::VersionBlocked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Cancelling disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_locked(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        // Prepare a registered and locked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (session, _verifier_certificate) = setup_wallet_disclosure_session(requested_format);
        wallet.session = Some(session);

        wallet.lock();

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Cancelling disclosure on a locked wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::Locked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Cancelling disclosure on a wallet without an active disclosure session should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("cancelling disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_vp_client_return_url(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let (session, _verifier_certificate) = setup_wallet_disclosure_session(requested_format);

        wallet.session = Some(session);

        wallet.mut_storage().expect_log_disclosure_event().never();

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
    }

    #[rstest]
    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_event_storage(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session and a faulty database.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(requested_format);
        wallet.session = Some(session);

        wallet
            .mut_storage()
            .expect_fetch_recent_wallet_events()
            .returning(move || Ok(vec![]));
        let events = setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .times(1)
            .return_once(|_, _, _, _, _| Err(StorageError::AlreadyOpened));

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
    #[rstest]
    #[tokio::test]
    async fn test_wallet_accept_disclosure_abridged(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] requested_format: CredentialFormat,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = setup_wallet_disclosure_session(requested_format);
        wallet.session = Some(session);

        // Set up the `disclose()` method to return the following.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        let disclose_return_url = RETURN_URL.clone();
        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .return_once(|_disclosable_attestations| Ok(Some(disclose_return_url)));

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .times(1)
            .return_once(|_| Ok(()));

        let (reader_certificate, _) = verifier_certificate.into_certificate_and_registration();
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
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let event_count = monitor_event_count(&mut wallet).await;

        let accept_return_url = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect("accepting disclosure should succeed");

        // Accepting disclosure should result in a `Wallet` without a disclosure session.
        assert_eq!(accept_return_url.as_ref(), Some(RETURN_URL.as_ref()));
        assert!(wallet.session.is_none());

        assert_eq!(event_count.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_blocked() {
        // Prepare a registered and unlocked wallet with an active disclosure session that is blocked.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        wallet.update_policy_repository.state = VersionState::Block;

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a blocked wallet should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::VersionBlocked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Accepting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_locked() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        wallet.lock();

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a locked wallet should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::Locked);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a wallet without an active disclosure session should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_unexpected_redirect_uri_purpose() {
        // Prepare a registered and unlocked wallet with an active disclosure based issuance session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        session.redirect_uri_purpose = RedirectUriPurpose::Issuance;

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a wallet that has a disclosure based issuance session should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(
            error,
            DisclosureError::UnexpectedRedirectUriPurpose {
                expected: RedirectUriPurpose::Issuance,
                found: RedirectUriPurpose::Browser,
            }
        );
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state_missing_attributes() {
        // Prepare a registered and unlocked wallet with an active disclosure session that has missing attributes.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        session.attestations = WalletDisclosureAttestations::Missing;

        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a wallet without an active disclosure session should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
    }

    #[tokio::test]
    #[should_panic(expected = "disclosure attestation count does not match query, expected 1, found 2")]
    async fn test_wallet_accept_disclosure_panic_query_index_out_of_bounds() {
        // Prepare a registered and unlocked wallet with an active disclosure based issuance session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::SdJwt);
        wallet.session = Some(session);

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .return_once(|| Ok(None));
        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a wallet while selecting a non-existant query index should result in a panic.
        let _ = wallet.accept_disclosure(&[0, 0], PIN.to_string()).await;
    }

    #[tokio::test]
    #[should_panic(expected = "selected disclosure attestation out of bounds for query index 0 with count 1: 1")]
    async fn test_wallet_accept_disclosure_panic_proposal_index_out_of_bounds() {
        // Prepare a registered and unlocked wallet with an active disclosure based issuance session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, _verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::SdJwt);
        wallet.session = Some(session);

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .return_once(|| Ok(None));
        wallet.mut_storage().expect_log_disclosure_event().never();

        // Accepting disclosure on a wallet while selecting a non-existant
        // attestation proposal should result in a panic.
        let _ = wallet.accept_disclosure(&[1], PIN.to_string()).await;
    }

    // TODO (PVW-3844): Add tests for continuing a PIN change when accepting disclosure.

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_increment_usage_count() {
        // Prepare a registered and unlocked wallet with an active disclosure session and a faulty database.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        let storage = wallet.mut_storage();
        storage
            .expect_increment_attestation_copies_usage_count()
            .times(1)
            .return_once(|_| Err(StorageError::NotOpened));

        storage.expect_fetch_data::<ChangePinData>().returning(|| Ok(None));

        let reader_certificate = verifier_certificate.certificate().clone();
        storage
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(vec![]),
                eq(reader_certificate),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        // Accepting disclosure on a wallet with a faulty database should result
        // in an error, the disclosure session should not be removed.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(error, DisclosureError::IncrementUsageCount(_));
        assert!(error.return_url().is_none());
        assert!(wallet.session.is_some());
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
        #[values(DataDisclosed::Disclosed, DataDisclosed::NotDisclosed)] data_shared: DataDisclosed,
    ) where
        F: Fn() -> E,
        E: Into<VpMessageClientError>,
    {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        let Some(Session::Disclosure(session)) = &wallet.session else {
            unreachable!();
        };

        let copy_ids = session
            .attestations
            .select_proposal(&[0])
            .values()
            .map(|attestation| attestation.attestation_copy_id())
            .collect_vec();

        // Check that the usage count got incremented for all of the attestation copy ids.
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .with(eq(copy_ids.clone()))
            .times(1)
            .returning(|_| Ok(()));

        // Verify that a disclosure error event will be recorded, with attestations if the data was shared.
        let reader_certificate = verifier_certificate.certificate().clone();
        let (first_event_attestations_tx, first_event_attestations_rx) = std::sync::mpsc::channel();
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                function(move |attestations: &Vec<_>| {
                    first_event_attestations_tx.send(attestations.clone()).unwrap();

                    attestations.len() == if data_shared == DataDisclosed::Disclosed { 1 } else { 0 }
                }),
                eq(reader_certificate),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let event_count = monitor_event_count(&mut wallet).await;

        let disclose_verifier_certificate = verifier_certificate.clone();
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        let mut disclosure_error = disclosure_session::DisclosureError::from(error_factory().into());
        // Fudge the `data_shared` property, since we cannot emulate an error that will make it be `false`.
        disclosure_error.data_shared = data_shared;
        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .return_once(|_disclosable_attestations| {
                Err((
                    Box::new(setup_disclosure_session_verifier_certificate(
                        disclose_verifier_certificate,
                        default_pid_credential_requests(CredentialFormat::MsoMdoc),
                    )),
                    disclosure_error,
                ))
            });

        // Accepting disclosure when the verifier responds with an invalid request error should result in an error.
        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        wallet.mut_storage().checkpoint();

        // Check the error type and its return URL and check if the wallet still has an active disclosure session.
        expected_error_type.check_error(&error, &verifier_certificate.registration().organization);
        if expect_return_url {
            assert_eq!(error.return_url(), Some(RETURN_URL.as_ref()));
        } else {
            assert!(error.return_url().is_none());
        }
        assert!(wallet.session.is_some());

        assert_eq!(event_count.load(Ordering::Relaxed), 2);

        // Repeating the disclosure with exactly the same error should result in an
        // increment in usage count and exactly the same disclosure error event.
        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };
        let disclose_verifier_certificate = verifier_certificate.clone();
        let mut disclosure_error = disclosure_session::DisclosureError::from(error_factory().into());
        disclosure_error.data_shared = data_shared;
        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .return_once(|_disclosable_attestations| {
                Err((
                    Box::new(setup_disclosure_session_verifier_certificate(
                        disclose_verifier_certificate,
                        default_pid_credential_requests(CredentialFormat::MsoMdoc),
                    )),
                    disclosure_error,
                ))
            });

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .with(eq(copy_ids))
            .times(1)
            .returning(|_| Ok(()));

        let reader_certificate = verifier_certificate.certificate().clone();
        let first_event_attestations = first_event_attestations_rx.try_recv().unwrap();
        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .with(
                always(),
                eq(first_event_attestations),
                eq(reader_certificate),
                eq(EventStatus::Error),
                eq(DisclosureType::Regular),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        wallet
            .mut_storage()
            .expect_fetch_recent_wallet_events()
            .returning(move || Ok(vec![]));

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        expected_error_type.check_error(&error, &verifier_certificate.registration().organization);
        if expect_return_url {
            assert_eq!(error.return_url(), Some(RETURN_URL.as_ref()));
        } else {
            assert!(error.return_url().is_none());
        }
        assert!(wallet.session.is_some());

        assert_eq!(event_count.load(Ordering::Relaxed), 3);
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
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        let Some(Session::Disclosure(session)) = &wallet.session else {
            unreachable!();
        };

        let copy_ids = session
            .attestations
            .select_proposal(&[0])
            .values()
            .map(|attestation| attestation.attestation_copy_id())
            .collect_vec();

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        // Check that the usage count got incremented for all of the attestation copy ids.
        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .with(eq(copy_ids))
            .times(1)
            .returning(|_| Ok(()));

        let reader_certificate = verifier_certificate.certificate().clone();
        match instruction_expectation {
            InstructionExpectation::Retry => {}
            InstructionExpectation::RetryWithEvent => {
                // Verify that a disclosure error event will be recorded.
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
                    .times(1)
                    .returning(|_, _, _, _, _| Ok(()));
            }
            InstructionExpectation::Termination => {
                // Verify that both a disclosure cancellation and error event are recorded.
                let error_reader_certificate = reader_certificate.clone();

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
                    .times(1)
                    .returning(|_, _, _, _, _| Ok(()));

                wallet
                    .mut_storage()
                    .expect_log_disclosure_event()
                    .with(
                        always(),
                        eq(vec![]),
                        eq(error_reader_certificate),
                        eq(EventStatus::Error),
                        eq(DisclosureType::Regular),
                    )
                    .times(1)
                    .returning(|_, _, _, _, _| Ok(()));
            }
        }

        let event_count = monitor_event_count(&mut wallet).await;

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        session
            .protocol_state
            .expect_disclose()
            .times(1)
            .return_once(move |_disclosable_attestations| {
                let mut session = setup_disclosure_session_verifier_certificate(
                    verifier_certificate,
                    default_pid_credential_requests(CredentialFormat::MsoMdoc),
                );

                if instruction_expectation == InstructionExpectation::Termination {
                    session.expect_terminate().times(1).return_once(|| Ok(None));
                }

                Err((
                    Box::new(session),
                    disclosure_session::DisclosureError::before_sharing(VpSessionError::Client(
                        VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(Box::new(
                            RemoteEcdsaKeyError::Instruction(instruction_error),
                        )))),
                    )),
                ))
            });

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
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

        let expected_event_count = match instruction_expectation {
            InstructionExpectation::Retry => 1,
            InstructionExpectation::RetryWithEvent => 2,
            InstructionExpectation::Termination => 3,
        };

        assert_eq!(event_count.load(Ordering::Relaxed), expected_event_count);
    }

    #[tokio::test]
    async fn test_start_disclosure_error_non_sd_claim_not_requested() {
        let my_attestation_type = "my.attestation.type";
        let my_sd_claim = "sd_claim";
        let my_first_non_sd_claim = "first_non_sd_claim";
        let my_second_non_sd_claim = "second_non_sd_claim";

        // Create a registered wallet
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Create a request that requests the sd claim and only 1 of the non-sd claims
        let credential_requests = NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
            &[my_attestation_type],
            &[&[my_sd_claim], &[my_second_non_sd_claim]], // NOTE: here we omit `my_first_non_sd_claim`
        )]);

        let _verifier_certificate = setup_disclosure_client_start(&mut wallet.disclosure_client, credential_requests);

        // Create metadata with an sd claim and 2 non-sd claims
        let mut type_metadata_with_non_selectively_disclosable_claim = UncheckedTypeMetadata::example_with_claim_names(
            my_attestation_type,
            &[
                (my_sd_claim, JsonSchemaPropertyType::String, None),
                (my_first_non_sd_claim, JsonSchemaPropertyType::String, None),
                (my_second_non_sd_claim, JsonSchemaPropertyType::String, None),
            ],
        );
        for claim in &mut type_metadata_with_non_selectively_disclosable_claim.claims {
            if [
                vec_nonempty![ClaimPath::SelectByKey(my_first_non_sd_claim.to_string())],
                vec_nonempty![ClaimPath::SelectByKey(my_second_non_sd_claim.to_string())],
            ]
            .contains(&claim.path)
            {
                claim.sd = sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata::Never;
            }
        }
        let type_metadata_with_non_selectively_disclocable_claim =
            NormalizedTypeMetadata::from_single_example(type_metadata_with_non_selectively_disclosable_claim);

        // Create a credential payload with an sd claim and 2 non-sd claims
        let previewable_payload = CredentialPayload::example_with_attributes(
            my_attestation_type,
            Attributes::example([
                ([my_sd_claim], AttributeValue::Text("Some Sd Claim".to_string())),
                (
                    [my_first_non_sd_claim],
                    AttributeValue::Text("Some Non Sd Claim".to_string()),
                ),
                (
                    [my_second_non_sd_claim],
                    AttributeValue::Text("Some Non Sd Claim".to_string()),
                ),
            ]),
            SigningKey::random(&mut OsRng).verifying_key(),
            &MockTimeGenerator::epoch(),
        );

        // Create an attestation for the above metadata and credential payload
        let attestation = example_stored_attestation_copy(
            CredentialFormat::SdJwt,
            previewable_payload,
            type_metadata_with_non_selectively_disclocable_claim,
        );

        // Mock the wallet database to return the attestation for the requested attestation type
        let (attestation_type, attestations) = (my_attestation_type, vec![attestation]);
        wallet
            .mut_storage()
            .expect_fetch_valid_unique_attestations_by_types_and_format()
            .withf(move |attestation_types, format, _| {
                *attestation_types == HashSet::from([attestation_type.to_owned()]) && *format == CredentialFormat::SdJwt
            })
            .times(1)
            .return_once(move |_, _, _| Ok(attestations));

        // The wallet will not check in the database if data was shared with the RP before.
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

        // Starting disclosure should not cause a disclosure event to be recorded.
        wallet.mut_storage().expect_log_disclosure_event().never();

        // Starting disclosure should fail with a non-selectively disclosable claim verification error
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect_err("starting disclosure should fail");

        assert_matches!(
            error,
            DisclosureError::NonSelectivelyDisclosableClaimsNotRequested(_, claims, attestation_type) if
                claims == vec![vec_nonempty![my_first_non_sd_claim.parse().unwrap()]] &&
                attestation_type == vec![my_attestation_type.to_string()]
        );

        wallet.mut_storage().checkpoint();
    }

    #[tokio::test]
    async fn test_accept_disclosure_error_revoked_user_request() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .times(1)
            .returning(|_| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        // UserRequest revocation calls reset_to_initial_state(), which clears storage.
        wallet.mut_storage().expect_clear().times(1).return_const(());

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        session.protocol_state.expect_disclose().times(1).return_once(move |_| {
            let session = setup_disclosure_session_verifier_certificate(
                verifier_certificate,
                default_pid_credential_requests(CredentialFormat::MsoMdoc),
            );

            Err((
                Box::new(session),
                wallet_revocation_error(AccountRevokedData {
                    revocation_reason: RevocationReason::UserRequest,
                    can_register_new_account: true,
                }),
            ))
        });

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(
            error,
            DisclosureError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::UserRequest,
                can_register_new_account: true
            }))
        );

        // After a UserRequest revocation, the wallet is fully reset: unregistered and locked.
        assert!(!wallet.registration.is_registered());
        assert!(wallet.is_locked());
        // The disclosure session is not placed back after a revocation.
        assert!(wallet.session.is_none());
    }

    #[tokio::test]
    async fn test_accept_disclosure_error_revoked_admin_request() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let (session, verifier_certificate) = setup_wallet_disclosure_session(CredentialFormat::MsoMdoc);
        wallet.session = Some(session);

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        wallet
            .mut_storage()
            .expect_increment_attestation_copies_usage_count()
            .times(1)
            .returning(|_| Ok(()));

        wallet
            .mut_storage()
            .expect_log_disclosure_event()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        // AdminRequest revocation stores the reason without resetting the wallet.
        wallet
            .mut_storage()
            .expect_insert_data::<AccountRevokedData>()
            .times(1)
            .returning(|_| Ok(()));

        let Some(Session::Disclosure(session)) = &mut wallet.session else {
            unreachable!();
        };

        session.protocol_state.expect_disclose().times(1).return_once(move |_| {
            let session = setup_disclosure_session_verifier_certificate(
                verifier_certificate,
                default_pid_credential_requests(CredentialFormat::MsoMdoc),
            );

            Err((
                Box::new(session),
                wallet_revocation_error(AccountRevokedData {
                    revocation_reason: RevocationReason::AdminRequest,
                    can_register_new_account: true,
                }),
            ))
        });

        let error = wallet
            .accept_disclosure(&[0], PIN.to_string())
            .await
            .expect_err("accepting disclosure should not succeed");

        assert_matches!(
            error,
            DisclosureError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: true
            }))
        );
        // After an AdminRequest revocation, the wallet remains registered.
        assert!(wallet.registration.is_registered());
        // The disclosure session is not placed back after a revocation.
        assert!(wallet.session.is_none());
    }

    /// Returns a disclosure error that simulates the WP having revoked our wallet.
    fn wallet_revocation_error(data: AccountRevokedData) -> disclosure_session::DisclosureError<VpSessionError> {
        disclosure_session::DisclosureError::before_sharing(VpSessionError::Client(VpClientError::DeviceResponse(
            mdoc::Error::Cose(CoseError::Signing(Box::new(RemoteEcdsaKeyError::Instruction(
                InstructionError::AccountRevoked(data),
            )))),
        )))
    }

    fn setup_pid_attributes_config() -> PidAttributesConfiguration {
        PidAttributesConfiguration {
            mso_mdoc: HashMap::from([(
                PID_ATTESTATION_TYPE.to_owned(),
                PidAttributePaths {
                    login: vec_nonempty!["login".to_owned()],
                    recovery_code: vec_nonempty![PID_ATTESTATION_TYPE.to_owned(), PID_RECOVERY_CODE.to_owned()],
                },
            )]),
            sd_jwt: HashMap::from([(
                PID_ATTESTATION_TYPE.to_owned(),
                PidAttributePaths {
                    login: vec_nonempty!["login".to_owned()],
                    recovery_code: vec_nonempty![PID_RECOVERY_CODE.to_owned()],
                },
            )]),
        }
    }

    #[rstest]
    #[case::mdoc_reco(
        CredentialFormat::MsoMdoc,
        PID_ATTESTATION_TYPE,
        vec_nonempty![ClaimPath::SelectByKey(PID_ATTESTATION_TYPE.to_owned()), ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())],
        true,
    )]
    #[case::mdoc_not_reco(
        CredentialFormat::MsoMdoc,
        PID_ATTESTATION_TYPE,
        vec_nonempty![ClaimPath::SelectByKey(PID_ATTESTATION_TYPE.to_owned()), ClaimPath::SelectByKey(PID_FAMILY_NAME.to_owned())],
        false,
    )]
    #[case::mdoc_unknown(
        CredentialFormat::MsoMdoc,
        "unknown_doctype",
        vec_nonempty![ClaimPath::SelectByKey("unknown_doctype".to_owned()), ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())],
        false,
    )]
    #[case::sdjwt_reco(
        CredentialFormat::SdJwt,
        PID_ATTESTATION_TYPE,
        vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())],
        true,
    )]
    #[case::sdjwt_not_reco(
        CredentialFormat::SdJwt,
        PID_ATTESTATION_TYPE,
        vec_nonempty![ClaimPath::SelectByKey(PID_FAMILY_NAME.to_owned())],
        false,
    )]
    #[case::sdjwt_unknown(
        CredentialFormat::SdJwt,
        "unknown_vct",
        vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_owned())],
        false,
    )]
    fn test_is_request_for_recovery_code(
        #[case] format: CredentialFormat,
        #[case] credential_type: &str,
        #[case] path: VecNonEmpty<ClaimPath>,
        #[case] expected: bool,
    ) {
        let pid_attributes = setup_pid_attributes_config();

        let request = match format {
            CredentialFormat::MsoMdoc => NormalizedCredentialRequest::MsoMdoc {
                id: "identifier".try_into().unwrap(),
                doctype_value: credential_type.to_owned(),
                claims: vec_nonempty![MdocAttributeRequest {
                    path,
                    intent_to_retain: Some(false),
                }],
            },
            CredentialFormat::SdJwt => NormalizedCredentialRequest::SdJwt {
                id: "identifier".try_into().unwrap(),
                vct_values: vec_nonempty![credential_type.to_owned()],
                claims: vec_nonempty![SdJwtAttributeRequest { path }],
            },
        };

        assert_eq!(is_request_for_recovery_code(request, &pid_attributes), expected);
    }
}
