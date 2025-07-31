#![expect(clippy::too_many_arguments)]

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
use p256::ecdsa::Signature;
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
use attestation_data::disclosure::DisclosedAttestation;
use attestation_data::x509::generate::mock::generate_reader_mock;
use crypto::factory::KeyFactory;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteKeyFactory;
use crypto::mock_remote::MockRemoteKeyFactoryError;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::server_keys::generate::mock::RP_CERT_CN;
use dcql::Query;
use dcql::normalized;
use http_utils::urls::BaseUrl;
use jwt::Jwt;
use mdoc::DeviceResponse;
use mdoc::SessionTranscript;
use mdoc::holder::Mdoc;
use mdoc::test::TestDocuments;
use mdoc::test::data::PID;
use mdoc::test::data::addr_street;
use mdoc::test::data::pid_full_name;
use mdoc::test::data::pid_given_name;
use openid4vc::ErrorResponse;
use openid4vc::GetRequestErrorCode;
use openid4vc::PostAuthResponseErrorCode;
use openid4vc::VpAuthorizationErrorCode;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::DisclosureUriSource;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::disclosure_session::VpDisclosureSession;
use openid4vc::disclosure_session::VpMessageClient;
use openid4vc::disclosure_session::VpMessageClientError;
use openid4vc::disclosure_session::VpSessionError;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::mock::test_document_to_mdoc;
use openid4vc::openid4vp::NormalizedVpAuthorizationRequest;
use openid4vc::openid4vp::RequestUriMethod;
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
use poa::Poa;
use poa::factory::PoaFactory;
use utils::generator::TimeGenerator;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_at_least::VecAtLeastTwoUnique;
use utils::vec_at_least::VecNonEmpty;

#[tokio::test]
async fn disclosure_direct() {
    let ca = Ca::generate("myca", Default::default()).unwrap();
    let auth_keypair = generate_reader_mock(&ca, None).unwrap();

    // RP assembles the Authorization Request and signs it into a JWS.
    let nonce = "nonce".to_string();
    let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
    let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let iso_auth_request = NormalizedVpAuthorizationRequest::new(
        normalized::mock::new_pid_example(),
        auth_keypair.certificate(),
        nonce.clone(),
        encryption_keypair.to_jwk_public_key().try_into().unwrap(),
        response_uri,
        None,
    )
    .unwrap();
    let auth_request = iso_auth_request.clone().into();
    let auth_request_jws = Jwt::sign_with_certificate(&auth_request, &auth_keypair).await.unwrap();

    // Wallet receives the signed Authorization Request and performs the disclosure.
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let jwe = disclosure_jwe(auth_request_jws, &[ca.to_trust_anchor()], &issuer_ca).await;

    // RP decrypts the response JWE and verifies the contained Authorization Response.
    let (auth_response, mdoc_nonce) = VpAuthorizationResponse::decrypt(&jwe, &encryption_keypair, &nonce).unwrap();
    let disclosed_attrs = auth_response
        .verify(
            &iso_auth_request,
            &[MOCK_WALLET_CLIENT_ID.to_string()],
            &mdoc_nonce,
            &MockTimeGenerator::default(),
            &[issuer_ca.to_trust_anchor()],
        )
        .unwrap();

    assert_eq!(
        disclosed_attrs[PID].attributes.clone().unwrap_mso_mdoc()[PID]["family_name"],
        AttributeValue::Text("De Bruijn".to_owned()),
    );
}

