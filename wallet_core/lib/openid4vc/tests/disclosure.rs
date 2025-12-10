use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use assert_matches::assert_matches;
use async_trait::async_trait;
use chrono::Utc;
use futures::FutureExt;
use indexmap::IndexMap;
use itertools::Itertools;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use ring::hmac;
use ring::rand;
use rstest::rstest;
use rustls_pki_types::TrustAnchor;
use url::Url;

use attestation_data::attributes::AttributeValue;
use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure::DisclosedAttestations;
use attestation_data::disclosure::DisclosedAttributes;
use attestation_data::test_credential::TestCredentials;
use attestation_data::test_credential::nl_pid_address_credentials_all;
use attestation_data::test_credential::nl_pid_address_minimal_address;
use attestation_data::test_credential::nl_pid_credentials_all;
use attestation_data::test_credential::nl_pid_credentials_family_name;
use attestation_data::test_credential::nl_pid_credentials_full_name;
use attestation_data::test_credential::nl_pid_credentials_given_name;
use attestation_data::test_credential::nl_pid_credentials_given_name_for_query_id;
use attestation_data::x509::generate::mock::generate_reader_mock_with_registration;
use attestation_types::pid_constants::EUDI_PID_ATTESTATION_TYPE;
use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
use attestation_types::pid_constants::PID_GIVEN_NAME;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteWscd as DisclosureMockRemoteWscd;
use crypto::mock_remote::MockRemoteWscdError;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
use crypto::server_keys::generate::mock::PID_ISSUER_CERT_CN;
use crypto::server_keys::generate::mock::RP_CERT_CN;
use crypto::wscd::DisclosureResult;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use crypto::x509::CertificateUsage;
use dcql::CredentialFormat;
use dcql::CredentialQueryIdentifier;
use dcql::Query;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::unique_id_vec::UniqueIdVec;
use http_utils::urls::BaseUrl;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithX5c;
use mdoc::DeviceResponse;
use mdoc::SessionTranscript;
use mdoc::holder::disclosure::PartialMdoc;
use openid4vc::ErrorResponse;
use openid4vc::GetRequestErrorCode;
use openid4vc::PostAuthResponseErrorCode;
use openid4vc::VpAuthorizationErrorCode;
use openid4vc::disclosure_session::DisclosableAttestations;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::DisclosureUriSource;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::disclosure_session::VpDisclosureSession;
use openid4vc::disclosure_session::VpMessageClient;
use openid4vc::disclosure_session::VpMessageClientError;
use openid4vc::disclosure_session::VpSessionError;
use openid4vc::mock::ExtendingVctRetrieverStub;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::openid4vp::NormalizedVpAuthorizationRequest;
use openid4vc::openid4vp::RequestUriMethod;
use openid4vc::openid4vp::VerifiablePresentation;
use openid4vc::openid4vp::VpAuthorizationRequest;
use openid4vc::openid4vp::VpAuthorizationResponse;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionToken;
use openid4vc::verifier::DisclosedAttributesError;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::DisclosureResultHandler;
use openid4vc::verifier::DisclosureResultHandlerError;
use openid4vc::verifier::EphemeralIdParameters;
use openid4vc::verifier::RpInitiatedUseCase;
use openid4vc::verifier::RpInitiatedUseCases;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::StatusResponse;
use openid4vc::verifier::UseCase;
use openid4vc::verifier::UseCases;
use openid4vc::verifier::Verifier;
use openid4vc::verifier::VerifierUrlParameters;
use openid4vc::verifier::VpToken;
use openid4vc::verifier::WalletAuthResponse;
use openid4vc::verifier::WalletInitiatedUseCase;
use openid4vc::verifier::WalletInitiatedUseCases;
use token_status_list::verification::client::mock::StatusListClientStub;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::TimeGenerator;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_nonempty;
use wscd::Poa;
use wscd::mock_remote::MockRemoteWscd;
use wscd::wscd::JwtPoaInput;

fn assert_disclosed_attestations_mdoc_pid(disclosed_attestations: &UniqueIdVec<DisclosedAttestations>) {
    assert_eq!(disclosed_attestations.len().get(), 1);
    let disclosed_attestations = disclosed_attestations.as_ref().first().unwrap();

    assert_eq!(disclosed_attestations.id, "mdoc_pid_example".try_into().unwrap());

    let disclosed_attestation = disclosed_attestations
        .attestations
        .iter()
        .exactly_one()
        .expect("there should be only one disclosed attestation");

    assert_eq!(disclosed_attestation.attestation_type, PID_ATTESTATION_TYPE);

    let DisclosedAttributes::MsoMdoc(attributes) = &disclosed_attestation.attributes else {
        panic!("disclosed attributes should be in mdoc format");
    };

    let name_space = attributes
        .get(PID_ATTESTATION_TYPE)
        .expect("disclosed attributes should include PID");

    assert_eq!(name_space.len(), 3);
    assert_eq!(
        name_space.get("bsn"),
        Some(&AttributeValue::Text("999999999".to_string()))
    );
    assert_eq!(
        name_space.get("given_name"),
        Some(&AttributeValue::Text("Willeke Liselotte".to_string()))
    );
    assert_eq!(
        name_space.get("family_name"),
        Some(&AttributeValue::Text("De Bruijn".to_string()))
    );
}

