use std::collections::HashSet;

use async_trait::async_trait;
use tracing::{info, instrument};
use url::Url;

use nl_wallet_mdoc::holder::{Mdoc, MdocDataSource};

use crate::{
    config::ConfigurationRepository,
    disclosure::{DisclosureUri, DisclosureUriError, MdocDisclosureSession},
    storage::{Storage, StorageError},
};

use super::Wallet;

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
    DisclosureSession(#[from] nl_wallet_mdoc::Error),
}

impl<C, S, K, A, D, P, R> Wallet<C, S, K, A, D, P, R>
where
    C: ConfigurationRepository,
    R: MdocDisclosureSession<Self>,
{
    #[instrument(skip_all)]
    pub async fn start_disclosure(&mut self, uri: &Url) -> Result<(), DisclosureError> {
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

        // Start the disclosure session based on the `ReaderEngagement` and
        // retain the return URL for if the session is completed successfully.
        let session = R::start(disclosure_uri, self, &config.mdoc_trust_anchors()).await?;
        self.disclosure_session.replace(session);

        // TODO: Return RP data and disclosure request.

        Ok(())
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

    // TODO: Test for `DisclosureSessionError` when we have more error invariants.
}
