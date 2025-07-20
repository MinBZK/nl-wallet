use std::collections::HashSet;

use assert_matches::assert_matches;
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;
use url::Url;

use attestation_data::disclosure::DisclosedAttestations;
use dcql::CredentialQueryFormat;
use dcql::normalized::AttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use http_utils::error::HttpJsonErrorBody;
use http_utils::tls::pinning::TlsPinningConfig;
use mdoc::test::TestDocuments;
use mdoc::test::data::addr_street;
use mdoc::test::data::pid_family_name;
use mdoc::test::data::pid_full_name;
use mdoc::test::data::pid_given_name;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::StatusResponse;
use openid4vc_server::verifier::DisclosedAttributesParams;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use tests_integration::common::*;
use wallet::DisclosureUriSource;
use wallet::errors::DisclosureError;
use wallet::mock::MockDigidSession;

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
#[case(SessionType::SameDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank",
    pid_full_name(),
    pid_full_name()
)]
#[case(SessionType::SameDevice,
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
#[case(SessionType::SameDevice,
    None,
    "multiple_cards",
    pid_given_name() + addr_street(),
    pid_given_name() + addr_street()
)]
#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_usecases_ok(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] usecase: String,
    #[case] test_documents: TestDocuments,
    #[case] expected_documents: TestDocuments,
) {
    let start_request = StartDisclosureRequest {
        usecase: usecase.clone(),
        credential_requests: Some(test_documents.into()),
        // The setup script is hardcoded to include "http://localhost:3004/" in the `ReaderRegistration`
        // contained in the certificate, so we have to specify a return URL prefixed with that.
        return_url_template,
    };

    let _retain = setup_digid_context();

    let pin = "112233";
    let (mut wallet, urls, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let client = reqwest::Client::new();

    let response = client
        .post(urls.verifier_internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse { session_token } = response.json::<StartDisclosureResponse>().await.unwrap();
    let mut status_url = urls.verifier_url.join(&format!("disclosure/sessions/{session_token}"));
    let status_query = serde_urlencoded::to_string(StatusParams {
        session_type: Some(session_type),
    })
    .unwrap();
    status_url.set_query(status_query.as_str().into());

    let mut disclosed_attributes_url = urls
        .verifier_internal_url
        .join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // after creating the session, it should have the status "Created"
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
    assert_eq!(proposal.attestations.len(), expected_documents.len());

    // after the first wallet interaction it should have the status "Waiting"
    assert_matches!(
        get_verifier_status(&client, status_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let return_url = wallet
        .accept_disclosure(pin.to_owned())
        .await
        .expect("Could not accept disclosure");

    // after disclosure, it should have the status "Done"
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

    let disclosed_documents = response.json::<DisclosedAttestations>().await.unwrap();

    expected_documents.assert_matches(
        &disclosed_documents
            .into_iter()
            .map(|(credential_type, attributes)| (credential_type, attributes.into()))
            .collect(),
    );
}

#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_without_pid() {
    let digid_context = MockDigidSession::start_context();
    digid_context.expect().return_once(|_, _: TlsPinningConfig, _| {
        let session = MockDigidSession::default();
        Ok((session, Url::parse("http://localhost/").unwrap()))
    });

    let pin = "112233";
    let (mut wallet, urls, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank_no_return_url".to_owned(),
        credential_requests: Some(
            vec![NormalizedCredentialRequest {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: "urn:eudi:pid:nl:1".to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(
                        vec!["urn:eudi:pid:nl:1".to_string(), "given_name".to_string()],
                        true,
                    ),
                    AttributeRequest::new_with_keys(
                        vec!["urn:eudi:pid:nl:1".to_string(), "family_name".to_string()],
                        false,
                    ),
                ],
            }]
            .try_into()
            .unwrap(),
        ),
        return_url_template: None,
    };
    let response = client
        .post(urls.verifier_internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the RP side of things?
    let StartDisclosureResponse { session_token } = response.json::<StartDisclosureResponse>().await.unwrap();

    let mut status_url = urls.verifier_url.join(&format!("disclosure/sessions/{session_token}"));
    let status_query = serde_urlencoded::to_string(StatusParams {
        session_type: Some(SessionType::SameDevice),
    })
    .unwrap();
    status_url.set_query(status_query.as_str().into());

    let disclosed_attributes_url = urls
        .verifier_internal_url
        .join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));

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
            requested_attributes,
            ..
        } if requested_attributes == HashSet::from([
            "urn:eudi:pid:nl:1/urn:eudi:pid:nl:1/given_name".to_string(),
            "urn:eudi:pid:nl:1/urn:eudi:pid:nl:1/family_name".to_string(),
        ])
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
