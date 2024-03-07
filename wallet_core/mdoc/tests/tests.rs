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
    iso::{device_retrieval::ItemsRequest, mdocs::DocType},
    server_keys::{KeyPair, KeyRing},
    server_state::MemorySessionStore,
    software_key_factory::SoftwareKeyFactory,
    unsigned::{Entry, UnsignedMdoc},
    utils::{
        issuer_auth::IssuerRegistration, keys::KeyFactory, reader_auth::ReaderRegistration, serialization,
        x509::Certificate,
    },
    verifier::{DisclosureData, SessionType, Verifier},
    IssuerSigned,
};
use wallet_common::{
    generator::TimeGenerator,
    keys::{EcdsaKey, WithIdentifier},
};
use webpki::TrustAnchor;

const ISSUANCE_DOC_TYPE: &str = "example_doctype";
const ISSUANCE_NAME_SPACE: &str = "example_namespace";
const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

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
) -> (MockDisclosureHttpClient, Arc<MockVerifier>, Certificate) {
    let reader_registration = ReaderRegistration {
        attributes: ReaderRegistration::create_attributes(
            ISSUANCE_DOC_TYPE.to_string(),
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS.iter().map(|(key, _)| key).copied(),
        ),
        ..ReaderRegistration::new_mock()
    };
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

/// Generates a valid `Mdoc`, based on an `UnsignedMdoc` and key identifier.
pub async fn mdoc_from_unsigned<KF>(unsigned_mdoc: UnsignedMdoc, key_factory: &KF) -> (Mdoc, Certificate)
where
    KF: KeyFactory,
{
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

    let mdoc_key = key_factory.generate_new().await.unwrap();
    let mdoc_public_key = (&mdoc_key.verifying_key().await.unwrap()).try_into().unwrap();
    let (issuer_signed, _) = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, &issuance_key)
        .await
        .unwrap();

    let mdoc = Mdoc::new::<KF::Key>(
        mdoc_key.identifier().to_string(),
        issuer_signed,
        &TimeGenerator,
        &[(ca.certificate().try_into().unwrap())],
    )
    .unwrap();

    (mdoc, ca.certificate().clone())
}

#[rstest]
#[case(SessionType::SameDevice, None)]
#[case(SessionType::SameDevice, Some("http://example.com/return_url".parse().unwrap()))]
#[case(SessionType::CrossDevice, None)]
#[case(SessionType::CrossDevice, Some("http://example.com/return_url".parse().unwrap()))]
#[tokio::test]
async fn test_disclosure(#[case] session_type: SessionType, #[case] return_url: Option<Url>) {
    let unsigned = UnsignedMdoc {
        doc_type: ISSUANCE_DOC_TYPE.to_string(),
        copy_count: 2,
        valid_from: chrono::Utc::now().into(),
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
    };

    let key_factory = SoftwareKeyFactory::default();
    let (mdoc, mdoc_ca) = mdoc_from_unsigned(unsigned, &key_factory).await;

    let mdoc_data_source = MockMdocDataSource::from(vec![MdocCopies {
        cred_copies: vec![mdoc],
    }]);

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
