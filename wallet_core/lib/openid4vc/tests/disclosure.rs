use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::Infallible;
use std::num::NonZeroU8;
use std::str::FromStr;
use std::sync::Arc;

use assert_matches::assert_matches;
use chrono::Utc;
use futures::future;
use itertools::Itertools;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
use mdoc::server_keys::generate::mock::generate_reader_mock;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use ring::hmac;
use ring::rand;
use rstest::rstest;
use rustls_pki_types::TrustAnchor;

use crypto::factory::KeyFactory;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteKeyFactory;
use crypto::mock_remote::MockRemoteKeyFactoryError;
use crypto::server_keys::generate::Ca;
use crypto::server_keys::KeyPair;
use http_utils::urls::BaseUrl;
use jwt::Jwt;
use mdoc::examples::example_items_requests;
use mdoc::examples::IsoCertTimeGenerator;
use mdoc::holder::mock::MockMdocDataSource as IsoMockMdocDataSource;
use mdoc::holder::DisclosureRequestMatch;
use mdoc::holder::Mdoc;
use mdoc::holder::MdocDataSource;
use mdoc::holder::StoredMdoc;
use mdoc::test::data::addr_street;
use mdoc::test::data::pid_full_name;
use mdoc::test::data::pid_given_name;
use mdoc::test::TestDocuments;
use mdoc::utils::reader_auth::ReaderRegistration;
use mdoc::verifier::ItemsRequests;
use mdoc::DeviceResponse;
use mdoc::DocType;
use mdoc::SessionTranscript;
use openid4vc::credential::MdocCopies;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::DisclosureUriSource;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpMessageClient;
use openid4vc::disclosure_session::VpMessageClientError;
use openid4vc::openid4vp::IsoVpAuthorizationRequest;
use openid4vc::openid4vp::VpAuthorizationRequest;
use openid4vc::openid4vp::VpAuthorizationResponse;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionToken;
use openid4vc::verifier::DisclosedAttributesError;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::StatusResponse;
use openid4vc::verifier::UseCase;
use openid4vc::verifier::Verifier;
use openid4vc::verifier::VerifierUrlParameters;
use openid4vc::verifier::VpToken;
use openid4vc::verifier::WalletAuthResponse;
use openid4vc::ErrorResponse;
use openid4vc::GetRequestErrorCode;
use openid4vc::PostAuthResponseErrorCode;
use openid4vc::VpAuthorizationErrorCode;
use poa::factory::PoaFactory;
use poa::Poa;
use poa::PoaError;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecAtLeastTwoUnique;

#[tokio::test]
async fn disclosure_direct() {
    let ca = Ca::generate("myca", Default::default()).unwrap();
    let auth_keypair = generate_reader_mock(&ca, None).unwrap();

    // RP assembles the Authorization Request and signs it into a JWS.
    let nonce = "nonce".to_string();
    let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
    let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let iso_auth_request = IsoVpAuthorizationRequest::new(
        &example_items_requests(),
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
            &mdoc_nonce,
            &IsoCertTimeGenerator,
            &[issuer_ca.to_trust_anchor()],
        )
        .unwrap();

    assert_eq!(
        disclosed_attrs["org.iso.18013.5.1.mDL"].attributes["org.iso.18013.5.1"]["family_name"],
        "Doe".into()
    );
}

