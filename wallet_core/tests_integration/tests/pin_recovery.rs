use std::assert_matches;

use db_test::DbSetup;
use hsm::test::HsmSetup;
use rstest::rstest;
use serial_test::serial;
use tests_integration::common::*;
use wallet::Pin;
use wallet::errors::InstructionError;
use wallet::errors::WalletUnlockError;

/// PIN recovery, against a PID issuer with or without the Credential Preview Endpoint.
#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc41_test_pin_recovery(#[values(true, false)] credential_preview_endpoint: bool) {
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
    wallet = do_pid_issuance(wallet, pin).await;

    let new_pin: Pin = "314159".into();
    wallet = do_pin_recovery(wallet, new_pin.clone()).await;

    // The wallet can now use the new PIN.
    wallet.check_pin(new_pin).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc46_test_pin_recovery_timeout() {
    let db_setup = DbSetup::create_clean().await;
    let hsm_setup = HsmSetup::new();
    let tempdir = tempfile::tempdir().unwrap();
    let pin: Pin = "112233".into();

    let (mut wallet, _, _) =
        setup_wallet_and_default_env(&db_setup, &hsm_setup, &tempdir, WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;
    wallet = do_pid_issuance(wallet, pin).await;

    // Put the wallet in timeout
    let wrong_pin: Pin = "332211".into();
    for _ in 0..3 {
        assert_matches!(
            wallet.check_pin(wrong_pin.clone()).await.unwrap_err(),
            WalletUnlockError::Instruction(InstructionError::IncorrectPin { .. })
        );
    }
    assert_matches!(
        wallet.check_pin(wrong_pin).await.unwrap_err(),
        WalletUnlockError::Instruction(InstructionError::Timeout { .. })
    );

    let new_pin: Pin = "314159".into();
    wallet = do_pin_recovery(wallet, new_pin.clone()).await;

    // The wallet can now use the new PIN.
    wallet.check_pin(new_pin).await.unwrap();
}
