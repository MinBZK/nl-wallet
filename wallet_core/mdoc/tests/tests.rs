use std::{ops::Add, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use ciborium::value::Value;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    examples::*,
    holder::*,
    iso::*,
    issuer::*,
    mock::{self, DebugCollapseBts, SoftwareKeyFactory},
    server_keys::{KeyRing, PrivateKey},
    server_state::{MemorySessionStore, SessionState, SessionStore},
    utils::{
        serialization::{cbor_deserialize, cbor_serialize},
        x509::Certificate,
    },
    Error,
};

type MockWallet = Wallet<MockHttpClient<MockIssuanceKeyring, MemorySessionStore<IssuanceData>>>;
type MockServer = Issuer<MockIssuanceKeyring, MemorySessionStore<IssuanceData>>;

/// Construct the example mdoc from the standard and disclose attributes
/// by running the example device request from the standard against it.
#[tokio::test]
async fn do_and_verify_iso_example_disclosure() {
    let device_request = DeviceRequest::example();

    // Examine some fields in the device request
    let items_request = device_request.doc_requests.first().unwrap().items_request.0.clone();
    assert_eq!(items_request.doc_type, EXAMPLE_DOC_TYPE);
    let requested_attrs = items_request.name_spaces.get(EXAMPLE_NAMESPACE).unwrap();
    let intent_to_retain = requested_attrs.get(EXAMPLE_ATTR_NAME).unwrap();
    assert!(intent_to_retain);
    println!("DeviceRequest: {:#?}", DebugCollapseBts::from(&device_request));

    // Verify reader's request
    let reader_trust_anchors = Examples::reader_trust_anchors();
    let session_transcript = ReaderAuthenticationBytes::example().0 .0.session_transcript;
    let certificate = device_request
        .doc_requests
        .first()
        .unwrap()
        .verify(session_transcript, &IsoCertTimeGenerator, reader_trust_anchors)
        .unwrap();
    let reader_x509_subject = certificate.unwrap().subject();

    // The reader's certificate contains who it is
    assert_eq!(
        reader_x509_subject.as_ref().unwrap().first().unwrap(),
        (&"CN".to_string(), &"reader".to_string())
    );
    println!("Reader: {:#?}", reader_x509_subject);

    // // Construct the mdoc from the example device response in the standard
    // let trust_anchors = Examples::iaca_trust_anchors();
    // let mdoc = mock::mdoc_from_example_device_response(trust_anchors);

    // // Do the disclosure and verify it
    // let wallet = Wallet::new(DummyHttpClient);
    // let storage = MdocsMap::try_from([mdoc]).unwrap();
    // let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;
    // let resp = wallet
    //     .disclose(
    //         &device_request,
    //         &session_transcript.clone(),
    //         &SoftwareKeyFactory::default(),
    //         &storage,
    //     )
    //     .await
    //     .unwrap();
    // println!("DeviceResponse: {:#?}", DebugCollapseBts(&resp));

    // let disclosed_attrs = resp
    //     .verify(None, &session_transcript, &IsoCertTimeGenerator, trust_anchors)
    //     .unwrap();
    // println!("DisclosedAttributes: {:#?}", DebugCollapseBts(&disclosed_attrs));

    // // The first disclosed attribute is the same as we saw earlier in the DeviceRequest
    // assert_disclosure_contains(
    //     &disclosed_attrs,
    //     EXAMPLE_DOC_TYPE,
    //     EXAMPLE_NAMESPACE,
    //     EXAMPLE_ATTR_NAME,
    //     &EXAMPLE_ATTR_VALUE,
    // );
}

// /// Disclose some of the attributes of the example mdoc from the spec.
// #[tokio::test]
// async fn iso_examples_custom_disclosure() {
//     let request = DeviceRequest::new(vec![ItemsRequest {
//         doc_type: EXAMPLE_DOC_TYPE.to_string(),
//         name_spaces: IndexMap::from([(
//             EXAMPLE_NAMESPACE.to_string(),
//             IndexMap::from([(EXAMPLE_ATTR_NAME.to_string(), false)]),
//         )]),
//         request_info: None,
//     }]);
//     println!("My Request: {:#?}", DebugCollapseBts(&request));

//     let trust_anchors = Examples::iaca_trust_anchors();
//     let mdoc = mock::mdoc_from_example_device_response(trust_anchors);

//     let storage = MdocsMap::try_from([mdoc]).unwrap();
//     let wallet = Wallet::new(DummyHttpClient);
//     let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;

//     let resp = wallet
//         .disclose(
//             &request,
//             &session_transcript.clone(),
//             &SoftwareKeyFactory::default(),
//             &storage,
//         )
//         .await
//         .unwrap();
//     println!("My DeviceResponse: {:#?}", DebugCollapseBts(&resp));

//     let disclosed_attrs = resp
//         .verify(None, &session_transcript, &IsoCertTimeGenerator, trust_anchors)
//         .unwrap();
//     println!("My Disclosure: {:#?}", DebugCollapseBts(&disclosed_attrs));

//     // The first disclosed attribute is the one we requested in our device request
//     assert_disclosure_contains(
//         &disclosed_attrs,
//         EXAMPLE_DOC_TYPE,
//         EXAMPLE_NAMESPACE,
//         EXAMPLE_ATTR_NAME,
//         &EXAMPLE_ATTR_VALUE,
//     );
// }

const ISSUANCE_DOC_TYPE: &str = "example_doctype";
const ISSUANCE_NAME_SPACE: &str = "example_namespace";
const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

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

