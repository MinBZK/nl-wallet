use std::assert_matches;
use std::time::Duration;

use db_test::DbSetup;
use hsm::test::HsmSetup;
use rstest::rstest;
use serial_test::serial;
use tests_integration::common::*;
use tokio::time::sleep;
use wallet::Pin;
use wallet::errors::InstructionError;
use wallet::errors::WalletUnlockError;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
#[serial(hsm)]
async fn ltc37_test_unlock_ok(
    #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
) {
    let db_setup = DbSetup::create().await;
    let hsm_setup = HsmSetup::new();
    let pin: Pin = "112234".into();

    let (mut wallet, _, _) = setup_wallet_and_default_env(&db_setup, &hsm_setup, vendor).await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;

    wallet.lock();
    assert!(wallet.is_locked());

    wallet.unlock(pin.clone()).await.expect("Should unlock wallet");
    assert!(!wallet.is_locked());

    wallet.lock();

    // Test multiple instructions
    wallet.unlock(pin).await.expect("Should unlock wallet");
    assert!(!wallet.is_locked());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc47_test_block() {
    let db_setup = DbSetup::create().await;
    let hsm_setup = HsmSetup::new();
    let correct_pin: Pin = "112234".into();

    let (mut settings, wp_root_ca) = wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url());
    settings.pin_policy.rounds = 1;
    settings.pin_policy.attempts_per_round = 2;
    settings.pin_policy.timeouts = vec![];

    let (mut wallet, _, _) = setup_wallet_and_env(
        &db_setup,
        &hsm_setup,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        (settings, wp_root_ca),
        pid_issuer_settings(db_setup.pid_issuer_url(), None),
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await;
    wallet = do_wallet_registration(wallet, correct_pin.clone()).await;

    wallet.lock();
    assert!(wallet.is_locked());

    let wrong_pin = "555555".into();
    let result = wallet
        .unlock(wrong_pin)
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        result,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 1,
            is_final_round: true
        })
    );
    assert!(wallet.is_locked());

    let another_wrong_pin = "555556".into();
    let result = wallet
        .unlock(another_wrong_pin)
        .await
        .expect_err("invalid pin should block wallet");
    assert_matches!(result, WalletUnlockError::Instruction(InstructionError::Blocked));
    assert!(wallet.is_locked());

    let result = wallet.unlock(correct_pin).await.expect_err("wallet should be blocked");
    assert_matches!(result, WalletUnlockError::Instruction(InstructionError::Blocked));
    assert!(wallet.is_locked());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc46_test_unlock_error() {
    let db_setup = DbSetup::create().await;
    let hsm_setup = HsmSetup::new();
    let correct_pin: Pin = "112234".into();
    let wrong_pin_1: Pin = "555555".into();
    let wrong_pin_2: Pin = "555556".into();
    let wrong_pin_3: Pin = "555557".into();

    let (mut wallet, _, _) = setup_wallet_and_default_env(&db_setup, &hsm_setup, WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, correct_pin.clone()).await;

    wallet.lock();
    assert!(wallet.is_locked());

    let r1 = wallet
        .unlock(wrong_pin_1)
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r1,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 3,
            is_final_round: false
        })
    );
    assert!(wallet.is_locked());

    let r2 = wallet
        .unlock(wrong_pin_2)
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r2,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 2,
            is_final_round: false
        })
    );
    assert!(wallet.is_locked());

    let r3 = wallet
        .unlock(wrong_pin_3.clone())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r3,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 1,
            is_final_round: false
        })
    );
    assert!(wallet.is_locked());

    // Sleeping before a timeout is expected influence timeout.
    sleep(Duration::from_millis(200)).await;

    let r4 = wallet
        .unlock(wrong_pin_3.clone())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r4,
        WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 200 })
    );
    assert!(wallet.is_locked());

    let r5 = wallet
        .unlock(wrong_pin_3.clone())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(r5, WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: t }) if t < 200);
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(200)).await;

    let r6 = wallet
        .unlock(wrong_pin_3.clone())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r6,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 3,
            is_final_round: false
        })
    );
    assert!(wallet.is_locked());

    let r7 = wallet
        .unlock(wrong_pin_3.clone())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r7,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 2,
            is_final_round: false
        })
    );
    assert!(wallet.is_locked());

    let r8 = wallet
        .unlock(wrong_pin_3.clone())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r8,
        WalletUnlockError::Instruction(InstructionError::IncorrectPin {
            attempts_left_in_round: 1,
            is_final_round: false
        })
    );
    assert!(wallet.is_locked());

    let r9 = wallet
        .unlock(wrong_pin_3)
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r9,
        WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 400 })
    );
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(400)).await;

    wallet.unlock(correct_pin).await.expect("should unlock wallet");
    assert!(!wallet.is_locked());
}
