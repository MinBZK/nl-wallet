use assert_matches::assert_matches;
use indexmap::IndexMap;
use reqwest::StatusCode;
use serial_test::serial;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    verifier::{SessionResult, SessionType, StatusResponse},
    ItemsRequest,
};
use wallet::{errors::DisclosureError, mock::MockDigidSession};
use wallet_server::verifier::{StartDisclosureRequest, StartDisclosureResponse};

use crate::common::*;

pub mod common;

async fn get_verifier_status(client: &reqwest::Client, session_url: Url) -> StatusResponse {
    let response = client.get(session_url).send().await.unwrap();

    assert!(response.status().is_success());

    response.json().await.unwrap()
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_disclosure_ok() {
    let digid_context = MockDigidSession::start_context();
    digid_context.expect().return_once(|_, _, _| {
        let mut session = MockDigidSession::default();

        session
            .expect_auth_url()
            .return_const(Url::parse("http://localhost/").unwrap());

        // Return a mock access token from the mock DigiD client that the `MockBsnLookup` always accepts.
        session
            .expect_get_access_token()
            .returning(|_| Ok("mock_token".to_string()));

        Ok(session)
    });

    let ws_settings = wallet_server_settings();

    let pin = "112233".to_string();
    let mut wallet = setup_wallet_and_env(wallet_provider_settings(), ws_settings.clone(), pid_issuer_settings()).await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "driving_license".to_owned(),
        session_type: SessionType::SameDevice,
        items_requests: vec![ItemsRequest {
            doc_type: "com.example.pid".to_owned(),
            request_info: None,
            name_spaces: IndexMap::from([(
                "com.example.pid".to_owned(),
                IndexMap::from_iter(
                    [("given_name", true), ("family_name", false)]
                        .iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), *intent_to_retain)),
                ),
            )]),
        }]
        .into(),
    };
    let response = client
        .post(
            ws_settings
                .internal_url
                .join("/sessions")
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
    } = response.json::<StartDisclosureResponse>().await.unwrap();

    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::Created
    );

    let mut url = engagement_url.clone();
    url.set_query(Some("session_type=same_device"));

    let proposal = wallet.start_disclosure(&url).await.expect("Could not start disclosure");
    assert_eq!(proposal.reader_registration.id, "some-service-id");
    assert_eq!(proposal.documents.len(), 1);

    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    wallet
        .accept_disclosure(pin)
        .await
        .expect("Could not accept disclosure");

    let expected_entries = vec![
        Entry {
            name: "family_name".into(),
            value: "De Bruijn".into(),
        },
        Entry {
            name: "given_name".into(),
            value: "Willeke Liselotte".into(),
        },
    ];
    assert_matches!(
        get_verifier_status(&client, session_url).await,
        StatusResponse::Done(SessionResult::Done { disclosed_attributes })
        if disclosed_attributes.get("com.example.pid").unwrap().get("com.example.pid").unwrap() == &expected_entries
    );
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_disclosure_without_pid() {
    let digid_context = MockDigidSession::start_context();
    digid_context.expect().return_once(|_, _, _| {
        let mut session = MockDigidSession::default();

        session
            .expect_auth_url()
            .return_const(Url::parse("http://localhost/").unwrap());

        // Return a mock access token from the mock DigiD client that the `MockBsnLookup` always accepts.
        session
            .expect_get_access_token()
            .returning(|_| Ok("mock_token".to_string()));

        Ok(session)
    });

    let ws_settings = wallet_server_settings();

    let pin = "112233".to_string();
    let mut wallet = setup_wallet_and_env(wallet_provider_settings(), ws_settings.clone(), pid_issuer_settings()).await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "driving_license".to_owned(),
        session_type: SessionType::SameDevice,
        items_requests: vec![ItemsRequest {
            doc_type: "com.example.pid".to_owned(),
            request_info: None,
            name_spaces: IndexMap::from([(
                "com.example.pid".to_owned(),
                IndexMap::from_iter(
                    [("given_name", true), ("family_name", false)]
                        .iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), *intent_to_retain)),
                ),
            )]),
        }]
        .into(),
    };
    let response = client
        .post(
            ws_settings
                .internal_url
                .join("/sessions")
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
    } = response.json::<StartDisclosureResponse>().await.unwrap();
    let response = client.get(session_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let mut url = engagement_url.clone();
    url.set_query(Some("session_type=same_device"));

    let error = wallet
        .start_disclosure(&url)
        .await
        .expect_err("Should return error that attributes are not available");

    assert_matches!(
        error,
        DisclosureError::AttributesNotAvailable {
            reader_registration: _,
            missing_attributes: attrs
        } if attrs
            .iter()
            .flat_map(|attr| attr.attributes.keys().map(|k| k.to_owned()).collect::<Vec<&str>>())
            .collect::<Vec<&str>>() == vec!["given_name", "family_name"]
    );
}
