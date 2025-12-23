use std::time::Duration;

use sea_orm::prelude::Uuid;
use serde::Deserialize;
use serial_test::serial;

use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
use http_utils::reqwest::default_reqwest_client_builder;
use token_status_list::verification::verifier::RevocationStatus;
use wallet::AttestationPresentation;

use tests_integration::common::*;

#[tokio::test]
#[serial(hsm)]
async fn test_revocation_ok() {
    let pin = "112233";

    let (mut wallet, _, issuance_urls) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
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
            if pid_attestation.validity.revocation_status == Some(RevocationStatus::Valid) {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Timeout waiting for revocation status");

    wallet.stop_background_revocation_checks();

    let client = default_reqwest_client_builder().build().unwrap();

    #[derive(Deserialize)]
    struct Batch {
        batch_id: Uuid,
        is_revoked: bool,
    }

    // Retrieve all batches
    let batch_values = client
        .get(issuance_urls.pid_issuer_internal_url.join("/batch/"))
        .send()
        .await
        .unwrap()
        .json::<Vec<Batch>>()
        .await
        .unwrap();

    let batch_ids = batch_values
        .into_iter()
        .filter_map(|batch| (!batch.is_revoked).then_some(batch.batch_id))
        .collect::<Vec<_>>();

    // Revoke all batches
    client
        .post(issuance_urls.pid_issuer_internal_url.join("/revoke/"))
        .json(&batch_ids)
        .send()
        .await
        .unwrap();

    // Check that the pid attestation now is revoked
    wallet.start_background_revocation_checks(Duration::from_millis(10));
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let pid_attestation = get_pid_attestation(&mut wallet).await;
            if pid_attestation.validity.revocation_status == Some(RevocationStatus::Revoked) {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Timeout waiting for revocation status");

    wallet.stop_background_revocation_checks();
}

async fn get_pid_attestation(wallet: &mut WalletWithStorage) -> AttestationPresentation {
    let attestations = wallet_attestations(wallet).await;
    attestations
        .into_iter()
        .find(|attestation| attestation.attestation_type == PID_ATTESTATION_TYPE)
        .expect("should have received PID attestation")
}
