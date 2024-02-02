use std::collections::HashSet;

use indexmap::IndexMap;
use platform_support::hw_keystore::PlatformEcdsaKey;
use tracing::{error, info, instrument};
use url::Url;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{MdocDataSource, ProposedAttributes, StoredMdoc},
    server_keys::KeysError,
    utils::{cose::CoseError, reader_auth::ReaderRegistration, x509::Certificate},
};

use crate::{
    account_provider::AccountProviderClient,
    config::ConfigurationRepository,
    disclosure::{
        DisclosureUriData, DisclosureUriError, MdocDisclosureMissingAttributes, MdocDisclosureProposal,
        MdocDisclosureSession, MdocDisclosureSessionState,
    },
    document::{DisclosureDocument, DocumentMdocError, MissingDisclosureAttributes},
    instruction::{InstructionClient, InstructionError, RemoteEcdsaKeyError, RemoteEcdsaKeyFactory},
    storage::{Storage, StorageError, StoredMdocCopy, WalletEvent},
    EventStatus,
};

use super::Wallet;

#[derive(Debug, Clone)]
pub struct DisclosureProposal {
    pub documents: Vec<DisclosureDocument>,
    pub reader_registration: ReaderRegistration,
    pub shared_data_with_relying_party_before: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum DisclosureError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("wallet is locked")]
    Locked,
    #[error("disclosure session is not in the correct state")]
    SessionState,
    #[error("could not parse disclosure URI: {0}")]
    DisclosureUri(#[source] DisclosureUriError),
    #[error("error in mdoc disclosure session: {0}")]
    DisclosureSession(#[source] nl_wallet_mdoc::Error),
    #[error("not all requested attributes are available, missing: {missing_attributes:?}")]
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        missing_attributes: Vec<MissingDisclosureAttributes>,
        shared_data_with_relying_party_before: bool,
    },
    #[error("could not interpret (missing) mdoc attributes: {0}")]
    MdocAttributes(#[source] DocumentMdocError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[source] InstructionError),
    #[error("could not increment usage count of mdoc copies in database: {0}")]
    IncrementUsageCount(#[source] StorageError),
    #[error("could not store history in database: {0}")]
    HistoryStorage(#[source] StorageError),
}

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    CR: ConfigurationRepository,
    MDS: MdocDisclosureSession<Self>,
    S: Storage,
{
    #[instrument(skip_all)]
    pub async fn start_disclosure(&mut self, uri: &Url) -> Result<DisclosureProposal, DisclosureError> {
        info!("Performing disclosure based on received URI: {}", uri);

        info!("Checking if registered");
        if self.registration.is_none() {
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

        let config = &self.config_repository.config().disclosure;

        // Assume that redirect URI creation is checked when updating the `Configuration`.
        let disclosure_redirect_uri_base = config.uri_base().unwrap();
        let disclosure_uri = DisclosureUriData::parse_from_uri(uri, &disclosure_redirect_uri_base)
            .map_err(DisclosureError::DisclosureUri)?;

        // Start the disclosure session based on the `ReaderEngagement`.
        let session = MDS::start(disclosure_uri, self, &config.rp_trust_anchors())
            .await
            .map_err(DisclosureError::DisclosureSession)?;

        let shared_data_with_relying_party_before = self
            .storage
            .read()
            .await
            .did_share_data_with_relying_party(session.rp_certificate())
            .await
            .map_err(DisclosureError::HistoryStorage)?;

        let proposal_session = match session.session_state() {
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session) => {
                // Translate the missing attributes into a `Vec<MissingDisclosureAttributes>`.
                // If this fails, return `DisclosureError::AttributeMdoc` instead.
                info!(
                    "At least one attribute is missing in order to satisfy the disclosure request, \
                    attempting to translate to MissingDisclosureAttributes"
                );

                let missing_attributes = missing_attr_session.missing_attributes().to_vec();
                let error = match MissingDisclosureAttributes::from_mdoc_missing_attributes(missing_attributes) {
                    Ok(attributes) => {
                        // If the missing attributes can be translated and shown to the user,
                        // store the session so that it will only be terminated on user interaction.
                        // This prevents gleaning of missing attributes by a verifier.
                        let reader_registration = session.reader_registration().clone().into();
                        self.disclosure_session.replace(session);

                        DisclosureError::AttributesNotAvailable {
                            reader_registration,
                            missing_attributes: attributes,
                            shared_data_with_relying_party_before,
                        }
                    }
                    // TODO: What to do when the missing attributes could not be translated?
                    //       In that case there is no way we can terminate the session with
                    //       user interaction, since the missing attributes cannot be presented.
                    Err(error) => DisclosureError::MdocAttributes(error),
                };

                return Err(error);
            }
            MdocDisclosureSessionState::Proposal(proposal_session) => proposal_session,
        };

        info!("All attributes in the disclosure request are present in the database, return a proposal to the user");

        // Prepare a `Vec<ProposedDisclosureDocument>` to report to the caller.
        let documents = proposal_session
            .proposed_attributes()
            .into_iter()
            .map(|(doc_type, attributes)| DisclosureDocument::from_mdoc_attributes(&doc_type, attributes))
            .collect::<Result<_, _>>()
            .map_err(DisclosureError::MdocAttributes)?;

        // Place this in a `DisclosureProposal`, along with a copy of the `ReaderRegistration`.
        let proposal = DisclosureProposal {
            documents,
            reader_registration: session.reader_registration().clone(),
            shared_data_with_relying_party_before,
        };

        // Retain the session as `Wallet` state.
        self.disclosure_session.replace(session);

        Ok(proposal)
    }

    async fn terminate_disclosure_session(&mut self, session: MDS) -> Result<(), DisclosureError> {
        // Prepare history events from session before terminating session
        let event = WalletEvent::new_disclosure(None, session.rp_certificate().clone(), EventStatus::Cancelled);

        session.terminate().await.map_err(DisclosureError::DisclosureSession)?;

        self.store_history_event(event)
            .await
            .map_err(DisclosureError::HistoryStorage)?;

        Ok(())
    }

    pub async fn cancel_disclosure(&mut self) -> Result<(), DisclosureError> {
        info!("Cancelling disclosure");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(DisclosureError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DisclosureError::Locked);
        }

        info!("Checking if a disclosure session is present");
        let session = self.disclosure_session.take().ok_or(DisclosureError::SessionState)?;

        self.terminate_disclosure_session(session).await
    }

    async fn log_empty_disclosure_error(&mut self, remote_party_certificate: Certificate, message: String) {
        let event = WalletEvent::new_disclosure(None, remote_party_certificate, EventStatus::Error(message));
        let _ = self.store_history_event(event).await.map_err(|e| {
            error!("Could not store error in history: {e}");
            e
        });
    }

    async fn log_disclosure_error(
        &mut self,
        session_proposal: Option<ProposedAttributes>,
        remote_party_certificate: Certificate,
        message: String,
    ) {
        let event = WalletEvent::new_disclosure(
            session_proposal.map(Into::into),
            remote_party_certificate,
            EventStatus::Error(message),
        );
        let _ = self.store_history_event(event).await.map_err(|e| {
            error!("Could not store error in history: {e}");
            e
        });
    }

    pub async fn accept_disclosure(&mut self, pin: String) -> Result<Option<Url>, DisclosureError>
    where
        S: Storage,
        PEK: PlatformEcdsaKey,
        APC: AccountProviderClient,
    {
        info!("Accepting disclosure");

        info!("Checking if registered");
        let registration_data = self
            .registration
            .as_ref()
            .ok_or_else(|| DisclosureError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DisclosureError::Locked);
        }

        info!("Checking if a disclosure session is present");
        let session = self.disclosure_session.as_ref().ok_or(DisclosureError::SessionState)?;

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

        if let Err(error) = self
            .storage
            .get_mut()
            .increment_mdoc_copies_usage_count(session_proposal.proposed_source_identifiers())
            .await
        {
            self.log_empty_disclosure_error(
                session.rp_certificate().clone(),
                "Failed to register shared mdoc copy".to_string(),
            )
            .await;
            return Err(DisclosureError::IncrementUsageCount(error));
        }

        // Prepare the `RemoteEcdsaKeyFactory` for signing using the provided PIN.
        let config = self.config_repository.config();

        let instruction_result_public_key = config.account_server.instruction_result_public_key.clone().into();
        let remote_instruction = InstructionClient::new(
            pin,
            &self.storage,
            &self.hw_privkey,
            &self.account_provider_client,
            registration_data,
            &config.account_server.base_url,
            &instruction_result_public_key,
        );
        let remote_key_factory = RemoteEcdsaKeyFactory::new(&remote_instruction);

        // Actually perform disclosure, casting any `InstructionError` that
        // occur during signing to `RemoteEcdsaKeyError::Instruction`.
        if let Err(error) = session_proposal.disclose(&&remote_key_factory).await {
            self.log_disclosure_error(
                error.data_shared.then(|| session_proposal.proposed_attributes()),
                session.rp_certificate().clone(),
                "Error occurred while disclosing attributes".to_owned(),
            )
            .await;

            let error = match error.error {
                nl_wallet_mdoc::Error::Cose(CoseError::Signing(error)) if error.is::<RemoteEcdsaKeyError>() => {
                    // This `unwrap()` is safe because of the `is()` check above.
                    match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                        RemoteEcdsaKeyError::Instruction(error) => DisclosureError::Instruction(error),
                        error => DisclosureError::DisclosureSession(nl_wallet_mdoc::Error::KeysError(
                            KeysError::KeyGeneration(error.into()),
                        )),
                    }
                }
                _ => DisclosureError::DisclosureSession(error.error),
            };

            if matches!(
                error,
                DisclosureError::Instruction(InstructionError::Timeout { .. } | InstructionError::Blocked)
            ) {
                // On a PIN timeout we should proactively terminate the disclosure session,
                // and lock the wallet, as the user is probably not the owner of the wallet.
                // The UI should catch this specific error and close the disclosure screens.

                let session = self.disclosure_session.take().unwrap();
                if let Err(terminate_error) = self.terminate_disclosure_session(session).await {
                    // Log the error, but do not return it from this method.
                    error!(
                        "Error while terminating disclosure session on PIN timeout: {}",
                        terminate_error
                    );
                }

                self.lock();
            }

            return Err(error);
        }

        // Clone the return URL if present, so we can return it from this method.
        let return_url = session_proposal.return_url().cloned();

        // Save data for disclosure in event log.
        let event = WalletEvent::new_disclosure(
            Some(session_proposal.proposed_attributes().into()),
            session.rp_certificate().clone(),
            EventStatus::Success,
        );
        self.store_history_event(event)
            .await
            .map_err(DisclosureError::HistoryStorage)?;

        // When disclosure is successful, we can remove the session.
        self.disclosure_session.take();

        Ok(return_url)
    }
}

