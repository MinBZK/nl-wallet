use std::{ops::Add, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use ciborium::value::Value;
use indexmap::{IndexMap, IndexSet};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    errors::Result,
    holder::{HttpClient, MdocCopies, Wallet},
    identifiers::AttributeIdentifier,
    issuer::{IssuanceData, Issuer},
    mock::{self, SoftwareKeyFactory},
    server_keys::{KeyRing, PrivateKey},
    server_state::MemorySessionStore,
    utils::{
        serialization::{cbor_deserialize, cbor_serialize},
        x509::Certificate,
    },
};

const ISSUANCE_DOC_TYPE: &str = "example_doctype";
const ISSUANCE_NAME_SPACE: &str = "example_namespace";
const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

type MockIssuanceServer = Issuer<MockKeyring, MemorySessionStore<IssuanceData>>;

struct MockKeyring {
    private_key: PrivateKey,
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

#[async_trait]
impl HttpClient for MockIssuanceHttpClient {
    async fn post<R, V>(&self, url: &Url, val: &V) -> Result<R>
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

fn setup_issuance_test() -> (Wallet<MockIssuanceHttpClient>, Arc<MockIssuanceServer>, Certificate) {
    let (issuance_key, ca) = mock::generate_issuance_key_and_ca().unwrap();

    // Setup issuer
    let issuance_server = MockIssuanceServer::new(
        "http://example.com".parse().unwrap(),
        MockKeyring {
            private_key: issuance_key,
        },
        MemorySessionStore::new(),
    )
    .into();

    let client = MockIssuanceHttpClient::new(Arc::clone(&issuance_server));
    let wallet = Wallet::new(client);

    (wallet, issuance_server, ca)
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
) -> Result<Option<Vec<MdocCopies>>>
where
    H: HttpClient,
{
    let service_engagement = issuance_server.new_session(request).await?;

    wallet.start_issuance(service_engagement).await?;

    if !user_consent {
        wallet.stop_issuance().await?;

        return Ok(None);
    }

    let mdocs = wallet
        .finish_issuance(&[ca.try_into().unwrap()], &SoftwareKeyFactory::default())
        .await?;

    Ok(mdocs.into())
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
        .expect("issuance should succeed")
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
    let mdocs = issuance_using_consent(false, new_issuance_request(), &mut wallet, server.as_ref(), &ca)
        .await
        .expect("issuance should succeed");
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
        .expect("issuance should succeed")
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
