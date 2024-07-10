use std::{collections::HashMap, str::FromStr, sync::Arc};

use chrono::Utc;
use itertools::Itertools;
use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
use ring::{hmac, rand};
use rstest::rstest;

use nl_wallet_mdoc::{
    examples::{Examples, IsoCertTimeGenerator},
    holder::{test::MockMdocDataSource, DisclosureRequestMatch, DisclosureUriSource, TrustAnchor},
    server_keys::KeyPair,
    server_state::{MemorySessionStore, SessionToken},
    software_key_factory::SoftwareKeyFactory,
    unsigned::Entry,
    utils::reader_auth::ReaderRegistration,
    verifier::{ReturnUrlTemplate, SessionType, SessionTypeReturnUrl},
    DeviceResponse, SessionTranscript,
};
use openid4vc::{
    disclosure_session::{DisclosureSession, VpMessageClient, VpMessageClientError},
    jwt,
    openid4vp::{IsoVpAuthorizationRequest, VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject},
    verifier::{DisclosureData, StatusResponse, UseCase, Verifier, VerifierUrlParameters, VpToken, WalletAuthResponse},
    ErrorResponse, VpAuthorizationErrorCode,
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
    let mdocs = MockMdocDataSource::default();
    let mdoc_nonce = "mdoc_nonce".to_string();

    // Verify the Authorization Request JWE and read the requested attributes.
    let (auth_request, cert) = VpAuthorizationRequest::verify(&auth_request, trust_anchors).unwrap();
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
    let mdocs = MockMdocDataSource::default();
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

#[rstest]
#[case(SessionType::SameDevice, DisclosureUriSource::Link)]
#[case(SessionType::CrossDevice, DisclosureUriSource::QrCode)]
#[tokio::test]
async fn test_client_and_server(#[case] session_type: SessionType, #[case] uri_source: DisclosureUriSource) {
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
            UseCase::new(disclosure_key, SessionTypeReturnUrl::SameDevice).unwrap(),
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
    let request_uri = request_uri_from_status_endpoint(verifier.as_ref(), &session_token, session_type).await;

    // Start session in the wallet
    let mdocs = MockMdocDataSource::default();
    let key_factory = SoftwareKeyFactory::default();
    let message_client = VerifierMockVpMessageClient::new(Arc::clone(&verifier));
    let session = DisclosureSession::start(message_client, &request_uri, uri_source, &mdocs, trust_anchors)
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
) -> String {
    let StatusResponse::Created { ul } = verifier
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
    else {
        panic!("unexpected state")
    };

    ul.as_ref().query().unwrap().to_string()
}

type MockVerifier = Verifier<MemorySessionStore<DisclosureData>>;

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
            .unwrap();

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
                &IsoCertTimeGenerator,
            )
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