/// The wallet side: verify the Authorization Request, compute the disclosure, and encrypt it into a JWE.
async fn disclosure_jwe(
    auth_request: Jwt<VpAuthorizationRequest>,
    trust_anchors: &[TrustAnchor<'_>],
    issuer_ca: &Ca,
) -> String {
    let mdocs = IsoMockMdocDataSource::new_example_resigned(issuer_ca).await;
    let mdoc_nonce = "mdoc_nonce".to_string();

    // Verify the Authorization Request JWE and read the requested attributes.
    let (auth_request, cert) = VpAuthorizationRequest::try_new(&auth_request, trust_anchors).unwrap();
    let auth_request = auth_request.validate(&cert, None).unwrap();

    // Check if we have the requested attributes.
    let session_transcript = SessionTranscript::new_oid4vp(
        &auth_request.response_uri,
        &auth_request.client_id,
        auth_request.nonce.clone(),
        &mdoc_nonce,
    );
    let DisclosureRequestMatch::Candidates(candidates) =
        DisclosureRequestMatch::new(auth_request.items_requests.as_ref().iter(), &mdocs, &session_transcript)
            .await
            .unwrap()
    else {
        panic!("should have found requested attributes")
    };

    // For each doctype, just choose the first candidate.
    let to_disclose = candidates.into_values().map(|mut docs| docs.pop().unwrap()).collect();

    // Compute the disclosure.
    let key_factory = MockRemoteKeyFactory::new_example();
    let (device_response, keys) = DeviceResponse::from_proposed_documents(to_disclose, &key_factory)
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
        Some(ReaderRegistration::new_mock_from_requests(&example_items_requests())),
    )
    .unwrap();

    // Initialize the "wallet"
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let mdocs = IsoMockMdocDataSource::new_example_resigned(&issuer_ca).await;

    // Start a session at the "RP"
    let message_client = DirectMockVpMessageClient::new(rp_keypair, vec![issuer_ca.to_trust_anchor().to_owned()]);
    let request_uri = message_client.start_session();

    // Perform the first part of the session, resulting in the proposed disclosure.
    let session = DisclosureSession::start(
        message_client,
        &request_uri,
        DisclosureUriSource::Link,
        &mdocs,
        &[ca.to_trust_anchor()],
    )
    .await
    .unwrap();

    let DisclosureSession::Proposal(proposal) = session else {
        panic!("should have requested attributes")
    };

    // Finish the disclosure.
    let key_factory = MockRemoteKeyFactory::new_example();
    proposal.disclose(&key_factory).await.unwrap();
}

// A mock implementation of the `VpMessageClient` trait that implements the RP side of OpenID4VP
// directly in its methods.
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
            ephemeral_id: vec![42],
            time: Utc::now(),
        })
        .unwrap();
        let request_uri = ("https://example.com/request_uri?".to_string() + &query)
            .parse()
            .unwrap();

        let nonce = "nonce".to_string();
        let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();

        let auth_request = IsoVpAuthorizationRequest::new(
            &example_items_requests(),
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
                &mdoc_nonce,
                &IsoCertTimeGenerator,
                &self.trust_anchors,
            )
            .unwrap();

        assert_eq!(
            disclosed_attrs["org.iso.18013.5.1.mDL"].attributes["org.iso.18013.5.1"]["family_name"],
            "Doe".into()
        );

        Ok(None)
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        panic!("error: {:?}", error)
    }
}

const NO_RETURN_URL_USE_CASE: &str = "no_return_url";
const DEFAULT_RETURN_URL_USE_CASE: &str = "default_return_url";
const ALL_RETURN_URL_USE_CASE: &str = "all_return_url";

struct MockMdocDataSource(HashMap<DocType, MdocCopies>);

impl From<Vec<Mdoc>> for MockMdocDataSource {
    fn from(value: Vec<Mdoc>) -> Self {
        MockMdocDataSource(
            value
                .into_iter()
                .map(|mdoc| (mdoc.doc_type().clone(), vec![mdoc].try_into().unwrap()))
                .collect(),
        )
    }
}

impl MdocDataSource for MockMdocDataSource {
    type MdocIdentifier = String;
    type Error = Infallible;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        let stored_mdocs = self
            .0
            .iter()
            .filter_map(|(doc_type, mdoc_copies)| {
                if doc_types.contains(doc_type.as_str()) {
                    return vec![StoredMdoc {
                        id: format!("{}_id", doc_type.clone()),
                        mdoc: mdoc_copies.first().clone(),
                    }]
                    .into();
                }

                None
            })
            .collect();

