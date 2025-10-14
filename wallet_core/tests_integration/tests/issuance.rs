use indexmap::IndexMap;
use itertools::Itertools;
use rstest::rstest;
use serial_test::serial;

use attestation_data::constants::*;
use attestation_data::disclosure::DisclosedAttestations;
use attestation_data::disclosure::DisclosedAttributes;
use attestation_data::issuable_document::IssuableDocument;
use attestation_types::claim_path::ClaimPath;
use dcql::CredentialFormat;
use dcql::normalized::MdocAttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::normalized::SdJwtAttributeRequest;
use openid4vc::ErrorResponse;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::verifier::StatusResponse;
use openid4vc_server::verifier::DisclosedAttributesParams;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use pid_issuer::pid::mock::mock_issuable_document_address;
use utils::vec_nonempty;
use wallet::AttestationAttributeValue;
use wallet::DisclosureAttestationOptions;
use wallet::DisclosureUriSource;
use wallet::attestation_data::Attribute;
use wallet::attestation_data::AttributeValue;
use wallet::errors::IssuanceError;
use wallet::openid4vc::SessionType;

use tests_integration::common::*;

#[tokio::test]
#[serial(hsm)]
async fn test_pid_ok() {
    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Verify that the first mdoc contains the bsn
    let attestations = wallet_attestations(&mut wallet).await;
    let pid_attestation = attestations
        .iter()
        .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
        .expect("should have received PID attestation");

    let bsn_attr = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key == vec![PID_BSN])
        .unwrap();

    assert_eq!(
        bsn_attr.value,
        AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
    );

    // Recovery code is hidden from presentation
    let recovery_code_result = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key == vec![PID_RECOVERY_CODE]);

    assert_eq!(recovery_code_result, None);
}

fn pid_without_optionals() -> IssuableDocument {
    IssuableDocument::try_new(
        PID_ATTESTATION_TYPE.to_string(),
        IndexMap::from_iter(vec![
            (
                PID_FAMILY_NAME.to_string(),
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
            ),
            (
                PID_GIVEN_NAME.to_string(),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
            ),
            (
                PID_BIRTH_DATE.to_string(),
                Attribute::Single(AttributeValue::Text("1997-05-10".to_string())),
            ),
            // only age_over_18 is optional in `eudi::pid::nl:1.json`
            (
                PID_BSN.to_string(),
                Attribute::Single(AttributeValue::Text("999991772".to_string())),
            ),
            (
                PID_RECOVERY_CODE.to_string(),
                Attribute::Single(AttributeValue::Text("123".to_string())),
            ),
        ])
        .into(),
    )
    .unwrap()
}

fn pid_missing_required() -> IssuableDocument {
    IssuableDocument::try_new(
        PID_ATTESTATION_TYPE.to_string(),
        IndexMap::from_iter(vec![
            (
                PID_FAMILY_NAME.to_string(),
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
            ),
            (
                PID_GIVEN_NAME.to_string(),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
            ),
            (
                PID_BIRTH_DATE.to_string(),
                Attribute::Single(AttributeValue::Text("1997-05-10".to_string())),
            ),
            // bsn is missing, which is required
        ])
        .into(),
    )
    .unwrap()
}

#[tokio::test]
#[serial(hsm)]
async fn test_pid_optional_attributes() {
    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(),
        (
            pid_issuer_settings().0,
            vec![pid_without_optionals(), mock_issuable_document_address()]
                .try_into()
                .unwrap(),
        ),
        issuance_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Verify that the first mdoc contains the bsn
    let attestations = wallet_attestations(&mut wallet).await;
    let pid_attestation = attestations
        .iter()
        .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
        .expect("should have received PID attestation");

    let bsn_attr = pid_attestation.attributes.iter().find(|a| a.key == vec![PID_BSN]);

    match bsn_attr {
        Some(bsn_attr) => assert_eq!(
            bsn_attr.value,
            AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
        ),
        None => panic!("BSN attribute not found"),
    }
    // Verify that we didn't get `age_over_18` issued
    let age_over_18 = pid_attestation.attributes.iter().find(|a| a.key == vec!["age_over_18"]);
    assert!(age_over_18.is_none());
}