/// The wallet side: verify the Authorization Request, gather the attestations and encrypt it into a JWE.
async fn disclosure_jwe(
    auth_request: Jwt<VpAuthorizationRequest>,
    trust_anchors: &[TrustAnchor<'_>],
    issuer_ca: &Ca,
) -> String {
    let mdoc_key = MockRemoteEcdsaKey::new(String::from("mdoc_key"), SigningKey::random(&mut OsRng));
    let mdocs = vec![Mdoc::new_mock_with_ca_and_key(issuer_ca, &mdoc_key).await];
    let mdoc_nonce = "mdoc_nonce".to_string();

    // Verify the Authorization Request JWE and read the requested attributes.
    let (auth_request, cert) = VpAuthorizationRequest::try_new(&auth_request, trust_anchors).unwrap();
    let auth_request = auth_request.validate(&cert, None).unwrap();

    // Compute the disclosure.
    let session_transcript = SessionTranscript::new_oid4vp(
        &auth_request.response_uri,
        &auth_request.client_id,
        auth_request.nonce.clone(),
        &mdoc_nonce,
    );
    let key_factory = MockRemoteKeyFactory::new(vec![mdoc_key]);
    let (device_response, keys) = DeviceResponse::sign_from_mdocs(mdocs, &session_transcript, &key_factory)
        .await
        .unwrap();

    let poa = match VecAtLeastTwoUnique::try_from(keys) {
        Ok(keys) => {
            let keys = keys.as_slice().iter().collect_vec().try_into().unwrap();
            let poa = key_factory
                .poa(keys, auth_request.client_id.clone(), Some(mdoc_nonce.clone()))
                .await
                .unwrap();
            Some(poa)
        }
        Err(_) => None,
    };

    // Put the disclosure in an Authorization Response and encrypt it.
    VpAuthorizationResponse::new_encrypted(device_response, &auth_request, &mdoc_nonce, poa).unwrap()
}

#[tokio::test]
async fn disclosure_using_message_client() {
    let ca = Ca::generate("myca", Default::default()).unwrap();
    let rp_keypair = generate_reader_mock(
        &ca,
        Some(ReaderRegistration::mock_from_dcql_query(&Query::new_pid_example())),
    )
    .unwrap();

    // Initialize the "wallet"
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();

    let mdoc_key = MockRemoteEcdsaKey::new(String::from("mdoc_key"), SigningKey::random(&mut OsRng));
    let mdocs = vec![Mdoc::new_mock_with_ca_and_key(&issuer_ca, &mdoc_key).await]
        .try_into()
        .unwrap();

    // Start a session at the "RP"
    let message_client = DirectMockVpMessageClient::new(rp_keypair, vec![issuer_ca.to_trust_anchor().to_owned()]);
    let request_uri = message_client.start_session();

    // Perform the first part, which creates the disclosure session.
    let client = VpDisclosureClient::new(message_client);
    let session = client
        .start(&request_uri, DisclosureUriSource::Link, &[ca.to_trust_anchor()])
        .await
        .unwrap();

    // Finish the disclosure by sending the attestations to the "RP".
    let key_factory = MockRemoteKeyFactory::new(vec![mdoc_key]);
    session.disclose(mdocs, &key_factory).await.unwrap();
}

/// A mock implementation of the `VpMessageClient` trait that implements the RP side of OpenID4VP
/// directly in its methods.
#[derive(Debug, Clone)]
struct DirectMockVpMessageClient {
    nonce: String,
    encryption_keypair: EcKeyPair,
    auth_keypair: KeyPair,
    auth_request: VpAuthorizationRequest,
    request_uri: BaseUrl,
    response_uri: BaseUrl,
    trust_anchors: Vec<TrustAnchor<'static>>,
}

impl DirectMockVpMessageClient {
    fn new(auth_keypair: KeyPair, trust_anchors: Vec<TrustAnchor<'static>>) -> Self {
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

        let nonce = "nonce".to_string();
        let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();

        let auth_request = NormalizedVpAuthorizationRequest::new(
            normalized::mock::new_pid_example(),
            auth_keypair.certificate(),
            nonce.clone(),
            encryption_keypair.to_jwk_public_key().try_into().unwrap(),
            response_uri.clone(),
            None,
        )
        .unwrap()
        .into();

        Self {
            nonce,
            encryption_keypair,
            auth_keypair,
            auth_request,
            request_uri,
            response_uri,
            trust_anchors,
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
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        assert_eq!(url, self.request_uri);

        let jws = Jwt::sign_with_certificate(&self.auth_request, &self.auth_keypair)
            .await
            .unwrap();
        Ok(jws)
    }

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        assert_eq!(url, self.response_uri);

        let (auth_response, mdoc_nonce) =
            VpAuthorizationResponse::decrypt(&jwe, &self.encryption_keypair, &self.nonce).unwrap();
        let disclosed_attrs = auth_response
            .verify(
                &self.auth_request.clone().try_into().unwrap(),
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &mdoc_nonce,
                &MockTimeGenerator::default(),
                &self.trust_anchors,
            )
            .unwrap();

        assert_eq!(
            disclosed_attrs[PID].attributes.clone().unwrap_mso_mdoc()[PID]["family_name"],
            AttributeValue::Text("De Bruijn".to_owned()),
        );

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
        _disclosed: &IndexMap<String, DisclosedAttestation>,
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
    pid_full_name(),
    pid_full_name().into(),
    pid_full_name()
)]
#[case(
    SessionType::SameDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    DEFAULT_RETURN_URL_USE_CASE,
    pid_full_name(),
    pid_full_name().into(),
    pid_full_name()
)]
#[case(
    SessionType::SameDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    ALL_RETURN_URL_USE_CASE,
    pid_full_name(),
    pid_full_name().into(),
    pid_full_name()
)]
#[case(
    SessionType::CrossDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_full_name(),
    pid_full_name().into(),
    pid_full_name()
)]
#[case(
    SessionType::CrossDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    DEFAULT_RETURN_URL_USE_CASE,
    pid_full_name(),
    pid_full_name().into(),
    pid_full_name()
)]
#[case(
    SessionType::CrossDevice,
    Some("https://example.com/return_url".parse().unwrap()),
    ALL_RETURN_URL_USE_CASE,
    pid_full_name(),
    pid_full_name().into(),
    pid_full_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_full_name(),
    pid_given_name().into(),
    pid_given_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_given_name(),
    pid_given_name().into(),
    pid_given_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_given_name() + addr_street(),
    (pid_given_name() + addr_street()).into(),
    pid_given_name() + addr_street()
)]
// attributes from different documents, so this case also tests the PoA
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_given_name() + addr_street(),
    (pid_given_name() + addr_street()).into(),
    pid_given_name() + addr_street()
)]
// TODO (PVW-4705): Re-enable this test once the attestation type limit for disclosure has been removed.
// #[case(
//     SessionType::SameDevice,
//     None,
//     NO_RETURN_URL_USE_CASE,
//     pid_full_name() + pid_full_name(),
//     (pid_given_name() + pid_given_name()).into(),
//     pid_given_name() + pid_given_name()
// )]
// #[case(
//     SessionType::SameDevice,
//     None,
//     NO_RETURN_URL_USE_CASE,
//     pid_given_name() + pid_given_name(),
//     (pid_given_name() + pid_given_name()).into(),
//     pid_given_name() + pid_given_name()
// )]
#[tokio::test]
async fn test_client_and_server(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] use_case: &str,
    #[case] stored_documents: TestDocuments,
    #[case] dcql_query: Query,
    #[case] expected_documents: TestDocuments,
    #[values(None, Some("query_param".to_string()))] result_query_param: Option<String>,
) {
    let (verifier, rp_trust_anchor, issuer_ca) = setup_verifier(&dcql_query, result_query_param.clone());

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
    let key_factory = MockRemoteKeyFactory::default();
    let session = start_disclosure_session(Arc::clone(&verifier), uri_source, &request_uri, rp_trust_anchor)
        .await
        .unwrap();

    // Finish the disclosure.
    let mdocs = test_documents_to_mdocs(stored_documents, &issuer_ca, &key_factory);
    let redirect_uri = session.disclose(mdocs, &key_factory).await.unwrap();

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
    let disclosed_documents = verifier
        .disclosed_attributes(&session_token, redirect_uri_nonce)
        .await
        .unwrap();

    expected_documents.assert_matches(
        &disclosed_documents
            .into_iter()
            .map(|(credential_type, attributes)| (credential_type, attributes.into()))
            .collect(),
    );
}

