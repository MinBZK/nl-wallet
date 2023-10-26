use tracing::{info, instrument};
use url::Url;

use crate::{
    config::ConfigurationRepository,
    disclosure::{DisclosureUri, DisclosureUriError, MdocDisclosureSession},
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
    R: MdocDisclosureSession,
{
    #[instrument(skip_all)]
    pub fn start_disclosure(&mut self, uri: &Url) -> Result<(), DisclosureError> {
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

        // Assume that redirect URI creation is checked when updating the `Configuration`.
        let disclosure_redirect_uri_base = self.config_repository.config().disclosure.uri_base().unwrap();
        let disclosure_uri = DisclosureUri::parse(uri, &disclosure_redirect_uri_base)?;

        // Start the disclosure session based on the `ReaderEngagement` and
        // retain the return URL for if the session is completed successfully.
        let session = R::start(disclosure_uri)?;
        self.disclosure_session.replace(session);

        Ok(())
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

        // Set up `DisclosureSession` to have `start()` called on it
        // with the items parsed from the disclosure URI.
        let session_start_context = MockMdocDisclosureSession::start_context();
        session_start_context
            .expect()
            .with(eq(DisclosureUri {
                reader_engagement_bytes: b"foobar".to_vec(),
                return_url: Url::parse("https://example.com").unwrap().into(),
            }))
            .return_once(|_| Ok(MockMdocDisclosureSession::default()));

        // Starting disclosure should not fail.
        wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .expect("Could not start disclosure");
    }

    #[tokio::test]
    async fn test_wallet_start_disclosure_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        wallet.lock();

        // Starting disclosure on a locked wallet should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
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
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
    }

    // TODO: Test for `DisclosureSessionError` when we have more error invariants.
}
