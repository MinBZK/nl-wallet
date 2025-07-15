use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serial_test::serial;
use url::Url;

use attestation_data::issuable_document::IssuableDocument;
use http_utils::urls::disclosure_based_issuance_base_uri;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::openid4vp::RequestUriMethod;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::verifier::VerifierUrlParameters;
use openid4vc::ErrorResponse;
use pid_issuer::pid::constants::*;
use pid_issuer::pid::mock::mock_issuable_document_address;
use tests_integration::common::*;
use wallet::attestation_data::Attribute;
use wallet::attestation_data::AttributeValue;
use wallet::errors::IssuanceError;
use wallet::mock::BSN_ATTR_NAME;
use wallet::mock::PID_DOCTYPE;
use wallet::openid4vc::SessionType;
use wallet::utils::BaseUrl;
use wallet::AttestationAttributeValue;
use wallet::AttestationPresentation;
use wallet::DisclosureUriSource;

pub async fn wallet_attestations(wallet: &mut WalletWithMocks) -> Vec<AttestationPresentation> {
    // Emit attestations into this local variable
    let attestations: Arc<std::sync::Mutex<Vec<AttestationPresentation>>> = Arc::new(std::sync::Mutex::new(vec![]));

    {
        let attestations = Arc::clone(&attestations);
        wallet
            .set_attestations_callback(Box::new(move |mut a| {
                let mut attestations = attestations.lock().unwrap();
                attestations.append(&mut a);
            }))
            .await
            .unwrap();
    }

    let attestations = attestations.lock().unwrap().to_vec();
    attestations
}

#[tokio::test]
#[serial(hsm)]
async fn test_pid_ok() {
    let _retain = setup_digid_context();

    let pin = "112233";
    let mut wallet = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Verify that the first mdoc contains the bsn
    let attestations = wallet_attestations(&mut wallet).await;
    let pid_attestation = attestations.first().unwrap();
    assert_eq!(pid_attestation.attestation_type, PID_DOCTYPE);

    let bsn_attr = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key == vec![BSN_ATTR_NAME])
        .unwrap();

    assert_eq!(
        bsn_attr.value,
        AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
    );

    let recovery_code_attr = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key == vec![PID_RECOVERY_CODE])
        .unwrap();

    assert_eq!(
        recovery_code_attr.value,
        AttestationAttributeValue::Basic(AttributeValue::Text("123".to_string()))
    );
}

fn universal_link(issuance_server_url: &BaseUrl) -> Url {
    let params = serde_urlencoded::to_string(VerifierUrlParameters {
        session_type: SessionType::SameDevice,
        ephemeral_id_params: None,
    })
    .unwrap();

    let mut issuance_server_url = issuance_server_url
        .join_base_url("/disclosure/university/request_uri")
        .into_inner();
    issuance_server_url.set_query(Some(&params));

    let query = serde_urlencoded::to_string(VpRequestUriObject {
        request_uri: issuance_server_url.try_into().unwrap(),
        request_uri_method: Some(RequestUriMethod::POST),
        client_id: "university.example.com".to_string(),
    })
    .unwrap();

    let mut uri = disclosure_based_issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()).into_inner();
    uri.set_query(Some(&query));

    uri
}

fn pid_without_optionals() -> IssuableDocument {
    IssuableDocument::try_new(
        MOCK_PID_DOCTYPE.to_string(),
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
        MOCK_PID_DOCTYPE.to_string(),
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
    let _retain = setup_digid_context();

    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
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
    let pid_attestation = attestations.first().unwrap();
    assert_eq!(pid_attestation.attestation_type, PID_DOCTYPE);

    let bsn_attr = pid_attestation.attributes.iter().find(|a| a.key == vec![BSN_ATTR_NAME]);

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
    let _retain = setup_digid_context();

    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
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

#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_ok() {
    let _context = setup_digid_context();

    let pin = "112233";
    let (mut wallet, _, issuance_url) = setup_wallet_and_env(
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

    let _proposal = wallet
        .start_disclosure(&universal_link(&issuance_url), DisclosureUriSource::Link)
        .await
        .unwrap();

    let attestation_previews = wallet.continue_disclosure_based_issuance(pin.to_owned()).await.unwrap();

    wallet.accept_issuance(pin.to_owned()).await.unwrap();

    // With collecting into this map, we willfully ignore the possibility here that the wallet might have
    // multiple attestation for a single attestation type.
    let attestations: HashMap<String, AttestationPresentation> = wallet_attestations(&mut wallet)
        .await
        .into_iter()
        .map(|att| (att.attestation_type.clone(), att))
        .collect();

    attestation_previews.iter().for_each(|preview| {
        let attestation = attestations.get(&preview.attestation_type).unwrap();
        assert_eq!(&attestation.attributes, &preview.attributes);
    });
}

#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_error_no_attributes() {
    let _context = setup_digid_context();

    let (issuance_server_settings, _, di_trust_anchor, di_tls_config) = issuance_server_settings();

    let pin = "112233";
    let (mut wallet, _, issuance_url) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        (issuance_server_settings, vec![], di_trust_anchor, di_tls_config),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let _proposal = wallet
        .start_disclosure(&universal_link(&issuance_url), DisclosureUriSource::Link)
        .await
        .unwrap();

    // If the issuer has no attestations to issue, we receive an empty vec and no error.
    let attestations = wallet.continue_disclosure_based_issuance(pin.to_owned()).await.unwrap();
    assert!(attestations.is_empty());
}
