use std::collections::HashSet;
use std::sync::Arc;

use indexmap::IndexMap;
use itertools::Itertools;
use tracing::error;
use tracing::info;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use mdoc::holder::MdocDataSource;
use mdoc::holder::StoredMdoc;
use mdoc::utils::cose::CoseError;
use mdoc::utils::issuer_auth::IssuerRegistration;
use mdoc::utils::reader_auth::ReaderRegistration;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::verifier::SessionType;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_common::http::TlsPinningConfig;
use wallet_common::update_policy::VersionState;
use wallet_common::urls;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::Attestation;
use crate::attestation::AttestationError;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::disclosure::DisclosureUriError;
use crate::disclosure::DisclosureUriSource;
use crate::disclosure::MdocDisclosureError;
use crate::disclosure::MdocDisclosureMissingAttributes;
use crate::disclosure::MdocDisclosureProposal;
use crate::disclosure::MdocDisclosureSession;
use crate::disclosure::MdocDisclosureSessionState;
use crate::errors::ChangePinError;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionError;
use crate::instruction::RemoteEcdsaKeyError;
use crate::instruction::RemoteEcdsaKeyFactory;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage;
use crate::storage::DataDisclosureStatus;
use crate::storage::DisclosureType;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredMdocCopy;
use crate::storage::WalletEvent;

use super::Wallet;

#[derive(Debug, Clone)]
pub struct DisclosureProposal {
    pub attestations: Vec<Attestation>,
    pub reader_registration: ReaderRegistration,
    pub shared_data_with_relying_party_before: bool,
    pub session_type: SessionType,
    pub disclosure_type: DisclosureType,
    pub disclosure_result: RedirectUriPurpose,
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
    #[error("could not parse disclosure URI: {0}")]
    DisclosureUri(#[source] DisclosureUriError),
    #[error("error in OpenID4VP disclosure session: {0}")]
    VpDisclosureSession(#[from] VpClientError),
    #[error("could not fetch if attributes were shared before: {0}")]
    HistoryRetrieval(#[source] StorageError),
    #[error("not all requested attributes are available, missing: {missing_attributes:?}")]
    #[category(pd)] // Might reveal information about what attributes are stored in the Wallet
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        missing_attributes: Vec<String>,
        shared_data_with_relying_party_before: bool,
        session_type: SessionType,
    },
    #[error("could not extract issuer registration from stored mdoc certificate: {0}")]
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
    pub fn return_url(&self) -> Option<&Url> {
        match self {
            Self::VpDisclosureSession(VpClientError::Request(error)) => error.redirect_uri().map(AsRef::as_ref),
            _ => None,
        }
    }
}

impl From<MdocDisclosureError> for DisclosureError {
    fn from(error: MdocDisclosureError) -> Self {
        // Note that the `.unwrap()` and `panic!()` statements below are safe,
        // as checking is performed within the guard statements.
        match error {
            // Upgrade any signing errors that are caused an instruction error to `DisclosureError::Instruction`.
            MdocDisclosureError::Vp(VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(error))))
                if matches!(
                    error.downcast_ref::<RemoteEcdsaKeyError>(),
                    Some(RemoteEcdsaKeyError::Instruction(_))
                ) =>
            {
                if let RemoteEcdsaKeyError::Instruction(error) = *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                    DisclosureError::Instruction(error)
                } else {
                    panic!()
                }
            }
            // Any other error should result in its generic top-level error variant.
            MdocDisclosureError::Vp(error) => DisclosureError::VpDisclosureSession(error),
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
pub struct WalletDisclosureSession<MDS> {
    redirect_uri_purpose: RedirectUriPurpose,
    protocol_state: MDS,
}

impl<MDS> WalletDisclosureSession<MDS> {
    pub fn new(redirect_uri_purpose: RedirectUriPurpose, protocol_state: MDS) -> Self {
        Self {
            redirect_uri_purpose,
            protocol_state,
        }
    }
}

// Normal sessions resulting in a redirect_uri will be the most common, so we provide a From impl for convenience.
impl<MDS> From<MDS> for WalletDisclosureSession<MDS> {
    fn from(protocol_state: MDS) -> Self {
        Self {
            redirect_uri_purpose: RedirectUriPurpose::Browser,
            protocol_state,
        }
    }
}

impl RedirectUriPurpose {
    fn from_uri(uri: &Url) -> Result<(Self, Url), DisclosureError> {
        let disclosure_base_uri = urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner();
        let disclosure_based_issuance_base_uri =
            urls::disclosure_based_issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner();

        if uri.as_str().starts_with(disclosure_based_issuance_base_uri.as_str()) {
            Ok((Self::Issuance, disclosure_based_issuance_base_uri))
        } else if uri.as_str().starts_with(disclosure_base_uri.as_str()) {
            Ok((Self::Browser, disclosure_base_uri))
        } else {
            Err(DisclosureError::DisclosureUri(DisclosureUriError::Malformed(
                uri.clone(),
            )))
        }
    }
}

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
    MDS: MdocDisclosureSession<Self>,
    S: Storage,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn start_disclosure(
        &mut self,
        uri: &Url,
        source: DisclosureUriSource,
    ) -> Result<DisclosureProposal, DisclosureError> {
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

        info!("Checking if there is already a disclosure session");
        if self.disclosure_session.is_some() {
            return Err(DisclosureError::SessionState);
        }

        let config = &self.config_repository.get().disclosure;

        let (disclosure_result, base_url) = RedirectUriPurpose::from_uri(uri)?;
        let disclosure_uri = MDS::parse_url(uri, &base_url).map_err(DisclosureError::DisclosureUri)?;

        // Start the disclosure session based on the parsed disclosure URI.
        let session = MDS::start(disclosure_uri, source, self, &config.rp_trust_anchors()).await?;

        let shared_data_with_relying_party_before = self
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(session.rp_certificate())
            .await
            .map_err(DisclosureError::HistoryRetrieval)?;

        let proposal_session = match session.session_state() {
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session) => {
                // TODO (PVW-3813): Attempt to translate the missing attributes using the TAS cache.
                //                  If translation fails, the missing attributes cannot be presented to
                //                  the user and we should simply never respond to the verifier in order
                //                  to prevent gleaning of absence of attestation.
                info!("At least one attribute is missing in order to satisfy the disclosure request");

                let reader_registration = session.reader_registration().clone().into();
                let missing_attributes = missing_attr_session
                    .missing_attributes()
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                let session_type = session.session_type();

                // Store the session so that it will only be terminated on user interaction.
                // This prevents gleaning of missing attributes by a verifier.
                self.disclosure_session
                    .replace(WalletDisclosureSession::new(disclosure_result, session));

                return Err(DisclosureError::AttributesNotAvailable {
                    reader_registration,
                    missing_attributes,
                    shared_data_with_relying_party_before,
                    session_type,
                });
            }
            MdocDisclosureSessionState::Proposal(proposal_session) => proposal_session,
        };

        info!("All attributes in the disclosure request are present in the database, return a proposal to the user");

        // Prepare a `IndexMap<DocType, ProposedDocumentAttributes>`.
        let proposed_attributes = proposal_session.proposed_attributes();

        let disclosure_type = storage::disclosure_type_for_proposed_attributes(&proposed_attributes);

        // Prepare a list of proposed attestations to report to the caller.
        let attestations: Vec<Attestation> = proposed_attributes
            .into_values()
            .map(|attributes| {
                let issuer_registration = IssuerRegistration::from_certificate(&attributes.issuer)
                    .map_err(DisclosureError::IssuerRegistration)?
                    .ok_or(DisclosureError::MissingIssuerRegistration)?;

                Attestation::create_for_disclosure(
                    attributes.type_metadata,
                    issuer_registration.organization,
                    attributes.attributes,
                )
                .map_err(DisclosureError::AttestationAttributes)
            })
            .try_collect()?;

        // Place this in a `DisclosureProposal`, along with a copy of the `ReaderRegistration`.
        let proposal = DisclosureProposal {
            attestations,
            reader_registration: session.reader_registration().clone(),
            shared_data_with_relying_party_before,
            session_type: session.session_type(),
            disclosure_type,
            disclosure_result,
        };

        // Retain the session as `Wallet` state.
        self.disclosure_session
            .replace(WalletDisclosureSession::new(disclosure_result, session));

        Ok(proposal)
    }

