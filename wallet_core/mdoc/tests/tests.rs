use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    ops::Add,
    sync::Arc,
};

use ciborium::value::Value;
use indexmap::IndexMap;
use rstest::rstest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{
    base64::{Base64, UrlSafe},
    formats::Unpadded,
    serde_as,
};
use url::Url;

use nl_wallet_mdoc::{
    holder::{DisclosureSession, HttpClient, HttpClientResult, Mdoc, MdocCopies, MdocDataSource, StoredMdoc},
    iso::mdocs::DocType,
    server_keys::{KeyPair, KeyRing},
    server_state::MemorySessionStore,
    software_key_factory::SoftwareKeyFactory,
    unsigned::{Entry, UnsignedMdoc},
    utils::{
        issuer_auth::IssuerRegistration,
        keys::KeyFactory,
        reader_auth::{AuthorizedAttribute, AuthorizedMdoc, AuthorizedNamespace, ReaderRegistration},
        serialization,
        x509::Certificate,
    },
    verifier::{DisclosureData, ItemsRequests, SessionType, Verifier},
    IssuerSigned,
};
use wallet_common::{
    generator::TimeGenerator,
    keys::{EcdsaKey, WithIdentifier},
};
use webpki::TrustAnchor;

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

fn reader_registration_for(items_requests: &ItemsRequests) -> ReaderRegistration {
    let attributes = items_requests
        .0
        .iter()
        .map(|items_request| {
            let namespaces: IndexMap<_, _> = items_request
                .name_spaces
                .iter()
                .map(|(namespace, attributes)| {
                    let authorized_attributes = attributes
                        .iter()
                        .map(|attribute| (attribute.0.clone(), AuthorizedAttribute {}))
                        .collect();

                    (namespace.clone(), AuthorizedNamespace(authorized_attributes))
                })
                .collect();

            (items_request.doc_type.clone(), AuthorizedMdoc(namespaces))
        })
        .collect();
    ReaderRegistration {
        attributes,
        ..ReaderRegistration::new_mock()
    }
}

