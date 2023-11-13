use std::collections::HashSet;

use async_trait::async_trait;
use indexmap::IndexMap;
use tracing::{info, instrument};
use url::Url;

use nl_wallet_mdoc::{
    holder::{HolderError, Mdoc, MdocDataSource},
    utils::reader_auth::ReaderRegistration,
};

use crate::{
    config::ConfigurationRepository,
    disclosure::{DisclosureUriData, DisclosureUriError, MdocDisclosureSession},
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
    DisclosureSession(#[source] nl_wallet_mdoc::Error),
    #[error("not all requested attributes are available, missing: {missing_attributes:?}")]
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        missing_attributes: Vec<MissingDisclosureAttributes>,
    },
    #[error("could not interpret (missing) mdoc attributes: {0}")]
    MdocAttributes(#[from] DocumentMdocError),
}

// Promote an `AttributesNotAvailable` error to a top-level error.
impl From<nl_wallet_mdoc::Error> for DisclosureError {
    fn from(value: nl_wallet_mdoc::Error) -> Self {
        match value {
            nl_wallet_mdoc::Error::Holder(HolderError::AttributesNotAvailable {
                reader_registration,
                missing_attributes,
            }) => {
                // Translate the missing attributes into a `Vec<MissingDisclosureAttributes>`.
                // If this fails, return `DisclosureError::AttributeMdoc` instead.
                match MissingDisclosureAttributes::from_mdoc_missing_attributes(missing_attributes) {
                    Ok(attributes) => DisclosureError::AttributesNotAvailable {
                        reader_registration,
                        missing_attributes: attributes,
                    },
                    Err(error) => error.into(),
                }
            }
            error => DisclosureError::DisclosureSession(error),
        }
    }
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

        // Prepare a `Vec<ProposedDisclosureDocument>` to report to the caller.
        let documents = session
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
    use assert_matches::assert_matches;
    use serial_test::serial;

    use nl_wallet_mdoc::{basic_sa_ext::Entry, DataElementValue};

    use crate::{disclosure::MockMdocDisclosureSession, Attribute, AttributeValue};

    use super::{super::tests::WalletWithMocks, *};

    const DISCLOSURE_URI: &str =
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com";

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

        MockMdocDisclosureSession::next_reader_registration_and_proposed_attributes(
            reader_registration,
            proposed_attributes,
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
            return_url: Url::parse("https://example.com").unwrap().into(),
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
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_disclosure_session() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` start to return the following error.
        MockMdocDisclosureSession::next_start_error(HolderError::NoDocumentRequests.into());

        // Starting disclosure with a malformed disclosure URI should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::DisclosureSession(_));
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_attributes_not_available() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` start to return the following error.
        MockMdocDisclosureSession::next_start_error(
            HolderError::AttributesNotAvailable {
                reader_registration: Default::default(),
                missing_attributes: vec!["com.example.pid/com.example.pid/age_over_18".parse().unwrap()],
            }
            .into(),
        );

        // Starting disclosure where an unavailable attribute is requested should result in an error.
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
    }

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure_error_mdoc_attributes_not_available() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up an `MdocDisclosureSession` start to return the following error.
        MockMdocDisclosureSession::next_start_error(
            HolderError::AttributesNotAvailable {
                reader_registration: Default::default(),
                missing_attributes: vec!["com.example.pid/com.example.pid/foobar".parse().unwrap()],
            }
            .into(),
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

        MockMdocDisclosureSession::next_reader_registration_and_proposed_attributes(
            Default::default(),
            proposed_attributes,
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
    }
}