#[tokio::test]
async fn test_client_and_server_cancel_after_created() {
    let dcql_query = Query::pid_full_name();
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
    let stored_documents = pid_full_name();
    let dcql_query = Query::pid_full_name();
    let session_type = SessionType::SameDevice;

    let (verifier, trust_anchor, issuer_ca) = setup_verifier(&dcql_query, None);

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

    // Start session in the wallet
    let key_factory = MockRemoteKeyFactory::default();
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
    let mdocs = test_documents_to_mdocs(stored_documents, &issuer_ca, &key_factory);
    let (_session, error) = session
        .disclose(mdocs, &key_factory)
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
    /// A mock key factory that returns a wrong PoA.
    #[derive(Default)]
    struct WrongPoaKeyFactory(MockRemoteKeyFactory);
    impl KeyFactory for WrongPoaKeyFactory {
        type Key = MockRemoteEcdsaKey;
        type Error = MockRemoteKeyFactoryError;

        async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
            self.0.generate_new_multiple(count).await
        }

        fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
            self.0.generate_existing(identifier, public_key)
        }

        async fn sign_with_new_keys(&self, _: Vec<u8>, _: u64) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
            unimplemented!()
        }

        async fn sign_multiple_with_existing_keys(
            &self,
            messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        ) -> Result<Vec<Vec<Signature>>, Self::Error> {
            self.0.sign_multiple_with_existing_keys(messages_and_keys).await
        }
    }

    impl PoaFactory for WrongPoaKeyFactory {
        type Key = MockRemoteEcdsaKey;
        type Error = MockRemoteKeyFactoryError;

        async fn poa(
            &self,
            keys: VecAtLeastTwoUnique<&Self::Key>,
            _: String,
            _: Option<String>,
        ) -> Result<Poa, Self::Error> {
            self.0.poa(keys, "".to_owned(), Some("".to_owned())).await
        }
    }

    let stored_documents = pid_full_name() + addr_street();
    let dcql_query: Query = (pid_given_name() + addr_street()).into();
    let session_type = SessionType::SameDevice;
    let use_case = NO_RETURN_URL_USE_CASE;

    let (verifier, rp_trust_anchor, issuer_ca) = setup_verifier(&dcql_query, None);

    // Start the session
    let session_token = verifier
        .new_session(use_case.to_string(), Some(dcql_query), None)
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
    let key_factory = WrongPoaKeyFactory::default();
    let session = start_disclosure_session(Arc::clone(&verifier), uri_source, &request_uri, rp_trust_anchor)
        .await
        .unwrap();

    // Finish the disclosure.
    let mdocs = test_documents_to_mdocs(stored_documents, &issuer_ca, &key_factory);
    let (_session, error) = session
        .disclose(mdocs, &key_factory)
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
    let (verifier, rp_trust_anchor, issuer_ca) = setup_wallet_initiated_usecase_verifier();

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

    let key_factory = MockRemoteKeyFactory::default();

    let session = start_disclosure_session(
        verifier,
        DisclosureUriSource::Link,
        &universal_link_query,
        rp_trust_anchor,
    )
    .await
    .unwrap();

    // Do the disclosure
    let mdocs = test_documents_to_mdocs(pid_full_name(), &issuer_ca, &key_factory);
    session.disclose(mdocs, &key_factory).await.unwrap().unwrap();
}

