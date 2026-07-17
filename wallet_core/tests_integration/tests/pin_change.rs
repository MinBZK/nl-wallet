use db_test::DbSetup;
use hsm::test::HsmSetup;
use serial_test::serial;
use tests_integration::common::*;
use wallet::Pin;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc45_test_pin_change() {
    let db_setup = DbSetup::create().await;
    let hsm_setup = HsmSetup::new();

    let old_pin: Pin = "123344".into();
    let new_pin: Pin = "123355".into();

    let (mut wallet, _, _) = setup_wallet_and_default_env(&db_setup, &hsm_setup, WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, old_pin.clone()).await;

    let begin = wallet.begin_change_pin(old_pin, new_pin.clone()).await;
    assert!(begin.is_ok(), "begin_change_pin failed: {:?}", begin.err());

    let result = wallet.continue_change_pin(&new_pin).await;
    assert!(result.is_ok(), "continue_change_pin failed: {:?}", result.err());
}
