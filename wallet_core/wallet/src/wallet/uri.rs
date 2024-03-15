use openid4vc::oidc::OidcClient;
use tracing::{info, instrument};
use url::Url;

use wallet_common::config::wallet_config::WalletConfiguration;

use crate::{
    config::{ConfigurationRepository, UNIVERSAL_LINK_BASE_URL},
    wallet::PidIssuanceSession,
};

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

impl<CR, S, PEK, APC, OIC, IS, MDS> Wallet<CR, S, PEK, APC, OIC, IS, MDS>
where
    CR: ConfigurationRepository,
    OIC: OidcClient,
{
    #[instrument(skip_all)]
    pub fn identify_uri(&self, uri_str: &str) -> Result<UriType, UriIdentificationError> {
        info!("Identifying type of URI: {}", uri_str);

        let uri = Url::parse(uri_str)?;

        let redirect_uri = WalletConfiguration::issuance_redirect_uri(UNIVERSAL_LINK_BASE_URL.to_owned()).to_owned();

        if matches!(self.issuance_session, Some(PidIssuanceSession::Digid(_)))
            && uri.as_str().starts_with(redirect_uri.as_str())
        {
            return Ok(UriType::PidIssuance(uri));
        }

        if uri
            .as_str()
            .starts_with(WalletConfiguration::disclosure_base_uri(UNIVERSAL_LINK_BASE_URL.to_owned()).as_str())
        {
            return Ok(UriType::Disclosure(uri));
        }

        Err(UriIdentificationError::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use openid4vc::oidc::MockOidcClient;

    use crate::{config::UNIVERSAL_LINK_BASE_URL, wallet::PidIssuanceSession};

    use super::{super::test::WalletWithMocks, *};

    #[tokio::test]
    async fn test_wallet_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Set up some URLs to work with.
        let example_uri = "https://example.com";

        let mut disclosure_uri_base =
            WalletConfiguration::disclosure_base_uri(UNIVERSAL_LINK_BASE_URL.to_owned()).to_owned();

        let digid_uri = WalletConfiguration::issuance_redirect_uri(UNIVERSAL_LINK_BASE_URL.to_owned());
        let digid_uri = digid_uri.as_str();

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
        wallet.issuance_session = Some(PidIssuanceSession::Digid(MockOidcClient::new()));

        // The wallet should now recognise the DigiD URI.
        assert_matches!(wallet.identify_uri(digid_uri).unwrap(), UriType::PidIssuance(_));

        // After clearing the `DigidSession`, the URI should not be recognised again.
        wallet.issuance_session = None;

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
