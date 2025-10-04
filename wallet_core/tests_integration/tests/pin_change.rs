use serial_test::serial;

use tests_integration::common::*;

#[tokio::test]
#[serial(hsm)]
async fn test_pin_change() {
    let old_pin_str = "123344";
    let new_pin_str = "123355";

    let (mut wallet, _, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, old_pin_str).await;

    let old_pin = old_pin_str.parse().unwrap();
    let new_pin = new_pin_str.parse().unwrap();

    let begin = wallet.begin_change_pin(old_pin, new_pin).await;
    assert!(begin.is_ok(), "begin_change_pin failed: {:?}", begin.err());

    let result = wallet.continue_change_pin(new_pin_str).await;
    assert!(result.is_ok(), "continue_change_pin failed: {:?}", result.err());
}

#[tokio::test]
#[serial(hsm)]
async fn test_pin_recovery() {
    let pin = "112233".to_string();
    let (mut wallet, _, _) = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, &pin).await;
    wallet = do_pid_issuance(wallet, pin).await;

    let new_pin = "314159".to_string();
    let uri = wallet.create_pin_recovery_redirect_uri().await.unwrap();
    wallet.continue_pin_recovery(uri).await.unwrap();
    wallet.complete_pin_recovery(new_pin.clone()).await.unwrap();

    // The wallet can now use the new PIN.
    wallet.check_pin(new_pin).await.unwrap();
}
