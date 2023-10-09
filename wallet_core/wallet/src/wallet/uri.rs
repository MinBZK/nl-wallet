use tracing::info;
use url::Url;

use crate::digid::DigidSession;

use super::Wallet;

#[derive(Debug)]
pub enum RedirectUriType {
    PidIssuance,
    Unknown,
}

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P>
where
    D: DigidSession,
{
    pub fn identify_redirect_uri(&self, redirect_uri: &Url) -> RedirectUriType {
        info!("Idetifying type of URI: {}", redirect_uri);

        if self
            .digid_session
            .as_ref()
            .map(|session| session.matches_received_redirect_uri(redirect_uri))
            .unwrap_or_default()
        {
            return RedirectUriType::PidIssuance;
        }

        RedirectUriType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::digid::MockDigidSession;

    use super::{super::tests::WalletWithMocks, *};

    #[test]
    fn test_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::default();

        // Set up two URLs to work with.
        let digid_url = Url::parse("redirect://here").unwrap();
        let example_url = Url::parse("https://exampl.com").unwrap();

        // The wallet should recognise neither of these URLs, as there is no `DigidSession`.
        assert_matches!(wallet.identify_redirect_uri(&digid_url), RedirectUriType::Unknown);
        assert_matches!(wallet.identify_redirect_uri(&example_url), RedirectUriType::Unknown);

        // Set up a `DigidSession` that will match only the first URL.
        let digid_session = {
            let mut digid_session = MockDigidSession::new();

            let digid_url_clone = digid_url.clone();
            digid_session
                .expect_matches_received_redirect_uri()
                .returning(move |url| url == &digid_url_clone);

            digid_session
        };
        wallet.digid_session = digid_session.into();

        // The wallet should now recognise the DigiD URL.
        assert_matches!(wallet.identify_redirect_uri(&digid_url), RedirectUriType::PidIssuance);
        assert_matches!(wallet.identify_redirect_uri(&example_url), RedirectUriType::Unknown);

        // After clearing the `DigidSession`, neither URL should be recognised again.
        wallet.digid_session = None;

        assert_matches!(wallet.identify_redirect_uri(&digid_url), RedirectUriType::Unknown);
        assert_matches!(wallet.identify_redirect_uri(&example_url), RedirectUriType::Unknown);
    }
}