#[tokio::test]
async fn test_wallet_initiated_usecase_verifier_cancel() {
    let (verifier, rp_trust_anchor, _issuer_ca) = setup_wallet_initiated_usecase_verifier();

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
    let dcql_query: Query = pid_full_name().into();

    let (verifier, rp_trust_anchor, _issuer_ca) = setup_verifier(&dcql_query, None);

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

fn setup_wallet_initiated_usecase_verifier() -> (Arc<MockWalletInitiatedUseCaseVerifier>, TrustAnchor<'static>, Ca) {
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();

    // Initialize the verifier
    let dcql_query: Query = pid_full_name().into();
    let reader_registration = Some(ReaderRegistration::mock_from_dcql_query(&dcql_query));
    let usecases = HashMap::from([(
        WALLET_INITIATED_RETURN_URL_USE_CASE.to_string(),
        WalletInitiatedUseCase::try_new(
            generate_reader_mock(&rp_ca, reader_registration.clone()).unwrap(),
            SessionTypeReturnUrl::SameDevice,
            dcql_query,
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
    ));

    (verifier, rp_ca.to_trust_anchor().to_owned(), issuer_ca)
}

fn setup_verifier(
    dcql_query: &Query,
    session_result_query_param: Option<String>,
) -> (Arc<MockRpInitiatedUseCaseVerifier>, TrustAnchor<'static>, Ca) {
    // Initialize key material
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();

    // Initialize the verifier
    let reader_registration = Some(ReaderRegistration::mock_from_dcql_query(dcql_query));
    let usecases = HashMap::from([
        (
            NO_RETURN_URL_USE_CASE.to_string(),
            RpInitiatedUseCase::try_new(
                generate_reader_mock(&rp_ca, reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::Neither,
                None,
                None,
            )
            .unwrap(),
        ),
        (
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            RpInitiatedUseCase::try_new(
                generate_reader_mock(&rp_ca, reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::SameDevice,
                None,
                None,
            )
            .unwrap(),
        ),
        (
            ALL_RETURN_URL_USE_CASE.to_string(),
            RpInitiatedUseCase::try_new(
                generate_reader_mock(&rp_ca, reader_registration).unwrap(),
                SessionTypeReturnUrl::Both,
                None,
                None,
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
    ));

    (verifier, rp_ca.to_trust_anchor().to_owned(), issuer_ca)
}

fn test_documents_to_mdocs<KF>(stored_documents: TestDocuments, issuer_ca: &Ca, key_factory: &KF) -> VecNonEmpty<Mdoc>
where
    KF: KeyFactory,
{
    stored_documents
        .into_iter()
        .map(|doc| {
            test_document_to_mdoc(doc, issuer_ca, key_factory)
                .now_or_never()
                .unwrap()
        })
        .collect_vec()
        .try_into()
        .unwrap()
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
type MockVerifier<T> = Verifier<MemorySessionStore<DisclosureData>, T>;

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
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
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

        Ok(jws)
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
