use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};

use nl_wallet_mdoc::{
    examples::{Examples, IsoCertTimeGenerator},
    holder::{DeviceRequestMatch, ProposedDocument, TrustAnchor},
    server_keys::KeyPair,
    software_key_factory::SoftwareKeyFactory,
    test::{example_items_requests, DebugCollapseBts},
    verifier::ItemsRequests,
    DeviceRequest, DeviceResponse, DeviceResponseVersion, SessionTranscript,
};
use openid4vc::{
    jwt,
    mock::MockMdocDataSource,
    openid4vp::{VpAuthorizationRequest, VpAuthorizationResponse},
};
use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt};

#[tokio::test]
async fn disclosure() {
    let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
    let auth_keypair = ca.generate_reader_mock(None).unwrap();

    // RP assembles the Authentication Request and signs it into a JWS.
    let nonce = "nonce".to_string();
    let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
    let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let auth_request = VpAuthorizationRequest::new(
        &example_items_requests(),
        auth_keypair.certificate(),
        nonce.clone(),
        encryption_keypair.to_jwk_public_key().try_into().unwrap(),
        response_uri,
    )
    .unwrap();
    let auth_request_jws = jwt::sign_with_certificate(&auth_request, &auth_keypair).await.unwrap();

    // Wallet receives the signed Authentication Request and performs the disclosure.
    let jwe = disclosure_jwe(auth_request_jws, &[ca.certificate().try_into().unwrap()]).await;

    // RP decrypts the response JWE and verifies the contained Authorization Response.
    let (auth_response, mdoc_nonce) = VpAuthorizationResponse::decrypt(jwe, encryption_keypair, nonce).unwrap();
    let disclosed_attrs = auth_response
        .verify(
            &auth_request,
            mdoc_nonce,
            &IsoCertTimeGenerator,
            Examples::iaca_trust_anchors(),
        )
        .unwrap();

    dbg!(DebugCollapseBts::from(disclosed_attrs));
}

/// The wallet side: verify the Authentication Request, compute the disclosure, and encrypt it into a JWE.
async fn disclosure_jwe(auth_request: Jwt<VpAuthorizationRequest>, trust_anchors: &[TrustAnchor<'_>]) -> String {
    let mdocs = MockMdocDataSource::default();
    let mdoc_nonce = "mdoc_nonce".to_string();

    // Verify the Authorization Request JWE and read the requested attributes.
    let (auth_request, _) = VpAuthorizationRequest::verify(&auth_request, trust_anchors).unwrap();
    let items_requests: ItemsRequests = auth_request.presentation_definition.direct().try_into().unwrap();
    let device_request = DeviceRequest::from_items_requests(items_requests.0.clone());

    // Check if we have the requested attributes.
    let session_transcript = SessionTranscript::new_oid4vp(
        auth_request.response_uri.as_ref().unwrap(),
        auth_request.oauth_request.client_id.clone(),
        auth_request.oauth_request.nonce.as_ref().unwrap().clone(),
        mdoc_nonce.clone(),
    );
    let DeviceRequestMatch::Candidates(candidates) = device_request
        .match_stored_documents(&mdocs, &session_transcript)
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
    VpAuthorizationResponse::new_encrypted(device_response, &auth_request, mdoc_nonce).unwrap()
}
