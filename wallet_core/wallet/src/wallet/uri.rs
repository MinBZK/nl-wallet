use tracing::info;
use url::Url;

use crate::digid::DigidSession;

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

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P>
where
    D: DigidSession,
{
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

        // TODO: actually implement disclosure recognition.
        if uri
            .as_str()
            .starts_with("walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure")
        {
            return Ok(UriType::Disclosure(uri));
        }

        Err(UriIdentificationError::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::digid::MockDigidSession;

    use super::{super::tests::WalletWithMocks, *};

    #[test]
    fn test_wallet_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::default();

        // Set up some URLs to work with.
        let digid_uri = "redirect://here";
        let disclosure_uri = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/foo";
        let example_uri = "https://exampl.com";

        // The placeholder disclosure URI should be recognised.
        assert_matches!(wallet.identify_uri(disclosure_uri).unwrap(), UriType::Disclosure(_));

        // The wallet should recognise neither of these URIs, as there is no `DigidSession`.
        assert_matches!(
            wallet.identify_uri(digid_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );
        assert_matches!(
            wallet.identify_uri(example_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );

        // Set up a `DigidSession` that will match only the first URI.
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
        assert_matches!(
            wallet.identify_uri(example_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );

        // After clearing the `DigidSession`, neither URI should be recognised again.
        wallet.digid_session = None;

        assert_matches!(
            wallet.identify_uri(digid_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );
        assert_matches!(
            wallet.identify_uri(example_uri).unwrap_err(),
            UriIdentificationError::Unknown
        );
    }
}