struct DummyHttpClient;

#[async_trait]
impl HttpClient for DummyHttpClient {
    async fn post<R, V>(&self, _: &Url, _: &V) -> Result<R, Error> {
        panic!("not implemented")
    }
}

struct MockHttpClient<K, S> {
    issuance_server: Arc<Issuer<K, S>>,
}

#[async_trait]
impl<K, S> HttpClient for MockHttpClient<K, S>
where
    K: KeyRing + Send + Sync,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    async fn post<R, V>(&self, url: &Url, val: &V) -> Result<R, Error>
    where
        V: Serialize + Sync,
        R: DeserializeOwned,
    {
        let session_token = url.path_segments().unwrap().last().unwrap().to_string();
        let val = &cbor_serialize(val).unwrap();
        let response = self
            .issuance_server
            .process_message(session_token.into(), val)
            .await
            .unwrap();
        let response = cbor_deserialize(response.as_slice()).unwrap();
        Ok(response)
    }
}

struct MockIssuanceKeyring {
    issuance_key: PrivateKey,
}
impl KeyRing for MockIssuanceKeyring {
    fn private_key(&self, _: &str) -> Option<&PrivateKey> {
        Some(&self.issuance_key)
    }
}

fn setup_issuance_test() -> (MockWallet, Arc<MockServer>, Certificate) {
    let (issuance_key, ca) = mock::generate_issuance_key_and_ca().unwrap();

    // Setup issuer
    let issuance_server = Arc::new(MockServer::new(
        "http://example.com".parse().unwrap(),
        MockIssuanceKeyring { issuance_key },
        MemorySessionStore::new(),
    ));

    let client = MockHttpClient {
        issuance_server: Arc::clone(&issuance_server),
    };
    let wallet = MockWallet::new(client);

    (wallet, issuance_server, ca)
}

#[tokio::test]
async fn issuance_and_disclosure() {
    // Agree with issuance
    let (mut wallet, server, ca) = setup_issuance_test();
    let mdocs = issuance_using_consent(true, new_issuance_request(), &mut wallet, Arc::clone(&server), &ca)
        .await
        .unwrap();
    assert_eq!(1, mdocs.len());

    // // We can disclose the mdoc that was just issued to us
    // let mdocs_map = mdocs.into_iter().flatten().collect::<Vec<_>>().try_into().unwrap();
    // custom_disclosure(wallet, ca, mdocs_map).await;

    // // Decline issuance
    // let (mut wallet, server, ca) = setup_issuance_test();
    // let mdocs = issuance_using_consent(false, new_issuance_request(), &mut wallet, Arc::clone(&server), &ca).await;
    // assert!(mdocs.is_none());

    // // Issue not-yet-valid mdocs
    // let now = Utc::now();
    // let mut request = new_issuance_request();
    // request
    //     .iter_mut()
    //     .for_each(|r| r.valid_from = now.add(Duration::days(132)).into());
    // assert!(request[0].valid_from.0 .0.parse::<DateTime<Utc>>().unwrap() > now);

    // let (mut wallet, server, ca) = setup_issuance_test();
    // let mdocs = issuance_using_consent(true, new_issuance_request(), &mut wallet, Arc::clone(&server), &ca)
    //     .await
    //     .unwrap();
    // assert_eq!(1, mdocs.len());
}

async fn issuance_using_consent(
    user_consent: bool,
    request: Vec<UnsignedMdoc>,
    wallet: &mut MockWallet,
    issuance_server: Arc<MockServer>,
    ca: &Certificate,
) -> Option<Vec<MdocCopies>> {
    let service_engagement = issuance_server.new_session(request).await.unwrap();

    wallet.start_issuance(service_engagement).await.unwrap();

    if !user_consent {
        wallet.stop_issuance().await.unwrap();
        return None;
    }

    let mdocs = wallet
        .finish_issuance(&[ca.try_into().unwrap()], &SoftwareKeyFactory::default())
        .await
        .unwrap();

    Some(mdocs)
}

// async fn custom_disclosure(wallet: MockWallet, ca: Certificate, mdocs: MdocsMap) {
//     assert!(!mdocs.list().is_empty());

//     // Create a request asking for one attribute
//     let request = DeviceRequest::new(vec![ItemsRequest {
//         doc_type: ISSUANCE_DOC_TYPE.to_string(),
//         name_spaces: IndexMap::from([(
//             ISSUANCE_NAME_SPACE.to_string(),
//             ISSUANCE_ATTRS.iter().map(|(key, _)| (key.to_string(), false)).collect(),
//         )]),
//         request_info: None,
//     }]);

//     // Do the disclosure and verify it
//     let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;
//     let disclosed = wallet
//         .disclose(
//             &request,
//             &session_transcript.clone(),
//             &SoftwareKeyFactory::default(),
//             &mdocs,
//         )
//         .await
//         .unwrap();
//     let disclosed_attrs = disclosed
//         .verify(None, &session_transcript, &TimeGenerator, &[(&ca).try_into().unwrap()])
//         .unwrap();
//     println!("Disclosure: {:#?}", DebugCollapseBts(&disclosed_attrs));

//     // Check the first disclosed attribute has the expected name and value
//     let attr = ISSUANCE_ATTRS.first().unwrap();
//     assert_disclosure_contains(
//         &disclosed_attrs,
//         ISSUANCE_DOC_TYPE,
//         ISSUANCE_NAME_SPACE,
//         attr.0,
//         &Value::Text(attr.1.to_string()),
//     );
// }
