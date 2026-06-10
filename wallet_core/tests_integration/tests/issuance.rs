use std::collections::HashSet;

use attestation_types::credential_format::Format;
use db_test::DbSetup;
use pid_issuer::pid::constants::PID_ATTESTATION_TYPE;
use pid_issuer::pid::constants::PID_BSN;
use pid_issuer::pid::constants::PID_RECOVERY_CODE;
use serial_test::serial;
use tests_integration::common::*;
use wallet::AttestationPresentation;
use wallet::attestation_data::AttributeValue;

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

    assert_eq!(bsn_attr.value, AttributeValue::Text("999991772".to_string()));

    // The recovery code should be hidden from presentation.
    let recovery_code_result = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq([PID_RECOVERY_CODE]));

    assert_eq!(recovery_code_result, None);
}