#[test]
fn disclosure_direct() {
    let ca = Ca::generate("myca", Default::default()).unwrap();
    let auth_keypair = ca.generate_reader_mock().unwrap();

    // RP assembles the Authorization Request and signs it into a JWS.
    let nonce = "nonce".to_string();
    let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
    let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let iso_auth_request = NormalizedVpAuthorizationRequest::new(
        NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
        auth_keypair.certificate(),
        nonce.clone(),
        encryption_keypair.to_jwk_public_key().try_into().unwrap(),
        response_uri,
        None,
    )
    .unwrap();
    let auth_request = iso_auth_request.clone().into();
    let auth_request_jws = SignedJwt::sign_with_certificate(&auth_request, &auth_keypair)
        .now_or_never()
        .unwrap()
        .unwrap();

    // Wallet receives the signed Authorization Request and performs the disclosure.
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let jwe = disclosure_jwe(&auth_request_jws.into(), &[ca.to_trust_anchor()], &issuer_ca);

    // RP decrypts the response JWE and verifies the contained Authorization Response.
    let disclosed_attestations = VpAuthorizationResponse::decrypt_and_verify(
        &jwe,
        &encryption_keypair,
        &iso_auth_request,
        &[MOCK_WALLET_CLIENT_ID.to_string()],
        &MockTimeGenerator::default(),
        &[issuer_ca.to_trust_anchor()],
        &ExtendingVctRetrieverStub,
        &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
            issuer_ca.generate_status_list_mock().unwrap(),
        ))),
        false,
    )
    .now_or_never()
    .unwrap()
    .unwrap();

    assert_disclosed_attestations_mdoc_pid(&disclosed_attestations);
}

