use std::time::Duration;

use assert_matches::assert_matches;
use serial_test::serial;

use attestation_data::issuable_document::IssuableDocument;
use db_test::DbSetup;
use dcql::CredentialFormat;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls::BaseUrl;
use openid4vc::disclosure_session::DisclosureUriSource;
use pid_issuer::pid::constants::PID_ATTESTATION_TYPE;
use token_status_list::status_list_service::BatchIsRevoked;
use token_status_list::verification::verifier::RevocationStatus;
use wallet::AttestationPresentation;
use wallet::errors::DisclosureError;

use tests_integration::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_revocation_pid_ok() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (mut wallet, _, issuance_urls) = setup_wallet_and_default_env(&db_setup, WalletDeviceVendor::Apple).await;
    wallet.stop_background_revocation_checks();
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    assert_revokeable(&mut wallet, PID_ATTESTATION_TYPE, issuance_urls.pid_issuer.internal).await;

    // Disclosing a revoked attestation should fail
    let err = wallet
        .start_disclosure(
            &universal_link(
                issuance_urls.issuance_server.public.as_base_url(),
                CredentialFormat::SdJwt,
            ),
            DisclosureUriSource::Link,
        )
        .await
        .expect_err("should not be able to disclose revoked attestation");

    assert_matches!(err, DisclosureError::AttributesNotAvailable { .. });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_revocation_degree_ok() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (settings, _, trust_anchor, tls_config) = issuance_server_settings(db_setup.issuance_server_url());
    let (mut wallet, _, issuance_urls) = setup_wallet_and_env(
        &db_setup,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings(db_setup.pid_issuer_url(), "123".to_string()),
        (
            settings,
            vec![IssuableDocument::new_mock_degree("MSc".to_string())],
            trust_anchor,
            tls_config,
        ),
    )
    .await;
    wallet.stop_background_revocation_checks();
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Perform issuance of university degrees based on the PID.
    let _ = do_degree_issuance(
        &mut wallet,
        pin.to_owned(),
        &issuance_urls.issuance_server.public,
        CredentialFormat::SdJwt,
    )
    .await;

    assert_revokeable(
        &mut wallet,
        "com.example.degree",
        issuance_urls.issuance_server.internal.clone(),
    )
    .await;
}

async fn assert_revokeable(wallet: &mut WalletWithStorage, attestation_type: &str, revocation_url: BaseUrl) {
    // Verify that the newly issued attestation has no revocation status yet
    let attestation = get_attestation(wallet, attestation_type, None).await;
    assert!(attestation.is_some());

    wallet.start_background_revocation_checks(Duration::from_millis(50));
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if get_attestation(wallet, attestation_type, Some(RevocationStatus::Valid))
                .await
                .is_some()
            {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Timeout waiting for revocation status");

    revoke_all_attestations(revocation_url).await;

    // Check that the attestation now is revoked
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if get_attestation(wallet, attestation_type, Some(RevocationStatus::Revoked))
                .await
                .is_some()
            {
                break;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Timeout waiting for revocation status");
}

async fn revoke_all_attestations(internal_url: BaseUrl) {
    let client = default_reqwest_client_builder().build().unwrap();

    // Retrieve all batches
    let batch_values = client
        .get(internal_url.join("/batch/"))
        .send()
        .await
        .unwrap()
        .json::<Vec<BatchIsRevoked>>()
        .await
        .unwrap();

    let batch_ids = batch_values
        .into_iter()
        .filter_map(|batch| (!batch.is_revoked).then_some(batch.batch_id))
        .collect::<Vec<_>>();

    // Revoke all batches
    client
        .post(internal_url.join("/revoke/"))
        .json(&batch_ids)
        .send()
        .await
        .unwrap();
}

async fn get_attestation(
    wallet: &mut WalletWithStorage,
    attestation_type: &str,
    revocation_status: Option<RevocationStatus>,
) -> Option<AttestationPresentation> {
    let attestations = wallet_attestations(wallet).await;

    attestations.into_iter().find(|attestation| {
        attestation.attestation_type == attestation_type && attestation.validity.revocation_status == revocation_status
    })
}