#[tokio::test]
#[serial(hsm)]
async fn test_pid_missing_required_attributes() {
    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(),
        (
            pid_issuer_settings().0,
            vec![pid_missing_required(), mock_issuable_document_address()]
                .try_into()
                .unwrap(),
        ),
        issuance_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    let redirect_url = wallet
        .create_pid_issuance_auth_url()
        .await
        .expect("should create PID issuance redirect URL");
    let _unsigned_mdocs = wallet
        .continue_pid_issuance(redirect_url)
        .await
        .expect("should continue PID issuance");
    let error = wallet
        .accept_issuance(pin.to_owned())
        .await
        .expect_err("should fail to accept issuance");

    assert!(matches!(
        error,
        IssuanceError::IssuerServer {
            error: IssuanceSessionError::CredentialRequest(ErrorResponse { error_description: Some(description), .. }),
            ..
            } if description.contains("\"urn:eudi:pid:nl:1\": \"bsn\" is a required property")
    ));
}

#[rstest]
#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_and_disclosure(
    #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] pid_format: CredentialFormat,
    #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] degree_format: CredentialFormat,
) {
    // Start with a wallet that contains the PID.
    let pin = "112233";
    let (mut wallet, urls, issuance_url) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;

    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Perform issuance of university degrees based on this PID.
    let _proposal = wallet
        .start_disclosure(&universal_link(&issuance_url, pid_format), DisclosureUriSource::Link)
        .await
        .unwrap();

    let attestation_previews = wallet
        .continue_disclosure_based_issuance(&[0], pin.to_owned())
        .await
        .unwrap();

    wallet.accept_issuance(pin.to_owned()).await.unwrap();

    let attestations = wallet_attestations(&mut wallet).await;

    // Check that every preview attestation is present in the wallet database after issuance.
    for preview in &attestation_previews {
        assert!(
            attestations
                .iter()
                .any(|attestation| attestation.attestation_type == preview.attestation_type
                    && attestation.attributes == preview.attributes)
        );
    }

    // Prepare a disclosure request and send this to the verifier.
    let credential_request = match degree_format {
        CredentialFormat::MsoMdoc => NormalizedCredentialRequest::MsoMdoc {
            id: "degree".parse().unwrap(),
            doctype_value: "com.example.degree".to_string(),
            claims: vec_nonempty![MdocAttributeRequest {
                path: vec_nonempty![
                    ClaimPath::SelectByKey("com.example.degree".to_string()),
                    ClaimPath::SelectByKey("education".to_string())
                ],
                intent_to_retain: Some(true),
            }],
        },
        CredentialFormat::SdJwt => NormalizedCredentialRequest::SdJwt {
            id: "degree".parse().unwrap(),
            vct_values: vec_nonempty!["com.example.degree".to_string()],
            claims: vec_nonempty![SdJwtAttributeRequest {
                path: vec_nonempty![ClaimPath::SelectByKey("education".to_string())],
            }],
        },
    };

    let start_request = StartDisclosureRequest {
        usecase: "job_finder".to_string(),
        dcql_query: Some(
            NormalizedCredentialRequests::try_from(vec![credential_request])
                .unwrap()
                .into(),
        ),
        return_url_template: Some("http://localhost:3004/return".parse().unwrap()),
    };

    let client = reqwest::Client::new();

    let StartDisclosureResponse { session_token } = client
        .post(urls.verifier_internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();

    // The the universal link the wallet can use for disclosure.
    let mut status_url = urls.verifier_url.join(&format!("disclosure/sessions/{session_token}"));
    let status_query = serde_urlencoded::to_string(StatusParams {
        session_type: Some(SessionType::SameDevice),
    })
    .unwrap();
    status_url.set_query(Some(status_query.as_str()));

    let StatusResponse::Created { ul } = client
        .get(status_url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
    else {
        panic!("should match StatusResponse::Created")
    };

    // Start the disclosure of the "education" field of a university degree.
    let proposal = wallet
        .start_disclosure(&ul.unwrap().into_inner(), DisclosureUriSource::Link)
        .await
        .expect("should start disclosure");

    assert_eq!(proposal.attestation_options.len().get(), 1);

    // Find the MSc degree within the two proposed attestations and disclose it.
    let msc_index = match proposal.attestation_options.first() {
        DisclosureAttestationOptions::Multiple(presentations) => {
            assert_eq!(presentations.len().get(), 2);

            presentations
                .iter()
                .position(|presentation| {
                    presentation
                        .attributes
                        .iter()
                        .find(|attribute| attribute.key.iter().eq(["education"]))
                        .is_some_and(|attribute| attribute.value.to_string() == "MSc")
                })
                .unwrap()
        }
        DisclosureAttestationOptions::Single(_) => panic!("expected multiple disclosure option for degree"),
    };

    let return_url = wallet
        .accept_disclosure(&[msc_index], pin.to_string())
        .await
        .expect("Could not accept disclosure");

    // Retrieve the disclosed attributes, for which we need the nonce returned to the wallet.
    let nonce = return_url
        .unwrap()
        .query_pairs()
        .find_map(|(key, value)| (key == "nonce").then(|| value.into_owned()))
        .unwrap();

    let mut disclosed_attributes_url = urls
        .verifier_internal_url
        .join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));
    let query = serde_urlencoded::to_string(DisclosedAttributesParams { nonce: Some(nonce) }).unwrap();
    disclosed_attributes_url.set_query(Some(query.as_str()));

    let disclosed_attestations = client
        .get(disclosed_attributes_url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<Vec<DisclosedAttestations>>()
        .await
        .unwrap();

    let attributes = &disclosed_attestations
        .iter()
        .exactly_one()
        .unwrap()
        .attestations
        .iter()
        .exactly_one()
        .unwrap()
        .attributes;

    // Check that the only attribute disclosed is the education.
    let education = match (degree_format, attributes) {
        (CredentialFormat::MsoMdoc, DisclosedAttributes::MsoMdoc(attributes)) => attributes
            .iter()
            .exactly_one()
            .ok()
            .and_then(|(name_space, attributes)| (name_space == "com.example.degree").then_some(attributes))
            .and_then(|attributes| attributes.into_iter().exactly_one().ok())
            .and_then(|(key, value)| (key == "education").then_some(value))
            .unwrap(),
        (CredentialFormat::SdJwt, DisclosedAttributes::SdJwt(attributes)) => attributes
            .flattened()
            .into_iter()
            .exactly_one()
            .ok()
            .and_then(|(path, value)| path.iter().eq(&["education"]).then_some(value))
            .unwrap(),
        _ => panic!("unexpected disclosed attributes format"),
    };

    assert_eq!(education.to_string(), "MSc");
}

#[rstest]
#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_error_no_attributes(
    #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] format: CredentialFormat,
) {
    let (issuance_server_settings, _, di_trust_anchor, di_tls_config) = issuance_server_settings();

    let pin = "112233";
    let (mut wallet, _, issuance_url) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(),
        pid_issuer_settings(),
        (issuance_server_settings, vec![], di_trust_anchor, di_tls_config),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let _proposal = wallet
        .start_disclosure(&universal_link(&issuance_url, format), DisclosureUriSource::Link)
        .await
        .unwrap();

    // If the issuer has no attestations to issue, we receive an empty vec and no error.
    let attestations = wallet
        .continue_disclosure_based_issuance(&[0], pin.to_owned())
        .await
        .unwrap();
    assert!(attestations.is_empty());
}