/// The wallet side: verify the Authorization Request, gather the attestations and encrypt it into a JWE.
fn disclosure_jwe(
    auth_request: &UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>,
    trust_anchors: &[TrustAnchor<'_>],
    issuer_ca: &Ca,
) -> String {
    let mdoc_key = MockRemoteEcdsaKey::new(String::from("mdoc_key"), SigningKey::random(&mut OsRng));
    let partial_mdocs = vec_nonempty![PartialMdoc::new_mock_with_ca_and_key(issuer_ca, &mdoc_key)];
    let encryption_nonce = "encryption_nonce".to_string();

    // Verify the Authorization Request JWE and read the requested attributes.
    let (auth_request, cert) = VpAuthorizationRequest::try_new(auth_request, trust_anchors).unwrap();
    let auth_request = auth_request.validate(&cert, None).unwrap();

    // Compute the disclosure.
    let session_transcript = SessionTranscript::new_oid4vp(
        &auth_request.response_uri,
        &auth_request.client_id,
        auth_request.nonce.clone(),
        &encryption_nonce,
    );
    let wscd = MockRemoteWscd::new(vec![mdoc_key]);
    let poa_input = JwtPoaInput::new(Some(auth_request.nonce.clone()), auth_request.client_id.clone());
    let (device_responses, poa) =
        DeviceResponse::sign_multiple_from_partial_mdocs(partial_mdocs, &session_transcript, &wscd, poa_input)
            .now_or_never()
            .unwrap()
            .unwrap();

    // Put the disclosure in an Authorization Response and encrypt it.
    VpAuthorizationResponse::new_encrypted(
        HashMap::from([(
            "mdoc_pid_example".try_into().unwrap(),
            VerifiablePresentation::MsoMdoc(device_responses),
        )]),
        &auth_request,
        &encryption_nonce,
        poa,
    )
    .unwrap()
}

#[rstest]
#[case(nl_pid_credentials_all())]
#[case(nl_pid_address_credentials_all())]
#[case(nl_pid_credentials_all() + nl_pid_address_credentials_all())]
#[tokio::test]
async fn disclosure_using_message_client(
    #[case] test_credentials: TestCredentials,
    #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] format: CredentialFormat,
) {
    let formats = std::iter::repeat_n(format, test_credentials.as_ref().len()).collect_vec();

    let ca = Ca::generate("myca", Default::default()).unwrap();
    let rp_keypair = generate_reader_mock_with_registration(
        &ca,
        ReaderRegistration::mock_from_dcql_query(&test_credentials.to_dcql_query(formats.iter().copied())),
    )
    .unwrap();

    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuer_keypair = issuer_ca
        .generate_key_pair(PID_ISSUER_CERT_CN, CertificateUsage::Mdl, Default::default())
        .unwrap();

    // Initialize the "wallet"
    let wscd = MockRemoteWscd::default();
    let disclosable_attestations = match format {
        CredentialFormat::MsoMdoc => {
            DisclosableAttestations::MsoMdoc(test_credentials.to_partial_mdocs(&issuer_keypair, &wscd))
        }
        CredentialFormat::SdJwt => {
            DisclosableAttestations::SdJwt(test_credentials.to_unsigned_sd_jwt_presentations(&issuer_keypair, &wscd))
        }
    }
    .try_into()
    .unwrap();

    // Start a session at the "RP"
    let message_client = DirectMockVpMessageClient::new(
        test_credentials,
        formats,
        rp_keypair,
        vec![issuer_ca.to_trust_anchor().to_owned()],
        issuer_ca.generate_status_list_mock_with_dn(PID_ISSUER_CERT_CN).unwrap(),
    );
    let request_uri = message_client.start_session();

    // Perform the first part, which creates the disclosure session.
    let client = VpDisclosureClient::new(message_client);
    let session = client
        .start(&request_uri, DisclosureUriSource::Link, &[ca.to_trust_anchor()])
        .await
        .unwrap();

    // Finish the disclosure by sending the attestations to the "RP".
    session
        .disclose(disclosable_attestations, &wscd, &MockTimeGenerator::default())
        .await
        .unwrap();
}

/// A mock implementation of the `VpMessageClient` trait that implements the RP side of OpenID4VP
/// directly in its methods.
#[derive(Debug, Clone)]
struct DirectMockVpMessageClient {
    test_credentials: TestCredentials,
    formats: Vec<CredentialFormat>,
    encryption_keypair: EcKeyPair,
    auth_keypair: KeyPair,
    auth_request: NormalizedVpAuthorizationRequest,
    request_uri: BaseUrl,
    response_uri: BaseUrl,
    trust_anchors: Vec<TrustAnchor<'static>>,
    status_list_keypair: KeyPair,
}

impl DirectMockVpMessageClient {
    fn new(
        test_credentials: TestCredentials,
        formats: Vec<CredentialFormat>,
        auth_keypair: KeyPair,
        trust_anchors: Vec<TrustAnchor<'static>>,
        status_list_keypair: KeyPair,
    ) -> Self {
        let query = serde_urlencoded::to_string(VerifierUrlParameters {
            session_type: SessionType::SameDevice,
            ephemeral_id_params: Some(EphemeralIdParameters {
                ephemeral_id: vec![42],
                time: Utc::now(),
            }),
        })
        .unwrap();
        let request_uri = ("https://example.com/request_uri?".to_string() + &query)
            .parse()
            .unwrap();

        let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();

        let auth_request = NormalizedVpAuthorizationRequest::new(
            test_credentials.to_normalized_credential_requests(formats.iter().copied()),
            auth_keypair.certificate(),
            "nonce".to_string(),
            encryption_keypair.to_jwk_public_key().try_into().unwrap(),
            response_uri.clone(),
            None,
        )
        .unwrap();

        Self {
            test_credentials,
            formats,
            encryption_keypair,
            auth_keypair,
            auth_request,
            request_uri,
            response_uri,
            trust_anchors,
            status_list_keypair,
        }
    }

    fn start_session(&self) -> String {
        serde_urlencoded::to_string(VpRequestUriObject {
            request_uri: self.request_uri.clone(),
            client_id: String::from(self.auth_keypair.certificate().san_dns_name().unwrap().unwrap()),
            request_uri_method: Default::default(),
        })
        .unwrap()
    }
}

impl VpMessageClient for DirectMockVpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
        _request_nonce: Option<String>,
    ) -> Result<UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>, VpMessageClientError> {
        assert_eq!(url, self.request_uri);

        let jws = SignedJwt::sign_with_certificate(&self.auth_request.clone().into(), &self.auth_keypair)
            .await
            .unwrap()
            .into();
        Ok(jws)
    }

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        assert_eq!(url, self.response_uri);

        let disclosed_attestations = VpAuthorizationResponse::decrypt_and_verify(
            &jwe,
            &self.encryption_keypair,
            &self.auth_request,
            &[MOCK_WALLET_CLIENT_ID.to_string()],
            &MockTimeGenerator::default(),
            &self.trust_anchors,
            &ExtendingVctRetrieverStub,
            &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                self.status_list_keypair.clone(),
            ))),
            false,
        )
        .await
        .unwrap();

        self.test_credentials
            .assert_matches_disclosed_attestations(&disclosed_attestations, self.formats.iter().copied());

        Ok(None)
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        panic!("error: {error:?}")
    }
}

const NO_RETURN_URL_USE_CASE: &str = "no_return_url";
const DEFAULT_RETURN_URL_USE_CASE: &str = "default_return_url";
const ALL_RETURN_URL_USE_CASE: &str = "all_return_url";
const WALLET_INITIATED_RETURN_URL_USE_CASE: &str = "wallet_initiated_return_url";

#[derive(Debug)]
pub struct MockDisclosureResultHandler {
    pub key: Option<String>,
}

impl MockDisclosureResultHandler {
    pub fn new(key: Option<String>) -> Self {
        Self { key }
    }
}

#[async_trait]
impl DisclosureResultHandler for MockDisclosureResultHandler {
    async fn disclosure_result(
        &self,
        _usecase_id: &str,
        _disclosed: &UniqueIdVec<DisclosedAttestations>,
    ) -> Result<HashMap<String, String>, DisclosureResultHandlerError> {
        Ok(self
            .key
            .as_ref()
            .map(|key| HashMap::from([(key.clone(), "foobar".to_string())]))
            .unwrap_or_default())
    }
}

