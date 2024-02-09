use tracing::{info, instrument};
use url::Url;
use wallet_common::config::wallet_config::DISCLOSURE_BASE_URI;

use crate::{config::ConfigurationRepository, digid::DigidSession};

use super::Wallet;

#[derive(Debug)]
pub enum UriType {
    PidIssuance(Url),
    Disclosure(Url),
}

#[derive(Debug, thiserror::Error)]
pub enum UriIdentificationError {
    #[error("could not parse URI: {0}")]
    Parse(#[from] url::ParseError),
    #[error("unknown URI")]
    Unknown,
}

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    CR: ConfigurationRepository,
    DGS: DigidSession,
{
    #[instrument(skip_all)]
    pub fn identify_uri(&self, uri_str: &str) -> Result<UriType, UriIdentificationError> {
        info!("Identifying type of URI: {}", uri_str);

        let uri = Url::parse(uri_str)?;

        if self
            .digid_session
            .as_ref()
            .map(|session| session.matches_received_redirect_uri(&uri))
            .unwrap_or_default()
        {
            return Ok(UriType::PidIssuance(uri));
        }

        if uri.as_str().starts_with(DISCLOSURE_BASE_URI.as_str()) {
            return Ok(UriType::Disclosure(uri));
        }

        Err(UriIdentificationError::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::digid::MockDigidSession;

    use super::{super::test::WalletWithMocks, *};

    #[tokio::test]
    async fn test_wallet_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Set up some URLs to work with.
        let example_uri = "https://example.com";
        let digid_uri = "redirect://here";

        let mut disclosure_uri_base = DISCLOSURE_BASE_URI.to_owned();

        // Add a trailing slash to the base path, if needed.
        if !disclosure_uri_base.path().ends_with('/') {
            disclosure_uri_base.path_segments_mut().unwrap().push("/");
        }

        let disclosure_uri = disclosure_uri_base.join("abcd").unwrap();

        // The example URI should not be recognised.
        assert_matches!(
            wallet.identify_uri(example_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );

        // The wallet should not recognise the DigiD URI, as there is no `DigidSession`.
        assert_matches!(
            wallet.identify_uri(digid_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );

        // Set up a `DigidSession` that will match the URI.
        let digid_session = {
            let mut digid_session = MockDigidSession::new();

            digid_session
                .expect_matches_received_redirect_uri()
                .returning(move |url| url.as_str() == digid_uri);

            digid_session
        };
        wallet.digid_session = digid_session.into();

        // The wallet should now recognise the DigiD URI.
        assert_matches!(wallet.identify_uri(digid_uri).unwrap(), UriType::PidIssuance(_));

        // After clearing the `DigidSession`, the URI should not be recognised again.
        wallet.digid_session = None;

        assert_matches!(
            wallet.identify_uri(digid_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );

        // The disclosure URI should be recognised.
        assert_matches!(
            wallet.identify_uri(disclosure_uri.as_str()).unwrap(),
            UriType::Disclosure(_)
        );
    }
}
