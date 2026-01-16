use std::collections::HashSet;

use assert_matches::assert_matches;
use itertools::Itertools;
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;
use url::Url;

use attestation_data::disclosure::DisclosedAttestations;
use attestation_data::test_credential::TestCredentials;
use dcql::CredentialFormat;
use dcql::CredentialQueryIdentifier;
use dcql::Query;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::unique_id_vec::UniqueIdVec;
use http_utils::error::HttpJsonErrorBody;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::StatusResponse;
use openid4vc_server::verifier::DisclosedAttributesParams;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use pid_issuer::pid::constants::EUDI_PID_ATTESTATION_TYPE;
use pid_issuer::pid::constants::PID_GIVEN_NAME;
use tests_integration::common::*;
use tests_integration::test_credential::new_mock_mdoc_pid_example;
use tests_integration::test_credential::nl_pid_credentials_family_name;
use tests_integration::test_credential::nl_pid_credentials_full_name;
use tests_integration::test_credential::nl_pid_credentials_given_name;
use tests_integration::test_credential::nl_pid_credentials_given_name_for_query_id;
use tests_integration::test_credential::nl_pid_full_name_and_minimal_address;
use wallet::DisclosureUriSource;
use wallet::errors::DisclosureError;

async fn get_verifier_status(client: &reqwest::Client, status_url: Url) -> StatusResponse {
    let response = client.get(status_url).send().await.unwrap();

    assert!(response.status().is_success());

    response.json().await.unwrap()
}

async fn assert_disclosure_ok(
    session_type: SessionType,
    usecase: String,
    return_url_template: Option<ReturnUrlTemplate>,
    format: CredentialFormat,
    dcql_query: Query,
    test_credentials: TestCredentials,
) {
    let start_request = StartDisclosureRequest {
        usecase: usecase.clone(),
        dcql_query: Some(dcql_query),
        // The setup script is hardcoded to include "http://localhost:3004/" in the `ReaderRegistration`
        // contained in the certificate, so we have to specify a return URL prefixed with that.
        return_url_template,
    };

    let pin = "112233";
    let (mut wallet, urls, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
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
    assert_eq!(
        proposal.attestation_options.len().get(),
        test_credentials.as_ref().len()
    );

    // after the first wallet interaction it should have the status "Waiting"
    assert_matches!(
        get_verifier_status(&client, status_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    // disclosed attributes endpoint should return a response with code Bad Request when the status is not DONE
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let attestation_count = test_credentials.as_ref().len();
    let return_url = wallet
        .accept_disclosure(&vec![0; attestation_count], pin.to_owned())
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

    let disclosed_attestations = response.json::<UniqueIdVec<DisclosedAttestations>>().await.unwrap();

    test_credentials.assert_matches_disclosed_attestations(
        &disclosed_attestations,
        std::iter::repeat_n(format, test_credentials.as_ref().len()),
    );
}

#[rstest]
#[case(
    SessionType::SameDevice,
    None,
    "xyz_bank_no_return_url",
    nl_pid_credentials_full_name()
)]
#[case(
    SessionType::SameDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    // Note that this use case is exactly the same as "xyz_bank_mdoc" and only differs for the demo RP.
    "xyz_bank_sd_jwt",
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::SameDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank_all_return_url",
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::CrossDevice,
    None,
    "xyz_bank_no_return_url",
    nl_pid_credentials_full_name()
)]
#[case(
    SessionType::CrossDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    // Note that this use case is exactly the same as "xyz_bank_mdoc" and only differs for the demo RP.
    "xyz_bank_sd_jwt",
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::CrossDevice,
    Some("http://localhost:3004/return".parse().unwrap()),
    "xyz_bank_all_return_url",
    nl_pid_credentials_full_name(),
)]
#[case(
    SessionType::SameDevice,
    None,
    "xyz_bank_no_return_url",
    nl_pid_credentials_given_name() + nl_pid_credentials_family_name(),
)]
#[case(
    SessionType::SameDevice,
    None,
    "xyz_bank_no_return_url",
    nl_pid_full_name_and_minimal_address()
)]
#[tokio::test]
#[serial(hsm)]
async fn ltc15_ltc16_test_disclosure_usecases_ok(
    #[case] session_type: SessionType,
    #[case] return_url_template: Option<ReturnUrlTemplate>,
    #[case] usecase: String,
    #[case] test_credentials: TestCredentials,
    #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] format: CredentialFormat,
) {
    let dcql_query = test_credentials.to_dcql_query(std::iter::repeat_n(format, test_credentials.as_ref().len()));

    assert_disclosure_ok(
        session_type,
        usecase,
        return_url_template,
        format,
        dcql_query,
        test_credentials,
    )
    .await;
}

#[tokio::test]
#[serial(hsm)]
async fn ltc15_test_disclosure_extended_vct_ok() {
    let session_type = SessionType::SameDevice;
    let return_url_template = None;
    let usecase = "xyz_bank_no_return_url".to_owned();
    let format = CredentialFormat::SdJwt;

    let query_id = "eudi_pid_given_name";
    let test_credentials = nl_pid_credentials_given_name_for_query_id(query_id);
    let mut dcql_query: Query = NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(
        &[EUDI_PID_ATTESTATION_TYPE],
        &[&[PID_GIVEN_NAME]],
    )])
    .into();
    dcql_query.credentials = dcql_query
        .credentials
        .into_iter()
        .map(|mut query| {
            query.id = CredentialQueryIdentifier::try_new(String::from(query_id)).unwrap();
            query
        })
        .collect_vec()
        .try_into()
        .unwrap();

    assert_disclosure_ok(
        session_type,
        usecase,
        return_url_template,
        format,
        dcql_query,
        test_credentials,
    )
    .await;
}

#[tokio::test]
#[serial(hsm)]
async fn ltc20_test_disclosure_without_pid() {
    let pin = "112233";
    let (mut wallet, urls, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank_no_return_url".to_owned(),
        dcql_query: Some(new_mock_mdoc_pid_example()),
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
            "urn:eudi:pid:nl:1/urn:eudi:pid:nl:1/bsn".to_string(),
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
