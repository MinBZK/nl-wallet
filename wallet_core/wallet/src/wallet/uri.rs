use std::sync::Arc;

use tracing::info;
use tracing::instrument;
use url::Url;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::urls;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::digid::DigidClient;
use crate::repository::Repository;
use crate::wallet::Session;

use super::Wallet;

#[derive(Debug)]
pub enum UriType {
    PidIssuance,
    PidRenewal,
    PinRecovery,
    Disclosure,
    DisclosureBasedIssuance,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum UriIdentificationError {
    #[error("could not parse URI: {0}")]
    Parse(#[from] url::ParseError),
    #[error("unknown URI")]
    Unknown(Url),
}

pub(super) fn identify_uri(uri: &Url) -> Option<UriType> {
    if uri
        .as_str()
        .starts_with(urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str())
    {
        return Some(UriType::PidIssuance);
    }

    if uri.as_str().starts_with(
        urls::disclosure_based_issuance_base_uri(&UNIVERSAL_LINK_BASE_URL)
            .as_ref()
            .as_str(),
    ) {
        return Some(UriType::DisclosureBasedIssuance);
    }

    if uri
        .as_str()
        .starts_with(urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str())
    {
        return Some(UriType::Disclosure);
    }

    None
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn identify_uri(&self, uri_str: &str) -> Result<UriType, UriIdentificationError> {
        info!("Identifying type of URI: {}", uri_str);

        let uri = Url::parse(uri_str)?;
        let uri_type = match identify_uri(&uri) {
            Some(uri_type) => uri_type,
            None => return Err(UriIdentificationError::Unknown(uri)),
        };

        if matches!(uri_type, UriType::PidIssuance) && !matches!(self.session, Some(Session::Digid(_))) {
            return Err(UriIdentificationError::Unknown(uri));
        }

        Ok(uri_type)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::config::UNIVERSAL_LINK_BASE_URL;
    use crate::digid::MockDigidSession;

    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    #[tokio::test]
    async fn test_wallet_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Set up some URLs to work with.
        let example_uri = "https://example.com/";

        let disclosure_uri_base = urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL);
        let disclosure_based_issuance_uri_base = urls::disclosure_based_issuance_base_uri(&UNIVERSAL_LINK_BASE_URL);

        let digid_uri = urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL);
        let digid_uri = digid_uri.as_ref().as_str();

        let disclosure_uri = disclosure_uri_base.join("abcd");
        let disclosure_based_issuance_uri = disclosure_based_issuance_uri_base.join("abcd");

        // The example URI should not be recognised.
        assert_matches!(
            wallet.identify_uri(example_uri).unwrap_err(),
            UriIdentificationError::Unknown(uri) if uri.as_str() == example_uri
        );

        // The wallet should not recognise the DigiD URI, as there is no `DigidSession`.
        assert_matches!(
            wallet.identify_uri(digid_uri).unwrap_err(),
            UriIdentificationError::Unknown(uri) if uri.as_str() == digid_uri
        );

        // Set up a `DigidSession` that will match the URI.
        wallet.session = Some(Session::Digid(MockDigidSession::new()));

        // The wallet should now recognise the DigiD URI.
        assert_matches!(wallet.identify_uri(digid_uri).unwrap(), UriType::PidIssuance);

        // After clearing the `DigidSession`, the URI should not be recognised again.
        wallet.session = None;

        assert_matches!(
            wallet.identify_uri(digid_uri).unwrap_err(),
            UriIdentificationError::Unknown(uri) if uri.as_str() == digid_uri
        );

        // The disclosure URI should be recognised.
        assert_matches!(
            wallet.identify_uri(disclosure_uri.as_str()).unwrap(),
            UriType::Disclosure
        );

        // The disclosure based issuance URI should be recognised.
        assert_matches!(
            wallet.identify_uri(disclosure_based_issuance_uri.as_str()).unwrap(),
            UriType::DisclosureBasedIssuance
        );
    }
}
