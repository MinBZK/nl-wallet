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
    verifier::DisclosedAttributes,
    ItemsRequest,
};
use openid4vc::{
    return_url::ReturnUrlTemplate,
    verifier::{SessionType, StatusResponse},
};
use tests_integration::common::*;
use wallet::{errors::DisclosureError, mock::MockDigidSession, DisclosureUriSource};
use wallet_common::http_error::HttpJsonErrorBody;
use wallet_server::verifier::{
    DisclosedAttributesParams, StartDisclosureRequest, StartDisclosureResponse, StatusParams,
};

async fn get_verifier_status(client: &reqwest::Client, status_url: Url) -> StatusResponse {
    let response = client.get(status_url).send().await.unwrap();

    assert!(response.status().is_success());

    response.json().await.unwrap()
}

#[rstest]
#[case(
    SessionType::SameDevice,
    None,
    "xyz_bank_no_return_url",
    pid_full_name(),
    pid_full_name()
)]
#[case(
    SessionType::SameDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank",
    pid_full_name(),
    pid_full_name())
]
#[case(
    SessionType::SameDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank_all_return_url",
    pid_full_name(),
    pid_full_name()
)]
#[case(
    SessionType::CrossDevice,
    None,
    "xyz_bank_no_return_url",
    pid_full_name(),
    pid_full_name()
)]
#[case(SessionType::CrossDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank",
    pid_full_name(),
    pid_full_name()
)]
#[case(SessionType::CrossDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank_all_return_url",
    pid_full_name(),
    pid_full_name()
)]
#[case(SessionType::SameDevice,
    None,
    "xyz_bank_no_return_url",
    pid_family_name() + pid_given_name(),
    pid_full_name()
)]
#[case(
    SessionType::SameDevice,
    None,
    "multiple_cards",
    pid_given_name() + addr_street(),
    pid_given_name() + addr_street()
)]
#[case(
    SessionType::SameDevice,
    None,
    "multiple_cards",
    pid_given_name() + addr_street(), pid_given_name() + addr_street()
)]
#[tokio::test]
#[serial]
async fn test_disclosure_usecases_ok(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] usecase: String,
    #[case] test_documents: TestDocuments,
    #[case] expected_documents: TestDocuments,
) {
    let start_request = StartDisclosureRequest {
        usecase: usecase.clone(),
        items_requests: test_documents.into(),
        // The setup script is hardcoded to include "http://localhost:3004/" in the `ReaderRegistration`
        // contained in the certificate, so we have to specify a return URL prefixed with that.
        return_url_template,
    };

    // ownerschip of context is required
    let _context = setup_digid_context();

    let ws_settings = wallet_server_settings();
    let ws_internal_url = wallet_server_internal_url(&ws_settings.requester_server, &ws_settings.urls.public_url);

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
        .post(ws_internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse { session_token } = response.json::<StartDisclosureResponse>().await.unwrap();
    let mut status_url = ws_settings
        .urls
        .public_url
        .join(&format!("disclosure/sessions/{session_token}"));
    let status_query = serde_urlencoded::to_string(StatusParams { session_type }).unwrap();
    status_url.set_query(status_query.as_str().into());

    let mut disclosed_attributes_url =
        ws_internal_url.join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // after creating the session it should have status "Created"
    let StatusResponse::Created { ul } = get_verifier_status(&client, status_url.clone()).await else {
        panic!("should match StatusResponse::Created")
    };

    // Determine the correct source for the session type.
    let source = match session_type {
        SessionType::SameDevice => DisclosureUriSource::Link,
        SessionType::CrossDevice => DisclosureUriSource::QrCode,
    };

    let proposal = wallet
        .start_disclosure(&ul.unwrap().into_inner(), source)
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

    // Check if we received a return URL when we should have, based on the use case and session type.
    let should_have_return_url = match (usecase, session_type) {
        (usecase, _) if usecase == "xyz_bank_no_return_url" || usecase == "multiple_cards" => false,
        (usecase, _) if usecase == "xyz_bank_all_return_url" => true,
        (_, SessionType::SameDevice) => true,
        (_, SessionType::CrossDevice) => false,
    };
    assert_eq!(return_url.is_some(), should_have_return_url);

    if let Some(url) = return_url {
        // If we have a return URL, test that requesting the disclosed attributes
        // without the nonce contained in it will result in an error.
        let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = serde_json::from_slice::<HttpJsonErrorBody<String>>(&response.bytes().await.unwrap())
            .expect("response body should deserialize to HttpJsonErrorBody");

        assert_eq!(body.r#type, "nonce");
        assert_eq!(body.status, Some(StatusCode::UNAUTHORIZED));

        // Copy the nonce from the received return URL to the disclosed attributes request.
        let nonce = url
            .query_pairs()
            .find(|(key, _)| key == "nonce")
            .map(|(_, value)| value.into_owned())
            .expect("nonce should be present on return URL");
        let disclosed_attributes_query =
            serde_urlencoded::to_string(DisclosedAttributesParams { nonce: nonce.into() }).unwrap();
        disclosed_attributes_url.set_query(disclosed_attributes_query.as_str().into());
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
    let digid_context = MockDigidSession::start_context();
    digid_context.expect().return_once(|_, _| {
        let session = MockDigidSession::default();
        Ok((session, Url::parse("http://localhost/").unwrap()))
    });

    let ws_settings = wallet_server_settings();
    let ws_internal_url = wallet_server_internal_url(&ws_settings.requester_server, &ws_settings.urls.public_url);

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
        usecase: "xyz_bank_no_return_url".to_owned(),
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
        .post(ws_internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the RP side of things?
    let StartDisclosureResponse { session_token } = response.json::<StartDisclosureResponse>().await.unwrap();

    let mut status_url = ws_settings
        .urls
        .public_url
        .join(&format!("disclosure/sessions/{session_token}"));
    let status_query = serde_urlencoded::to_string(StatusParams {
        session_type: SessionType::SameDevice,
    })
    .unwrap();
    status_url.set_query(status_query.as_str().into());

    let disclosed_attributes_url =
        ws_internal_url.join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));

    assert_matches!(
        get_verifier_status(&client, status_url.clone()).await,
        StatusResponse::Created { .. }
    );

    let response = client.get(status_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let res = response.json::<StatusResponse>().await.unwrap();
    let StatusResponse::Created { ul } = res else {
        panic!("should match StatusResponse::Created")
    };

    let error = wallet
        .start_disclosure(&ul.unwrap().into_inner(), DisclosureUriSource::Link)
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
