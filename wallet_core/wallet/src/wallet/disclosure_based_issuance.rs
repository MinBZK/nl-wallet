use std::sync::Arc;

use tracing::info;
use tracing::instrument;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use http_utils::tls::pinning::TlsPinningConfig;
use mdoc::utils::auth::Organization;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpMessageClientError;
use openid4vc::issuance_session::IssuanceSession as Openid4vcIssuanceSession;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use openid4vc::PostAuthResponseErrorCode;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::Attestation;
use crate::disclosure::MdocDisclosureSession;
use crate::errors::UpdatePolicyError;
use crate::issuance::DigidSession;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::wallet::Session;

use super::disclosure::RedirectUriPurpose;
use super::DisclosureError;
use super::IssuanceError;
use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DisclosureBasedIssuanceError {
    #[error("disclosure failed: {0}")]
    Disclosure(#[from] DisclosureError),
    #[error("retrieving attribute previews failed: {0}")]
    Issuance(#[from] IssuanceError),
    #[error("missing redirect URI from verifier response")]
    #[category(critical)]
    MissingRedirectUri(Box<Organization>),
    #[error("missing query in redirect URI")]
    #[category(critical)]
    MissingRedirectUriQuery(Box<Organization>),
    #[error("failed to deserialize Credential Offer: {0}")]
    #[category(pd)]
    UrlDecoding(#[source] serde_urlencoded::de::Error, Box<Organization>),
    #[error("no grants found in Credential Offer")]
    #[category(critical)]
    MissingGrants(Box<Organization>),
    #[error("no Authorization Code found in Credential Offer")]
    #[category(critical)]
    MissingAuthorizationCode(Box<Organization>),
    #[error("unexpected scheme: expected '{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}', found '{0}'")]
    #[category(critical)]
    UnexpectedScheme(String, Box<Organization>),
}

// This method requires the caller to be aware which flow it is in: ordinary disclosure
// (in which case it should call accept_disclosure()), or disclosure based issuance.
// Calling this method in a non-disclosure based issuance setting will result in an error.
// Alternatively, we could have made accept_disclosure() return an enum, containing either
// the redirect URI (in case of ordinary disclosure), or a `Vec<Attestation>`.
// However, the `flutter_api` already knows which flow it is in anyway, because it displays
// different things to the user in each flow. So keeping this a distinct method is more
// pragmatic.
impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    DS: DigidSession,
    IS: Openid4vcIssuanceSession,
    MDS: MdocDisclosureSession<Self>,
    WIC: Default,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_disclosure_based_issuance(
        &mut self,
        pin: String,
    ) -> Result<Vec<Attestation>, DisclosureBasedIssuanceError> {
        let config = self.config_repository.get();

        info!("Checking if a disclosure session is present");
        let Some(Session::Disclosure(session)) = &self.session else {
            return Err(DisclosureBasedIssuanceError::Disclosure(DisclosureError::SessionState));
        };

        let organization = session.protocol_state().reader_registration().organization.clone();

        let redirect_uri = match self
            .perform_disclosure(pin, RedirectUriPurpose::Issuance, config.as_ref())
            .await
        {
            Ok(Some(redirect_uri)) if redirect_uri.scheme() == OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME => redirect_uri,

            Ok(Some(redirect_uri)) => Err(DisclosureBasedIssuanceError::UnexpectedScheme(
                redirect_uri.scheme().to_string(),
                Box::new(organization.clone()),
            ))?,

            Ok(None) => Err(DisclosureBasedIssuanceError::MissingRedirectUri(Box::new(
                organization.clone(),
            )))?,

            // If the issuer has no attestations to issue, return an empty Vec.
            Err(DisclosureError::VpClient(VpClientError::Request(VpMessageClientError::AuthPostResponse(err))))
                if err.error_response.error == PostAuthResponseErrorCode::NoIssuableAttestations =>
            {
                return Ok(vec![]);
            }

            Err(err) => Err(err)?,
        };

        let query = redirect_uri
            .query()
            .ok_or(DisclosureBasedIssuanceError::MissingRedirectUriQuery(Box::new(
                organization.clone(),
            )))?;

        let CredentialOfferContainer { credential_offer } = serde_urlencoded::from_str(query)
            .map_err(|e| DisclosureBasedIssuanceError::UrlDecoding(e, Box::new(organization.clone())))?;

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code: credential_offer
                    .grants
                    .ok_or(DisclosureBasedIssuanceError::MissingGrants(Box::new(
                        organization.clone(),
                    )))?
                    .authorization_code()
                    .ok_or(DisclosureBasedIssuanceError::MissingAuthorizationCode(Box::new(
                        organization.clone(),
                    )))?,
            },
            code_verifier: None,
            client_id: Some(NL_WALLET_CLIENT_ID.to_string()),
            redirect_uri: None,
        };

        let previews = self
            .issuance_fetch_previews(
                token_request,
                credential_offer.credential_issuer,
                &config.mdoc_trust_anchors(),
                false,
            )
            .await?;

        Ok(previews)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use parking_lot::lock_api::Mutex;

    use openid4vc::credential::CredentialOffer;
    use openid4vc::credential::CredentialOfferContainer;
    use openid4vc::credential::GrantPreAuthorizedCode;
    use openid4vc::credential::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
    use openid4vc::credential_formats::CredentialFormats;
    use openid4vc::disclosure_session::VpClientError;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::token::CredentialPreview;
    use openid4vc::verifier::DisclosureResultHandlerError;
    use openid4vc::verifier::PostAuthResponseError;
    use openid4vc::verifier::ToPostAuthResponseErrorCode;
    use openid4vc::DisclosureErrorResponse;
    use openid4vc::PostAuthResponseErrorCode;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use utils::vec_at_least::VecNonEmpty;

    use crate::disclosure::MdocDisclosureError;
    use crate::disclosure::MdocDisclosureSessionState;
    use crate::disclosure::MockMdocDisclosureProposal;
    use crate::disclosure::MockMdocDisclosureSession;
    use crate::issuance;
    use crate::wallet::disclosure::DisclosureSession;
    use crate::wallet::disclosure::RedirectUriPurpose;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::WalletWithMocks;
    use crate::wallet::test::ISSUER_KEY;
    use crate::wallet::DisclosureBasedIssuanceError;
    use crate::wallet::DisclosureError;
    use crate::wallet::Session;

    const PIN: &str = "051097";

    #[tokio::test]
    async fn test_wallet_accept_disclosure_based_issuance() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Setup wallet disclosure state
        let credential_offer = serde_urlencoded::to_string(CredentialOfferContainer {
            credential_offer: CredentialOffer {
                credential_issuer: "https://issuer.example.com".parse().unwrap(),
                credential_configuration_ids: vec![],
                grants: Some(openid4vc::credential::Grants::PreAuthorizedCode {
                    pre_authorized_code: GrantPreAuthorizedCode::new("123".to_string().into()),
                }),
            },
        })
        .unwrap();
        let credential_offer = format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://?{credential_offer}")
            .parse()
            .unwrap();
        wallet.session = Some(Session::Disclosure(DisclosureSession::new(
            RedirectUriPurpose::Issuance,
            MockMdocDisclosureSession {
                session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                    disclose_return_url: Some(credential_offer),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )));

        // Setup wallet issuance state
        let (unsigned_mdoc, metadata) = issuance::mock::create_example_unsigned_mdoc();
        let (_, _, metadata_documents) = TypeMetadataDocuments::from_single_example(metadata);
        let normalized_type_metadata = vec![
            metadata_documents
                .clone()
                .into_normalized(&unsigned_mdoc.doc_type)
                .unwrap()
                .0,
        ];
        let credential_formats = CredentialFormats::try_new(
            VecNonEmpty::try_from(vec![CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer_certificate: ISSUER_KEY.issuance_key.certificate().clone(),
                type_metadata: metadata_documents.clone(),
            }])
            .unwrap(),
        )
        .unwrap();
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            Ok((
                MockIssuanceSession::new(),
                vec![(credential_formats, normalized_type_metadata)],
            ))
        });

        // Accept disclosure based issuance
        let previews = wallet
            .continue_disclosure_based_issuance(PIN.to_owned())
            .await
            .expect("Accepting disclosure based issuance should not have resulted in an error");

        assert!(!previews.is_empty())
    }

    #[derive(thiserror::Error, Debug)]
    #[error("mock error")]
    pub struct MockError;

    impl ToPostAuthResponseErrorCode for MockError {
        fn to_error_code(&self) -> PostAuthResponseErrorCode {
            PostAuthResponseErrorCode::NoIssuableAttestations
        }
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_based_issuance_no_attestations() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Setup an disclosure based issuance session returning an error that means there are no attestations to offer.
        wallet.session = Some(Session::Disclosure(DisclosureSession::new(
            RedirectUriPurpose::Issuance,
            MockMdocDisclosureSession {
                session_state: MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal {
                    next_error: Mutex::new(Some(MdocDisclosureError::Vp(
                        VpClientError::Request(
                            DisclosureErrorResponse {
                                error_response: PostAuthResponseError::HandlingDisclosureResult(
                                    DisclosureResultHandlerError(Box::new(MockError)),
                                )
                                .into(),
                                redirect_uri: None,
                            }
                            .into(),
                        )
                        .into(),
                    ))),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )));

        let previews = wallet
            .continue_disclosure_based_issuance(PIN.to_owned())
            .await
            .expect("Accepting disclosure based issuance should not have resulted in an error");

        // By offering zero attestations to issue, the issuer says that it has no attestations to offer.
        assert!(previews.is_empty());
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_based_issuance_error_wrong_redirect_uri_purpose() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let disclosure_session = MockMdocDisclosureSession::default();
        wallet.session = Some(Session::Disclosure(DisclosureSession::new(
            RedirectUriPurpose::Browser,
            disclosure_session,
        )));

        let error = wallet
            .continue_disclosure_based_issuance(PIN.to_owned())
            .await
            .expect_err("Accepting disclosure based issuance should have resulted in an error");

        assert_matches!(
            error,
            DisclosureBasedIssuanceError::Disclosure(DisclosureError::UnexpectedRedirectUriPurpose {
                expected: RedirectUriPurpose::Browser,
                found: RedirectUriPurpose::Issuance,
            })
        );
    }
}
