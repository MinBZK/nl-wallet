use serial_test::serial;

use tests_integration::common::*;

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
