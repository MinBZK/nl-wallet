use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    num::NonZeroU8,
    str::FromStr,
    sync::Arc,
};

use assert_matches::assert_matches;
use chrono::Utc;
use futures::future;
use itertools::Itertools;
use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
use ring::{hmac, rand};
use rstest::rstest;

use nl_wallet_mdoc::{
    examples::{Examples, IsoCertTimeGenerator},
    holder::{
        mock::MockMdocDataSource as IsoMockMdocDataSource, DisclosureRequestMatch, Mdoc, MdocCopies, MdocDataSource,
        StoredMdoc, TrustAnchor,
    },
    server_keys::KeyPair,
    software_key_factory::SoftwareKeyFactory,
    test::{
        data::{addr_street, pid_full_name, pid_given_name},
        TestDocuments,
    },
    unsigned::Entry,
    utils::reader_auth::ReaderRegistration,
    verifier::ItemsRequests,
    DeviceResponse, DocType, SessionTranscript,
};
use openid4vc::{
    disclosure_session::{
        DisclosureSession, DisclosureUriSource, VpClientError, VpMessageClient, VpMessageClientError,
    },
    jwt,
    openid4vp::{IsoVpAuthorizationRequest, VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject},
    return_url::ReturnUrlTemplate,
    server_state::{MemorySessionStore, SessionToken},
    verifier::{
        DisclosedAttributesError, DisclosureData, SessionType, SessionTypeReturnUrl, StatusResponse, UseCase, Verifier,
        VerifierUrlParameters, VpToken, WalletAuthResponse,
    },
    ErrorResponse, GetRequestErrorCode, PostAuthResponseErrorCode, VpAuthorizationErrorCode,
};
use wallet_common::{
    config::wallet_config::BaseUrl, generator::TimeGenerator, jwt::Jwt, trust_anchor::OwnedTrustAnchor,
};

#[tokio::test]
async fn disclosure_direct() {
    let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
    let auth_keypair = ca.generate_reader_mock(None).unwrap();

    // RP assembles the Authorization Request and signs it into a JWS.
    let nonce = "nonce".to_string();
    let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
    let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let iso_auth_request = IsoVpAuthorizationRequest::new(
        &Examples::items_requests(),
        auth_keypair.certificate(),
        nonce.clone(),
        encryption_keypair.to_jwk_public_key().try_into().unwrap(),
        response_uri,
        None,
    )
    .unwrap();
    let auth_request = iso_auth_request.clone().into();
    let auth_request_jws = jwt::sign_with_certificate(&auth_request, &auth_keypair).await.unwrap();

    // Wallet receives the signed Authorization Request and performs the disclosure.
    let jwe = disclosure_jwe(auth_request_jws, &[ca.certificate().try_into().unwrap()]).await;

    // RP decrypts the response JWE and verifies the contained Authorization Response.
    let (auth_response, mdoc_nonce) = VpAuthorizationResponse::decrypt(&jwe, &encryption_keypair, &nonce).unwrap();
    let disclosed_attrs = auth_response
        .verify(
            &iso_auth_request,
            &mdoc_nonce,
            &IsoCertTimeGenerator,
            Examples::iaca_trust_anchors(),
        )
        .unwrap();

    assert_eq!(
        *disclosed_attrs["org.iso.18013.5.1.mDL"].attributes["org.iso.18013.5.1"]
            .first()
            .unwrap(),
        Entry {
            name: "family_name".to_string(),
            value: "Doe".into()
        }
    );
}