#[rstest]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    nl_pid_credentials_full_name()
)]
#[case(
    SessionType::SameDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    DEFAULT_RETURN_URL_USE_CASE,
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::SameDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    ALL_RETURN_URL_USE_CASE,
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::CrossDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    nl_pid_credentials_full_name()
)]
#[case(
    SessionType::CrossDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    DEFAULT_RETURN_URL_USE_CASE,
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::CrossDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    ALL_RETURN_URL_USE_CASE,
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    nl_pid_credentials_given_name()
)]
// attributes from different documents, so this case also tests the PoA
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    nl_pid_credentials_given_name() + nl_pid_address_minimal_address(),
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    nl_pid_credentials_given_name() + nl_pid_credentials_family_name(),
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    nl_pid_credentials_given_name() + nl_pid_credentials_full_name() + nl_pid_credentials_all(),
)]
#[tokio::test]
async fn test_client_and_server(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] use_case: &str,
    #[case] test_credentials: TestCredentials,
    #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] format: CredentialFormat,
    #[values(None, Some("query_param".to_string()))] result_query_param: Option<String>,
) {
    let formats = std::iter::repeat_n(format, test_credentials.as_ref().len()).collect_vec();
    let dcql_query = test_credentials.to_dcql_query(formats.iter().copied());

    let (verifier, rp_trust_anchor, issuer_keypair) = setup_verifier(&dcql_query, result_query_param.clone());

    // Start the session
    let session_token = verifier
        .new_session(use_case.to_string(), Some(dcql_query), return_url_template)
        .await
        .unwrap();

    // frontend receives the UL to feed to the wallet when fetching the session status
    let request_uri = request_uri_from_status_endpoint(&verifier, &session_token, session_type).await;

    // Determine the correct source for the session type
    let uri_source = match session_type {
        SessionType::SameDevice => DisclosureUriSource::Link,
        SessionType::CrossDevice => DisclosureUriSource::QrCode,
    };

    // Start session in the wallet
    let wscd = MockRemoteWscd::default();
    let session = start_disclosure_session(Arc::clone(&verifier), uri_source, &request_uri, rp_trust_anchor)
        .await
        .unwrap();

    // Finish the disclosure.
    let disclosable_attestations = match format {
        CredentialFormat::MsoMdoc => {
            DisclosableAttestations::MsoMdoc(test_credentials.to_partial_mdocs(&issuer_keypair, &wscd))
        }
        CredentialFormat::SdJwt => {
            DisclosableAttestations::SdJwt(test_credentials.to_unsigned_sd_jwt_presentations(&issuer_keypair, &wscd))
        }
    }
    .try_into()
    .unwrap();
    let redirect_uri = session
        .disclose(disclosable_attestations, &wscd, &MockTimeGenerator::default())
        .await
        .unwrap();

    // Check if we received a redirect URI when we should have, based on the use case and session type.
    let should_have_redirect_uri = match (use_case, session_type) {
        (use_case, _) if use_case == NO_RETURN_URL_USE_CASE => false,
        (use_case, _) if use_case == ALL_RETURN_URL_USE_CASE => true,
        (_, SessionType::SameDevice) => true,
        (_, SessionType::CrossDevice) => false,
    };
    assert_eq!(redirect_uri.is_some(), should_have_redirect_uri);

    let redirect_uri_query_pairs: IndexMap<String, String> = redirect_uri
        .as_ref()
        .map(|uri| {
            uri.as_ref()
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        })
        .unwrap_or_default();

    if redirect_uri.is_some() && result_query_param.is_some_and(|param| !redirect_uri_query_pairs.contains_key(&param))
    {
        panic!("expected query parameter not found in redirect URI");
    }

    let redirect_uri_nonce = redirect_uri_query_pairs.get("nonce").cloned();

    // If we have a redirect URI (nonce), then fetching the attributes without a nonce or with a wrong one should fail.
    if redirect_uri_nonce.is_some() {
        let error = verifier
            .disclosed_attributes(&session_token, None)
            .await
            .expect_err("fetching disclosed attributes without a return URL nonce should fail");
        assert_matches!(error, DisclosedAttributesError::RedirectUriNonceMissing);

        let error = verifier
            .disclosed_attributes(&session_token, "incorrect".to_string().into())
            .await
            .expect_err("fetching disclosed attributes with incorrect return URL nonce should fail");
        assert_matches!(
            error,
            DisclosedAttributesError::RedirectUriNonceMismatch(nonce) if nonce == "incorrect"
        );
    }

    // Retrieve the attributes disclosed by the wallet
    let disclosed_attestations = verifier
        .disclosed_attributes(&session_token, redirect_uri_nonce)
        .await
        .unwrap();

    test_credentials.assert_matches_disclosed_attestations(&disclosed_attestations, formats.iter().copied());
}