        Ok(stored_mdocs)
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
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_given_name() + addr_street(),
    pid_given_name().into(),
    pid_given_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_full_name(),
    (pid_given_name() + pid_given_name()).into(),
    pid_given_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    NO_RETURN_URL_USE_CASE,
    pid_given_name(),
    (pid_given_name() + pid_given_name()).into(),
    pid_given_name()
)]
#[tokio::test]
async fn test_client_and_server(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] use_case: &str,
    #[case] stored_documents: TestDocuments,
    #[case] requested_documents: ItemsRequests,
    #[case] expected_documents: TestDocuments,
) {
    let (verifier, rp_trust_anchor, issuer_ca) = setup_verifier(&requested_documents);

    // Start the session
    let session_token = verifier
        .new_session(requested_documents, use_case.to_string(), return_url_template)
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
    let session = start_disclosure_session(
        Arc::clone(&verifier),
        stored_documents,
        &issuer_ca,
        uri_source,
        &request_uri,
        rp_trust_anchor,
        &key_factory,
    )
    .await
    .unwrap();

    let DisclosureSession::Proposal(proposal) = session else {
        panic!("should have requested attributes")
    };

    // Finish the disclosure.
    let redirect_uri = proposal.disclose(&key_factory).await.unwrap();

    // Check if we received a redirect URI when we should have, based on the use case and session type.
    let should_have_redirect_uri = match (use_case, session_type) {
        (use_case, _) if use_case == NO_RETURN_URL_USE_CASE => false,
        (use_case, _) if use_case == ALL_RETURN_URL_USE_CASE => true,
        (_, SessionType::SameDevice) => true,
        (_, SessionType::CrossDevice) => false,
    };
    assert_eq!(redirect_uri.is_some(), should_have_redirect_uri);

    let redirect_uri_nonce = redirect_uri.and_then(|uri| {
        uri.as_ref()
            .query_pairs()
            .find_map(|(name, val)| (name == "nonce").then(|| val.to_string()))
    });

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

    expected_documents.assert_matches(&disclosed_documents);
}

#[tokio::test]
async fn test_client_and_server_cancel_after_created() {
    let stored_documents = pid_full_name();
    let items_requests = pid_full_name().into();
    let session_type = SessionType::SameDevice;

    let (verifier, trust_anchor, issuer_ca) = setup_verifier(&items_requests);

    // Start the session
    let session_token = verifier
        .new_session(
            items_requests,
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
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
    let error = start_disclosure_session(
        Arc::clone(&verifier),
        stored_documents,
        &issuer_ca,
        DisclosureUriSource::Link,
        &request_uri,
        trust_anchor,
        &MockRemoteKeyFactory::default(),
    )
    .await
    .expect_err("should not be able to start the disclosure session in the wallet");

    assert_matches!(
        error,
        VpClientError::Request(VpMessageClientError::AuthGetResponse(error))
            if error.error_response.error == GetRequestErrorCode::CancelledSession
    );
}

#[tokio::test]
async fn test_client_and_server_cancel_after_wallet_start() {
    let stored_documents = pid_full_name();
    let items_requests = pid_full_name().into();
    let session_type = SessionType::SameDevice;

    let (verifier, trust_anchor, issuer_ca) = setup_verifier(&items_requests);

    // Start the session
    let session_token = verifier
        .new_session(
            items_requests,
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
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
        stored_documents,
        &issuer_ca,
        DisclosureUriSource::Link,
        &request_uri,
        trust_anchor,
        &key_factory,
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
    let DisclosureSession::Proposal(proposal) = session else {
        panic!("should have requested attributes")
    };

    let error = proposal
        .disclose(&key_factory)
        .await
        .expect_err("should not be able to disclose attributes");

    assert_matches!(
        error.error,
        VpClientError::Request(VpMessageClientError::AuthPostResponse(error))
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

        fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
            self.0.generate_existing(identifier, public_key)
        }

        async fn sign_multiple_with_existing_keys(
            &self,
            messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        ) -> Result<Vec<Vec<Signature>>, Self::Error> {
            self.0.sign_multiple_with_existing_keys(messages_and_keys).await
        }

        async fn sign_with_new_keys(&self, _: Vec<u8>, _: u64) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
            unimplemented!()
        }

        async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
            self.0.generate_new_multiple(count).await
        }
    }

    impl PoaFactory for WrongPoaKeyFactory {
        type Key = MockRemoteEcdsaKey;
        type Error = PoaError;

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
    let items_requests = (pid_given_name() + addr_street()).into();
    let session_type = SessionType::SameDevice;
    let use_case = NO_RETURN_URL_USE_CASE;

    let (verifier, rp_trust_anchor, issuer_ca) = setup_verifier(&items_requests);

    // Start the session
    let session_token = verifier
        .new_session(items_requests, use_case.to_string(), None)
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
    let session = start_disclosure_session(
        Arc::clone(&verifier),
        stored_documents,
        &issuer_ca,
        uri_source,
        &request_uri,
        rp_trust_anchor,
        &key_factory,
    )
    .await
    .unwrap();

    let DisclosureSession::Proposal(proposal) = session else {
        panic!("should have requested attributes")
    };

    // Finish the disclosure.
    let error = proposal
        .disclose(&key_factory)
        .await
        .expect_err("should not be able to disclose attributes");
    assert_matches!(
        error.error,
        VpClientError::Request(VpMessageClientError::AuthPostResponse(error))
            if error.error_response.error == PostAuthResponseErrorCode::InvalidRequest
    );
}

