use std::assert_matches;
use std::collections::HashSet;

use attestation_data::attributes::Attributes;
use attestation_types::credential_format::Format;
use db_test::DbSetup;
use openid4vc::ErrorResponse;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::wallet_issuance::WalletIssuanceError;
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
use serial_test::serial;
use tests_integration::common::*;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use wallet::AttestationAttributeValue;
use wallet::AttestationPresentation;
use wallet::PidIssuancePurpose;
use wallet::attestation_data::AttributeValue;
use wallet::errors::IssuanceError;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc1_test_pid_ok() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (mut wallet, _, _) = setup_wallet_and_default_env(&db_setup, WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let attestations = wallet_attestations(&mut wallet).await;

    assert_eq!(attestations.len(), 2);

    for pid_attestation in &attestations {
        test_pid_attestation(pid_attestation)
    }

    // Both formats should have been issued.
    let formats = attestations
        .iter()
        .map(|attestation| attestation.format)
        .collect::<HashSet<_>>();

    assert!(formats.contains(&Format::MsoMdoc));
    assert!(formats.contains(&Format::SdJwt));

    // After the wallet is enrolled and has a PID, the PID can be renewed.
    wallet = do_pid_renewal(wallet, pin.to_owned()).await;

    let attestations = wallet_attestations(&mut wallet).await;

    assert_eq!(attestations.len(), 2);

    for pid_attestation in &attestations {
        test_pid_attestation(pid_attestation)
    }

    // Check that both formats are still present.
    let renewed_formats = attestations
        .iter()
        .map(|attestation| attestation.format)
        .collect::<HashSet<_>>();

    assert_eq!(formats, renewed_formats);
}

fn test_pid_attestation(pid_attestation: &AttestationPresentation) {
    // A PID attestation should have the PID attestation type.
    assert_eq!(pid_attestation.attestation_type, PID_ATTESTATION_TYPE);

    // It should all contain the BSN.
    let bsn_attr = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq([PID_BSN]))
        .unwrap();

    assert_eq!(
        bsn_attr.value,
        AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
    );

    // The recovery code should be hidden from presentation.
    let recovery_code_result = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq([PID_RECOVERY_CODE]));

    assert_eq!(recovery_code_result, None);
}

fn pid_without_optionals_with_address() -> VecNonEmpty<IssuableDocument> {
    vec_nonempty![Format::MsoMdoc, Format::SdJwt]
        .into_nonempty_iter()
        .map(|format| {
            IssuableDocument::try_new_with_random_id(
                format,
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
        })
        .collect()
}

fn pid_missing_required_with_address() -> VecNonEmpty<IssuableDocument> {
    vec_nonempty![Format::MsoMdoc, Format::SdJwt]
        .into_nonempty_iter()
        .map(|format| {
            IssuableDocument::try_new_with_random_id(
                format,
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
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc1_test_pid_optional_attributes() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (mut wallet, _, _) = setup_wallet_and_env(
        &db_setup,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        (
            pid_issuer_settings(db_setup.pid_issuer_url()).0,
            pid_without_optionals_with_address(),
        ),
        issuance_server_settings(db_setup.issuance_server_url()),
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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc2_test_pid_missing_required_attributes() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (mut wallet, _, _) = setup_wallet_and_env(
        &db_setup,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        (
            pid_issuer_settings(db_setup.pid_issuer_url()).0,
            pid_missing_required_with_address(),
        ),
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin).await;

    let redirect_uri = wallet
        .create_pid_issuance_auth_url(PidIssuancePurpose::Enrollment)
        .await
        .expect("should create PID issuance redirect URL");

    let _unsigned_mdocs = wallet
        .continue_issuance(fake_oidc_redirect(redirect_uri).await)
        .await
        .expect("should continue PID issuance");
    let error = wallet
        .accept_issuance(pin.to_owned())
        .await
        .expect_err("should fail to accept issuance");

    assert_matches!(
        &error,
        IssuanceError::IssuerServer {
            error: WalletIssuanceError::CredentialRequest(response),
            ..
        } if matches!(
            response.as_ref(),
            ErrorResponse { error_description: Some(description), .. }
                if description.contains("\"urn:eudi:pid:nl:1\": \"bsn\" is a required property")
        )
    );
}