impl<CR, S, PEK, APC, DGS, PIC, MDS> MdocDataSource for Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
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
                        .get(mdoc.doc_type.as_str())
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
    use std::sync::{atomic::Ordering, Arc, Mutex};

    use assert_matches::assert_matches;
    use itertools::Itertools;
    use mockall::predicate::*;
    use rstest::rstest;
    use serial_test::serial;

    use nl_wallet_mdoc::{
        basic_sa_ext::Entry,
        holder::{HolderError, Mdoc, ProposedDocumentAttributes},
        iso::disclosure::SessionStatus,
        verifier::SessionType,
        DataElementValue,
    };
    use uuid::uuid;

    use crate::{
        disclosure::{MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal, MockMdocDisclosureSession},
        wallet::test::ISSUER_KEY,
        Attribute, AttributeValue, EventStatus,
    };

    use super::{super::test::WalletWithMocks, *};

    const DISCLOSURE_URI: &str =
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com&session_type=same_device";
    const PROPOSED_ID: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let reader_registration = ReaderRegistration::new_mock();
        let proposed_attributes = IndexMap::from([(
            "com.example.pid".to_string(),
            ProposedDocumentAttributes {
                attributes: IndexMap::from([(
                    "com.example.pid".to_string(),
                    vec![Entry {
                        name: "age_over_18".to_string(),
                        value: DataElementValue::Bool(true),
                    }],
                )]),
                issuer: ISSUER_KEY.issuance_key.certificate().clone(),
            },
        )]);
        let proposal_session = MockMdocDisclosureProposal {
            proposed_source_identifiers: vec![PROPOSED_ID],
            proposed_attributes,
            ..Default::default()
        };

        MockMdocDisclosureSession::next_fields(
            reader_registration,
            MdocDisclosureSessionState::Proposal(proposal_session),
        );

        // Starting disclosure should not fail.
        let proposal = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect("Could not start disclosure");

        // Test that the `Wallet` now contains a `DisclosureSession`
        // with the items parsed from the disclosure URI.
        assert_matches!(wallet.disclosure_session, Some(session) if session.disclosure_uri == DisclosureUriData {
            reader_engagement_bytes: b"foobar".to_vec(),
            return_url: Some(Url::parse("https://example.com").unwrap()),
            session_type: SessionType::SameDevice,
        });

        // Test that the returned `DisclosureProposal` contains the
        // `ReaderRegistration` we set up earlier, as well as the
        // proposed attributes converted to a `ProposedDisclosureDocument`.
        assert_eq!(proposal.documents.len(), 1);
        let document = proposal.documents.first().unwrap();
        assert_eq!(document.doc_type, "com.example.pid");
        assert_eq!(document.attributes.len(), 1);
        assert_matches!(
            document.attributes.first().unwrap(),
            (
                &"age_over_18",
                Attribute {
                    key_labels: _,
                    value: AttributeValue::Boolean(true)
                }
            )
        );

        // Starting disclosure should not cause mdoc copy usage counts to be incremented.
        assert!(wallet.storage.get_mut().mdoc_copies_usage_counts.is_empty());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Starting disclosure on a locked wallet should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::Locked);
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Starting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_session_state() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Start an active disclosure session.
        wallet.disclosure_session = MockMdocDisclosureSession::default().into();

        // Starting disclosure on a wallet with an active disclosure should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_some());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_disclosure_uri() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Starting disclosure on a wallet with a malformed disclosure URI should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse("http://example.com").unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureUri(_));
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_disclosure_session() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` start to return the following error.
        MockMdocDisclosureSession::next_start_error(HolderError::NoAttributesRequested.into());

        // Starting disclosure with a malformed disclosure URI should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureSession(_));
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_attributes_not_available() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec!["com.example.pid/com.example.pid/age_over_18".parse().unwrap()];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session),
        );

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::AttributesNotAvailable {
                reader_registration: _,
                missing_attributes,
                shared_data_with_relying_party_before,
            } if !shared_data_with_relying_party_before && missing_attributes[0].doc_type == "com.example.pid" &&
                 *missing_attributes[0].attributes.first().unwrap().0 == "age_over_18"
        );
        assert!(wallet.disclosure_session.is_some());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_mdoc_attributes_not_available() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec!["com.example.pid/com.example.pid/foobar".parse().unwrap()];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session),
        );

        // Starting disclosure where an attribute that is both unavailable
        // and unknown is requested should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::MdocAttributes(DocumentMdocError::UnknownAttribute {
                doc_type,
                name_space,
                name,
                value: None,
            }) if doc_type == "com.example.pid" && name_space == "com.example.pid" && name == "foobar"
        );
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_mdoc_attributes() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let proposed_attributes = IndexMap::from([(
            "com.example.pid".to_string(),
            ProposedDocumentAttributes {
                attributes: IndexMap::from([(
                    "com.example.pid".to_string(),
                    vec![Entry {
                        name: "foo".to_string(),
                        value: DataElementValue::Text("bar".to_string()),
                    }],
                )]),
                issuer: ISSUER_KEY.issuance_key.certificate().clone(),
            },
        )]);
        let proposal_session = MockMdocDisclosureProposal {
            proposed_attributes,
            ..Default::default()
        };

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::Proposal(proposal_session),
        );

        // Starting disclosure where unknown attributes are requested should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::MdocAttributes(DocumentMdocError::UnknownAttribute {
                doc_type,
                name_space,
                name,
                value: Some(DataElementValue::Text(value)),
            }) if doc_type == "com.example.pid" && name_space == "com.example.pid" && name == "foo" && value == "bar"
        );
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let reader_registration = ReaderRegistration::new_mock();
        let proposed_attributes = IndexMap::from([(
            "com.example.pid".to_string(),
            ProposedDocumentAttributes {
                attributes: IndexMap::from([(
                    "com.example.pid".to_string(),
                    vec![Entry {
                        name: "age_over_18".to_string(),
                        value: DataElementValue::Bool(true),
                    }],
                )]),
                issuer: ISSUER_KEY.issuance_key.certificate().clone(),
            },
        )]);
        let proposal_session = MockMdocDisclosureProposal {
            proposed_source_identifiers: vec![PROPOSED_ID],
            proposed_attributes,
            ..Default::default()
        };

        MockMdocDisclosureSession::next_fields(
            reader_registration,
            MdocDisclosureSessionState::Proposal(proposal_session),
        );

        // Start a disclosure session, to ensure a proper session exists that can be cancelled.
        let _ = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect("Could not start disclosure");

        // Verify disclosure session is not yet terminated
        let was_terminated = Arc::clone(&wallet.disclosure_session.as_ref().unwrap().was_terminated);
        assert!(!was_terminated.load(Ordering::Relaxed));

        // Verify no history events are yet logged
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert!(events.is_empty());

        // Cancelling disclosure should result in a `Wallet` without a disclosure
        // session, while the session that was there should be terminated.
        wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

        // Verify disclosure session is terminated
        assert!(wallet.disclosure_session.is_none());
        assert!(was_terminated.load(Ordering::Relaxed));

        // Verify a Disclosure Cancel event is logged
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: EventStatus::Cancelled,
                ..
            }
        );

        // Cancelling disclosure should not cause mdoc copy usage counts to be incremented.
        assert!(wallet.storage.get_mut().mdoc_copies_usage_counts.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_cancel_disclosure_missing_attributes() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec![
            "com.example.pid/com.example.pid/bsn".parse().unwrap(),
            "com.example.pid/com.example.pid/age_over_18".parse().unwrap(),
        ];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        MockMdocDisclosureSession::next_fields(
            ReaderRegistration::new_mock(),
            MdocDisclosureSessionState::MissingAttributes(missing_attr_session),
        );

        // Starting disclosure where an unavailable attribute is requested should result in an error.
        // As an exception, this error should leave the `Wallet` with an active disclosure session.
        let _error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");
        assert!(wallet.disclosure_session.is_some());

        // Verify disclosure session is not yet terminated
        let was_terminated = Arc::clone(&wallet.disclosure_session.as_ref().unwrap().was_terminated);
        assert!(!was_terminated.load(Ordering::Relaxed));

        // Verify no history events are yet logged
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert!(events.is_empty());

        // Cancelling disclosure should result in a `Wallet` without a disclosure
        // session, while the session that was there should be terminated.
        wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

        // Verify disclosure session is terminated
        assert!(wallet.disclosure_session.is_none());
        assert!(was_terminated.load(Ordering::Relaxed));

        // Verify a single Disclosure Error event is logged
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: EventStatus::Cancelled,
                ..
            }
        );
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_locked() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.disclosure_session = MockMdocDisclosureSession::default().into();

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
        let mut wallet = WalletWithMocks::new_unregistered().await;

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
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

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
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let return_url = Url::parse("https://example.com/return/here").unwrap();

        let proposed_attributes = IndexMap::from([(
            "com.example.pid".to_string(),
            ProposedDocumentAttributes {
                attributes: IndexMap::from([(
                    "com.example.pid".to_string(),
                    vec![Entry {
                        name: "age_over_18".to_string(),
                        value: DataElementValue::Bool(true),
                    }],
                )]),
                issuer: ISSUER_KEY.issuance_key.certificate().clone(),
            },
        )]);
        let disclosure_session = MockMdocDisclosureProposal {
            return_url: return_url.clone().into(),
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

        wallet.disclosure_session = disclosure_session.into();

        // Accepting disclosure should succeed and give us the return URL.
        let accept_result = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect("Could not accept disclosure");

        assert_matches!(accept_result, Some(result_return_url) if result_return_url == return_url);

        // Check that the disclosure session is no longer
        // present and that the disclosure count is 1.
        assert!(wallet.disclosure_session.is_none());
        assert!(!wallet.is_locked());
        assert_eq!(disclosure_count.load(Ordering::Relaxed), 1);

        // Verify a single Disclosure Success event is logged, and documents are shared
        // Also verify that `did_share_data_with_relying_party()` now returns `true`
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: EventStatus::Success,
                documents: Some(_),
                reader_certificate,
                ..
            } if wallet.storage.read().await.did_share_data_with_relying_party(reader_certificate).await.unwrap()
        );

        // Test that the usage count got incremented for the proposed mdoc copy id.
        assert_eq!(wallet.storage.get_mut().mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            wallet
                .storage
                .get_mut()
                .mdoc_copies_usage_counts
                .get(&PROPOSED_ID)
                .copied()
                .unwrap_or_default(),
            1
        );
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_locked() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.disclosure_session = MockMdocDisclosureSession::default().into();

        wallet.lock();

        // Accepting disclosure on a locked wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::Locked);
        assert!(wallet.disclosure_session.is_some());
        assert!(wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0)
            }
            _ => unreachable!(),
        };

        // The mdoc copy usage counts should not be incremented.
        assert!(wallet.storage.get_mut().mdoc_copies_usage_counts.is_empty());

        // Verify no Disclosure events are logged
        assert!(wallet.storage.get_mut().fetch_wallet_events().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Accepting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(wallet.disclosure_session.is_none());
        assert!(wallet.is_locked());

        // Verify no Disclosure events are logged
        assert!(wallet.storage.get_mut().fetch_wallet_events().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Accepting disclosure on a wallet without an active
        // disclosure session should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_none());
        assert!(!wallet.is_locked());

        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::MissingAttributes(Default::default()),
            ..Default::default()
        };
        wallet.disclosure_session = disclosure_session.into();

        // Accepting disclosure on a wallet that has a disclosure session
        // with missing attributes should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());

        // The mdoc copy usage counts should not be incremented.
        assert!(wallet.storage.get_mut().mdoc_copies_usage_counts.is_empty());

        // Verify no Disclosure events are logged
        assert!(wallet.storage.get_mut().fetch_wallet_events().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_disclosure_session() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                proposed_source_identifiers: vec![PROPOSED_ID],
                next_error: Mutex::new(
                    nl_wallet_mdoc::Error::Holder(HolderError::DisclosureResponse(SessionStatus::DecodingError)).into(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };
        wallet.disclosure_session = disclosure_session.into();

        // Accepting disclosure when the verifier responds with
        // a decoding error should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureSession(_));
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0)
            }
            _ => unreachable!(),
        };

        // Test that the usage count got incremented for the proposed mdoc copy id.
        assert_eq!(wallet.storage.get_mut().mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            wallet
                .storage
                .get_mut()
                .mdoc_copies_usage_counts
                .get(&PROPOSED_ID)
                .copied()
                .unwrap_or_default(),
            1
        );

        // Verify a Disclosure error event is logged, with no documents shared
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure { status: EventStatus::Error(error), documents: None, .. }
            if error == "Error occurred while disclosing attributes"
        );

        // Set up the disclosure session to return a different error.
        match wallet.disclosure_session.as_ref().unwrap().session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                proposal
                    .next_error
                    .lock()
                    .unwrap()
                    .replace(nl_wallet_mdoc::Error::Cose(CoseError::Signing(
                        RemoteEcdsaKeyError::KeyNotFound("foobar".to_string()).into(),
                    )))
            }
            _ => unreachable!(),
        };

        // Accepting disclosure when the Wallet Provider responds that key with
        // a particular identifier is not present should result in an error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureSession(_));
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0)
            }
            _ => unreachable!(),
        };

        // Test that the usage count got incremented again for the proposed mdoc copy id.
        assert_eq!(wallet.storage.get_mut().mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            wallet
                .storage
                .get_mut()
                .mdoc_copies_usage_counts
                .get(&PROPOSED_ID)
                .copied()
                .unwrap_or_default(),
            2
        );

        // Verify another Disclosure error event is logged, with no documents shared
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 2);
        assert_matches!(
            &events[1],
            WalletEvent::Disclosure { status: EventStatus::Error(error), documents: None, .. }
            if error == "Error occurred while disclosing attributes"
        );
    }

    #[rstest]
    #[case(InstructionError::IncorrectPin { leftover_attempts: 1, is_final_attempt: false }, false)]
    #[case(InstructionError::Timeout { timeout_millis: 10_000 }, true)]
    #[case(InstructionError::Blocked, true)]
    #[case(InstructionError::InstructionValidation, false)]
    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_instruction(
        #[case] instruction_error: InstructionError,
        #[case] expect_termination: bool,
    ) {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                proposed_source_identifiers: vec![PROPOSED_ID],
                next_error: Mutex::new(
                    nl_wallet_mdoc::Error::Cose(CoseError::Signing(
                        RemoteEcdsaKeyError::Instruction(instruction_error).into(),
                    ))
                    .into(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };
        wallet.disclosure_session = disclosure_session.into();

        let was_terminated = Arc::clone(&wallet.disclosure_session.as_ref().unwrap().was_terminated);
        assert!(!was_terminated.load(Ordering::Relaxed));

        // Accepting disclosure when the Wallet Provider responds with an `InstructionError` indicating
        // that the account is blocked should result in a `DisclosureError::Instruction` error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
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
        assert_eq!(wallet.storage.get_mut().mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            wallet
                .storage
                .get_mut()
                .mdoc_copies_usage_counts
                .get(&PROPOSED_ID)
                .copied()
                .unwrap_or_default(),
            1
        );

        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();

        if expect_termination {
            // Verify both a disclosure cancellation and error event are logged
            assert_eq!(events.len(), 2);
            assert_matches!(
                &events[0],
                WalletEvent::Disclosure {
                    status: EventStatus::Cancelled,
                    ..
                }
            );
            assert_matches!(
                &events[1],
                WalletEvent::Disclosure { status: EventStatus::Error(error), documents: None, .. }
                if error == "Error occurred while disclosing attributes"
            );
        } else {
            // Verify a disclosure error event is logged
            assert_eq!(events.len(), 1);
            assert_matches!(
                &events[0],
                WalletEvent::Disclosure { status: EventStatus::Error(error), documents: None, .. }
                if error == "Error occurred while disclosing attributes"
            );
        }
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_error_holder_attributes_are_shared() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let disclosure_session = MockMdocDisclosureSession {
            session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                proposed_source_identifiers: vec![PROPOSED_ID],
                next_error: Mutex::new(nl_wallet_mdoc::Error::Holder(HolderError::ReaderAuthMissing).into()),
                attributes_shared: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        wallet.disclosure_session = disclosure_session.into();

        // Accepting disclosure when the Wallet Provider responds with an `InstructionError` indicating
        // that the account is blocked should result in a `DisclosureError::Instruction` error.
        let error = wallet
            .accept_disclosure(PIN.to_string())
            .await
            .expect_err("Accepting disclosure should have resulted in an error");

        assert_matches!(
            error,
            DisclosureError::DisclosureSession(nl_wallet_mdoc::Error::Holder(HolderError::ReaderAuthMissing))
        );
        assert!(wallet.disclosure_session.is_some());
        assert!(!wallet.is_locked());
        match wallet.disclosure_session.as_ref().unwrap().session_state {
            MdocDisclosureSessionState::Proposal(ref proposal) => {
                assert_eq!(proposal.disclosure_count.load(Ordering::Relaxed), 0)
            }
            _ => unreachable!(),
        };

        // Test that the usage count got incremented for the proposed mdoc copy id.
        assert_eq!(wallet.storage.get_mut().mdoc_copies_usage_counts.len(), 1);
        assert_eq!(
            wallet
                .storage
                .get_mut()
                .mdoc_copies_usage_counts
                .get(&PROPOSED_ID)
                .copied()
                .unwrap_or_default(),
            1
        );

        // Verify a Disclosure error event is logged, and documents are shared
        let events = wallet.storage.get_mut().fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(
            &events[0],
            WalletEvent::Disclosure {
                status: EventStatus::Error(error),
                documents: Some(_),
                reader_certificate,
                ..
            } if error == "Error occurred while disclosing attributes" &&
                wallet.storage.read().await.did_share_data_with_relying_party(reader_certificate).await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_mdoc_by_doc_types() {
        // Prepare a wallet in initial state.
        let wallet = WalletWithMocks::new_unregistered().await;

        // Create some fake `Mdoc` entries to place into wallet storage.
        let mdoc1 = Mdoc::new_example_mock();
        let mdoc2 = {
            let mut mdoc2 = mdoc1.clone();

            mdoc2.doc_type = "com.example.doc_type".to_string();

            mdoc2
        };

        // Place 3 copies of each `Mdoc` into `MockStorage`.
        wallet
            .storage
            .write()
            .await
            .insert_mdocs(vec![
                vec![mdoc1.clone(), mdoc1.clone(), mdoc1.clone()].into(),
                vec![mdoc2.clone(), mdoc2.clone(), mdoc2.clone()].into(),
            ])
            .await
            .unwrap();

        // Call the `MdocDataSource.mdoc_by_doc_types()` method on the `Wallet`.
        let mdoc_by_doc_types = wallet
            .mdoc_by_doc_types(&["com.example.doc_type", "org.iso.18013.5.1.mDL"].into())
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
        let wallet = WalletWithMocks::new_unregistered().await;

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
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Set up `MockStorage` to return an error when performing a query.
        wallet.storage.get_mut().has_query_error = true;

        // Calling the `MdocDataSource.mdoc_by_doc_types()` method
        // on the `Wallet` should forward the `StorageError`.
        let error = wallet
            .mdoc_by_doc_types(&Default::default())
            .await
            .expect_err("Getting mdocs by doc types from wallet should result in an error");

        assert_matches!(error, StorageError::Database(_));
    }
}
