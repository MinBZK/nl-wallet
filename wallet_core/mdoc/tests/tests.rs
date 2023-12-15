use std::{ops::Add, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use ciborium::value::Value;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    holder::*,
    issuer::*,
    mock::{self, SoftwareKeyFactory},
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

    // Decline issuance
    let (mut wallet, server, ca) = setup_issuance_test();
    let mdocs = issuance_using_consent(false, new_issuance_request(), &mut wallet, Arc::clone(&server), &ca).await;
    assert!(mdocs.is_none());

    // Issue not-yet-valid mdocs
    let now = Utc::now();
    let mut request = new_issuance_request();
    request
        .iter_mut()
        .for_each(|r| r.valid_from = now.add(Duration::days(132)).into());
    assert!(request[0].valid_from.0 .0.parse::<DateTime<Utc>>().unwrap() > now);

    let (mut wallet, server, ca) = setup_issuance_test();
    let mdocs = issuance_using_consent(true, new_issuance_request(), &mut wallet, Arc::clone(&server), &ca)
        .await
        .unwrap();
    assert_eq!(1, mdocs.len());
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
