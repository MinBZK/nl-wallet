use std::{collections::HashSet, convert::Infallible};

use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};

use nl_wallet_mdoc::{
    examples::{Examples, IsoCertTimeGenerator},
    holder::{DeviceRequestMatch, Mdoc, MdocDataSource, ProposedDocument, StoredMdoc, TrustAnchor},
    server_keys::KeyPair,
    software_key_factory::SoftwareKeyFactory,
    unsigned::Entry,
    verifier::ItemsRequests,
    DeviceRequest, DeviceResponse, DeviceResponseVersion, SessionTranscript,
};
use openid4vc::{
    jwt,
    openid4vp::{VpAuthorizationRequest, VpAuthorizationResponse},
};
use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt};

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
            &auth_request,
            mdoc_nonce,
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
    let auth_request = VpAuthorizationRequest::verify(&auth_request, trust_anchors).unwrap();
    let items_requests: ItemsRequests = auth_request.presentation_definition.direct().try_into().unwrap();
    let device_request = DeviceRequest::from_items_requests(items_requests.0.clone());

    // Check if we have the requested attributes.
    let session_transcript = SessionTranscript::new_oid4vp(
        auth_request.response_uri.as_ref().unwrap(),
        &auth_request.oauth_request.client_id,
        auth_request.oauth_request.nonce.as_ref().unwrap().clone(),
        &mdoc_nonce,
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
    VpAuthorizationResponse::new_encrypted(device_response, &auth_request, &mdoc_nonce).unwrap()
}

/// A type that implements `MdocDataSource` and simply returns
/// the [`Mdoc`] contained in `DeviceResponse::example()`, if its
/// `doc_type` is requested.
#[derive(Debug)]
pub struct MockMdocDataSource {
    pub mdocs: Vec<Mdoc>,
}
pub type MdocIdentifier = String;
impl Default for MockMdocDataSource {
    fn default() -> Self {
        MockMdocDataSource {
            mdocs: vec![Mdoc::new_example_mock()],
        }
    }
}

impl MdocDataSource for MockMdocDataSource {
    type MdocIdentifier = MdocIdentifier;
    type Error = Infallible;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        let stored_mdocs = self
            .mdocs
            .iter()
            .filter(|mdoc| doc_types.contains(mdoc.doc_type.as_str()))
            .cloned()
            .enumerate()
            .map(|(index, mdoc)| StoredMdoc {
                id: format!("id_{}", index + 1),
                mdoc,
            })
            .collect();

        Ok(vec![stored_mdocs])
    }
}
