use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    num::NonZeroU8,
    sync::Arc,
};

use futures::future;
use rstest::rstest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{
    base64::{Base64, UrlSafe},
    formats::Unpadded,
    serde_as,
};
use url::Url;
use webpki::TrustAnchor;

use nl_wallet_mdoc::{
    holder::{DisclosureSession, HttpClient, HttpClientResult, Mdoc, MdocCopies, MdocDataSource, StoredMdoc},
    iso::mdocs::DocType,
    server_keys::{KeyPair, KeyRing},
    server_state::MemorySessionStore,
    software_key_factory::SoftwareKeyFactory,
    test::{
        data::{addr_street, pid_full_name, pid_given_name},
        TestDocuments,
    },
    utils::{reader_auth::ReaderRegistration, serialization, x509::Certificate},
    verifier::{DisclosureData, ItemsRequests, SessionType, Verifier},
};

type MockVerifier = Verifier<MockKeyring, MemorySessionStore<DisclosureData>>;

struct MockKeyring {
    private_key: KeyPair,
}

impl MockKeyring {
    pub fn new(private_key: KeyPair) -> Self {
        MockKeyring { private_key }
    }
}

impl KeyRing for MockKeyring {
    fn private_key(&self, _: &str) -> Option<&KeyPair> {
        Some(&self.private_key)
    }
}

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

        let session_data = self.verifier.process_message(&msg, session_token.into()).await.unwrap();

        let response_msg = serialization::cbor_serialize(&session_data).unwrap();
        let response = serialization::cbor_deserialize(response_msg.as_slice()).unwrap();

        Ok(response)
    }
}

fn setup_verifier_test(
    mdoc_trust_anchors: &[TrustAnchor<'_>],
    authorized_requests: &ItemsRequests,
) -> (MockDisclosureHttpClient, Arc<MockVerifier>, Certificate) {
    let reader_registration = ReaderRegistration::new_mock_from_requests(authorized_requests);
    let ca = KeyPair::generate_reader_mock_ca().unwrap();
    let disclosure_key = ca.generate_reader_mock(reader_registration.into()).unwrap();

    let verifier = MockVerifier::new(
        "http://example.com".parse().unwrap(),
        MockKeyring::new(disclosure_key),
        MemorySessionStore::new(Default::default()),
        mdoc_trust_anchors.iter().map(|anchor| anchor.into()).collect(),
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
#[case(SessionType::SameDevice, None, pid_full_name(), pid_full_name().into(), pid_full_name())]
#[case(SessionType::SameDevice, Some("http://example.com/return_url".parse().unwrap()), pid_full_name(), pid_full_name().into(), pid_full_name())]
#[case(SessionType::CrossDevice, None, pid_full_name(), pid_full_name().into(), pid_full_name())]
#[case(SessionType::CrossDevice, Some("http://example.com/return_url".parse().unwrap()), pid_full_name(), pid_full_name().into(), pid_full_name())]
#[case(SessionType::SameDevice, None, pid_full_name(), pid_given_name().into(), pid_given_name())]
#[case(SessionType::SameDevice, None, pid_given_name(), pid_given_name().into(), pid_given_name())]
#[case(SessionType::SameDevice, None, pid_given_name() + addr_street(), (pid_given_name() + addr_street()).into(), pid_given_name() + addr_street())]
#[case(SessionType::SameDevice, None, pid_given_name() + addr_street(), (pid_given_name() + addr_street()).into(), pid_given_name() + addr_street())]
#[case(SessionType::SameDevice, None, pid_given_name() + addr_street(), pid_given_name().into(), pid_given_name())]
#[case(SessionType::SameDevice, None, pid_full_name(), (pid_given_name() + pid_given_name()).into(), pid_given_name())]
#[case(SessionType::SameDevice, None, pid_given_name(), (pid_given_name() + pid_given_name()).into(), pid_given_name())]
#[tokio::test]
async fn test_disclosure(
    #[case] session_type: SessionType,
    #[case] return_url: Option<Url>,
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

    let (session_id, reader_engagement) = verifier
        .new_session(
            requested_documents,
            session_type,
            Default::default(),
            return_url.is_some(),
        )
        .await
        .expect("creating new verifier session should succeed");

    // Encode the `ReaderEngagement` and start the disclosure session on the holder side.
    let reader_engagement_bytes = serialization::cbor_serialize(&reader_engagement).unwrap();
    let disclosure_session = DisclosureSession::start(
        verifier_client,
        &reader_engagement_bytes,
        return_url,
        session_type,
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

    // Note: same as in wallet_server, but not needed anywhere else in this crate
    #[serde_as]
    #[derive(Deserialize)]
    struct DisclosedAttributesParams {
        #[serde_as(as = "Option<Base64<UrlSafe, Unpadded>>")]
        transcript_hash: Option<Vec<u8>>,
    }

    let transcript_hash = disclosure_session_proposal.return_url().and_then(|u| {
        serde_urlencoded::from_str::<DisclosedAttributesParams>(u.query().unwrap_or(""))
            .expect("query of return URL should always parse")
            .transcript_hash
    });

    let disclosed_documents = verifier
        .disclosed_attributes(&session_id, transcript_hash)
        .await
        .expect("verifier disclosed attributes should be present");

    expected_documents.assert_matches(&disclosed_documents);
}
