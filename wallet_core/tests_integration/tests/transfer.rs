use rstest::rstest;
use serial_test::serial;

use tests_integration::common::do_pid_issuance;
use tests_integration::common::do_wallet_registration;
use tests_integration::common::setup_env_default;
use tests_integration::common::setup_wallet;
use wallet::TransferSessionState;

#[tokio::test]
#[rstest]
#[serial(hsm)]
async fn test_wallet_transfer() {
    let (config_server_config, mock_device_config, wallet_config, _, _) = setup_env_default().await;

    let source_wallet_pin = "112233";
    let mut source_wallet = setup_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    source_wallet = do_wallet_registration(source_wallet, source_wallet_pin).await;
    source_wallet = do_pid_issuance(source_wallet, source_wallet_pin.to_owned()).await;

    let destination_wallet_pin = "332211";
    let mut destination_wallet = setup_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    destination_wallet = do_wallet_registration(destination_wallet, destination_wallet_pin).await;
    destination_wallet = do_pid_issuance(destination_wallet, destination_wallet_pin.to_owned()).await;

    let _url = destination_wallet.init_transfer().await.unwrap();

    let state = destination_wallet.get_transfer_status().await.unwrap();
    assert_eq!(state, TransferSessionState::Created);

    let state = source_wallet.get_transfer_status().await.unwrap();
    assert_eq!(state, TransferSessionState::Created);
}
