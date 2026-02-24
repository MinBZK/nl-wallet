use serial_test::serial;

use attestation_data::attributes::Attributes;
use attestation_data::issuable_document::IssuableDocument;
use openid4vc::ErrorResponse;
use openid4vc::issuance_session::IssuanceSessionError;
use pid_issuer::pid::constants::PID_ADDRESS_GROUP;
use pid_issuer::pid::constants::PID_ATTESTATION_TYPE;
use pid_issuer::pid::constants::PID_BIRTH_DATE;
use pid_issuer::pid::constants::PID_BSN;
use pid_issuer::pid::constants::PID_FAMILY_NAME;
use pid_issuer::pid::constants::PID_GIVEN_NAME;
use pid_issuer::pid::constants::PID_RECOVERY_CODE;
use pid_issuer::pid::constants::PID_RESIDENT_CITY;
use pid_issuer::pid::constants::PID_RESIDENT_COUNTRY;
use pid_issuer::pid::constants::PID_RESIDENT_HOUSE_NUMBER;
use pid_issuer::pid::constants::PID_RESIDENT_POSTAL_CODE;
use pid_issuer::pid::constants::PID_RESIDENT_STREET;
use wallet::AttestationAttributeValue;
use wallet::PidIssuancePurpose;
use wallet::attestation_data::AttributeValue;
use wallet::errors::IssuanceError;

use tests_integration::common::*;

#[tokio::test]
#[serial(hsm)]
async fn ltc1_test_pid_ok() {
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
        .find(|a| a.key.iter().eq([PID_BSN]))
        .unwrap();

    assert_eq!(
        bsn_attr.value,
        AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
    );

    // Recovery code is hidden from presentation
    let recovery_code_result = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq([PID_RECOVERY_CODE]));

    assert_eq!(recovery_code_result, None);

    // After the wallet is enrolled and has a PID, the PID can be renewed.
    wallet = do_pid_renewal(wallet, pin.to_owned()).await;

    let attestations = wallet_attestations(&mut wallet).await;
    assert_eq!(
        attestations
            .iter()
            .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
            .iter()
            .count(),
        1
    );
}

fn pid_without_optionals_with_address() -> IssuableDocument {
    IssuableDocument::try_new_with_random_id(
        PID_ATTESTATION_TYPE.to_string(),
        Attributes::example([
            (vec![PID_FAMILY_NAME], AttributeValue::Text("De Bruijn".to_string())),
            (
                vec![PID_GIVEN_NAME],
                AttributeValue::Text("Willeke Liselotte".to_string()),
            ),
            (vec![PID_BIRTH_DATE], AttributeValue::Text("1997-05-10".to_string())),
            // only age_over_18 is optional in `eudi:pid:nl:1`
            (vec![PID_BSN], AttributeValue::Text("999991772".to_string())),
            (vec![PID_RECOVERY_CODE], AttributeValue::Text("123".to_string())),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
                AttributeValue::Text("Turfmarkt".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
                AttributeValue::Text("147".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
                AttributeValue::Text("2511 DP".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
                AttributeValue::Text("Den Haag".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
                AttributeValue::Text("Nederland".to_string()),
            ),
        ]),
    )
    .unwrap()
}

fn pid_missing_required_with_address() -> IssuableDocument {
    IssuableDocument::try_new_with_random_id(
        PID_ATTESTATION_TYPE.to_string(),
        Attributes::example([
            (vec![PID_FAMILY_NAME], AttributeValue::Text("De Bruijn".to_string())),
            (
                vec![PID_GIVEN_NAME],
                AttributeValue::Text("Willeke Liselotte".to_string()),
            ),
            (vec![PID_BIRTH_DATE], AttributeValue::Text("1997-05-10".to_string())),
            // bsn is missing, which is required
            (vec![PID_RECOVERY_CODE], AttributeValue::Text("123".to_string())),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
                AttributeValue::Text("Turfmarkt".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
                AttributeValue::Text("147".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
                AttributeValue::Text("2511 DP".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
                AttributeValue::Text("Den Haag".to_string()),
            ),
            (
                vec![PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
                AttributeValue::Text("Nederland".to_string()),
            ),
        ]),
    )
    .unwrap()
}

#[tokio::test]
#[serial(hsm)]
async fn ltc1_test_pid_optional_attributes() {
    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(),
        (
            pid_issuer_settings("123".to_string()).0,
            vec![pid_without_optionals_with_address()].try_into().unwrap(),
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

    let bsn_attr = pid_attestation.attributes.iter().find(|a| a.key.iter().eq([PID_BSN]));

    match bsn_attr {
        Some(bsn_attr) => assert_eq!(
            bsn_attr.value,
            AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
        ),
        None => panic!("BSN attribute not found"),
    }
    // Verify that we didn't get `age_over_18` issued
    let age_over_18 = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq(["age_over_18"]));
    assert!(age_over_18.is_none());
}

#[tokio::test]
#[serial(hsm)]
async fn ltc2_test_pid_missing_required_attributes() {
    let pin = "112233";
    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(),
        (
            pid_issuer_settings("123".to_string()).0,
            vec![pid_missing_required_with_address()].try_into().unwrap(),
        ),
        issuance_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    let redirect_url = wallet
        .create_pid_issuance_auth_url(PidIssuancePurpose::Enrollment)
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
