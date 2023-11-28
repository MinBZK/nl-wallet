use std::collections::HashSet;

use async_trait::async_trait;
use indexmap::IndexMap;
use tracing::{info, instrument};
use url::Url;

use nl_wallet_mdoc::{
    holder::{Mdoc, MdocDataSource},
    utils::reader_auth::ReaderRegistration,
};

use crate::{
    config::ConfigurationRepository,
    disclosure::{
        DisclosureUriData, DisclosureUriError, MdocDisclosureMissingAttributes, MdocDisclosureProposal,
        MdocDisclosureSession, MdocDisclosureSessionState,
    },
    document::{DocumentMdocError, MissingDisclosureAttributes, ProposedDisclosureDocument},
    storage::{Storage, StorageError},
};

use super::Wallet;

#[derive(Debug, Clone)]
pub struct DisclosureProposal {
    pub documents: Vec<ProposedDisclosureDocument>,
    pub reader_registration: ReaderRegistration,
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
    DisclosureUri(#[from] DisclosureUriError),
    #[error("error in mdoc disclosure session: {0}")]
    DisclosureSession(#[from] nl_wallet_mdoc::Error),
    #[error("not all requested attributes are available, missing: {missing_attributes:?}")]
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        missing_attributes: Vec<MissingDisclosureAttributes>,
    },
    #[error("could not interpret (missing) mdoc attributes: {0}")]
    MdocAttributes(#[from] DocumentMdocError),
}

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    CR: ConfigurationRepository,
    MDS: MdocDisclosureSession<Self>,
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
        let disclosure_uri = DisclosureUriData::parse_from_uri(uri, &disclosure_redirect_uri_base)?;

        // Start the disclosure session based on the `ReaderEngagement`.
        let session = MDS::start(disclosure_uri, self, &config.rp_trust_anchors()).await?;

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
                        }
                    }
                    // TODO: What to do when the missing attributes could not be translated?
                    //       In that case there is no way we can terminate the session with
                    //       user interaction, since the missing attributes cannot be presented.
                    Err(error) => error.into(),
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
            .map(|(doc_type, attributes)| ProposedDisclosureDocument::from_mdoc_attributes(&doc_type, attributes))
            .collect::<Result<_, _>>()?;

        // Place this in a `DisclosureProposal`, along with a copy of the `ReaderRegistration`.
        let proposal = DisclosureProposal {
            documents,
            reader_registration: session.reader_registration().clone(),
        };

        // Retain the session as `Wallet` state.
        self.disclosure_session.replace(session);

        Ok(proposal)
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

        session.terminate().await?;

        Ok(())
    }

    // TODO: Implement disclosure method.
}

