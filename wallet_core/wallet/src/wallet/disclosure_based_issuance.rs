use std::sync::Arc;

use openid4vc::disclosure_session::DisclosureSession;
use tracing::info;
use tracing::instrument;

use attestation_data::auth::Organization;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::PostAuthResponseErrorCode;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpMessageClientError;
use openid4vc::issuance_session::IssuanceSession as Openid4vcIssuanceSession;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::AttestationPresentation;
use crate::digid::DigidClient;
use crate::errors::UpdatePolicyError;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::wallet::Session;

use super::DisclosureError;
use super::IssuanceError;
use super::Wallet;
use super::disclosure::RedirectUriPurpose;

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
impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    DC: DigidClient,
    IS: Openid4vcIssuanceSession,
    DCC: DisclosureClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_disclosure_based_issuance(
        &mut self,
        pin: String,
    ) -> Result<Vec<AttestationPresentation>, DisclosureBasedIssuanceError> {
        let config = self.config_repository.get();

        info!("Checking if a disclosure session is present");
        let Some(Session::Disclosure(session)) = &self.session else {
            return Err(DisclosureBasedIssuanceError::Disclosure(DisclosureError::SessionState));
        };

        let organization = session
            .protocol_state
            .verifier_certificate()
            .registration()
            .organization
            .clone();

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
                &config.issuer_trust_anchors(),
                false,
            )
            .await?;

        Ok(previews)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use uuid::Uuid;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::disclosure_type::DisclosureType;
    use attestation_data::x509::generate::mock::generate_reader_mock;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use mdoc::holder::disclosure::PartialMdoc;
    use openid4vc::DisclosureErrorResponse;
    use openid4vc::PostAuthResponseErrorCode;
    use openid4vc::credential::CredentialOffer;
    use openid4vc::credential::CredentialOfferContainer;
    use openid4vc::credential::GrantPreAuthorizedCode;
    use openid4vc::credential::Grants;
    use openid4vc::credential::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
    use openid4vc::disclosure_session;
    use openid4vc::disclosure_session::VerifierCertificate;
    use openid4vc::disclosure_session::VpClientError;
    use openid4vc::disclosure_session::VpSessionError;
    use openid4vc::disclosure_session::mock::MockDisclosureSession;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::verifier::DisclosureResultHandlerError;
    use openid4vc::verifier::PostAuthResponseError;
    use openid4vc::verifier::ToPostAuthResponseErrorCode;
    use utils::generator::mock::MockTimeGenerator;

    use crate::attestation::AttestationPresentation;
    use crate::storage::DisclosableAttestation;
    use crate::storage::PartialAttestation;

    use super::super::DisclosureBasedIssuanceError;
    use super::super::Session;
    use super::super::disclosure::DisclosureError;
    use super::super::disclosure::RedirectUriPurpose;
    use super::super::disclosure::WalletDisclosureSession;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::super::test::create_example_preview_data;

    const PIN: &str = "051097";

    fn setup_wallet_disclosure_session() -> WalletDisclosureSession<MockDisclosureSession> {
        let reader_ca = Ca::generate_reader_mock_ca().unwrap();
        let reader_key_pair = generate_reader_mock(&reader_ca, Some(ReaderRegistration::new_mock())).unwrap();
        let verifier_certificate = VerifierCertificate::try_new(reader_key_pair.into()).unwrap().unwrap();

        let mut disclosure_session = MockDisclosureSession::new();
        disclosure_session
            .expect_verifier_certificate()
            .return_const(verifier_certificate);

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let mdoc_key = MockRemoteEcdsaKey::new("mdoc_key".to_string(), SigningKey::random(&mut OsRng));
        let partial_mdoc = Box::new(PartialMdoc::new_mock_with_ca_and_key(&ca, &mdoc_key));
        let disclosable_attestation = DisclosableAttestation::new(
            Uuid::new_v4(),
            PartialAttestation::MsoMdoc { partial_mdoc },
            AttestationPresentation::new_mock(),
        );

        WalletDisclosureSession::new_proposal(
            RedirectUriPurpose::Issuance,
            DisclosureType::Regular,
            IndexMap::from([("id".try_into().unwrap(), disclosable_attestation)]),
            disclosure_session,
        )
    }

    #[tokio::test]
    async fn test_wallet_accept_disclosure_based_issuance() {
        // Prepare a registered and unlocked wallet with an active disclosure session.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Setup wallet disclosure state
        let credential_offer = serde_urlencoded::to_string(CredentialOfferContainer {
            credential_offer: CredentialOffer {
                credential_issuer: "https://issuer.example.com".parse().unwrap(),
                credential_configuration_ids: vec![],
                grants: Some(Grants::PreAuthorizedCode {
                    pre_authorized_code: GrantPreAuthorizedCode::new("123".to_string().into()),
                }),
            },
        })
        .unwrap();
        let credential_offer = format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://?{credential_offer}")
            .parse()
            .unwrap();

        let mut disclosure_session = setup_wallet_disclosure_session();
        disclosure_session
            .protocol_state
            .expect_disclose()
            .return_once(|_| Ok(Some(credential_offer)));
        wallet.session = Some(Session::Disclosure(disclosure_session));

        // Setup wallet issuance state
        let credential_preview = create_example_preview_data(&MockTimeGenerator::default());
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            let mut client = MockIssuanceSession::new();

            client
                .expect_normalized_credential_previews()
                .return_const(vec![credential_preview]);

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            Ok(client)
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
        let mut disclosure_session = setup_wallet_disclosure_session();
        disclosure_session.protocol_state.expect_disclose().return_once(|_| {
            Err((
                MockDisclosureSession::new(),
                disclosure_session::DisclosureError::new(
                    true,
                    VpSessionError::Client(VpClientError::Request(
                        DisclosureErrorResponse {
                            error_response: PostAuthResponseError::HandlingDisclosureResult(
                                DisclosureResultHandlerError::new(MockError),
                            )
                            .into(),
                            redirect_uri: None,
                        }
                        .into(),
                    )),
                ),
            ))
        });
        wallet.session = Some(Session::Disclosure(disclosure_session));

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

        let mut disclosure_session = setup_wallet_disclosure_session();
        disclosure_session.redirect_uri_purpose = RedirectUriPurpose::Browser;
        wallet.session = Some(Session::Disclosure(disclosure_session));

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