#[tokio::test]
async fn test_client_and_server_cancel_after_created() {
    let dcql_query = Query::new_mock_mdoc_pid_example();
    let session_type = SessionType::SameDevice;

    let (verifier, trust_anchor, _issuer_ca) = setup_verifier(&dcql_query, None);

    // Start the session
    let session_token = verifier
        .new_session(
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            Some(dcql_query),
            Some(ReturnUrlTemplate::from_str("https://example.com/redirect_uri/{session_token}").unwrap()),
        )
        .await
        .unwrap();

    // The front-end receives the UL to feed to the wallet when fetching the session status
    // (this also verifies that the status is Created)
    let request_uri = request_uri_from_status_endpoint(&verifier, &session_token, session_type).await;

    // Cancel the session
    verifier
        .cancel(&session_token)
        .await
        .expect("should be able to cancel newly created session");

    // The session should now be cancelled
    let status_response = request_status_endpoint(&verifier, &session_token, None).await;

    assert_matches!(status_response, StatusResponse::Cancelled);

    // Starting the session in the wallet should result in an error
    let Err(error) = start_disclosure_session(
        Arc::clone(&verifier),
        DisclosureUriSource::Link,
        &request_uri,
        trust_anchor,
    )
    .await
    else {
        panic!("should not be able to start the disclosure session in the wallet")
    };

    assert_matches!(
        error,
        VpSessionError::Client(VpClientError::Request(VpMessageClientError::AuthGetResponse(error)))
            if error.error_response.error == GetRequestErrorCode::CancelledSession
    );
}

#[tokio::test]
async fn test_client_and_server_cancel_after_wallet_start() {
    let test_credentials = nl_pid_credentials_all();
    let dcql_query = test_credentials.to_dcql_query([CredentialFormat::SdJwt]);

    let (verifier, trust_anchor, issuer_keypair) = setup_verifier(&dcql_query, None);

    // Start the session
    let session_token = verifier
        .new_session(
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            Some(dcql_query),
            Some(ReturnUrlTemplate::from_str("https://example.com/redirect_uri/{session_token}").unwrap()),
        )
        .await
        .unwrap();

    // The front-end receives the UL to feed to the wallet when fetching the session status
    // (this also verifies that the status is Created)
    let request_uri = request_uri_from_status_endpoint(&verifier, &session_token, SessionType::SameDevice).await;

    // Start session in the wallet
    let wscd = MockRemoteWscd::default();
    let session = start_disclosure_session(
        Arc::clone(&verifier),
        DisclosureUriSource::Link,
        &request_uri,
        trust_anchor,
    )
    .await
    .unwrap();

    // Cancel the session
    verifier
        .cancel(&session_token)
        .await
        .expect("should be able to cancel session that is waiting for response");

    // The session should now be cancelled
    let status_response = request_status_endpoint(&verifier, &session_token, None).await;

    assert_matches!(status_response, StatusResponse::Cancelled);

    // Disclosing attributes at this point should result in an error.
    let presentations = test_credentials.to_unsigned_sd_jwt_presentations(&issuer_keypair, &wscd);
    let disclosable_attestations = DisclosableAttestations::SdJwt(presentations).try_into().unwrap();
    let (_session, error) = session
        .disclose(disclosable_attestations, &wscd, &MockTimeGenerator::default())
        .await
        .expect_err("should not be able to disclose attributes");

    assert_matches!(
        error.error,
        VpSessionError::Client(VpClientError::Request(VpMessageClientError::AuthPostResponse(error)))
            if error.error_response.error == PostAuthResponseErrorCode::CancelledSession
    );
}

#[tokio::test]
async fn test_disclosure_invalid_poa() {
    /// A mock WSCD that returns a wrong PoA.
    #[derive(Default)]
    struct WrongPoaWscd(MockRemoteWscd);

    impl DisclosureWscd for WrongPoaWscd {
        type Key = MockRemoteEcdsaKey;
        type Error = MockRemoteWscdError;
        type Poa = Poa;

        fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
            self.0.new_key(identifier, public_key)
        }

        async fn sign(
            &self,
            messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
            poa_input: <Self::Poa as WscdPoa>::Input,
        ) -> Result<DisclosureResult<Self::Poa>, Self::Error> {
            let mut result = self.0.sign(messages_and_keys, poa_input).await?;

            result.poa.as_mut().unwrap().set_payload("wrong_payload".to_string());

            Ok(result)
        }
    }

    impl AsRef<DisclosureMockRemoteWscd> for WrongPoaWscd {
        fn as_ref(&self) -> &DisclosureMockRemoteWscd {
            let Self(inner) = self;

            inner.as_ref()
        }
    }

    let test_credentials = nl_pid_credentials_full_name() + nl_pid_address_minimal_address();
    let dcql_query = test_credentials.to_dcql_query([CredentialFormat::SdJwt, CredentialFormat::SdJwt]);
    let use_case = NO_RETURN_URL_USE_CASE;

    let (verifier, rp_trust_anchor, issuer_keypair) = setup_verifier(&dcql_query, None);

    // Start the session
    let session_token = verifier
        .new_session(use_case.to_string(), Some(dcql_query), None)
        .await
        .unwrap();

    // frontend receives the UL to feed to the wallet when fetching the session status
    let request_uri = request_uri_from_status_endpoint(&verifier, &session_token, SessionType::SameDevice).await;

    // Start session in the wallet
    let wscd = WrongPoaWscd::default();
    let session = start_disclosure_session(
        Arc::clone(&verifier),
        DisclosureUriSource::Link,
        &request_uri,
        rp_trust_anchor,
    )
    .await
    .unwrap();

    // Finish the disclosure.
    let presentations = test_credentials.to_unsigned_sd_jwt_presentations(&issuer_keypair, &wscd);
    let disclosable_attestations = DisclosableAttestations::SdJwt(presentations).try_into().unwrap();
    let (_session, error) = session
        .disclose(disclosable_attestations, &wscd, &MockTimeGenerator::default())
        .await
        .expect_err("should not be able to disclose attributes");
    assert_matches!(
        error.error,
        VpSessionError::Client(VpClientError::Request(VpMessageClientError::AuthPostResponse(error)))
            if error.error_response.error == PostAuthResponseErrorCode::InvalidRequest
    );
}