    /// When we have missing attributes, we don't have a proposal -> empty proposed_attributes.
    /// When we do have a proposal, give us the proposed attributes then. In both cases, empty
    /// or "real", use from_proposed_attributes to determine the disclosure_type.
    async fn terminate_disclosure_session(&mut self, session: MDS) -> Result<Option<Url>, DisclosureError> {
        let proposed_attributes = match session.session_state() {
            MdocDisclosureSessionState::MissingAttributes(_) => None,
            MdocDisclosureSessionState::Proposal(proposal_session) => Some(proposal_session.proposed_attributes()),
        };

        let event = WalletEvent::new_disclosure_cancel(
            proposed_attributes,
            session.rp_certificate().clone(),
            session.reader_registration().clone(),
            DataDisclosureStatus::NotDisclosed,
        );

        let return_url = session.terminate().await?;

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

        let has_active_session = self.disclosure_session.is_some();

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
        let session = self.disclosure_session.take().ok_or(DisclosureError::SessionState)?;

        self.terminate_disclosure_session(session.protocol_state).await
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
        self.perform_disclosure(pin, RedirectUriPurpose::Browser).await
    }

    #[instrument(skip_all)]
    pub(crate) async fn perform_disclosure(
        &mut self,
        pin: String,
        redirect_uri_purpose: RedirectUriPurpose,
    ) -> Result<Option<Url>, DisclosureError>
    where
        S: Storage,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        APC: AccountProviderClient,
        WIC: Default,
    {
        info!("Accepting disclosure");

        let config = &self.config_repository.get().update_policy_server;

        info!("Fetching update policy");
        self.update_policy_repository.fetch(&config.http_config).await?;

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
        let session = self.disclosure_session.as_ref().ok_or(DisclosureError::SessionState)?;

        if session.redirect_uri_purpose != redirect_uri_purpose {
            return Err(DisclosureError::UnexpectedRedirectUriPurpose {
                expected: session.redirect_uri_purpose,
                found: redirect_uri_purpose,
            });
        }

        let session = &session.protocol_state;

        let session_proposal = match session.session_state() {
            MdocDisclosureSessionState::Proposal(session_proposal) => session_proposal,
            _ => return Err(DisclosureError::SessionState),
        };

        // Increment the disclosure counts of the mdoc copies referenced in the proposal,
        // so that for the next disclosure different copies are used.

        // NOTE: If the disclosure fails and is retried, the disclosure count will jump by
        //       more than 1, since the same copies are shared with the verifier again.
        //       It is necessary to increment the disclosure count before sending the mdocs
        //       to the verifier, as we do not know if disclosure fails before or after the
        //       verifier has received the attributes.

        let result = self
            .storage
            .write()
            .await
            .increment_mdoc_copies_usage_count(session_proposal.proposed_source_identifiers())
            .await;

        if let Err(error) = result {
            let event = WalletEvent::new_disclosure_error(
                session_proposal.proposed_attributes(),
                session.rp_certificate().clone(),
                session.reader_registration().clone(),
                DataDisclosureStatus::NotDisclosed,
            );
            if let Err(e) = self.store_history_event(event).await {
                error!("Could not store error in history: {e}");
            }

            return Err(DisclosureError::IncrementUsageCount(error));
        }

        // Prepare the `RemoteEcdsaKeyFactory` for signing using the provided PIN.
        let config = self.config_repository.get();

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

        // Actually perform disclosure, casting any `InstructionError` that
        // occur during signing to `RemoteEcdsaKeyError::Instruction`.
        let result = session_proposal.disclose(&remote_key_factory).await;
        let return_url = match result {
            Ok(return_url) => return_url,
            Err(error) => {
                let disclosure_error = DisclosureError::from(error.error);

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
                        session_proposal.proposed_attributes(),
                        session.rp_certificate().clone(),
                        session.reader_registration().clone(),
                        data_status,
                    );
                    if let Err(e) = self.store_history_event(event).await {
                        error!("Could not store error in history: {e}");
                    }
                }

                if matches!(
                    disclosure_error,
                    DisclosureError::Instruction(InstructionError::Timeout { .. } | InstructionError::Blocked)
                ) {
                    // On a PIN timeout we should proactively terminate the disclosure session
                    // and lock the wallet, as the user is probably not the owner of the wallet.
                    // The UI should catch this specific error and close the disclosure screens.

                    let session = self.disclosure_session.take().unwrap();
                    if let Err(terminate_error) = self.terminate_disclosure_session(session.protocol_state).await {
                        // Log the error, but do not return it from this method.
                        error!(
                            "Error while terminating disclosure session on PIN timeout: {}",
                            terminate_error
                        );
                    }

                    self.lock.lock();
                }

                return Err(disclosure_error);
            }
        };

        // Get some data from the session that we need for the event, then remove the disclosure session,
        // as disclosure is now successful. Any errors that occur after this point will result in the
        // `Wallet` not having an active disclosure session anymore.
        let proposed_attributes = session_proposal.proposed_attributes();
        let reader_certificate = session.rp_certificate().clone();
        let reader_registration = session.reader_registration().clone();

        self.disclosure_session.take();

        // Save data for disclosure in event log.
        let event = WalletEvent::new_disclosure_success(
            proposed_attributes,
            reader_certificate,
            reader_registration,
            DataDisclosureStatus::Disclosed,
        );
        self.store_history_event(event)
            .await
            .map_err(DisclosureError::EventStorage)?;

        Ok(return_url)
    }
}

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> MdocDataSource for Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
{
    type MdocIdentifier = Uuid;
    type Error = StorageError;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        // Build an `IndexMap<>` to group `StoredMdoc` entries with the same `doc_type`.
        let mdocs_by_doc_type = self
            .storage
            .read()
            .await
            .fetch_unique_mdocs_by_doctypes(doc_types)
            .await?
            .into_iter()
            .fold(
                IndexMap::<_, Vec<_>>::with_capacity(doc_types.len()),
                |mut mdocs_by_doc_type, StoredMdocCopy { mdoc_copy_id, mdoc, .. }| {
                    // Re-use the `doc_types` string slices, which should contain all `Mdoc` doc types.
                    let doc_type = *doc_types
                        .get(mdoc.doc_type().as_str())
                        .expect("Storage returned mdoc with unexpected doc_type");
                    mdocs_by_doc_type
                        .entry(doc_type)
                        .or_default()
                        .push(StoredMdoc { id: mdoc_copy_id, mdoc });

                    mdocs_by_doc_type
                },
            );
        // Take only the values of this `HashMap`, which is what we need for the return type.
        let mdocs = mdocs_by_doc_type.into_values().collect();

        Ok(mdocs)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use itertools::Itertools;
    use mockall::predicate::*;
    use parking_lot::Mutex;
    use rstest::rstest;
    use serial_test::serial;
    use uuid::uuid;

    use mdoc::holder::Mdoc;
    use mdoc::holder::ProposedAttributes;
    use mdoc::holder::ProposedDocumentAttributes;
    use mdoc::test::data::PID;
    use mdoc::unsigned::Entry;
    use mdoc::DataElementValue;
    use openid4vc::attributes::AttributeError;
    use openid4vc::attributes::AttributeValue;
    use openid4vc::disclosure_session::VpMessageClientError;
    use openid4vc::DisclosureErrorResponse;
    use openid4vc::ErrorResponse;
    use openid4vc::GetRequestErrorCode;
    use openid4vc::PostAuthResponseErrorCode;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::TypeMetadata;

    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;
    use crate::config::UNIVERSAL_LINK_BASE_URL;
    use crate::disclosure::MockMdocDisclosureMissingAttributes;
    use crate::disclosure::MockMdocDisclosureProposal;
    use crate::disclosure::MockMdocDisclosureSession;
    use crate::storage::DisclosureStatus;
    use crate::AttestationAttribute;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::super::test::ISSUER_KEY;
    use super::*;

    static DISCLOSURE_URI: LazyLock<Url> =
        LazyLock::<Url>::new(|| urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).join("Zm9vYmFy"));
    const PROPOSED_ID: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

    fn setup_proposed_attributes(attrs: &[(&str, DataElementValue)]) -> ProposedAttributes {
        let metadata_props = attrs
            .iter()
            .map(|(name, value)| {
                (
                    *name,
                    match value {
                        DataElementValue::Text(_) => JsonSchemaPropertyType::String,
                        DataElementValue::Bool(_) => JsonSchemaPropertyType::Boolean,
                        DataElementValue::Integer(_) => JsonSchemaPropertyType::Integer,
                        DataElementValue::Float(_) => JsonSchemaPropertyType::Number,
                        DataElementValue::Null => JsonSchemaPropertyType::Null,
                        _ => unimplemented!(),
                    },
                    None,
                )
            })
            .collect::<Vec<_>>();

        IndexMap::from([(
            "com.example.pid".to_string(),
            ProposedDocumentAttributes {
                type_metadata: TypeMetadata::example_with_claim_names("com.example.pid", &metadata_props),
                attributes: IndexMap::from([(
                    "com.example.pid".to_string(),
                    attrs
                        .iter()
                        .map(|(name, value)| Entry {
                            name: String::from(*name),
                            value: value.clone(),
                        })
                        .collect::<Vec<_>>(),
                )]),
                issuer: ISSUER_KEY.issuance_key.certificate().clone(),
            },
        )])
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_start_disclosure() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let reader_registration = ReaderRegistration::new_mock();
        let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
        let proposal_session = MockMdocDisclosureProposal {
            proposed_source_identifiers: vec![PROPOSED_ID],
            proposed_attributes,
            ..Default::default()
        };

        MockMdocDisclosureSession::next_fields(
            reader_registration,
            MdocDisclosureSessionState::Proposal(proposal_session),
            None,
        );

        // Starting disclosure should not fail.
        let proposal = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::QrCode)
            .await
            .expect("Could not start disclosure");

        // Test that the `Wallet` now contains a `DisclosureSession`.
        assert_matches!(
            wallet.disclosure_session,
            Some(session) if session.protocol_state.disclosure_uri_source == DisclosureUriSource::QrCode
        );

        // Test that the returned `DisclosureProposal` contains the
        // `ReaderRegistration` we set up earlier, as well as the
        // proposed attributes converted to a `ProposedDisclosureDocument`.
        assert_eq!(proposal.attestations.len(), 1);
        let document = proposal.attestations.first().unwrap();
        assert_eq!(document.attestation_type, "com.example.pid");
        assert_eq!(document.attributes.len(), 1);
        assert_matches!(
            document.attributes.first().unwrap(),
            AttestationAttribute {
                value: AttestationAttributeValue::Basic(AttributeValue::Bool(true)),
                ..
            }
        );

        // Starting disclosure should not cause mdoc copy usage counts to be incremented.
        assert!(wallet.storage.read().await.mdoc_copies_usage_counts.is_empty());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.lock();

        // Starting disclosure on a locked wallet should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::Locked);
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Starting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_session_state() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Start an active disclosure session.
        wallet.disclosure_session = Some(MockMdocDisclosureSession::default().into());

        // Starting disclosure on a wallet with an active disclosure should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_disclosure_uri() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Starting disclosure on a wallet with a malformed disclosure URI should result in an error.
        // (The `MockMdocDisclosureSession` used by `WalletWithMocks` rejects URLs containing an `invalid`
        // query parameter.)
        let error = wallet
            .start_disclosure(
                &Url::parse("http://example.com?invalid").unwrap(),
                DisclosureUriSource::Link,
            )
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureUri(_));
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_start_disclosure_error_disclosure_session() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up an `MdocDisclosureSession` start to return the following error.
        MockMdocDisclosureSession::next_start_error(VpClientError::MissingSessionType.into());

        // Starting disclosure which returns an error should forward that error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::VpDisclosureSession(VpClientError::MissingSessionType)
        );
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_start_disclosure_error_return_url() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up an `MdocDisclosureSession` start to return the following error.
        let return_url = Url::parse("https://example.com/return/here").unwrap();
        MockMdocDisclosureSession::next_start_error(
            VpClientError::Request(VpMessageClientError::AuthGetResponse(DisclosureErrorResponse {
                error_response: ErrorResponse {
                    error: GetRequestErrorCode::ServerError,
                    error_description: None,
                    error_uri: None,
                },
                redirect_uri: Some(return_url.clone().try_into().unwrap()),
            }))
            .into(),
        );

        // Starting disclosure where the verifier returns responds with a HTTP error body containing
        // a redirect URI should result in that URI being available on the returned error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::VpDisclosureSession(_));
        assert_eq!(error.return_url(), Some(&return_url));
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_start_disclosure_error_attributes_not_available() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec!["com.example.pid/com.example.pid/age_over_18".parse().unwrap()];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session),
            None,
        );

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::AttributesNotAvailable {
                reader_registration: _,
                missing_attributes,
                shared_data_with_relying_party_before,
                session_type: SessionType::SameDevice,
            } if !shared_data_with_relying_party_before &&
                missing_attributes == vec!["com.example.pid/com.example.pid/age_over_18"]
        );
        assert!(wallet.disclosure_session.is_some());
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_start_disclosure_error_mdoc_attributes() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let mut proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);

        proposed_attributes
            .get_mut("com.example.pid")
            .unwrap()
            .attributes
            .get_mut("com.example.pid")
            .unwrap()
            .push(Entry {
                name: "foo".to_string(),
                value: DataElementValue::Text("bar".to_string()),
            });

        let proposal_session = MockMdocDisclosureProposal {
            proposed_attributes,
            ..Default::default()
        };

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::Proposal(proposal_session),
            None,
        );

        // Starting disclosure where unknown attributes are requested should result in an error.
        let error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::AttestationAttributes(
                AttestationError::Attribute(AttributeError::SomeAttributesNotProcessed(claims)))
                if claims == IndexMap::from([
                    (String::from("com.example.pid"),
                    vec![Entry {
                        name: String::from("foo"),
                        value: ciborium::value::Value::Text(String::from("bar"))
                    }]
                )]
            )
        );

        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_cancel_disclosure() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let reader_registration = ReaderRegistration::new_mock();
        let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
        let proposal_session = MockMdocDisclosureProposal {
            proposed_source_identifiers: vec![PROPOSED_ID],
            proposed_attributes,
            ..Default::default()
        };

        let return_url = Url::parse("https://example.com/return/here").unwrap();

        MockMdocDisclosureSession::next_fields(
            reader_registration,
            MdocDisclosureSessionState::Proposal(proposal_session),
            Some(return_url.clone()),
        );

        // Start a disclosure session, to ensure a proper session exists that can be cancelled.
        let _ = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect("Could not start disclosure");

        // Verify disclosure session is not yet terminated
        let was_terminated = Arc::clone(
            &wallet
                .disclosure_session
                .as_ref()
                .unwrap()
                .protocol_state
                .was_terminated,
        );
        assert!(!was_terminated.load(Ordering::Relaxed));

        // Get latest emitted recent_history events
        let latest_events = events.lock().pop().unwrap();
        // Verify no history events are yet logged
        assert!(latest_events.is_empty());

        // Cancelling disclosure should result in a `Wallet` without a disclosure
        // session, while the session that was there should be terminated.
        let cancel_return_url = wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

        assert_eq!(cancel_return_url, Some(return_url));

        // Verify disclosure session is terminated
        assert!(wallet.disclosure_session.is_none());
        assert!(was_terminated.load(Ordering::Relaxed));

        // Get latest emitted recent_history events
        let events = events.lock().pop().unwrap();
        // Verify a Disclosure Cancel event is logged
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: DisclosureStatus::Cancelled,
                ..
            }
        );

        // Cancelling disclosure should not cause mdoc copy usage counts to be incremented.
        assert!(wallet.storage.read().await.mdoc_copies_usage_counts.is_empty());
    }

    #[tokio::test]
    #[serial(MockMdocDisclosureSession)]
    async fn test_wallet_cancel_disclosure_missing_attributes() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec![
            "com.example.pid/com.example.pid/bsn".parse().unwrap(),
            "com.example.pid/com.example.pid/age_over_18".parse().unwrap(),
        ];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        let return_url = Url::parse("https://example.com/return/here").unwrap();

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session),
            Some(return_url.clone()),
        );

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let _error = wallet
            .start_disclosure(&DISCLOSURE_URI, DisclosureUriSource::Link)
            .await
            .expect_err("Starting disclosure should have resulted in an error");
        assert!(wallet.disclosure_session.is_some());

        // Verify disclosure session is not yet terminated
        let was_terminated = Arc::clone(
            &wallet
                .disclosure_session
                .as_ref()
                .unwrap()
                .protocol_state
                .was_terminated,
        );
        assert!(!was_terminated.load(Ordering::Relaxed));

        // Get latest emitted recent_history events
        let latest_events = events.lock().pop().unwrap();
        // Verify no history events are yet logged
        assert!(latest_events.is_empty());

        // Cancelling disclosure should result in a `Wallet` without a disclosure
        // session, while the session that was there should be terminated.
        let cancel_return_url = wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

        assert_eq!(cancel_return_url, Some(return_url));

        // Verify disclosure session is terminated
        assert!(wallet.disclosure_session.is_none());
        assert!(was_terminated.load(Ordering::Relaxed));

        // Get latest emitted recent_history events
        let events = events.lock().pop().unwrap();
        // Verify a single Disclosure Error event is logged
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: DisclosureStatus::Cancelled,
                ..
            }
        );
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_locked() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.disclosure_session = Some(MockMdocDisclosureSession::default().into());

        wallet.lock();

        // Cancelling disclosure on a locked wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("Cancelling disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::Locked);
        assert!(wallet.disclosure_session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Cancelling disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("Cancelling disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Cancelling disclosure on a wallet without an active
        // disclosure session should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("Cancelling disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_none());
    }

    const PIN: &str = "051097";

    #[tokio::test]
    async fn test_wallet_accept_disclosure() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let return_url = Url::parse("https://example.com/return/here").unwrap();

        let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
        let disclosure_session = MockMdocDisclosureProposal {
            disclose_return_url: return_url.clone().into(),
            proposed_source_identifiers: vec![PROPOSED_ID],
            proposed_attributes,
            ..Default::default()
        };

        // Create a `MockMdocDisclosureSession` with the return URL and the `MockMdocDisclosureProposal`,
        // copy the disclosure count and check that it is 0.
        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(disclosure_session),
            ..Default::default()
        };
        let disclosure_count = match disclosure_session.session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => Arc::clone(&proposal.disclosure_count),
            _ => unreachable!(),
        };
        assert_eq!(disclosure_count.load(Ordering::Relaxed), 0);

        let reader_certificate = disclosure_session.certificate.clone();

        wallet.disclosure_session = Some(disclosure_session.into());

        // Accepting disclosure should succeed and give us the return URL.
        let accept_result = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect("Could not accept disclosure");

        assert_matches!(accept_result, Some(result_return_url) if result_return_url == return_url);

        // Check that the disclosure session is no longer
        // present and that the disclosure count is 1.
        assert!(wallet.disclosure_session.is_none());
        assert!(!wallet.is_locked());
        assert_eq!(disclosure_count.load(Ordering::Relaxed), 1);

        // Get latest emitted recent_history events
        let events = events.lock().pop().unwrap();

        // Verify a single Disclosure Success event is logged, and documents are shared
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: DisclosureStatus::Success,
                attestations,
                ..
            } if !attestations.is_empty()
        );
        // Verify that `did_share_data_with_relying_party()` now returns `true`
        assert!(wallet
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(&reader_certificate)
            .await
            .unwrap());

        // Test that the usage count got incremented for the proposed mdoc copy id.
        let mdoc_copies_usage_counts = &wallet.storage.read().await.mdoc_copies_usage_counts;
        assert_eq!(mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
            1
        );
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_locked() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        wallet.disclosure_session = Some(MockMdocDisclosureSession::default().into());

        wallet.lock();

        // Accepting disclosure on a locked wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::Locked);
        assert!(wallet.disclosure_session.is_some());
        assert!(wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().protocol_state.session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);
            }
            _ => unreachable!(),
        };

        // The mdoc copy usage counts should not be incremented.
        assert!(wallet.storage.read().await.mdoc_copies_usage_counts.is_empty());

        // Verify no Disclosure events are logged
        assert!(wallet
            .storage
            .read()
            .await
            .fetch_wallet_events()
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Accepting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(wallet.disclosure_session.is_none());
        assert!(wallet.is_locked());

        // Verify no Disclosure events are logged
        assert!(wallet
            .storage
            .read()
            .await
            .fetch_wallet_events()
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state_no_session() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Accepting disclosure on a wallet without an active
        // disclosure session should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_none());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state_missing_attributes() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::MissingAttributes(Default::default()),
            ..Default::default()
        };
        wallet.disclosure_session = Some(disclosure_session.into());

        // Accepting disclosure on a wallet that has a disclosure session
        // with missing attributes should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());

        // The mdoc copy usage counts should not be incremented.
        assert!(wallet.storage.read().await.mdoc_copies_usage_counts.is_empty());

        // Verify no Disclosure events are logged
        assert!(wallet
            .storage
            .read()
            .await
            .fetch_wallet_events()
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_disclosure_session() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let response = DisclosureErrorResponse {
            error_response: ErrorResponse {
                error: PostAuthResponseErrorCode::InvalidRequest,
                error_description: None,
                error_uri: None,
            },
            redirect_uri: None,
        };
        let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                proposed_source_identifiers: vec![PROPOSED_ID],
                proposed_attributes,
                next_error: Mutex::new(Some(MdocDisclosureError::Vp(VpClientError::Request(
                    VpMessageClientError::AuthPostResponse(response),
                )))),
                ..Default::default()
            }),
            ..Default::default()
        };
        wallet.disclosure_session = Some(disclosure_session.into());

        // Accepting disclosure when the verifier responds with
        // an invalid request error should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::VpDisclosureSession(VpClientError::Request(VpMessageClientError::AuthPostResponse(_)))
        );
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().protocol_state.session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);
            }
            _ => unreachable!(),
        };

        // Test that the usage count got incremented for the proposed mdoc copy id.
        assert_eq!(wallet.storage.read().await.mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            wallet
                .storage
                .read()
                .await
                .mdoc_copies_usage_counts
                .get(&PROPOSED_ID)
                .copied()
                .unwrap_or_default(),
            1
        );

        // Get latest emitted recent_history events
        let events = events.lock().pop().unwrap();
        // Verify a Disclosure error event is logged, with no documents shared
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: DisclosureStatus::Error,
                attestations,
                ..
            } if attestations.is_empty()
        );

        // Set up the disclosure session to return a different error.
        match wallet.disclosure_session.as_ref().unwrap().protocol_state.session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                proposal
                    .next_error
                    .lock()
                    .replace(MdocDisclosureError::Vp(VpClientError::DeviceResponse(
                        mdoc::Error::Cose(CoseError::Signing(
                            RemoteEcdsaKeyError::KeyNotFound("foobar".to_string()).into(),
                        )),
                    )))
            }
            _ => unreachable!(),
        };

        // Accepting disclosure when the wallet provider responds that key with
        // a particular identifier is not present should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::VpDisclosureSession(VpClientError::DeviceResponse(mdoc::Error::Cose(CoseError::Signing(
                _
            ))))
        );
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().protocol_state.session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);
            }
            _ => unreachable!(),
        };

        // Test that the usage count got incremented again for the proposed mdoc copy id.
        let mdoc_copies_usage_counts = &wallet.storage.read().await.mdoc_copies_usage_counts;
        assert_eq!(mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
            2
        );

        // Verify another Disclosure error event is logged, with no documents shared
        let events = wallet.storage.read().await.fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 2);
        assert_matches!(
            &events[1],
            WalletEvent::Disclosure {
                status: DisclosureStatus::Error,
                attestations,
                ..
            } if attestations.is_empty()
        );
    }

    #[rstest]
    #[case(InstructionError::IncorrectPin{ attempts_left_in_round: 1, is_final_round: false }, false, false)]
    #[case(InstructionError::Timeout{ timeout_millis: 10_000 }, true, true)]
    #[case(InstructionError::Blocked, true, true)]
    #[case(InstructionError::InstructionValidation, false, true)]
    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_instruction(
        #[case] instruction_error: InstructionError,
        #[case] expect_termination: bool,
        #[case] expect_history_log: bool,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                proposed_source_identifiers: vec![PROPOSED_ID],
                proposed_attributes,
                next_error: Mutex::new(Some(MdocDisclosureError::Vp(VpClientError::DeviceResponse(
                    mdoc::Error::Cose(CoseError::Signing(
                        RemoteEcdsaKeyError::Instruction(instruction_error).into(),
                    )),
                )))),
                ..Default::default()
            }),
            ..Default::default()
        };
        wallet.disclosure_session = Some(disclosure_session.into());

        let was_terminated = Arc::clone(
            &wallet
                .disclosure_session
                .as_ref()
                .unwrap()
                .protocol_state
                .was_terminated,
        );
        assert!(!was_terminated.load(Ordering::Relaxed));

        // Accepting disclosure when the verifier responds with an `InstructionError` indicating
        // that the account is blocked should result in a `DisclosureError::Instruction` error.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::Instruction(_));

        if expect_termination {
            // If the disclosure session should be terminated, there
            // should be no session, the wallet should be locked and
            // the session should be terminated at the remote end.
            assert!(wallet.disclosure_session.is_none());
            assert!(wallet.is_locked());
            assert!(was_terminated.load(Ordering::Relaxed));
        } else {
            // Otherwise, the session should still be present, the wallet
            // unlocked and the session should not be terminated.
            assert!(wallet.disclosure_session.is_some());
            assert!(!wallet.is_locked());
            assert!(!was_terminated.load(Ordering::Relaxed));
        }

        // Test that the usage count got incremented for the proposed mdoc copy id.
        let mdoc_copies_usage_counts = &wallet.storage.read().await.mdoc_copies_usage_counts;
        assert_eq!(mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
            1
        );

        // Get latest emitted recent_history events
        let events = events.lock().pop().unwrap();

        match (expect_termination, expect_history_log) {
            (true, true) => {
                // Verify both a disclosure cancellation and error event are logged
                assert_eq!(events.len(), 2);
                assert_matches!(
                    &events[0],
                    WalletEvent::Disclosure {
                        status: DisclosureStatus::Cancelled,
                        ..
                    }
                );
                assert_matches!(
                    &events[1],
                    WalletEvent::Disclosure {
                        status: DisclosureStatus::Error,
                        attestations,
                        ..
                    } if attestations.is_empty()
                );
            }
            (false, true) => {
                // Verify a disclosure error event is logged
                assert_eq!(events.len(), 1);
                assert_matches!(
                    &events[0],
                    WalletEvent::Disclosure {
                        status: DisclosureStatus::Error,
                        attestations,
                        ..
                    } if attestations.is_empty()
                );
            }
            (false, false) => {
                assert_eq!(events.len(), 0);
            }
            (true, false) => {
                panic!(
                    "In practice this cannot happen, as the InstructionError cannot be both (Timeout or Blocked) and \
                     IncorrectPin"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_holder_attributes_are_shared() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        let proposed_attributes = setup_proposed_attributes(&[("age_over_18", DataElementValue::Bool(true))]);
        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                proposed_source_identifiers: vec![PROPOSED_ID],
                proposed_attributes,
                next_error: Mutex::new(Some(MdocDisclosureError::Vp(VpClientError::MissingReaderRegistration))),
                attributes_shared: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        let reader_certificate = disclosure_session.certificate.clone();

        wallet.disclosure_session = Some(disclosure_session.into());

        // Accepting disclosure when the verifier responds with an error indicating that
        // attributes were shared should result in a disclosure event being recorded.
        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::VpDisclosureSession(VpClientError::MissingReaderRegistration)
        );
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().protocol_state.session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0);
            }
            _ => unreachable!(),
        };

        // Test that the usage count got incremented for the proposed mdoc copy id.
        let mdoc_copies_usage_counts = &wallet.storage.read().await.mdoc_copies_usage_counts;
        assert_eq!(mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            mdoc_copies_usage_counts.get(&PROPOSED_ID).copied().unwrap_or_default(),
            1
        );

        // Get latest emitted recent_history events
        let events = events.lock().pop().unwrap();
        // Verify a Disclosure error event is logged, and documents are shared
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: DisclosureStatus::Error,
                attestations,
                ..
            } if !attestations.is_empty()
        );
        assert!(wallet
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(&reader_certificate)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_wrong_redirect_uri_purpose() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let disclosure_session = MockMdocDisclosureSession { ..Default::default() };
        wallet.disclosure_session = Some(WalletDisclosureSession::new(
            RedirectUriPurpose::Issuance,
            disclosure_session,
        ));

        let error = wallet
            .accept_disclosure(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::UnexpectedRedirectUriPurpose {
                expected: RedirectUriPurpose::Issuance,
                found: RedirectUriPurpose::Browser,
            }
        );
    }

    #[tokio::test]
    async fn test_mdoc_by_doc_types() {
        // Prepare a wallet in initial state.
        let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Create some fake `Mdoc` entries to place into wallet storage.
        let mdoc1 = Mdoc::new_mock().await;
        let mdoc2 = Mdoc::new_mock_with_doctype("com.example.doc_type").await;

        // Place 3 copies of each `Mdoc` into `MockStorage`.
        wallet
            .storage
            .write()
            .await
            .insert_mdocs(vec![
                vec![mdoc1.clone(), mdoc1.clone(), mdoc1.clone()].try_into().unwrap(),
                vec![mdoc2.clone(), mdoc2.clone(), mdoc2.clone()].try_into().unwrap(),
            ])
            .await
            .unwrap();

        // Call the `MdocDataSource.mdoc_by_doc_types()` method on the `Wallet`.
        let mdoc_by_doc_types = wallet
            .mdoc_by_doc_types(&["com.example.doc_type", PID].into())
            .await
            .expect("Could not get mdocs by doc types from wallet");

        // The result should be one copy of each distinct `Mdoc`,
        // while retaining the original insertion order.
        assert_eq!(mdoc_by_doc_types.len(), 2);
        assert_eq!(mdoc_by_doc_types[0].len(), 1);
        assert_eq!(mdoc_by_doc_types[1].len(), 1);

        assert_matches!(&mdoc_by_doc_types[0][0], StoredMdoc { mdoc, .. } if *mdoc == mdoc1);
        assert_matches!(&mdoc_by_doc_types[1][0], StoredMdoc { mdoc, .. } if *mdoc == mdoc2);

        let unique_ids = mdoc_by_doc_types
            .into_iter()
            .flat_map(|stored_mdocs| stored_mdocs.into_iter().map(|stored_mdoc| stored_mdoc.id))
            .unique()
            .collect::<Vec<_>>();

        assert_eq!(unique_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_mdoc_by_doc_types_empty() {
        // Prepare a wallet in initial state.
        let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Calling the `MdocDataSource.mdoc_by_doc_types()` method
        // on the `Wallet` should return an empty result.
        let mdoc_by_doc_types = wallet
            .mdoc_by_doc_types(&Default::default())
            .await
            .expect("Could not get mdocs by doc types from wallet");

        assert!(mdoc_by_doc_types.is_empty());
    }

    #[tokio::test]
    async fn test_mdoc_by_doc_types_error() {
        // Prepare a wallet in initial state.
        let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Set up `MockStorage` to return an error when performing a query.
        wallet.storage.write().await.has_query_error = true;

        // Calling the `MdocDataSource.mdoc_by_doc_types()` method
        // on the `Wallet` should forward the `StorageError`.
        let error = wallet
            .mdoc_by_doc_types(&Default::default())
            .await
            .expect_err("Getting mdocs by doc types from wallet should result in an error");

        assert_matches!(error, StorageError::Database(_));
    }
}
