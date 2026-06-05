use std::sync::Arc;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::urls;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::return_url::credential_offer_base_uri;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKeyHolder;
use tracing::info;
use tracing::instrument;
use url::Url;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::Wallet;
use crate::PidIssuancePurpose;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::repository::Repository;
use crate::wallet::Session;
use crate::wallet::issuance::SessionState;
use crate::wallet::issuance::WalletIssuanceSession;
use crate::wallet::pin_recovery::PinRecoverySession;

#[derive(Debug)]
pub enum UriType {
    PidIssuance,
    PidRenewal,
    PinRecovery,
    Disclosure,
    DisclosureBasedIssuance,
    Transfer,
    CredentialOffer,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum UriIdentificationError {
    #[error("could not parse URI: {0}")]
    Parse(#[from] url::ParseError),
    #[error("unknown URI")]
    Unknown(Url),
}

/// Custom URL schemes for disclosure flows.
const DISCLOSURE_URL_SCHEMES: &[&str] = &["eu-eaap", "openid4vp", "haip-vp"];

/// Custom URL schemes for credential offer issuance flows.
const CREDENTIAL_OFFER_URI_SCHEMES: &[&str] = &["eu-eaa-offer", "haip-vci", "openid-credential-offer"];

pub(super) fn identify_uri(uri: &Url) -> Option<UriType> {
    let uri_str = uri.as_str();

    if uri_str.starts_with(urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str()) {
        return Some(UriType::PidIssuance);
    }

    if uri_str.starts_with(
        urls::disclosure_based_issuance_base_uri(&UNIVERSAL_LINK_BASE_URL)
            .as_ref()
            .as_str(),
    ) {
        return Some(UriType::DisclosureBasedIssuance);
    }

    if uri_str.starts_with(urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str()) {
        return Some(UriType::Disclosure);
    }

    if uri_str.starts_with(credential_offer_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str()) {
        return Some(UriType::CredentialOffer);
    }

    if uri_str.starts_with(urls::transfer_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().as_str()) {
        return Some(UriType::Transfer);
    }

    if DISCLOSURE_URL_SCHEMES.contains(&uri.scheme()) {
        return Some(UriType::Disclosure);
    }

    if CREDENTIAL_OFFER_URI_SCHEMES.contains(&uri.scheme()) {
        return Some(UriType::CredentialOffer);
    }

    None
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn identify_uri(&self, uri_str: &str) -> Result<UriType, UriIdentificationError> {
        info!("Identifying type of URI: {}", uri_str);

        let uri = Url::parse(uri_str)?;
        let uri_type = match identify_uri(&uri) {
            // The authorization return URL should only be handled if we're doing either PID issuance or PIN recovery.
            Some(UriType::PidIssuance)
                if matches!(
                    self.session,
                    Some(Session::Issuance(WalletIssuanceSession::Pid {
                        purpose: PidIssuancePurpose::Enrollment,
                        session_state: SessionState::Authorization { .. },
                    })),
                ) =>
            {
                UriType::PidIssuance
            }
            Some(UriType::PidIssuance)
                if matches!(
                    self.session,
                    Some(Session::Issuance(WalletIssuanceSession::Pid {
                        purpose: PidIssuancePurpose::Renewal,
                        session_state: SessionState::Authorization { .. },
                    })),
                ) =>
            {
                UriType::PidRenewal
            }
            Some(UriType::PidIssuance)
                if matches!(
                    self.session,
                    Some(Session::PinRecovery(PinRecoverySession::OAuth { .. }))
                ) =>
            {
                UriType::PinRecovery
            }

            // If we're not doing PID issuance or PIN recovery then the authorization return URL is unexpected,
            // so return an error in that case (and of course also when the URI was not recognized).
            Some(UriType::PidIssuance) | None => return Err(UriIdentificationError::Unknown(uri)),

            // Just pass through any other URI types.
            Some(uri_type) => uri_type,
        };

        Ok(uri_type)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use openid4vc::wallet_issuance::mock::MockAuthorizationSession;
    use rstest::rstest;

    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::*;
    use crate::config::UNIVERSAL_LINK_BASE_URL;
    use crate::wallet::issuance::SessionState;
    use crate::wallet::issuance::WalletIssuanceSession;

    #[tokio::test]
    async fn test_wallet_identify_redirect_uri() {
        // Prepare an unregistered wallet.
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Set up some URLs to work with.
        let example_uri = "https://example.com/";

        let disclosure_uri_base = urls::disclosure_base_uri(&UNIVERSAL_LINK_BASE_URL);
        let disclosure_based_issuance_uri_base = urls::disclosure_based_issuance_base_uri(&UNIVERSAL_LINK_BASE_URL);

        let authorization_uri = urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL);
        let authorization_uri = authorization_uri.as_ref().as_str();

        let disclosure_uri = disclosure_uri_base.join("abcd");
        let disclosure_based_issuance_uri = disclosure_based_issuance_uri_base.join("abcd");

        let transfer_uri = urls::transfer_base_uri(&UNIVERSAL_LINK_BASE_URL).join("fghi");

        // The example URI should not be recognised.
        assert_matches!(
            wallet.identify_uri(example_uri).unwrap_err(),
            UriIdentificationError::Unknown(uri) if uri.as_str() == example_uri
        );

        // The wallet should not recognise the authorization URI, as there is no authorization session.
        assert_matches!(
            wallet.identify_uri(authorization_uri).unwrap_err(),
            UriIdentificationError::Unknown(uri) if uri.as_str() == authorization_uri
        );

        // Set up an enrollment session that will match the URI.
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Pid {
            purpose: PidIssuancePurpose::Enrollment,
            session_state: SessionState::Authorization {
                authorization_session: MockAuthorizationSession::new(),
            },
        }));

        // The wallet should now recognise the authorization URI.
        assert_matches!(wallet.identify_uri(authorization_uri).unwrap(), UriType::PidIssuance);

        // Set up a PID renewal session that will match the URI.
        wallet.session = Some(Session::Issuance(WalletIssuanceSession::Pid {
            purpose: PidIssuancePurpose::Renewal,
            session_state: SessionState::Authorization {
                authorization_session: MockAuthorizationSession::new(),
            },
        }));

        // The wallet should now recognise the authorization URI.
        assert_matches!(wallet.identify_uri(authorization_uri).unwrap(), UriType::PidRenewal);

        // After clearing the session, the URI should not be recognised again.
        wallet.session = None;

        assert_matches!(
            wallet.identify_uri(authorization_uri).unwrap_err(),
            UriIdentificationError::Unknown(uri) if uri.as_str() == authorization_uri
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

        // The transfer URI should be recognised.
        assert_matches!(wallet.identify_uri(transfer_uri.as_str()).unwrap(), UriType::Transfer);

        // The credential offer URI should be recognised.
        let credential_offer_uri = credential_offer_base_uri(&UNIVERSAL_LINK_BASE_URL).join("offer123");
        assert_matches!(
            wallet.identify_uri(credential_offer_uri.as_str()).unwrap(),
            UriType::CredentialOffer
        );
    }

    #[rstest]
    #[case("eu-eaap://")]
    #[case("openid4vp://")]
    #[case("haip-vp://")]
    #[case("haip-vp://request?client_id=verifier&request_uri=https%3A%2F%2Fexample.com")]
    #[tokio::test]
    async fn test_wallet_identify_disclosure_scheme(#[case] uri: &str) {
        let wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;
        let actual = wallet.identify_uri(uri).expect("uri is identifiable");
        assert_matches!(actual, UriType::Disclosure);
    }

    #[rstest]
    #[case("eu-eaa-offer://")]
    #[case("haip-vci://")]
    #[case("openid-credential-offer://")]
    #[case("openid-credential-offer://request?credential_offer=%7B%22issuer%22%3A%22https%3A%2F%2Fexample.com%22%7D")] // credential_offer={"issuer":"https://example.com"}
    #[tokio::test]
    async fn test_wallet_identify_credential_offer_scheme(#[case] uri: &str) {
        let wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;
        let actual = wallet.identify_uri(uri).expect("uri is identifiable");
        assert_matches!(actual, UriType::CredentialOffer);
    }
}
