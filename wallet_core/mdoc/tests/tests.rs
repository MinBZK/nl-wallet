use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    num::NonZeroU8,
    sync::Arc,
};

use assert_matches::assert_matches;
use base64::prelude::*;
use futures::future;
use ring::{hmac, rand};
use rstest::rstest;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use webpki::TrustAnchor;

use nl_wallet_mdoc::{
    holder::{
        DisclosureSession, DisclosureUriSource, HttpClient, HttpClientResult, Mdoc, MdocCopies, MdocDataSource,
        StoredMdoc,
    },
    iso::mdocs::DocType,
    server_keys::KeyPair,
    server_state::MemorySessionStore,
    software_key_factory::SoftwareKeyFactory,
    test::{
        data::{addr_street, pid_full_name, pid_given_name},
        TestDocuments,
    },
    utils::{
        reader_auth::ReaderRegistration,
        serialization::{self, cbor_deserialize},
        x509::Certificate,
    },
    verifier::{
        DisclosureData, ItemsRequests, ReturnUrlTemplate, SessionType, SessionTypeReturnUrl, StatusResponse, UseCase,
        VerificationError, Verifier,
    },
    Error, ReaderEngagement,
};
use wallet_common::generator::TimeGenerator;

const NO_RETURN_URL_USE_CASE: &str = "no_return_url";
const DEFAULT_RETURN_URL_USE_CASE: &str = "default_return_url";
const ALL_RETURN_URL_USE_CASE: &str = "all_return_url";

type MockVerifier = Verifier<MemorySessionStore<DisclosureData>>;

struct MockDisclosureHttpClient {
    verifier: Arc<MockVerifier>,
}

impl MockDisclosureHttpClient {
    pub fn new(verifier: Arc<MockVerifier>) -> Self {
        MockDisclosureHttpClient { verifier }
    }
}

impl HttpClient for MockDisclosureHttpClient {
    async fn post<R, V>(&self, url: &Url, val: &V) -> HttpClientResult<R>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        let session_token = url.path_segments().unwrap().last().unwrap().to_string();
        let msg = serialization::cbor_serialize(val).unwrap();

        let session_data = self
            .verifier
            .process_message(&msg, &session_token.into(), url.clone())
            .await
            .unwrap();

        let response_msg = serialization::cbor_serialize(&session_data).unwrap();
        let response = serialization::cbor_deserialize(response_msg.as_slice()).unwrap();

        Ok(response)
    }
}