/// The wallet side: verify the Authorization Request, compute the disclosure, and encrypt it into a JWE.
async fn disclosure_jwe(auth_request: Jwt<VpAuthorizationRequest>, trust_anchors: &[TrustAnchor<'_>]) -> String {
    let mdocs = IsoMockMdocDataSource::default();
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
    let key_factory = SoftwareKeyFactory::default();
    let device_response = DeviceResponse::from_proposed_documents(to_disclose, &key_factory)
        .await
        .unwrap();

    // Put the disclosure in an Authorization Response and encrypt it.
    VpAuthorizationResponse::new_encrypted(device_response, &auth_request, &mdoc_nonce).unwrap()
}

#[tokio::test]
async fn disclosure_using_message_client() {
    let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
    let trust_anchors = &[ca.certificate().try_into().unwrap()];
    let rp_keypair = ca
        .generate_reader_mock(Some(ReaderRegistration::new_mock_from_requests(
            &Examples::items_requests(),
        )))
        .unwrap();

    // Initialize the "wallet"
    let mdocs = IsoMockMdocDataSource::default();
    let key_factory = &SoftwareKeyFactory::default();

    // Start a session at the "RP"
    let message_client = DirectMockVpMessageClient::new(rp_keypair);
    let request_uri = message_client.start_session();

    // Perform the first part of the session, resulting in the proposed disclosure.
    let session = DisclosureSession::start(
        message_client,
        &request_uri,
        DisclosureUriSource::Link,
        &mdocs,
        trust_anchors,
    )
    .await
    .unwrap();

    let DisclosureSession::Proposal(proposal) = session else {
        panic!("should have requested attributes")
    };

    // Finish the disclosure.
    proposal.disclose(key_factory).await.unwrap();
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
}

impl DirectMockVpMessageClient {
    fn new(auth_keypair: KeyPair) -> Self {
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
            &Examples::items_requests(),
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
        }
    }

    fn start_session(&self) -> String {
        serde_urlencoded::to_string(VpRequestUriObject {
            request_uri: self.request_uri.clone(),
            client_id: self.auth_keypair.certificate().san_dns_name().unwrap().unwrap(),
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

        let jws = jwt::sign_with_certificate(&self.auth_request, &self.auth_keypair)
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
                Examples::iaca_trust_anchors(),
            )
            .unwrap();

        assert_eq!(
            *disclosed_attrs["org.iso.18013.5.1.mDL"].attributes["org.iso.18013.5.1"]
                .first()
                .unwrap(),
            Entry {
                name: "family_name".to_string(),
                value: "Doe".into()
            }
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
                .map(|mdoc| (mdoc.doc_type.clone(), vec![mdoc].into()))
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
                        mdoc: mdoc_copies.cred_copies.first().unwrap().clone(),
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
    let (session, key_factory) = start_disclosure_session(
        Arc::clone(&verifier),
        stored_documents,
        &issuer_ca,
        uri_source,
        &request_uri,
        &rp_trust_anchor,
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
        &trust_anchor,
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
    let (session, key_factory) = start_disclosure_session(
        Arc::clone(&verifier),
        stored_documents,
        &issuer_ca,
        DisclosureUriSource::Link,
        &request_uri,
        &trust_anchor,
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

fn setup_verifier(items_requests: &ItemsRequests) -> (Arc<MockVerifier>, OwnedTrustAnchor, KeyPair) {
    // Initialize key material
    let issuer_ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let rp_ca = KeyPair::generate_reader_mock_ca().unwrap();
    let rp_trust_anchor: TrustAnchor = rp_ca.certificate().try_into().unwrap();

    // Initialize the verifier
    let reader_registration = Some(ReaderRegistration::new_mock_from_requests(items_requests));
    let usecases = HashMap::from([
        (
            NO_RETURN_URL_USE_CASE.to_string(),
            UseCase::try_new(
                rp_ca.generate_reader_mock(reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::Neither,
            )
            .unwrap(),
        ),
        (
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            UseCase::try_new(
                rp_ca.generate_reader_mock(reader_registration.clone()).unwrap(),
                SessionTypeReturnUrl::SameDevice,
            )
            .unwrap(),
        ),
        (
            ALL_RETURN_URL_USE_CASE.to_string(),
            UseCase::try_new(
                rp_ca.generate_reader_mock(reader_registration).unwrap(),
                SessionTypeReturnUrl::Both,
            )
            .unwrap(),
        ),
    ])
    .into();

    let verifier = Arc::new(MockVerifier::new(
        usecases,
        MemorySessionStore::default(),
        vec![OwnedTrustAnchor::from(&(issuer_ca.certificate().try_into().unwrap()))],
        hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
    ));

    (verifier, (&rp_trust_anchor).into(), issuer_ca)
}

async fn start_disclosure_session(
    verifier: Arc<MockVerifier>,
    stored_documents: TestDocuments,
    issuer_ca: &KeyPair,
    uri_source: DisclosureUriSource,
    request_uri: &str,
    trust_anchor: &OwnedTrustAnchor,
) -> Result<
    (
        DisclosureSession<VerifierMockVpMessageClient, String>,
        SoftwareKeyFactory,
    ),
    VpClientError,
> {
    let key_factory = SoftwareKeyFactory::default();

    // Populate the wallet with the specified test documents
    let mdocs = future::join_all(
        stored_documents
            .into_iter()
            .map(|doc| async { doc.sign(issuer_ca, &key_factory, NonZeroU8::new(1).unwrap()).await }),
    )
    .await;
    let mdocs = MockMdocDataSource::from(mdocs);

    // Start session in the wallet
    DisclosureSession::start(
        VerifierMockVpMessageClient::new(verifier),
        request_uri,
        uri_source,
        &mdocs,
        &[(trust_anchor).into()],
    )
    .await
    .map(|session| (session, key_factory))
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

type MockVerifier = Verifier<MemorySessionStore<DisclosureData>>;

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
        let session_token = SessionToken::new(path_segments[path_segments.len() - 2]);

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
        let session_token = SessionToken::new(path_segments[path_segments.len() - 2]);

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
