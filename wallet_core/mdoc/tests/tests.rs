use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    ops::Add,
    sync::Arc,
};

use chrono::{DateTime, Duration, Utc};
use ciborium::value::Value;
use indexmap::{IndexMap, IndexSet};
use rstest::rstest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{
    base64::{Base64, UrlSafe},
    formats::Unpadded,
    serde_as,
};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    holder::{DisclosureSession, HttpClient, HttpClientResult, MdocCopies, MdocDataSource, StoredMdoc, Wallet},
    identifiers::AttributeIdentifier,
    iso::{device_retrieval::ItemsRequest, mdocs::DocType},
    issuer::{IssuanceData, Issuer},
    server_keys::{KeyRing, PrivateKey},
    server_state::MemorySessionStore,
    software_key_factory::SoftwareKeyFactory,
    utils::{
        auth::reader_auth::mock::reader_registration_mock,
        reader_auth::ReaderRegistration,
        serialization,
        x509::{Certificate, CertificateType},
    },
    verifier::{DisclosureData, SessionType, Verifier},
};
use webpki::TrustAnchor;

const ISSUANCE_DOC_TYPE: &str = "example_doctype";
const ISSUANCE_NAME_SPACE: &str = "example_namespace";
const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

type MockIssuanceServer = Issuer<MockKeyring, MemorySessionStore<IssuanceData>>;
type MockVerifier = Verifier<MockKeyring, MemorySessionStore<DisclosureData>>;

struct MockKeyring {
    private_key: PrivateKey,
}

impl MockKeyring {
    pub fn new(private_key: PrivateKey) -> Self {
        MockKeyring { private_key }
    }
}

impl KeyRing for MockKeyring {
    fn private_key(&self, _: &str) -> Option<&PrivateKey> {
        Some(&self.private_key)
    }
}

struct MockIssuanceHttpClient {
    issuance_server: Arc<MockIssuanceServer>,
}

impl MockIssuanceHttpClient {
    pub fn new(issuance_server: Arc<MockIssuanceServer>) -> Self {
        MockIssuanceHttpClient { issuance_server }
    }
}