fn setup_verifier(items_requests: &ItemsRequests) -> (Arc<MockVerifier>, TrustAnchor<'static>, Ca) {
    // Initialize key material
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();

    // Initialize the verifier
    let reader_registration = Some(ReaderRegistration::new_mock_from_requests(items_requests));
    let usecases = HashMap::from([
        (
            NO_RETURN_URL_USE_CASE.to_string(),
            UseCase::try_new(
                generate_reader_mock(&rp_ca, reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::Neither,
            )
            .unwrap(),
        ),
        (
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            UseCase::try_new(
                generate_reader_mock(&rp_ca, reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::SameDevice,
            )
            .unwrap(),
        ),
        (
            ALL_RETURN_URL_USE_CASE.to_string(),
            UseCase::try_new(
                generate_reader_mock(&rp_ca, reader_registration).unwrap(),
                SessionTypeReturnUrl::Both,
            )
            .unwrap(),
        ),
    ])
    .into();

    let verifier = Arc::new(MockVerifier::new(
        usecases,
        Arc::new(MemorySessionStore::default()),
        vec![issuer_ca.to_trust_anchor().to_owned()],
        hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
    ));

    (verifier, rp_ca.to_trust_anchor().to_owned(), issuer_ca)
}

async fn start_disclosure_session<KF, K>(
    verifier: Arc<MockVerifier>,
    stored_documents: TestDocuments,
    issuer_ca: &Ca,
    uri_source: DisclosureUriSource,
    request_uri: &str,
    trust_anchor: TrustAnchor<'static>,
    key_factory: &KF,
) -> Result<DisclosureSession<VerifierMockVpMessageClient, String>, VpClientError>
where
    KF: KeyFactory<Key = K>,
{
    // Populate the wallet with the specified test documents
    let mdocs = future::join_all(
        stored_documents
            .into_iter()
            .map(|doc| async { doc.sign(issuer_ca, key_factory, NonZeroU8::new(1).unwrap()).await }),
    )
    .await;
    let mdocs = MockMdocDataSource::from(mdocs);

    // Start session in the wallet
    DisclosureSession::start(
        VerifierMockVpMessageClient::new(verifier),
        request_uri,
        uri_source,
        &mdocs,
        &[trust_anchor],
    )
    .await
}

async fn request_uri_from_status_endpoint(
    verifier: &MockVerifier,
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
    verifier: &MockVerifier,
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

type MockVerifier = Verifier<MemorySessionStore<DisclosureData>, SigningKey>;

#[derive(Debug)]
struct VerifierMockVpMessageClient {
    verifier: Arc<MockVerifier>,
}

impl VerifierMockVpMessageClient {
    pub fn new(verifier: Arc<MockVerifier>) -> Self {
        VerifierMockVpMessageClient { verifier }
    }
}

impl VpMessageClient for VerifierMockVpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        let path_segments = url.as_ref().path_segments().unwrap().collect_vec();
        let session_token = path_segments[path_segments.len() - 2].to_owned().into();

        let jws = self
            .verifier
            .process_get_request(
                &session_token,
                format!("https://example.com/verifier_base_url/{session_token}/response_uri")
                    .parse()
                    .unwrap(),
                url.as_ref().query(),
                wallet_nonce,
            )
            .await
            .map_err(|error| VpMessageClientError::AuthGetResponse(error.into()))?;

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
            .map_err(|error| VpMessageClientError::AuthPostResponse(error.into()))?;

        Ok(response.redirect_uri)
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        panic!("error: {:?}", error)
    }
}
