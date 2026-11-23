use std::collections::HashMap;
use std::collections::HashSet;

use attestation_types::credential_format::Format;
use db_test::DbSetup;
use hsm::test::HsmSetup;
use pid_issuer::pid::rulebook::constants::PID_BSN;
use pid_issuer::pid::rulebook::constants::PID_MDOC_DOCTYPE;
use pid_issuer::pid::rulebook::constants::PID_NL_MDOC_NAMESPACE;
use pid_issuer::pid::rulebook::constants::PID_NL_SD_JWT_VCT;
use pid_issuer::pid::rulebook::constants::PID_RECOVERY_CODE;
use rstest::rstest;
use serial_test::serial;
use tests_integration::common::*;
use wallet::AttestationIdentity;
use wallet::AttestationPresentation;
use wallet::AttributeValue;
use wallet::Pin;

/// PID issuance and renewal, against a PID issuer with or without the Credential Preview Endpoint.
#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc1_test_pid_ok(#[values(true, false)] credential_preview_endpoint: bool) {
    let db_setup = DbSetup::create_clean().await;
    let hsm_setup = HsmSetup::new();
    let tempdir = tempfile::tempdir().unwrap();
    let pin: Pin = "112233".into();

    let mut pid_issuer_settings = pid_issuer_settings(db_setup.pid_issuer_url(), None);
    pid_issuer_settings
        .authorizing_issuer_settings
        .issuer_settings
        .credential_preview_endpoint = credential_preview_endpoint;

    let (mut wallet, _, _) = setup_wallet_and_env(
        &db_setup,
        &hsm_setup,
        &tempdir,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings,
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin.clone()).await;

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

    let identities = identities_by_format(&attestations);

    // After the wallet is enrolled and has a PID, the PID can be renewed.
    wallet = do_pid_renewal(wallet, pin).await;

    let attestations = wallet_attestations(&mut wallet).await;

    assert_eq!(attestations.len(), 2);

    for pid_attestation in &attestations {
        test_pid_attestation(pid_attestation)
    }

    // Check that renewal updated the stored PID in both formats, instead of adding a new PID.
    assert_eq!(identities_by_format(&attestations), identities);
}

fn identities_by_format(attestations: &[AttestationPresentation]) -> HashMap<Format, AttestationIdentity> {
    attestations
        .iter()
        .map(|attestation| (attestation.format, attestation.identity))
        .collect()
}

fn test_pid_attestation(pid_attestation: &AttestationPresentation) {
    // A PID attestation should have the PID attestation type for its format.
    let expected_attestation_type = match pid_attestation.format {
        Format::MsoMdoc => PID_MDOC_DOCTYPE,
        Format::SdJwt => PID_NL_SD_JWT_VCT,
    };
    assert_eq!(pid_attestation.attestation_type, expected_attestation_type);

    // An mdoc's claim key is namespaced, unlike SD-JWT's flat one.
    let (expected_bsn_key, expected_recovery_code_key): (Vec<&str>, Vec<&str>) = match pid_attestation.format {
        Format::MsoMdoc => (
            vec![PID_NL_MDOC_NAMESPACE, PID_BSN],
            vec![PID_NL_MDOC_NAMESPACE, PID_RECOVERY_CODE],
        ),
        Format::SdJwt => (vec![PID_BSN], vec![PID_RECOVERY_CODE]),
    };

    // It should all contain the BSN.
    let bsn_attr = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq(expected_bsn_key.clone()))
        .unwrap();

    assert_eq!(bsn_attr.value, AttributeValue::Text("999991772".to_string()));

    // The recovery code should be hidden from presentation.
    let recovery_code_result = pid_attestation
        .attributes
        .iter()
        .find(|a| a.key.iter().eq(expected_recovery_code_key.clone()));

    assert_eq!(recovery_code_result, None);
}
