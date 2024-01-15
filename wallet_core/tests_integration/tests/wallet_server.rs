use std::collections::HashMap;

use base64::prelude::*;
use indexmap::IndexMap;
use p256::pkcs8::EncodePrivateKey;
use reqwest::StatusCode;
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionToken,
    utils::{
        reader_auth::ReaderRegistration,
        serialization::cbor_deserialize,
        x509::{Certificate, CertificateType},
    },
    verifier::SessionType,
    ItemsRequest, ReaderEngagement,
};
use wallet_server::{
    settings::{KeyPair, Settings},
    store::DisclosureSessionStore,
    verifier::{StartDisclosureRequest, StartDisclosureResponse},
};

use crate::common::*;

pub mod common;

fn wallet_server_settings() -> Settings {
    let mut settings = common::wallet_server_settings();
    settings.usecases = HashMap::new();
    settings.trust_anchors = Vec::new();

    #[cfg(feature = "postgres")]
    {
        settings.store_url = "postgres://postgres@127.0.0.1:5432/wallet_server".parse().unwrap();
    }
    #[cfg(not(feature = "postgres"))]
    {
        settings.store_url = "memory://".parse().unwrap();
    }

    let (ca, ca_privkey) = Certificate::new_ca("ca.example.com").unwrap();
    let (cert, cert_privkey) = Certificate::new(
        &ca,
        &ca_privkey,
        "cert.example.com",
        CertificateType::ReaderAuth(Box::new(ReaderRegistration::mock_reader_registration()).into()),
    )
    .unwrap();

    settings.usecases.insert(
        "example_usecase".to_owned(),
        KeyPair {
            certificate: cert.as_bytes().to_vec().into(),
            private_key: cert_privkey
                .to_pkcs8_der()
                .expect("could not serialize private key")
                .as_bytes()
                .to_vec()
                .into(),
        },
    );

    settings
}

fn parse_wallet_url(engagement_url: Url) -> Url {
    let reader_engagement: ReaderEngagement = cbor_deserialize(
        BASE64_URL_SAFE_NO_PAD
            .decode(
                engagement_url
                    .path_segments()
                    .expect("no path in engagement_url")
                    .last()
                    .expect("empty path in engagement_url"),
            )
            .unwrap()
            .as_slice(),
    )
    .unwrap();

    reader_engagement
        .0
        .connection_methods
        .expect("no connection_methods in reader_engagement")
        .first()
        .expect("empty connection_methods in reader_engagement")
        .0
        .connection_options
        .0
        .uri
        .clone()
}

#[tokio::test]
async fn test_start_session() {
    let settings = wallet_server_settings();
    let sessions = DisclosureSessionStore::init(settings.store_url.clone()).await.unwrap();

    start_wallet_server(settings.clone(), sessions).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "example_usecase".to_owned(),
        session_type: SessionType::SameDevice,
        items_requests: vec![ItemsRequest {
            doc_type: "example_doctype".to_owned(),
            request_info: None,
            name_spaces: IndexMap::from([(
                "example_namespace".to_owned(),
                IndexMap::from_iter(
                    [("first_name", true), ("family_name", false)]
                        .iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), *intent_to_retain)),
                ),
            )]),
        }]
        .into(),
        return_url_template: None,
    };
    let response = client
        .post(
            settings
                .internal_url
                .join("sessions")
                .expect("could not join url with endpoint"),
        )
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the RP side of things?
    let StartDisclosureResponse {
        session_url,
        engagement_url,
        ..
    } = response.json::<StartDisclosureResponse>().await.unwrap();
    let response = client.get(session_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the wallet side of things?
    let wallet_url = parse_wallet_url(engagement_url);
    let response = client.post(wallet_url).body("hello").send().await.unwrap();

    assert_ne!(response.status(), StatusCode::NOT_FOUND);

    // TODO construct a valid body when we have the code to do so, until then this is a bad request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_session_not_found() {
    let settings = wallet_server_settings();
    let sessions = DisclosureSessionStore::init(settings.store_url.clone()).await.unwrap();

    start_wallet_server(settings.clone(), sessions).await;

    let client = reqwest::Client::new();
    // does it exist for the RP side of things?
    let response = client
        .get(
            settings
                .internal_url
                .join(&format!("/{}/status", SessionToken::new()))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // does it exist for the wallet side of things?
    let response = client
        .post(settings.public_url.join(&format!("/{}", SessionToken::new())).unwrap())
        // .json(...) // there's no way to construct a valid body here
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
