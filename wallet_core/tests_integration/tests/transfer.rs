use assert_matches::assert_matches;
use serial_test::serial;
use tempfile::TempDir;

use dcql::CredentialFormat;
use openid4vc::disclosure_session::DisclosureUriSource;
use tests_integration::common::WalletWithStorage;
use tests_integration::common::do_pid_issuance;
use tests_integration::common::do_wallet_registration;
use tests_integration::common::setup_env_default;
use tests_integration::common::setup_tempfile_wallet;
use tests_integration::common::universal_link;
use tests_integration::common::wallet_attestations;
use wallet::TransferSessionState;
use wallet::errors::ChangePinError;
use wallet::errors::InstructionError;

async fn assert_states(
    expected_state: TransferSessionState,
    destination: &mut WalletWithStorage,
    source: &mut WalletWithStorage,
) {
    assert_state(expected_state, source).await;
    assert_state(expected_state, destination).await;
}

async fn assert_state(expected_state: TransferSessionState, wallet: &mut WalletWithStorage) {
    assert_eq!(wallet.get_transfer_status().await.unwrap(), expected_state);
}

async fn init_wallets(source_wallet_pin: &str, destination_wallet_pin: &str) -> (WalletWithStorage, WalletWithStorage) {
    let source_tempdir = TempDir::new().unwrap();
    let destination_tempdir = TempDir::new().unwrap();

    let (config_server_config, mock_device_config, wallet_config, issuance_url, _) = setup_env_default().await;

    let mut source = setup_tempfile_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.google_key_holder(),
        source_tempdir,
    )
    .await;
    source = do_wallet_registration(source, source_wallet_pin).await;
    source = do_pid_issuance(source, source_wallet_pin.to_owned()).await;
    source
        .start_disclosure(
            &universal_link(&issuance_url, CredentialFormat::SdJwt),
            DisclosureUriSource::Link,
        )
        .await
        .unwrap();
    source
        .continue_disclosure_based_issuance(source_wallet_pin.to_owned())
        .await
        .unwrap();
    source.accept_issuance(source_wallet_pin.to_owned()).await.unwrap();

    let mut destination = setup_tempfile_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
        destination_tempdir,
    )
    .await;
    destination = do_wallet_registration(destination, destination_wallet_pin).await;
    destination = do_pid_issuance(destination, destination_wallet_pin.to_string()).await;

    (source, destination)
}

#[tokio::test]
#[serial(hsm)]
async fn test_wallet_transfer() {
    let source_wallet_pin = "112233";
    let destination_wallet_pin = "332211";
    let (mut source, mut destination) = init_wallets(source_wallet_pin, destination_wallet_pin).await;

    let url = destination.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination).await;

    source.confirm_transfer(url).await.unwrap();

    assert_states(TransferSessionState::ReadyForTransfer, &mut destination, &mut source).await;

    // Other instructions are not allowed during transfer
    let err = destination
        .begin_change_pin(String::from(destination_wallet_pin), String::from("565656"))
        .await
        .expect_err("should fail during transfer");
    assert_matches!(
        err,
        ChangePinError::Instruction(InstructionError::InstructionValidation)
    );
    let err = source
        .begin_change_pin(String::from(source_wallet_pin), String::from("565656"))
        .await
        .expect_err("should fail during transfer");
    assert_matches!(
        err,
        ChangePinError::Instruction(InstructionError::InstructionValidation)
    );

    source.send_wallet_payload(source_wallet_pin.to_string()).await.unwrap();

    assert_state(TransferSessionState::ReadyForDownload, &mut source).await;
    assert_state(TransferSessionState::Success, &mut destination).await;
    assert_state(TransferSessionState::Success, &mut source).await;

    // Check if the content of the destination wallet
    let attestations = wallet_attestations(&mut destination).await;
    assert_eq!(3, attestations.len());

    // Check that the source wallet is empty
    let attestations = wallet_attestations(&mut source).await;
    assert!(attestations.is_empty());
}

#[tokio::test]
#[serial(hsm)]
async fn test_wallet_transfer_canceled_from_source() {
    let source_wallet_pin = "112233";
    let destination_wallet_pin = "332211";
    let (mut source, mut destination) = init_wallets(source_wallet_pin, destination_wallet_pin).await;

    let url = destination.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination).await;

    source.confirm_transfer(url).await.unwrap();

    assert_states(TransferSessionState::ReadyForTransfer, &mut destination, &mut source).await;

    source.cancel_transfer().await.unwrap();

    assert_state(TransferSessionState::Canceled, &mut destination).await;
}

#[tokio::test]
#[serial(hsm)]
async fn test_wallet_transfer_canceled_from_destination() {
    let source_wallet_pin = "112233";
    let destination_wallet_pin = "332211";
    let (mut source, mut destination) = init_wallets(source_wallet_pin, destination_wallet_pin).await;

    let url = destination.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination).await;

    source.confirm_transfer(url).await.unwrap();

    assert_states(TransferSessionState::ReadyForTransfer, &mut destination, &mut source).await;

    source.send_wallet_payload(source_wallet_pin.to_string()).await.unwrap();

    assert_state(TransferSessionState::ReadyForDownload, &mut source).await;

    destination.cancel_transfer().await.unwrap();

    assert_state(TransferSessionState::Canceled, &mut source).await;
}
