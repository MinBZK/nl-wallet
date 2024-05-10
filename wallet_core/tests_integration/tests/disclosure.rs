use assert_matches::assert_matches;
use indexmap::IndexMap;
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;
use url::Url;

use nl_wallet_mdoc::{
    test::{
        data::{addr_street, pid_family_name, pid_full_name, pid_given_name},
        TestDocuments,
    },
    verifier::{DisclosedAttributes, ReturnUrlTemplate, SessionType, StatusResponse},
    ItemsRequest,
};
use openid4vc::{oidc::MockOidcClient, token::TokenRequest};
use tests_integration_common::*;
use wallet::errors::DisclosureError;
use wallet_common::utils;
use wallet_server::verifier::{StartDisclosureRequest, StartDisclosureResponse};

async fn get_verifier_status(client: &reqwest::Client, status_url: Url) -> StatusResponse {
    let response = client.get(status_url).send().await.unwrap();

    assert!(response.status().is_success());

    response.json().await.unwrap()
}

#[rstest]
#[case(SessionType::SameDevice, None, "xyz_bank", pid_full_name(), pid_full_name())]
#[case(SessionType::SameDevice, Some("http://localhost:3004/return".parse().unwrap()), "xyz_bank", pid_full_name(), pid_full_name())]
#[case(SessionType::CrossDevice, None, "xyz_bank", pid_full_name(), pid_full_name())]
#[case(SessionType::CrossDevice, Some("http://localhost:3004/return".parse().unwrap()), "xyz_bank", pid_full_name(), pid_full_name())]
#[case(SessionType::SameDevice, None, "xyz_bank", pid_family_name() + pid_given_name(), pid_full_name())]
#[case(
    SessionType::SameDevice,
    None,
    "multiple_cards",
    pid_given_name() + addr_street(),
    pid_given_name() + addr_street()
)]
#[case(SessionType::SameDevice, None, "multiple_cards", pid_given_name() + addr_street(), pid_given_name() + addr_street())]
#[tokio::test]
#[serial]
async fn test_disclosure_usecases_ok(
    #[case] session_type: SessionType,
    #[case] return_url: Option<ReturnUrlTemplate>,
    #[case] usecase: String,
    #[case] test_documents: TestDocuments,
    #[case] expected_documents: TestDocuments,
) {
    let start_request = StartDisclosureRequest {
        usecase,
        session_type,
        items_requests: test_documents.into(),
        // The setup script is hardcoded to include "http://localhost:3004/" in the `ReaderRegistration`
        // contained in the certificate, so we have to specify a return URL prefixed with that.
        return_url_template: return_url,
    };

    let digid_context = MockOidcClient::start_context();
    digid_context.expect().return_once(|_, _, _, _| {
        let mut session = MockOidcClient::default();

        session.expect_into_token_request().return_once(|_url| {
            Ok(TokenRequest {
                grant_type: openid4vc::token::TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: utils::random_string(32).into(),
                },
                code_verifier: Some("my_code_verifier".to_string()),
                client_id: Some("my_client_id".to_string()),
                redirect_uri: Some("redirect://here".parse().unwrap()),
            })
        });

        Ok((session, Url::parse("http://localhost/").unwrap()))
    });

    let ws_settings = wallet_server_settings();

    let pin = "112233".to_string();
    let mut wallet = setup_wallet_and_env(
        config_server_settings(),
        wallet_provider_settings(),
        ws_settings.clone(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    let client = reqwest::Client::new();

    let response = client
        .post(ws_settings.internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse {
        status_url,
        mut disclosed_attributes_url,
    } = response.json::<StartDisclosureResponse>().await.unwrap();

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // after creating the session it should have status "Created"
    let engagement_url = match get_verifier_status(&client, status_url.clone()).await {
        StatusResponse::Created { engagement_url } => engagement_url,
        _ => panic!("should match StatusResponse::Created"),
    };

    let proposal = wallet
        .start_disclosure(&engagement_url)
        .await
        .expect("should start disclosure");
    assert_eq!(proposal.documents.len(), expected_documents.len());

    // after the first wallet interaction it should have status "Waiting"
    assert_matches!(
        get_verifier_status(&client, status_url.clone()).await,
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
    assert_matches!(get_verifier_status(&client, status_url).await, StatusResponse::Done);

    // passing the transcript_hash this way only works reliably if it is the only query paramater (which should be the case here)
    if let Some(url) = return_url {
        disclosed_attributes_url.set_query(url.query());
    }

    let response = client.get(disclosed_attributes_url).send().await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);

    let disclosed_documents = response.json::<DisclosedAttributes>().await.unwrap();

    expected_documents.assert_matches(&disclosed_documents);
}

#[tokio::test]
#[serial]
async fn test_disclosure_without_pid() {
    let digid_context = MockOidcClient::start_context();
    digid_context.expect().return_once(|_, _, _, _| {
        let session = MockOidcClient::default();
        Ok((session, Url::parse("http://localhost/").unwrap()))
    });

    let ws_settings = wallet_server_settings();

    let pin = "112233".to_string();
    let mut wallet = setup_wallet_and_env(
        config_server_settings(),
        wallet_provider_settings(),
        ws_settings.clone(),
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
                        .into_iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), intent_to_retain)),
                ),
            )]),
        }]
        .into(),
        return_url_template: None,
    };
    let response = client
        .post(ws_settings.internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the RP side of things?
    let StartDisclosureResponse {
        status_url,
        disclosed_attributes_url,
    } = response.json::<StartDisclosureResponse>().await.unwrap();

    assert_matches!(
        get_verifier_status(&client, status_url.clone()).await,
        StatusResponse::Created { .. }
    );

    let response = client.get(status_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let res = response.json::<StatusResponse>().await.unwrap();
    let engagement_url = match res {
        StatusResponse::Created { engagement_url } => engagement_url,
        _ => panic!("should match StatusResponse::Created"),
    };

    let mut url = engagement_url.clone();
    url.set_query(Some("session_type=same_device"));

    let error = wallet
        .start_disclosure(&url)
        .await
        .expect_err("Should return error that attributes are not available");

    assert_matches!(
        get_verifier_status(&client, status_url.clone()).await,
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
        get_verifier_status(&client, status_url.clone()).await,
        StatusResponse::Cancelled
    );

    let response = client.get(status_url).send().await.unwrap();
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