impl HttpClient for MockIssuanceHttpClient {
    async fn post<R, V>(&self, url: &Url, val: &V) -> HttpClientResult<R>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        let session_token = url.path_segments().unwrap().last().unwrap().to_string();
        let val = serialization::cbor_serialize(val).unwrap();
        let response = self
            .issuance_server
            .process_message(session_token.into(), &val)
            .await
            .unwrap();
        let response = serialization::cbor_deserialize(response.as_slice()).unwrap();
        Ok(response)
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

fn setup_issuance_test() -> (Wallet<MockIssuanceHttpClient>, Arc<MockIssuanceServer>, Certificate) {
    let (issuance_key, ca) = PrivateKey::generate_mock_with_ca().unwrap();

    // Setup issuer
    let issuance_server = MockIssuanceServer::new(
        "http://example.com".parse().unwrap(),
        MockKeyring::new(issuance_key),
        MemorySessionStore::new(),
    )
    .into();

    let client = MockIssuanceHttpClient::new(Arc::clone(&issuance_server));
    let wallet = Wallet::new(client);

    (wallet, issuance_server, ca)
}

pub const RP_CA_CN: &str = "ca.rp.example.com";
pub const RP_CERT_CN: &str = "cert.rp.example.com";

fn setup_verifier_test(
    mdoc_trust_anchors: &[TrustAnchor<'_>],
) -> (MockDisclosureHttpClient, Arc<MockVerifier>, Certificate) {
    let reader_registration = ReaderRegistration {
        attributes: ReaderRegistration::create_attributes(
            ISSUANCE_DOC_TYPE.to_string(),
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS.iter().map(|(key, _)| key).copied(),
        ),
        ..reader_registration_mock()
    };
    let (ca, ca_privkey) = Certificate::new_ca(RP_CA_CN).unwrap();
    let (disclosure_cert, disclosure_privkey) = Certificate::new(
        &ca,
        &ca_privkey,
        RP_CERT_CN,
        CertificateType::ReaderAuth(Box::new(reader_registration).into()),
    )
    .unwrap();
    let disclosure_key = PrivateKey::new(disclosure_privkey, disclosure_cert);

    let verifier = MockVerifier::new(
        "http://example.com".parse().unwrap(),
        MockKeyring::new(disclosure_key),
        MemorySessionStore::new(),
        mdoc_trust_anchors.iter().map(|anchor| anchor.into()).collect(),
    )
    .into();
    let client = MockDisclosureHttpClient::new(Arc::clone(&verifier));

    (client, verifier, ca)
}

struct MockMdocDataSource(HashMap<DocType, MdocCopies>);

impl From<Vec<MdocCopies>> for MockMdocDataSource {
    fn from(value: Vec<MdocCopies>) -> Self {
        MockMdocDataSource(
            value
                .into_iter()
                .map(|copies| (copies.cred_copies.first().unwrap().doc_type.clone(), copies))
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

fn new_issuance_request() -> Vec<UnsignedMdoc> {
    let now = chrono::Utc::now();
    vec![UnsignedMdoc {
        doc_type: ISSUANCE_DOC_TYPE.to_string(),
        copy_count: 2,
        valid_from: now.into(),
        valid_until: chrono::Utc::now().add(chrono::Duration::days(365)).into(),
        attributes: IndexMap::from([(
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS
                .iter()
                .map(|(key, val)| Entry {
                    name: key.to_string(),
                    value: Value::Text(val.to_string()),
                })
                .collect(),
        )]),
    }]
}

async fn issuance_using_consent<H>(
    user_consent: bool,
    request: Vec<UnsignedMdoc>,
    wallet: &mut Wallet<H>,
    issuance_server: &MockIssuanceServer,
    ca: &Certificate,
) -> Option<Vec<MdocCopies>>
where
    H: HttpClient,
{
    let service_engagement = issuance_server
        .new_session(request)
        .await
        .expect("starting a new issuance session on the server should succeed");

    wallet
        .start_issuance(service_engagement)
        .await
        .expect("starting issuance on the Wallet should succeed");

    if !user_consent {
        wallet
            .stop_issuance()
            .await
            .expect("stopping issuance on the Wallet should succeed");

        return None;
    }

    let mdocs = wallet
        .finish_issuance(&[ca.try_into().unwrap()], &SoftwareKeyFactory::default())
        .await
        .expect("finishing issuance on the Wallet should succeed");

    mdocs.into()
}

#[tokio::test]
async fn test_issuance() {
    let expected_identifiers = ISSUANCE_ATTRS
        .iter()
        .map(|(key, _)| AttributeIdentifier {
            doc_type: ISSUANCE_DOC_TYPE.to_string(),
            namespace: ISSUANCE_NAME_SPACE.to_string(),
            attribute: key.to_string(),
        })
        .collect::<IndexSet<_>>();

    // Agree with issuance
    let (mut wallet, server, ca) = setup_issuance_test();
    let mdocs = issuance_using_consent(true, new_issuance_request(), &mut wallet, server.as_ref(), &ca)
        .await
        .unwrap();
    assert_eq!(1, mdocs.len());
    let mdoc_copies = mdocs.first().unwrap();
    assert_eq!(mdoc_copies.cred_copies.len(), 2);
    assert_eq!(
        mdoc_copies
            .cred_copies
            .first()
            .unwrap()
            .issuer_signed_attribute_identifiers(),
        expected_identifiers,
    );

    // Decline issuance
    let (mut wallet, server, ca) = setup_issuance_test();
    let mdocs = issuance_using_consent(false, new_issuance_request(), &mut wallet, server.as_ref(), &ca).await;
    assert!(mdocs.is_none());

    // Issue not-yet-valid mdocs
    let now = Utc::now();
    let mut request = new_issuance_request();
    request
        .iter_mut()
        .for_each(|r| r.valid_from = now.add(Duration::days(132)).into());
    assert!(request[0].valid_from.0 .0.parse::<DateTime<Utc>>().unwrap() > now);

    let (mut wallet, server, ca) = setup_issuance_test();
    let mdocs = issuance_using_consent(true, new_issuance_request(), &mut wallet, server.as_ref(), &ca)
        .await
        .unwrap();
    assert_eq!(1, mdocs.len());
    let mdoc_copies = mdocs.first().unwrap();
    assert_eq!(mdoc_copies.cred_copies.len(), 2);
    assert_eq!(
        mdoc_copies
            .cred_copies
            .first()
            .unwrap()
            .issuer_signed_attribute_identifiers(),
        expected_identifiers,
    );
}

#[rstest]
#[case(SessionType::SameDevice, None)]
#[case(SessionType::SameDevice, Some("http://example.com/return_url".parse().unwrap()))]
#[case(SessionType::CrossDevice, None)]
#[case(SessionType::CrossDevice, Some("http://example.com/return_url".parse().unwrap()))]
#[tokio::test]
async fn test_issuance_and_disclosure(#[case] session_type: SessionType, #[case] return_url: Option<Url>) {
    // Perform issuance and save result in `MockMdocDataSource`.
    let (mut wallet, issuance_server, mdoc_ca) = setup_issuance_test();
    let mdoc_data_source: MockMdocDataSource = issuance_using_consent(
        true,
        new_issuance_request(),
        &mut wallet,
        issuance_server.as_ref(),
        &mdoc_ca,
    )
    .await
    .unwrap()
    .into();

    // Prepare a request and start issuance on the verifier side.
    let (verifier_client, verifier, verifier_ca) = setup_verifier_test(&[(&mdoc_ca).try_into().unwrap()]);
    let items_requests = vec![ItemsRequest {
        doc_type: ISSUANCE_DOC_TYPE.to_string(),
        name_spaces: IndexMap::from([(
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS.iter().map(|(key, _)| (key.to_string(), false)).collect(),
        )]),
        request_info: None,
    }]
    .into();

    let (session_id, reader_engagement) = verifier
        .new_session(items_requests, session_type, Default::default(), return_url.is_some())
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
        .disclose(&SoftwareKeyFactory::default())
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

    // Check the disclosed attributes.
    let attributes_iter = disclosed_attributes
        .get(ISSUANCE_DOC_TYPE)
        .expect("disclosed attributes should contain doc_type")
        .get(ISSUANCE_NAME_SPACE)
        .expect("disclosed attributes should contain namespace")
        .iter()
        .map(|entry| (entry.name.as_str(), entry.value.as_text().unwrap()));
    itertools::assert_equal(attributes_iter, ISSUANCE_ATTRS.iter().copied());
}
