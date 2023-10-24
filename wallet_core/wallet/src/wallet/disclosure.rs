use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use tracing::{info, instrument};
use url::Url;

use crate::{
    config::ConfigurationRepository,
    disclosure::{DisclosureSession, DisclosureSessionError},
    utils,
};

use super::Wallet;

const PARAM_RETURN_URL: &str = "return_url";

#[derive(Debug, thiserror::Error)]
pub enum DisclosureError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("wallet is locked")]
    Locked,
    #[error("disclosure session is not in the correct state")]
    SessionState,
    #[error("{0}")]
    DisclosureUriError(#[from] DisclosureUriError),
    #[error("{0}")]
    DisclosureSession(#[from] DisclosureSessionError),
}

#[derive(Debug, thiserror::Error)]
pub enum DisclosureUriError {
    #[error("recieved URI is malformed for disclosure: {0}")]
    Malformed(Url),
    #[error("could not decode reader engagement from disclosure URI: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("could not parse return URL from disclosure URI: {0}")]
    ReturnUrl(#[from] url::ParseError),
}

/// Parse the `ReaderEngagement`` bytes and a possible return URL from the disclosure URI.
/// The `base_uri` argument is contained in the `Configuration`.
fn parse_disclosure_uri(uri: &Url, base_uri: &Url) -> Result<(Vec<u8>, Option<Url>), DisclosureUriError> {
    // Check if both URIs can have path segments (see below) and
    // if the the base URI is actually a base of the disclosure URI.
    if uri.cannot_be_a_base() || base_uri.cannot_be_a_base() || !uri.as_str().starts_with(base_uri.as_str()) {
        return Err(DisclosureUriError::Malformed(uri.clone()));
    }

    // Get the number of path segments in the base URI, taking a trailing slash into account.
    let mut base_path_segment_count = base_uri.path_segments().map(|s| s.count()).unwrap_or_default();
    if base_uri.path().ends_with('/') {
        base_path_segment_count -= 1;
    }

    // Get the first path segment from the disclosure URI that is beyond
    // that of the base and check that it is not an empty string.
    let mut path_segments_iter = uri.path_segments().unwrap().skip(base_path_segment_count);
    let path_segment = path_segments_iter.next();
    let reader_engagement_base64 = path_segment.ok_or_else(|| DisclosureUriError::Malformed(uri.clone()))?;

    if reader_engagement_base64.is_empty() {
        return Err(DisclosureUriError::Malformed(uri.clone()));
    }

    // If there are additional path segments, consider that an error.
    if path_segments_iter.next().is_some() {
        return Err(DisclosureUriError::Malformed(uri.clone()));
    }

    // Decode the `ReaderEngagement` bytes from base64.
    let reader_engagement_bytes = URL_SAFE_NO_PAD.decode(reader_engagement_base64)?;

    // Parse an optional return URL from the query parameters.
    let return_url = utils::url::url_find_first_query_value(uri, PARAM_RETURN_URL)
        .map(|url| Url::parse(url.as_ref()))
        .transpose()?;

    Ok((reader_engagement_bytes, return_url))
}

impl<C, S, K, A, D, P, R> Wallet<C, S, K, A, D, P, R>
where
    C: ConfigurationRepository,
    R: DisclosureSession,
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
        let (reader_engagement_bytes, return_url) = parse_disclosure_uri(uri, &disclosure_redirect_uri_base)?;

        // Start the disclosure session based on the `ReaderEngagement` and
        // retain the return URL for if the session is completed successfully.
        let session = R::start(&reader_engagement_bytes, return_url)?;
        self.disclosure_session.replace(session);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use rstest::rstest;
    use serial_test::serial;

    use crate::disclosure::MockDisclosureSession;

    use super::{super::tests::WalletWithMocks, *};

    #[rstest]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy",
        "scheme://host.name/some/path",
        b"foobar",
        None
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy",
        "scheme://host.name/some/path/",
        b"foobar",
        None
    )]
    #[case("scheme://host.name/Zm9vYmFy", "scheme://host.name", b"foobar", None)]
    #[case("scheme://host.name/Zm9vYmFy", "scheme://host.name/", b"foobar", None)]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com",
        "scheme://host.name/some/path",
        b"foobar",
        Some("https://example.com")
    )]
    fn test_parse_disclosure_uri(
        #[case] uri: Url,
        #[case] base_uri: Url,
        #[case] expected_bytes: &[u8],
        #[case] expected_return_url: Option<&str>,
    ) {
        let (reader_engagement_bytes, return_url) =
            parse_disclosure_uri(&uri, &base_uri).expect("Could not parse disclosure URI");

        assert_eq!(reader_engagement_bytes, expected_bytes);
        assert_eq!(return_url, expected_return_url.map(|url| Url::parse(url).unwrap()));
    }

    #[rstest]
    #[case("httsp://example.com/Zm9vYmFy", "scheme://host.name")]
    #[case("scheme://host.name/some/path/Zm9vYmFy/blah", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path/")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path/")]
    fn test_parse_disclosure_uri_error_malformed(#[case] uri: Url, #[case] base_uri: Url) {
        let error =
            parse_disclosure_uri(&uri, &base_uri).expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Malformed(_));
    }

    #[rstest]
    #[case("scheme://host.name/some/path/foobar", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/Zm9vYmFyCg==", "scheme://host.name/some/path")]
    fn test_parse_disclosure_uri_error_base64(#[case] uri: Url, #[case] base_uri: Url) {
        let error =
            parse_disclosure_uri(&uri, &base_uri).expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Base64(_));
    }

    #[rstest]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=foobar",
        "scheme://host.name/some/path"
    )]
    fn test_parse_disclosure_uri_error_return_url(#[case] uri: Url, #[case] base_uri: Url) {
        let error =
            parse_disclosure_uri(&uri, &base_uri).expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::ReturnUrl(_));
    }

    const DISCLOSURE_URI: &str =
        "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com";

    #[tokio::test]
    #[serial]
    async fn test_wallet_start_disclosure() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::registered().await;

        // Set up `DisclosureSession` to have `start()` called on it
        // with the items parsed from the disclosure URI.
        let session_start_context = MockDisclosureSession::start_context();
        session_start_context
            .expect()
            .with(
                eq(b"foobar".to_vec()),
                eq(Some(Url::parse("https://example.com").unwrap())),
            )
            .return_once(|_, _| Ok(MockDisclosureSession::default()));

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

        wallet.disclosure_session = MockDisclosureSession::default().into();

        // Starting disclosure on a wallet with an active disclosure should result in an error.
        let error = wallet
            .start_disclosure(&Url::parse(DISCLOSURE_URI).unwrap())
            .expect_err("Starting disclosure should have resulted in an error");

        assert_matches!(error, DisclosureError::SessionState);
    }

    // TODO: Test for `DisclosureSessionError` when we have more error invariants.
}