fn setup_verifier_test(
    mdoc_trust_anchors: &[TrustAnchor<'_>],
    items_requests: &ItemsRequests,
) -> (MockDisclosureHttpClient, Arc<MockVerifier>, Certificate) {
    let reader_registration = reader_registration_for(items_requests);
    let ca = KeyPair::generate_reader_mock_ca().unwrap();
    let disclosure_key = ca.generate_reader_mock(reader_registration.into()).unwrap();

    let verifier = MockVerifier::new(
        "http://example.com".parse().unwrap(),
        MockKeyring::new(disclosure_key),
        MemorySessionStore::new(),
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

/// Generates a valid `Mdoc`, based on an `UnsignedMdoc` and key identifier.
pub async fn mdoc_from_unsigned<KF>(ca: &KeyPair, unsigned_mdoc: UnsignedMdoc, key_factory: &KF) -> Mdoc
where
    KF: KeyFactory,
{
    let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

    let mdoc_key = key_factory.generate_new().await.unwrap();
    let mdoc_public_key = (&mdoc_key.verifying_key().await.unwrap()).try_into().unwrap();
    let (issuer_signed, _) = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, &issuance_key)
        .await
        .unwrap();

    Mdoc::new::<KF::Key>(
        mdoc_key.identifier().to_string(),
        issuer_signed,
        &TimeGenerator,
        &[(ca.certificate().try_into().unwrap())],
    )
    .unwrap()
}

struct AttributesWithValue(&'static str, &'static str, Vec<Entry>);

impl From<(&'static str, &'static str, Vec<(&'static str, Value)>)> for AttributesWithValue {
    fn from((doc_type, namespace, attributes): (&'static str, &'static str, Vec<(&'static str, Value)>)) -> Self {
        Self::new(doc_type, namespace, attributes)
    }
}

impl AttributesWithValue {
    fn new(doc_type: &'static str, namespace: &'static str, attributes: Vec<(&'static str, Value)>) -> Self {
        Self(
            doc_type,
            namespace,
            attributes
                .into_iter()
                .map(|(name, value)| Entry {
                    name: name.into(),
                    value,
                })
                .collect(),
        )
    }

    fn doc_type(&self) -> &'static str {
        self.0
    }
    fn namespace(&self) -> &'static str {
        self.1
    }
    fn attributes(&self) -> &Vec<Entry> {
        &self.2
    }
}

impl From<AttributesWithValue> for UnsignedMdoc {
    fn from(value: AttributesWithValue) -> Self {
        Self {
            doc_type: value.doc_type().to_string(),
            copy_count: 2,
            valid_from: chrono::Utc::now().into(),
            valid_until: chrono::Utc::now().add(chrono::Duration::days(365)).into(),
            attributes: IndexMap::from([(value.namespace().to_string(), value.attributes().to_vec())]),
        }
    }
}

fn request_full_name() -> ItemsRequests {
    ItemsRequests::from(IndexMap::from_iter(vec![(
        "passport",
        IndexMap::from_iter([("identity", vec!["first_name", "family_name"])]),
    )]))
}

fn full_name() -> Vec<AttributesWithValue> {
    vec![(
        "passport",
        "identity",
        vec![("first_name", "John".into()), ("family_name", "Doe".into())],
    )
        .into()]
}

fn request_first_name() -> ItemsRequests {
    ItemsRequests::from(IndexMap::from_iter(vec![(
        "passport",
        IndexMap::from_iter([("identity", vec!["first_name"])]),
    )]))
}

fn request_first_name_double() -> ItemsRequests {
    ItemsRequests::from(vec![
        ("passport", "identity", "first_name"),
        ("passport", "identity", "first_name"),
    ])
}

fn first_name() -> Vec<AttributesWithValue> {
    vec![("passport", "identity", vec![("first_name", "John".into())]).into()]
}

fn request_two_cards() -> ItemsRequests {
    ItemsRequests::from(IndexMap::from_iter(vec![
        ("passport", IndexMap::from_iter([("identity", vec!["first_name"])])),
        ("driver_license", IndexMap::from_iter([("residence", vec!["city"])])),
    ]))
}

fn two_cards() -> Vec<AttributesWithValue> {
    vec![
        ("passport", "identity", vec![("first_name", "John".into())]).into(),
        ("driver_license", "residence", vec![("city", "Ons Dorp".into())]).into(),
    ]
}

#[rstest]
#[case(SessionType::SameDevice, None, full_name(), request_full_name(), full_name())]
#[case(SessionType::SameDevice, Some("http://example.com/return_url".parse().unwrap()), full_name(), request_full_name(), full_name())]
#[case(SessionType::CrossDevice, None, full_name(), request_full_name(), full_name())]
#[case(SessionType::CrossDevice, Some("http://example.com/return_url".parse().unwrap()), full_name(), request_full_name(), full_name())]
#[case(SessionType::SameDevice, None, full_name(), request_first_name(), first_name())]
#[case(SessionType::SameDevice, None, first_name(), request_first_name(), first_name())]
#[case(SessionType::SameDevice, None, two_cards(), request_two_cards(), two_cards())]
#[case(SessionType::SameDevice, None, two_cards(), request_first_name(), first_name())]
#[case(
    SessionType::SameDevice,
    None,
    full_name(),
    request_first_name_double(),
    first_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    first_name(),
    request_first_name_double(),
    first_name()
)]
#[tokio::test]
async fn test_issuance_and_disclosure(
    #[case] session_type: SessionType,
    #[case] return_url: Option<Url>,
    #[case] stored_attributes: Vec<AttributesWithValue>,
    #[case] requested_attributes: ItemsRequests,
    #[case] expected_attributes: Vec<AttributesWithValue>,
) {
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let key_factory = SoftwareKeyFactory::default();

    let mdocs = {
        let mut mdocs = vec![];

        for doc in stored_attributes {
            let unsigned = UnsignedMdoc::from(doc);
            let mdoc = mdoc_from_unsigned(&ca, unsigned, &key_factory).await;
            mdocs.push(mdoc);
        }

        mdocs
    };

    let mdoc_ca = ca.certificate().clone();

    let mdoc_data_source = MockMdocDataSource::from(mdocs);

    // Prepare a request and start issuance on the verifier side.
    let authorized_attributes = requested_attributes.clone();
    let (verifier_client, verifier, verifier_ca) =
        setup_verifier_test(&[(&mdoc_ca).try_into().unwrap()], &authorized_attributes);

    let (session_id, reader_engagement) = verifier
        .new_session(
            requested_attributes,
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

    let disclosed_attributes = verifier
        .disclosed_attributes(&session_id, transcript_hash)
        .await
        .expect("verifier disclosed attributes should be present");

    for AttributesWithValue(doc_type, namespace, expected_entries) in expected_attributes.into_iter() {
        // Check the disclosed attributes.
        itertools::assert_equal(
            disclosed_attributes
                .get(doc_type)
                .expect("expected doc_type not received")
                .get(namespace)
                .expect("expected namespace not received"),
            &expected_entries,
        );
    }
}
