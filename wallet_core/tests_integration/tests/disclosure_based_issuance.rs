use std::assert_matches;

use attestation_data::disclosure::DisclosedAttestations;
use attestation_data::disclosure::DisclosedAttributes;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use db_test::DbSetup;
use dcql::normalized::NormalizedCredentialRequest;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::normalized::SdJwtAttributeRequest;
use hsm::test::HsmSetup;
use itertools::Itertools;
use openid4vc::verifier::StatusResponse;
use openid4vc_server::verifier::DisclosedAttributesParams;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use rstest::rstest;
use serial_test::serial;
use tests_integration::common::*;
use utils::vec_nonempty;
use wallet::AttestationIdentity;
use wallet::AttributeValue;
use wallet::DisclosureAttestationOptions;
use wallet::DisclosureBasedIssuanceResult;
use wallet::DisclosureUriSource;
use wallet::Pin;
use wallet::openid4vc::SessionType;

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc5_test_disclosure_based_issuance_and_disclosure(
    #[values(Format::MsoMdoc, Format::SdJwt)] pid_format: Format,
) {
    let db_setup = DbSetup::create_clean().await;
    let hsm_setup = HsmSetup::new();
    let tempdir = tempfile::tempdir().unwrap();
    let pin: Pin = "112233".into();

    // Start with a wallet that contains the PID.
    let (mut wallet, disclosure_urls, issuance_data) =
        setup_wallet_and_default_env(&db_setup, &hsm_setup, &tempdir, WalletDeviceVendor::Apple).await;

    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    // Perform issuance of university degrees based on this PID.
    let attestation_previews = do_degree_issuance(
        &mut wallet,
        pin.clone(),
        &issuance_data.issuance_server.public,
        &issuance_data.degree_client_ids,
        pid_format,
    )
    .await;

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
    let credential_request = NormalizedCredentialRequest::SdJwt {
        id: "degree".parse().unwrap(),
        vct_values: vec_nonempty!["com.example.degree".to_string()],
        claims: vec_nonempty![SdJwtAttributeRequest {
            path: vec_nonempty![ClaimPath::SelectByKey("education".to_string())],
        }],
        aki: vec![],
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
        .post(disclosure_urls.verifier_internal_url.join("disclosure/sessions"))
        .json(&start_request)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();

    // The universal link the wallet can use for disclosure.
    let mut status_url = disclosure_urls
        .verifier_url
        .join(&format!("disclosure/sessions/{session_token}"));
    let status_query = serde_qs::to_string(&StatusParams {
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
                        .is_some_and(
                            |attribute| matches!(&attribute.value, AttributeValue::Text(string) if string == "MSc"),
                        )
                })
                .unwrap()
        }
        DisclosureAttestationOptions::Single(_) => panic!("expected multiple disclosure option for degree"),
    };

    let return_url = wallet
        .accept_disclosure(&[msc_index], pin)
        .await
        .expect("Could not accept disclosure");

    // Retrieve the disclosed attributes, for which we need the nonce returned to the wallet.
    let nonce = return_url
        .unwrap()
        .query_pairs()
        .find_map(|(key, value)| (key == "nonce").then(|| value.into_owned()))
        .unwrap();

    let mut disclosed_attributes_url = disclosure_urls
        .verifier_internal_url
        .join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));
    let query = serde_qs::to_string(&DisclosedAttributesParams { nonce: Some(nonce) }).unwrap();
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
    let education = match attributes {
        DisclosedAttributes::SdJwt(attributes) => attributes
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
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc10_test_disclosure_based_issuance_error_no_attributes(
    #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
) {
    let db_setup = DbSetup::create_clean().await;
    let hsm_setup = HsmSetup::new();
    let tempdir = tempfile::tempdir().unwrap();
    let pin: Pin = "112233".into();

    let (issuance_server_settings, _, di_trust_anchor, di_tls_config) =
        issuance_server_settings(db_setup.issuance_server_url());
    let (mut wallet, _, issuance_urls) = setup_wallet_and_env(
        &db_setup,
        &hsm_setup,
        &tempdir,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings(db_setup.pid_issuer_url(), None),
        (issuance_server_settings, vec![], di_trust_anchor, di_tls_config),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    let _proposal = wallet
        .start_disclosure(
            &universal_link(
                issuance_urls.issuance_server.public.as_base_url(),
                &issuance_urls.degree_client_ids,
                format,
            ),
            DisclosureUriSource::Link,
        )
        .await
        .unwrap();

    let result = wallet.continue_disclosure_based_issuance(&[0], pin).await.unwrap();
    assert_matches!(result, DisclosureBasedIssuanceResult::NoIssuableAttestations);
}

/// Disclosure based issuance against an issuance server configured without the Credential Preview Endpoint.
///
/// As the wallet receives no previews, the degrees are issued right after the disclosure, using the same PIN. They are
/// presented for review, and stored once committed.
#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_disclosure_based_issuance_without_previews(#[values(Format::MsoMdoc, Format::SdJwt)] format: Format) {
    let db_setup = DbSetup::create_clean().await;
    let hsm_setup = HsmSetup::new();
    let tempdir = tempfile::tempdir().unwrap();
    let pin: Pin = "112233".into();

    let (mut issuance_server_settings, issuable_documents, di_trust_anchor, di_tls_config) =
        issuance_server_settings(db_setup.issuance_server_url());
    issuance_server_settings.issuer_settings.credential_preview_endpoint = false;

    let (mut wallet, _, issuance_urls) = setup_wallet_and_env(
        &db_setup,
        &hsm_setup,
        &tempdir,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings(db_setup.pid_issuer_url(), None),
        (
            issuance_server_settings,
            issuable_documents,
            di_trust_anchor,
            di_tls_config,
        ),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    let _proposal = wallet
        .start_disclosure(
            &universal_link(
                issuance_urls.issuance_server.public.as_base_url(),
                &issuance_urls.degree_client_ids,
                format,
            ),
            DisclosureUriSource::Link,
        )
        .await
        .unwrap();

    let DisclosureBasedIssuanceResult::IssuedForReview(reviewed_attestations) =
        wallet.continue_disclosure_based_issuance(&[0], pin).await.unwrap()
    else {
        panic!("expected attestations issued for review");
    };

    // Only the two degrees should be issued, by the same issuer.
    assert_eq!(reviewed_attestations.len().get(), 2);
    assert!(
        reviewed_attestations
            .iter()
            .all(|attestation| attestation.attestation_type == "com.example.degree")
    );
    let issuer = reviewed_attestations.first().issuer.clone();
    assert!(reviewed_attestations.iter().all(|degree| degree.issuer == issuer));

    wallet.commit_issued_credentials().await.unwrap();

    let attestations = wallet_attestations(&mut wallet).await;
    let degrees = attestations
        .iter()
        .filter(|attestation| attestation.attestation_type == "com.example.degree")
        .collect_vec();
    assert_eq!(degrees.len(), 2);
    assert!(degrees.iter().all(|degree| degree.issuer == issuer));
}

/// Disclosure based issuance of the same degrees twice, against an issuance server with or without the Credential
/// Preview Endpoint. As the degrees are issued with the same contents, the second issuance renews the stored degrees.
#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_disclosure_based_issuance_renewal(#[values(true, false)] credential_preview_endpoint: bool) {
    let db_setup = DbSetup::create_clean().await;
    let hsm_setup = HsmSetup::new();
    let tempdir = tempfile::tempdir().unwrap();
    let pin: Pin = "112233".into();

    let (mut issuance_server_settings, issuable_documents, di_trust_anchor, di_tls_config) =
        issuance_server_settings(db_setup.issuance_server_url());
    issuance_server_settings.issuer_settings.credential_preview_endpoint = credential_preview_endpoint;

    let (mut wallet, _, issuance_urls) = setup_wallet_and_env(
        &db_setup,
        &hsm_setup,
        &tempdir,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings(db_setup.pid_issuer_url(), None),
        (
            issuance_server_settings,
            issuable_documents,
            di_trust_anchor,
            di_tls_config,
        ),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

    issue_degrees(&mut wallet, &pin, &issuance_urls, credential_preview_endpoint).await;

    let identities = degree_identities(&mut wallet).await;
    assert_eq!(identities.len(), 2);

    // Issuing the same degrees again should update the stored degrees, instead of adding new ones.
    issue_degrees(&mut wallet, &pin, &issuance_urls, credential_preview_endpoint).await;

    let renewed_identities = degree_identities(&mut wallet).await;
    assert_eq!(renewed_identities.len(), identities.len());
    assert!(renewed_identities.iter().all(|identity| identities.contains(identity)));
}

/// Perform disclosure based issuance of the degrees, which should be offered with previews if `expect_previews` is set.
async fn issue_degrees(wallet: &mut WalletWithStorage, pin: &Pin, issuance_urls: &IssuerData, expect_previews: bool) {
    let _proposal = wallet
        .start_disclosure(
            &universal_link(
                issuance_urls.issuance_server.public.as_base_url(),
                &issuance_urls.degree_client_ids,
                Format::SdJwt,
            ),
            DisclosureUriSource::Link,
        )
        .await
        .unwrap();

    match wallet
        .continue_disclosure_based_issuance(&[0], pin.clone())
        .await
        .unwrap()
    {
        DisclosureBasedIssuanceResult::Previews(_) => {
            assert!(expect_previews, "expected no previews");

            wallet.accept_issuance(pin.clone()).await.unwrap();
        }
        DisclosureBasedIssuanceResult::IssuedForReview(_) => {
            assert!(!expect_previews, "expected previews");

            wallet.commit_issued_credentials().await.unwrap();
        }
        DisclosureBasedIssuanceResult::NoIssuableAttestations => panic!("expected issuable degrees"),
    }
}

/// The identities of the degrees stored in the wallet.
async fn degree_identities(wallet: &mut WalletWithStorage) -> Vec<AttestationIdentity> {
    wallet_attestations(wallet)
        .await
        .into_iter()
        .filter(|attestation| attestation.attestation_type == "com.example.degree")
        .map(|attestation| attestation.identity)
        .collect()
}
