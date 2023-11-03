use std::collections::HashSet;

use async_trait::async_trait;
use tracing::{info, instrument};
use url::Url;

use nl_wallet_mdoc::{
    holder::{HolderError, Mdoc, MdocDataSource},
    utils::reader_auth::ReaderRegistration,
};

use crate::{
    config::ConfigurationRepository,
    disclosure::{DisclosureUri, DisclosureUriError, MdocDisclosureSession},
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
    DisclosureUriError(#[from] DisclosureUriError),
    #[error("error in mdoc disclosure session: {0}")]
    DisclosureSession(#[source] nl_wallet_mdoc::Error),
    #[error("could not interpret mdoc attributes: {0}")]
    AttributeMdoc(#[from] DocumentMdocError),
    #[error("not all requested attributes are available, missing: {missing_attributes:?}")]
    AttributesNotAvailable {
        reader_registration: Box<ReaderRegistration>,
        missing_attributes: Vec<MissingDisclosureAttributes>,
    },
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

impl<C, S, K, A, D, P, R> Wallet<C, S, K, A, D, P, R>
where
    C: ConfigurationRepository,
    R: MdocDisclosureSession<Self>,
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

        let config = self.config_repository.config();

        // Assume that redirect URI creation is checked when updating the `Configuration`.
        let disclosure_redirect_uri_base = config.disclosure.uri_base().unwrap();
        let disclosure_uri = DisclosureUri::parse(uri, &disclosure_redirect_uri_base)?;

        // Start the disclosure session based on the `ReaderEngagement`.
        let session = R::start(disclosure_uri, self, &config.mdoc_trust_anchors()).await?;

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
impl<C, S, K, A, D, P, R> MdocDataSource for Wallet<C, S, K, A, D, P, R>
where
    C: Send + Sync,
    S: Storage + Send + Sync,
    K: Send + Sync,
    A: Send + Sync,
    D: Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    type Error = StorageError;

    async fn mdoc_by_doc_types(&self, doc_types: &HashSet<&str>) -> std::result::Result<Vec<Mdoc>, Self::Error> {
        // TODO: Retain UUIDs and increment use count on mdoc_copy when disclosure takes place.
        let mdocs = self
            .storage
            .read()
            .await
            .fetch_unique_mdocs_by_doctypes(doc_types)
            .await?
            .into_iter()
            .map(|(_, mdoc)| mdoc)
            .collect();

        Ok(mdocs)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use serial_test::serial;

    use crate::disclosure::MockMdocDisclosureSession;

    use super::{super::tests::WalletWithMocks, *};

    const DISCLOSURE_URI: &str =
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com";

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Starting disclosure should not fail.
        wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .await
            .expect("Could not start disclosure");

        // Test that the `Wallet` now contains a `DisclosureSession`
        // with the items parsed from the disclosure URI.
        assert_matches!(wallet.disclosure_session, Some(session) if session.disclosure_uri == DisclosureUri {
            reader_engagement_bytes: b"foobar".to_vec(),
            return_url: Url::parse("https://example.com").unwrap().into(),
        });

        // TODO: Test returned `DisclosureProposal`.
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

    // TODO: Test for `DisclosureSessionError`.
}
