use std::time::Duration;

use assert_matches::assert_matches;
use serial_test::serial;
use tokio::time::sleep;

use tests_integration::common::*;
use wallet::errors::InstructionError;
use wallet::errors::WalletUnlockError;

#[tokio::test]
#[serial]
async fn test_unlock_ok() {
    let pin = "112234".to_string();

    let mut wallet = setup_wallet_and_default_env().await;
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

#[tokio::test]
#[serial]
async fn test_block() {
    let pin = "112234".to_string();

    let (mut settings, wp_root_ca) = wallet_provider_settings();
    settings.pin_policy.rounds = 1;
    settings.pin_policy.attempts_per_round = 2;
    settings.pin_policy.timeouts = vec![];

    let mut wallet = setup_wallet_and_env(
        config_server_settings(),
        update_policy_server_settings(),
        (settings, wp_root_ca),
        wallet_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;

    wallet.lock();
    assert!(wallet.is_locked());

    let result = wallet
        .unlock("555555".to_string())
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

    let result = wallet
        .unlock("555556".to_string())
        .await
        .expect_err("invalid pin should block wallet");
    assert_matches!(result, WalletUnlockError::Instruction(InstructionError::Blocked));
    assert!(wallet.is_locked());

    let result = wallet
        .unlock("112234".to_string())
        .await
        .expect_err("wallet should be blocked");
    assert_matches!(result, WalletUnlockError::Instruction(InstructionError::Blocked));
    assert!(wallet.is_locked());
}

#[tokio::test]
#[serial]
async fn test_unlock_error() {
    let pin = "112234".to_string();
    let mut wallet = setup_wallet_and_default_env().await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;

    wallet.lock();
    assert!(wallet.is_locked());

    let r1 = wallet
        .unlock("555555".to_string())
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
        .unlock("555556".to_string())
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
        .unlock("555557".to_string())
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
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r4,
        WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 200 })
    );
    assert!(wallet.is_locked());

    let r5 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(r5, WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: t }) if t < 200);
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(200)).await;

    let r6 = wallet
        .unlock("555557".to_string())
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
        .unlock("555557".to_string())
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
        .unlock("555557".to_string())
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

    let r8 = wallet
        .unlock("555557".to_string())
        .await
        .expect_err("invalid pin should return error");
    assert_matches!(
        r8,
        WalletUnlockError::Instruction(InstructionError::Timeout { timeout_millis: 400 })
    );
    assert!(wallet.is_locked());

    sleep(Duration::from_millis(400)).await;

    wallet.unlock(pin).await.expect("should unlock wallet");
    assert!(!wallet.is_locked());
}