fn setup_verifier_test(
    mdoc_trust_anchors: &[TrustAnchor<'_>],
    authorized_requests: &ItemsRequests,
) -> (MockDisclosureHttpClient, Arc<MockVerifier>, Certificate) {
    let reader_registration = Some(ReaderRegistration::new_mock_from_requests(authorized_requests));
    let ca = KeyPair::generate_reader_mock_ca().unwrap();

    let use_cases = HashMap::from([
        (
            NO_RETURN_URL_USE_CASE.to_string(),
            UseCase {
                key_pair: ca.generate_reader_mock(reader_registration.clone()).unwrap(),
                session_type_return_url: SessionTypeReturnUrl::Neither,
            },
        ),
        (
            DEFAULT_RETURN_URL_USE_CASE.to_string(),
            UseCase {
                key_pair: ca.generate_reader_mock(reader_registration.clone()).unwrap(),
                session_type_return_url: SessionTypeReturnUrl::SameDevice,
            },
        ),
        (
            ALL_RETURN_URL_USE_CASE.to_string(),
            UseCase {
                key_pair: ca.generate_reader_mock(reader_registration).unwrap(),
                session_type_return_url: SessionTypeReturnUrl::Both,
            },
        ),
    ])
    .into();

    let verifier = MockVerifier::new(
        use_cases,
        MemorySessionStore::default(),
        mdoc_trust_anchors.iter().map(|anchor| anchor.into()).collect(),
        hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
    )
    .into();
    let client = MockDisclosureHttpClient::new(Arc::clone(&verifier));

    (client, verifier, ca.into())
}

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
async fn test_disclosure(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] use_case: &str,
    #[case] stored_documents: TestDocuments,
    #[case] requested_documents: ItemsRequests,
    #[case] expected_documents: TestDocuments,
) {
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let key_factory = SoftwareKeyFactory::default();

    let mdocs = future::join_all(
        stored_documents
            .into_iter()
            .map(|doc| async { doc.sign(&ca, &key_factory, NonZeroU8::new(1).unwrap()).await }),
    )
    .await;

    let mdoc_ca = ca.certificate().clone();

    let mdoc_data_source = MockMdocDataSource::from(mdocs);

    // Prepare a request and start disclosure on the verifier side.
    let authorized_documents = &requested_documents;
    let (verifier_client, verifier, verifier_ca) =
        setup_verifier_test(&[(&mdoc_ca).try_into().unwrap()], authorized_documents);

    let session_token = verifier
        .new_session(requested_documents, use_case.to_string(), return_url_template)
        .await
        .expect("creating new verifier session should succeed");

    let response = verifier
        .status_response(
            &session_token,
            session_type,
            &"https://app.example.com/app".parse().unwrap(),
            &"https://example.com/disclosure".parse().unwrap(),
            &TimeGenerator,
        )
        .await
        .expect("should return status response for session");

    let engagement_url = match response {
        StatusResponse::Created { engagement_url } => engagement_url,
        _ => panic!("should match StatusResponse::Created"),
    };

    // Deserialize the `ReaderEngagement` from the URL, just to make sure it's the correct type
    let reader_engagement: ReaderEngagement = cbor_deserialize(
        BASE64_URL_SAFE_NO_PAD
            .decode(engagement_url.path_segments().unwrap().last().unwrap())
            .expect("serializing an engagement should never fail")
            .as_slice(),
    )
    .expect("should always deserialize");

    // Determine the correct source for the session type.
    let reader_engagement_source = match session_type {
        SessionType::SameDevice => DisclosureUriSource::Link,
        SessionType::CrossDevice => DisclosureUriSource::QrCode,
    };

    // Encode the `ReaderEngagement` and start the disclosure session on the holder side.
    let reader_engagement_bytes = serialization::cbor_serialize(&reader_engagement).unwrap();
    let disclosure_session = DisclosureSession::start(
        verifier_client,
        &reader_engagement_bytes,
        reader_engagement_source,
        &mdoc_data_source,
        &[(&verifier_ca).try_into().unwrap()],
    )
    .await
    .expect("starting disclosure session should succeed");

    // Actually disclose the requested attributes.
    let disclosure_session_proposal = match disclosure_session {
        DisclosureSession::Proposal(proposal) => proposal,
        _ => panic!("disclosure session should contain proposal"),
    };

    // Get the disclosed attributes from the verifier session.
    disclosure_session_proposal
        .disclose(&key_factory)
        .await
        .expect("disclosure of proposed attributes should succeed");

    let return_url_nonce = disclosure_session_proposal
        .return_url()
        .and_then(|return_url| return_url.query_pairs().find(|(key, _)| key == "nonce"))
        .map(|(_, nonce)| nonce.into_owned());

    // Check if we received a return URL when we should have, based on the use case and session type.
    let should_have_return_url = match (use_case, session_type) {
        (use_case, _) if use_case == NO_RETURN_URL_USE_CASE => false,
        (use_case, _) if use_case == ALL_RETURN_URL_USE_CASE => true,
        (_, SessionType::SameDevice) => true,
        (_, SessionType::CrossDevice) => false,
    };
    assert_eq!(return_url_nonce.is_some(), should_have_return_url);

    if return_url_nonce.is_some() {
        let error = verifier
            .disclosed_attributes(&session_token, None)
            .await
            .expect_err("fetching disclosed attributes without a return URL nonce should fail");

        assert_matches!(error, Error::Verification(VerificationError::ReturnUrlNonceMissing));

        let error = verifier
            .disclosed_attributes(&session_token, "incorrect".to_string().into())
            .await
            .expect_err("fetching disclosed attributes with incorrect return URL nonce should fail");

        assert_matches!(
            error,
            Error::Verification(VerificationError::ReturnUrlNonceMismatch(nonce)) if nonce == "incorrect"
        );
    }

    let disclosed_documents = verifier
        .disclosed_attributes(&session_token, return_url_nonce)
        .await
        .expect("verifier disclosed attributes should be present");

    expected_documents.assert_matches(&disclosed_documents);
}
