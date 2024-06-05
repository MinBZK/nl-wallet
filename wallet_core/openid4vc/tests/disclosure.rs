use std::{collections::HashMap, str::FromStr, sync::Arc};

use itertools::Itertools;
use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
use ring::{hmac, rand};
use rstest::rstest;

use nl_wallet_mdoc::{
    examples::{Examples, IsoCertTimeGenerator},
    holder::{DisclosureRequestMatch, ProposedDocument, TrustAnchor},
    server_keys::KeyPair,
    server_state::{MemorySessionStore, SessionToken},
    software_key_factory::SoftwareKeyFactory,
    unsigned::Entry,
    utils::reader_auth::ReaderRegistration,
    verifier::{ReturnUrlTemplate, SessionType, SessionTypeReturnUrl, UseCase},
    DeviceResponse, DeviceResponseVersion, SessionTranscript,
};
use openid4vc::{
    disclosure_session::{DisclosureSession, VpMessageClient, VpMessageClientError},
    jwt,
    mock::MockMdocDataSource,
    openid4vp::{VpAuthorizationErrorCode, VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject},
    verifier::{DisclosureData, StatusResponse, Verifier, WalletAuthResponse},
    ErrorResponse,
};
use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt, trust_anchor::OwnedTrustAnchor};

#[tokio::test]
async fn disclosure() {
    let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
    let auth_keypair = ca.generate_reader_mock(None).unwrap();

    // RP assembles the Authorization Request and signs it into a JWS.
    let nonce = "nonce".to_string();
    let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
    let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let auth_request = VpAuthorizationRequest::new(
        &Examples::items_requests(),
        auth_keypair.certificate(),
        nonce.clone(),
        encryption_keypair.to_jwk_public_key().try_into().unwrap(),
        response_uri,
    )
    .unwrap();
    let auth_request_jws = jwt::sign_with_certificate(&auth_request, &auth_keypair).await.unwrap();

    // Wallet receives the signed Authorization Request and performs the disclosure.
    let jwe = disclosure_jwe(auth_request_jws, &[ca.certificate().try_into().unwrap()]).await;

    // RP decrypts the response JWE and verifies the contained Authorization Response.
    let (auth_response, mdoc_nonce) = VpAuthorizationResponse::decrypt(&jwe, &encryption_keypair, &nonce).unwrap();
    let disclosed_attrs = auth_response
        .verify(
            &auth_request.try_into().unwrap(),
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
    let mdocs = MockMdocDataSource::default();
    let mdoc_nonce = "mdoc_nonce".to_string();

    // Verify the Authorization Request JWE and read the requested attributes.
    let (auth_request, _) = VpAuthorizationRequest::verify(&auth_request, trust_anchors).unwrap();

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
    let documents = ProposedDocument::sign_multiple(&key_factory, to_disclose)
        .await
        .unwrap();
    let device_response = DeviceResponse {
        version: DeviceResponseVersion::V1_0,
        documents: Some(documents),
        document_errors: None,
        status: 0,
    };

    // Put the disclosure in an Authorization Response and encrypt it.
    VpAuthorizationResponse::new_encrypted(device_response, &auth_request, &mdoc_nonce).unwrap()
}

#[rstest]
#[case(SessionType::SameDevice)]
#[case(SessionType::CrossDevice)]
#[tokio::test]
async fn test_client_and_server(#[case] session_type: SessionType) {
    let items_requests = Examples::items_requests();

    // Initialize key material
    let ca = KeyPair::generate_reader_mock_ca().unwrap();
    let disclosure_key = ca
        .generate_reader_mock(Some(ReaderRegistration::new_mock_from_requests(&items_requests)))
        .unwrap();
    let trust_anchors = &[ca.certificate().try_into().unwrap()];

    // Initialize the verifier
    let verifier = Arc::new(MockVerifier::new(
        HashMap::from([(
            "usecase_id".to_string(),
            UseCase {
                key_pair: disclosure_key,
                session_type_return_url: SessionTypeReturnUrl::SameDevice,
            },
        )])
        .into(),
        MemorySessionStore::default(),
        Examples::iaca_trust_anchors()
            .iter()
            .map(OwnedTrustAnchor::from)
            .collect_vec(),
        hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
    ));

    // Start the session
    let session_token = verifier
        .new_session(
            items_requests,
            "usecase_id".to_string(),
            Some(ReturnUrlTemplate::from_str("https://example.com/redirect_uri/{session_token}").unwrap()),
        )
        .await
        .unwrap();

    // frontend receives the UL to feed to the wallet when fetching the session status
    let request_uri_object = request_uri_from_status_endpoint(verifier.as_ref(), &session_token, session_type).await;

    // Start session in the wallet
    let mdocs = MockMdocDataSource::default();
    let key_factory = SoftwareKeyFactory::default();
    let message_client = MockVpMessageClient::new(Arc::clone(&verifier));
    let session = DisclosureSession::start(message_client, request_uri_object, &mdocs, trust_anchors)
        .await
        .unwrap();

    let DisclosureSession::Proposal(proposal) = session else {
        panic!("should have requested attributes")
    };

    // Finish the disclosure.
    let redirect_uri = proposal.disclose(&key_factory).await.unwrap();

    // If a redirect URI is present then the wallet would navigate to it, informing the RP of the
    // redirect URI nonce which it will need when retrieving the disclosed attributes.
    let redirect_uri_nonce = redirect_uri.and_then(|uri| {
        uri.as_ref()
            .query_pairs()
            .find_map(|(name, val)| if name == "nonce" { Some(val.to_string()) } else { None })
    });

    // Retrieve the attributes disclosed by the wallet
    let disclosed = verifier
        .disclosed_attributes(&session_token, redirect_uri_nonce)
        .await
        .unwrap();

    assert_eq!(
        *disclosed["org.iso.18013.5.1.mDL"].attributes["org.iso.18013.5.1"]
            .first()
            .unwrap(),
        Entry {
            name: "family_name".to_string(),
            value: "Doe".into()
        }
    );
}

async fn request_uri_from_status_endpoint(
    verifier: &MockVerifier,
    session_token: &SessionToken,
    session_type: SessionType,
) -> VpRequestUriObject {
    let StatusResponse::Created { ul } = verifier
        .status_response(
            session_token,
            &"https://example.com/ul".parse().unwrap(),
            &"https://example.com/verifier_base_url".parse().unwrap(),
            session_type,
        )
        .await
        .unwrap()
    else {
        panic!("unexpected state")
    };

    serde_urlencoded::from_str(ul.as_ref().query().unwrap()).unwrap()
}

type MockVerifier = Verifier<MemorySessionStore<DisclosureData>>;

struct MockVpMessageClient {
    verifier: Arc<MockVerifier>,
}

impl MockVpMessageClient {
    pub fn new(verifier: Arc<MockVerifier>) -> Self {
        MockVpMessageClient { verifier }
    }
}

impl VpMessageClient for MockVpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        let session_token = SessionToken::new(url.as_ref().path_segments().unwrap().last().unwrap());

        let url_params = serde_urlencoded::from_str(url.as_ref().query().unwrap()).unwrap();

        let jws = self
            .verifier
            .process_get_request(&session_token, &"https://example.com".parse().unwrap(), url_params)
            .await
            .unwrap();

        Ok(jws)
    }

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        let session_token = SessionToken::new(url.as_ref().path_segments().unwrap().last().unwrap());

        let response = self
            .verifier
            .process_authorization_response(&session_token, WalletAuthResponse::Response(jwe), &IsoCertTimeGenerator)
            .await
            .unwrap();

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
