use std::sync::Arc;

use tracing::info;
use tracing::instrument;
use url::Url;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::urls;

use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::issuance::DigidSession;
use crate::repository::Repository;
use crate::wallet::PidIssuanceSession;

use super::Wallet;

#[derive(Debug)]
pub enum UriType {
    PidIssuance(Url),
    Disclosure(Url),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum UriIdentificationError {
    #[error("could not parse URI: {0}")]
    #[category(pd)]
    Parse(#[from] url::ParseError),
    #[error("unknown URI")]
    #[category(critical)]
    Unknown,
}

impl<CR, S, PEK, APC, DS, IS, MDS, WIC, UR> Wallet<CR, S, PEK, APC, DS, IS, MDS, WIC, UR>
where
    CR: Repository<Arc<WalletConfiguration>>,
    DS: DigidSession,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn identify_uri(&self, uri_str: &str) -> Result<UriType, UriIdentificationError> {
        info!("Identifying type of URI: {}", uri_str);

        let uri = Url::parse(uri_str)?;

        if matches!(self.issuance_session, Some(PidIssuanceSession::Digid(_)))
            && uri
                .as_str()
                .starts_with(urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str())
        {
            return Ok(UriType::PidIssuance(uri));
        }

        if uri
            .as_str()
            .starts_with(urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str())
        {
            return Ok(UriType::Disclosure(uri));
        }

        Err(UriIdentificationError::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::config::UNIVERSAL_LINK_BASE_URL;
    use crate::issuance::MockDigidSession;
    use crate::wallet::PidIssuanceSession;

    use super::super::test::WalletWithMocks;
    use super::*;

    #[tokio::test]
    async fn test_wallet_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Set up some URLs to work with.
        let example_uri = "https://example.com";

        let disclosure_uri_base = urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL);

        let digid_uri = urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL);
        let digid_uri = digid_uri.as_ref().as_str();

        let disclosure_uri = disclosure_uri_base.join("abcd");

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
        wallet.issuance_session = Some(PidIssuanceSession::Digid(MockDigidSession::new()));

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
