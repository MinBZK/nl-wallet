use std::time::Duration;

use serial_test::serial;

use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
use token_status_list::verification::verifier::RevocationStatus;
use wallet::AttestationPresentation;

use tests_integration::common::*;

#[tokio::test]
#[serial(hsm)]
async fn test_revocation_ok() {
    let pin = "112233";

    let (mut wallet, _, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Verify that the newly issued PID has no revocation status yet
    let attestations = wallet_attestations(&mut wallet).await;
    let pid_attestation = attestations
        .iter()
        .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
        .expect("should have received PID attestation");

    assert!(pid_attestation.validity.revocation_status.is_none());

    wallet.stop_background_revocation_checks();
    wallet.start_background_revocation_checks(Duration::from_millis(10));

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let pid_attestation = get_pid_attestation(&mut wallet).await;
            if pid_attestation.validity.revocation_status.is_some() {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Timeout waiting for revocation status");

    wallet.stop_background_revocation_checks();
    let pid_attestation = get_pid_attestation(&mut wallet).await;

    assert_eq!(
        Some(RevocationStatus::Valid),
        pid_attestation.validity.revocation_status,
    );
}

async fn get_pid_attestation(wallet: &mut WalletWithStorage) -> AttestationPresentation {
    let attestations = wallet_attestations(wallet).await;
    attestations
        .into_iter()
        .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
        .expect("should have received PID attestation")
}