#[tokio::test]
async fn test_wallet_initiated_usecase_verifier() {
    let (verifier, test_credentials, rp_trust_anchor, issuer_keypair) = setup_wallet_initiated_usecase_verifier();

    let mut request_uri: Url = format!("https://example.com/{WALLET_INITIATED_RETURN_URL_USE_CASE}/request_uri")
        .parse()
        .unwrap();
    request_uri.set_query(Some(
        &serde_urlencoded::to_string(VerifierUrlParameters {
            session_type: SessionType::SameDevice,
            ephemeral_id_params: None,
        })
        .unwrap(),
    ));

    let universal_link_query = serde_urlencoded::to_string(VpRequestUriObject {
        request_uri: request_uri.try_into().unwrap(),
        request_uri_method: Some(RequestUriMethod::POST),
        client_id: RP_CERT_CN.to_string(),
    })
    .unwrap();

    let wscd = MockRemoteWscd::default();

    let session = start_disclosure_session(
        verifier,
        DisclosureUriSource::Link,
        &universal_link_query,
        rp_trust_anchor,
    )
    .await
    .unwrap();

    // Do the disclosure
    let presentations = test_credentials.to_unsigned_sd_jwt_presentations(&issuer_keypair, &wscd);
    let disclosable_attestations = DisclosableAttestations::SdJwt(presentations).try_into().unwrap();
    session
        .disclose(disclosable_attestations, &wscd, &MockTimeGenerator::default())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_wallet_initiated_usecase_verifier_cancel() {
    let (verifier, _test_credentials, rp_trust_anchor, _issuer_keypair) = setup_wallet_initiated_usecase_verifier();

    let mut request_uri: Url = format!("https://example.com/{WALLET_INITIATED_RETURN_URL_USE_CASE}/request_uri")
        .parse()
        .unwrap();
    request_uri.set_query(Some(
        &serde_urlencoded::to_string(VerifierUrlParameters {
            session_type: SessionType::SameDevice,
            ephemeral_id_params: None,
        })
        .unwrap(),
    ));

    let universal_link_query = serde_urlencoded::to_string(VpRequestUriObject {
        request_uri: request_uri.try_into().unwrap(),
        request_uri_method: Some(RequestUriMethod::POST),
        client_id: RP_CERT_CN.to_string(),
    })
    .unwrap();

    let session = start_disclosure_session(
        verifier,
        DisclosureUriSource::Link,
        &universal_link_query,
        rp_trust_anchor,
    )
    .await
    .unwrap();

    // Cancel and verify that we don't get a return URL
    assert!(session.terminate().await.unwrap().is_none());
}

#[tokio::test]
async fn test_rp_initiated_usecase_verifier_cancel() {
    let dcql_query = nl_pid_credentials_full_name().to_dcql_query([CredentialFormat::SdJwt]);

    let (verifier, rp_trust_anchor, _issuer_keypair) = setup_verifier(&dcql_query, None);

    // Start the session
    let session_token = verifier
        .new_session(
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            Some(dcql_query),
            Some(ReturnUrlTemplate::from_str("https://example.com/redirect_uri/{session_token}").unwrap()),
        )
        .await
        .unwrap();

    // The front-end receives the UL to feed to the wallet when fetching the session status
    // (this also verifies that the status is Created)
    let request_uri = request_uri_from_status_endpoint(&verifier, &session_token, SessionType::SameDevice).await;

    // Start session in the wallet
    let session = start_disclosure_session(
        Arc::clone(&verifier),
        DisclosureUriSource::Link,
        &request_uri,
        rp_trust_anchor,
    )
    .await
    .unwrap();

    // Cancel and verify that we don't get a return URL
    assert!(session.terminate().await.unwrap().is_some());
}

#[tokio::test]
async fn test_rp_initiated_usecase_verifier_disclose_extending_credential() {
    let query_id = "eudi_pid_given_name";
    let test_credentials = nl_pid_credentials_given_name_for_query_id(query_id);
    let mut dcql_query: Query = NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
        &[EUDI_PID_ATTESTATION_TYPE],
        &[&[PID_GIVEN_NAME]],
    )])
    .into();
    dcql_query.credentials = dcql_query
        .credentials
        .into_iter()
        .map(|mut query| {
            query.id = CredentialQueryIdentifier::try_new(String::from(query_id)).unwrap();
            query
        })
        .collect_vec()
        .try_into()
        .unwrap();

    let (verifier, rp_trust_anchor, issuer_keypair) = setup_verifier(&dcql_query, None);

    // Start the session
    let session_token = verifier
        .new_session(
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            Some(dcql_query),
            Some(ReturnUrlTemplate::from_str("https://example.com/redirect_uri/{session_token}").unwrap()),
        )
        .await
        .unwrap();

    // The front-end receives the UL to feed to the wallet when fetching the session status
    // (this also verifies that the status is Created)
    let request_uri = request_uri_from_status_endpoint(&verifier, &session_token, SessionType::SameDevice).await;

    let wscd = MockRemoteWscd::default();

    // Start session in the wallet
    let session = start_disclosure_session(
        Arc::clone(&verifier),
        DisclosureUriSource::Link,
        &request_uri,
        rp_trust_anchor,
    )
    .await
    .unwrap();

    // Do the disclosure
    let presentations = test_credentials.to_unsigned_sd_jwt_presentations(&issuer_keypair, &wscd);
    let disclosable_attestations = DisclosableAttestations::SdJwt(presentations).try_into().unwrap();
    session
        .disclose(disclosable_attestations, &wscd, &MockTimeGenerator::default())
        .await
        .unwrap()
        .unwrap();
}