#[async_trait]
impl<CR, S, PEK, APC, DGS, PIC, MDS> MdocDataSource for Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    CR: Send + Sync,
    S: Storage + Send + Sync,
    PEK: Send + Sync,
    APC: Send + Sync,
    DGS: Send + Sync,
    PIC: Send + Sync,
    MDS: Send + Sync,
{
    type Error = StorageError;

    async fn mdoc_by_doc_types(&self, doc_types: &HashSet<&str>) -> std::result::Result<Vec<Vec<Mdoc>>, Self::Error> {
        // TODO: Retain UUIDs and increment use count on mdoc_copy when disclosure takes place.

        // Build an `IndexMap<>` to group `Mdoc` entries with the same `doc_type`.
        let mdocs_by_doc_type = self
            .storage
            .read()
            .await
            .fetch_unique_mdocs_by_doctypes(doc_types)
            .await?
            .into_iter()
            .fold(
                IndexMap::<_, Vec<_>>::with_capacity(doc_types.len()),
                |mut mdocs_by_doc_type, (_, mdoc)| {
                    // Re-use the `doc_types` string slices, which should contain all `Mdoc` doc types.
                    let doc_type = *doc_types
                        .get(mdoc.doc_type.as_str())
                        .expect("Storage returned mdoc with unexpected doc_type");
                    mdocs_by_doc_type.entry(doc_type).or_default().push(mdoc);

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
    use std::sync::{atomic::Ordering, Arc};

    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use serial_test::serial;

    use nl_wallet_mdoc::{
        basic_sa_ext::Entry, examples::Examples, holder::HolderError, mock as mdoc_mock, verifier::SessionType,
        DataElementValue,
    };

    use crate::{
        disclosure::{MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal, MockMdocDisclosureSession},
        Attribute, AttributeValue,
    };

    use super::{super::tests::WalletWithMocks, *};

    const DISCLOSURE_URI: &str =
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com&session_type=same_device";

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let reader_registration = ReaderRegistration {
            id: "1234".to_string(),
            ..Default::default()
        };
        let proposed_attributes = IndexMap::from([(
            "com.example.pid".to_string(),
            IndexMap::from([(
                "com.example.pid".to_string(),
                vec![Entry {
                    name: "age_over_18".to_string(),
                    value: DataElementValue::Bool(true),
                }],
            )]),
        )]);
        let mut proposal_session = MockMdocDisclosureProposal::default();
        proposal_session
            .expect_proposed_attributes()
            .return_const(proposed_attributes);

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
        assert_eq!(proposal.reader_registration.id, "1234");
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
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::registered().await;

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
        let mut wallet = WalletWithMocks::default();

        // Starting disclosure on an unregistered wallet should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::NotRegistered);
        assert!(wallet.digid_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_session_state() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::registered().await;

        wallet.disclosure_session = MockMdocDisclosureSession::default().into();

        // Starting disclosure on a wallet with an active disclosure should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.digid_session.is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_disclosure_uri() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Starting disclosure on a wallet with a malformed disclosure URI should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse("http://example.com").unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureUri(_));
        assert!(wallet.digid_session.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_disclosure_session() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

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
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec!["com.example.pid/com.example.pid/age_over_18".parse().unwrap()];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        MockMdocDisclosureSession::next_fields(
            Default::default(),
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
                missing_attributes
            } if missing_attributes[0].doc_type == "com.example.pid" &&
                 *missing_attributes[0].attributes.first().unwrap().0 == "age_over_18"
        );
        assert!(wallet.disclosure_session.is_some());
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_mdoc_attributes_not_available() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` start to return that attributes are not available.
        let missing_attributes = vec!["com.example.pid/com.example.pid/foobar".parse().unwrap()];
        let mut missing_attr_session = MockMdocDisclosureMissingAttributes::default();
        missing_attr_session
            .expect_missing_attributes()
            .return_const(missing_attributes);

        MockMdocDisclosureSession::next_fields(
            Default::default(),
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
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` to be returned with the following values.
        let proposed_attributes = IndexMap::from([(
            "com.example.pid".to_string(),
            IndexMap::from([(
                "com.example.pid".to_string(),
                vec![Entry {
                    name: "foo".to_string(),
                    value: DataElementValue::Text("bar".to_string()),
                }],
            )]),
        )]);
        let mut proposal_session = MockMdocDisclosureProposal::default();
        proposal_session
            .expect_proposed_attributes()
            .return_const(proposed_attributes);

        MockMdocDisclosureSession::next_fields(
            Default::default(),
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
    #[serial]
    async fn test_wallet_cancel_disclosure() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::registered().await;

        // Create a `MockMdocDisclosureSession` and store it on the `Wallet`.
        // Save and check its `was_terminated` value.
        let disclosure_session = MockMdocDisclosureSession::default();
        let was_terminated = Arc::clone(&disclosure_session.was_terminated);

        assert!(!was_terminated.load(Ordering::Relaxed));

        // Cancelling disclosure should result in a `Wallet` without a disclosure
        // session, while the session that was there should be terminated.
        wallet.disclosure_session = disclosure_session.into();
        wallet.cancel_disclosure().await.expect("Could not cancel disclosure");

        assert!(wallet.disclosure_session.is_none());
        assert!(was_terminated.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_wallet_cancel_disclosure_error_locked() {
        // Prepare a registered and locked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::registered().await;

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
        let mut wallet = WalletWithMocks::default();

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
        let mut wallet = WalletWithMocks::registered().await;

        // Cancelling disclosure on a wallet without an active
        // disclosure session should result in an error.
        let error = wallet
            .cancel_disclosure()
            .await
            .expect_err("Cancelling disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
        assert!(wallet.disclosure_session.is_none());
    }

    #[tokio::test]
    async fn test_mdoc_by_doc_types() {
        // Prepare a wallet in initial state.
        let wallet = WalletWithMocks::default();

        // Create some fake `Mdoc` entries to place into wallet storage.
        let trust_anchors = Examples::iaca_trust_anchors();
        let mdoc1 = mdoc_mock::mdoc_from_example_device_response(trust_anchors);
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
        assert_eq!(mdoc_by_doc_types, vec![vec![mdoc1], vec![mdoc2]]);
    }

    #[tokio::test]
    async fn test_mdoc_by_doc_types_empty() {
        // Prepare a wallet in initial state.
        let wallet = WalletWithMocks::default();

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
        let wallet = WalletWithMocks::default();

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
