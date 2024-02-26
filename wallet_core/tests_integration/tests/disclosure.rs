use assert_matches::assert_matches;
use indexmap::IndexMap;
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    server_state::SessionToken,
    verifier::{DisclosedAttributes, SessionType, StatusResponse},
    ItemsRequest,
};
use wallet::{errors::DisclosureError, mock::MockDigidSession};
use wallet_server::verifier::{ReturnUrlTemplate, StartDisclosureRequest, StartDisclosureResponse};

use crate::common::*;

pub mod common;

async fn get_verifier_status(client: &reqwest::Client, session_url: Url) -> StatusResponse {
    let response = client.get(session_url).send().await.unwrap();

    assert!(response.status().is_success());

    response.json().await.unwrap()
}

#[rstest]
#[case(SessionType::SameDevice, None)]
#[case(SessionType::SameDevice, Some("http://localhost:3004/return".parse().unwrap()))]
#[case(SessionType::CrossDevice, None)]
#[case(SessionType::CrossDevice, Some("http://localhost:3004/return".parse().unwrap()))]
#[tokio::test]
#[serial]
async fn test_disclosure_ok(#[case] session_type: SessionType, #[case] return_url: Option<ReturnUrlTemplate>) {
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
    let mut wallet = setup_wallet_and_env(
        config_server_settings(),
        wallet_provider_settings(),
        ws_settings.clone(),
        pid_issuer_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank".to_owned(),
        session_type,
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
        // The setup script is hardcoded to include "http://localhost:3004/" in the `ReaderRegistration`
        // contained in the certificate, so we have to specify a return URL prefixed with that.
        return_url_template: return_url,
    };
    let response = client
        .post(
            ws_settings
                .internal_url
                .join("sessions")
                .expect("could not join url with endpoint"),
        )
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse {
        session_url,
        engagement_url,
        mut disclosed_attributes_url,
    } = response.json::<StartDisclosureResponse>().await.unwrap();

    // after creating the session it should have status "Created"
    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::Created
    );

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let proposal = wallet
        .start_disclosure(&engagement_url)
        .await
        .expect("Could not start disclosure");
    assert_eq!(proposal.documents.len(), 1);

    // after the first wallet interaction it should have status "Waiting"
    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let return_url = wallet
        .accept_disclosure(pin)
        .await
        .expect("Could not accept disclosure");

    // after disclosure it should have status "Done"
    assert_matches!(get_verifier_status(&client, session_url).await, StatusResponse::Done);

    // passing the transcript_hash this way only works reliably it is the only query paramater (which should be the case here)
    if let Some(url) = return_url {
        disclosed_attributes_url.set_query(url.query());
    }

    let response = client.get(disclosed_attributes_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);

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
    let disclosed_attributes = response.json::<DisclosedAttributes>().await.unwrap();

    // verify the disclosed attributes
    assert_eq!(
        disclosed_attributes
            .get("com.example.pid")
            .unwrap()
            .get("com.example.pid")
            .unwrap(),
        &expected_entries
    );
}

#[tokio::test]
#[serial]
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
    let mut wallet = setup_wallet_and_env(
        config_server_settings(),
        wallet_provider_settings(),
        ws_settings.clone(),
        pid_issuer_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank".to_owned(),
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
        return_url_template: None,
    };
    let response = client
        .post(
            ws_settings
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
        disclosed_attributes_url,
    } = response.json::<StartDisclosureResponse>().await.unwrap();

    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::Created
    );

    let mut url = engagement_url.clone();
    url.set_query(Some("session_type=same_device"));

    let error = wallet
        .start_disclosure(&url)
        .await
        .expect_err("Should return error that attributes are not available");

    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    assert_matches!(
        error,
        DisclosureError::AttributesNotAvailable {
            missing_attributes: attrs,
            ..
        } if attrs
            .iter()
            .flat_map(|attr| attr.attributes.keys().map(|k| k.to_owned()).collect::<Vec<&str>>())
            .collect::<Vec<&str>>() == vec!["given_name", "family_name"]
    );

    wallet.cancel_disclosure().await.expect("Could not cancel disclosure");
    assert_matches!(
        get_verifier_status(&client, session_url.clone()).await,
        StatusResponse::Cancelled
    );

    let response = client.get(session_url).send().await.unwrap();
    let status = response.status();
    // a cancelled disclosure should have status 200
    assert_eq!(status, StatusCode::OK);

    let status = response.json::<StatusResponse>().await.unwrap();
    // and report the status as cancelled
    assert_matches!(status, StatusResponse::Cancelled);

    let response = client.get(disclosed_attributes_url).send().await.unwrap();
    // a cancelled disclosure does not result in any disclosed attributes
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_disclosure_not_found() {
    let settings = wallet_server_settings();
    start_wallet_server(settings.clone()).await;

    let client = reqwest::Client::new();
    // check if a freshly generated token returns a 404 on the status URL
    let response = client
        .get(
            settings
                .public_url
                .join(&format!("/{}/status", SessionToken::from("does_not_exist".to_owned())))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // check if a freshly generated token returns a 404 on the wallet URL
    let response = client
        .post(
            settings
                .public_url
                .join(&format!("/{}", SessionToken::from("does_not_exist".to_owned())))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // check if a freshly generated token returns a 404 on the disclosed_attributes URL
    let response = client
        .get(
            settings
                .internal_url
                .join(&format!(
                    "/{}/disclosed_attributes",
                    SessionToken::from("does_not_exist".to_owned())
                ))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