fn setup_wallet_initiated_usecase_verifier() -> (
    Arc<MockWalletInitiatedUseCaseVerifier>,
    TestCredentials,
    TrustAnchor<'static>,
    KeyPair,
) {
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();

    let issuer_keypair = issuer_ca
        .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
        .unwrap();

    // Initialize the verifier
    let test_credentials = nl_pid_credentials_full_name();
    let dcql_query = test_credentials.to_dcql_query([CredentialFormat::SdJwt]);
    let reader_registration = ReaderRegistration::mock_from_dcql_query(&dcql_query);
    let usecases = HashMap::from([(
        WALLET_INITIATED_RETURN_URL_USE_CASE.to_string(),
        WalletInitiatedUseCase::try_new(
            generate_reader_mock_with_registration(&rp_ca, reader_registration.clone()).unwrap(),
            SessionTypeReturnUrl::SameDevice,
            dcql_query.try_into().unwrap(),
            "https://example.com/redirect_uri".parse().unwrap(),
        )
        .unwrap(),
    )]);

    let verifier = Arc::new(MockWalletInitiatedUseCaseVerifier::new(
        WalletInitiatedUseCases::new(usecases),
        Arc::new(MemorySessionStore::default()),
        vec![issuer_ca.to_trust_anchor().to_owned()],
        Some(Box::new(MockDisclosureResultHandler::new(None))),
        vec![MOCK_WALLET_CLIENT_ID.to_string()],
        HashMap::default(),
        RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
            issuer_ca.generate_status_list_mock_with_dn(ISSUANCE_CERT_CN).unwrap(),
        ))),
    ));

    (
        verifier,
        test_credentials,
        rp_ca.to_trust_anchor().to_owned(),
        issuer_keypair,
    )
}

fn setup_verifier(
    dcql_query: &Query,
    session_result_query_param: Option<String>,
) -> (Arc<MockRpInitiatedUseCaseVerifier>, TrustAnchor<'static>, KeyPair) {
    // Initialize key material
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();

    let issuer_keypair = issuer_ca
        .generate_key_pair(PID_ISSUER_CERT_CN, CertificateUsage::Mdl, Default::default())
        .unwrap();

    // Initialize the verifier
    let reader_registration = ReaderRegistration::mock_from_dcql_query(dcql_query);
    let usecases = HashMap::from([
        (
            NO_RETURN_URL_USE_CASE.to_string(),
            RpInitiatedUseCase::try_new(
                generate_reader_mock_with_registration(&rp_ca, reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::Neither,
                None,
                None,
                false,
            )
            .unwrap(),
        ),
        (
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            RpInitiatedUseCase::try_new(
                generate_reader_mock_with_registration(&rp_ca, reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::SameDevice,
                None,
                None,
                false,
            )
            .unwrap(),
        ),
        (
            ALL_RETURN_URL_USE_CASE.to_string(),
            RpInitiatedUseCase::try_new(
                generate_reader_mock_with_registration(&rp_ca, reader_registration).unwrap(),
                SessionTypeReturnUrl::Both,
                None,
                None,
                false,
            )
            .unwrap(),
        ),
    ]);

    let sessions = Arc::new(MemorySessionStore::default());

    let usecases = RpInitiatedUseCases::new(
        usecases,
        hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
        Arc::clone(&sessions),
    );

    let verifier = Arc::new(MockRpInitiatedUseCaseVerifier::new(
        usecases,
        sessions,
        vec![issuer_ca.to_trust_anchor().to_owned()],
        Some(Box::new(MockDisclosureResultHandler::new(session_result_query_param))),
        vec![MOCK_WALLET_CLIENT_ID.to_string()],
        HashMap::from([(
            String::from(EUDI_PID_ATTESTATION_TYPE),
            vec_nonempty![String::from(PID_ATTESTATION_TYPE)],
        )]),
        RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
            issuer_ca.generate_status_list_mock_with_dn(PID_ISSUER_CERT_CN).unwrap(),
        ))),
    ));

    (verifier, rp_ca.to_trust_anchor().to_owned(), issuer_keypair)
}

