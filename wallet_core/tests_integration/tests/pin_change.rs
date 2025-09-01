use serial_test::serial;

use tests_integration::common::*;

#[tokio::test]
#[serial(hsm)]
async fn test_pin_change() {
    let settings_and_ca = wallet_provider_settings();
    let old_pin_str = "123344";
    let new_pin_str = "123355";

    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        settings_and_ca,
        verification_server_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await;
    wallet = do_wallet_registration(wallet, old_pin_str).await;

    let old_pin = old_pin_str.parse().unwrap();
    let new_pin = new_pin_str.parse().unwrap();

    let begin = wallet.begin_change_pin(old_pin, new_pin).await;
    assert!(begin.is_ok(), "begin_change_pin failed: {:?}", begin.err());

    let result = wallet.continue_change_pin(new_pin_str).await;
    assert!(result.is_ok(), "continue_change_pin failed: {:?}", result.err());
}