async fn start_disclosure_session<US, UC>(
    verifier: Arc<MockVerifier<US>>,
    uri_source: DisclosureUriSource,
    request_uri: &str,
    trust_anchor: TrustAnchor<'static>,
) -> Result<VpDisclosureSession<VerifierMockVpMessageClient<MockVerifier<US>>>, VpSessionError>
where
    US: UseCases<UseCase = UC, Key = SigningKey>,
    UC: UseCase<Key = SigningKey>,
{
    let client = VpDisclosureClient::new(VerifierMockVpMessageClient::new(verifier));

    // Start session in the wallet
    client.start(request_uri, uri_source, &[trust_anchor]).await
}

async fn request_uri_from_status_endpoint(
    verifier: &MockRpInitiatedUseCaseVerifier,
    session_token: &SessionToken,
    session_type: SessionType,
) -> String {
    let StatusResponse::Created { ul: Some(ul) } =
        request_status_endpoint(verifier, session_token, Some(session_type)).await
    else {
        panic!("unexpected state")
    };

    ul.as_ref().query().unwrap().to_string()
}

async fn request_status_endpoint(
    verifier: &MockRpInitiatedUseCaseVerifier,
    session_token: &SessionToken,
    session_type: Option<SessionType>,
) -> StatusResponse {
    verifier
        .status_response(
            session_token,
            session_type,
            &"https://example.com/ul".parse().unwrap(),
            format!("https://example.com/verifier_base_url/{session_token}/request_uri")
                .parse()
                .unwrap(),
            &TimeGenerator,
        )
        .await
        .unwrap()
}

type MockRpInitiatedUseCaseVerifier = MockVerifier<RpInitiatedUseCases<SigningKey, MemorySessionStore<DisclosureData>>>;
type MockWalletInitiatedUseCaseVerifier = MockVerifier<WalletInitiatedUseCases<SigningKey>>;
type MockVerifier<T> = Verifier<MemorySessionStore<DisclosureData>, T, StatusListClientStub<SigningKey>>;

#[derive(Debug)]
struct VerifierMockVpMessageClient<T> {
    verifier: Arc<T>,
}

impl<T> VerifierMockVpMessageClient<T> {
    pub fn new(verifier: Arc<T>) -> Self {
        VerifierMockVpMessageClient { verifier }
    }
}

impl<T> Clone for VerifierMockVpMessageClient<T> {
    fn clone(&self) -> Self {
        Self {
            verifier: Arc::clone(&self.verifier),
        }
    }
}

impl<US, UC> VpMessageClient for VerifierMockVpMessageClient<MockVerifier<US>>
where
    US: UseCases<UseCase = UC, Key = SigningKey>,
    UC: UseCase<Key = SigningKey>,
{
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>, VpMessageClientError> {
        let path_segments = url.as_ref().path_segments().unwrap().collect_vec();
        let session_token: SessionToken = path_segments[path_segments.len() - 2].to_owned().into();

        let jws = self
            .verifier
            .process_get_request(
                session_token.as_ref(),
                &"https://example.com/verifier_base_url".parse().unwrap(),
                url.as_ref().query(),
                wallet_nonce,
            )
            .await
            .map_err(|error| VpMessageClientError::AuthGetResponse(Box::new(error.into())))?;

        Ok(jws.into())
    }

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        let path_segments = url.as_ref().path_segments().unwrap().collect_vec();
        let session_token = path_segments[path_segments.len() - 2].to_owned().into();

        let response = self
            .verifier
            .process_authorization_response(
                &session_token,
                WalletAuthResponse::Response(VpToken { vp_token: jwe }),
                &TimeGenerator,
            )
            .await
            .map_err(|error| VpMessageClientError::AuthPostResponse(Box::new(error.into())))?;

        Ok(response.redirect_uri)
    }

    async fn send_error(
        &self,
        url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        let path_segments = url.as_ref().path_segments().unwrap().collect_vec();
        let session_token = path_segments[path_segments.len() - 2].to_owned().into();

        let response = self
            .verifier
            .process_authorization_response(&session_token, WalletAuthResponse::Error(error), &TimeGenerator)
            .await
            .map_err(|error| VpMessageClientError::AuthPostResponse(Box::new(error.into())))?;

        Ok(response.redirect_uri)
    }
}
