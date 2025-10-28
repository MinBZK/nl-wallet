use assert_matches::assert_matches;
use serial_test::serial;
use tempfile::TempDir;

use dcql::CredentialFormat;
use openid4vc::disclosure_session::DisclosureUriSource;
use tests_integration::common::WalletWithStorage;
use tests_integration::common::do_pid_issuance;
use tests_integration::common::do_wallet_registration;
use tests_integration::common::setup_env_default;
use tests_integration::common::setup_file_wallet;
use tests_integration::common::universal_link;
use tests_integration::common::wallet_attestations;
use wallet::TransferSessionState;
use wallet::errors::ChangePinError;
use wallet::errors::InstructionError;

struct WalletData {
    pub wallet: WalletWithStorage,
    pub pin: String,
    pub tempdir: TempDir,
}

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

async fn init_wallets() -> (WalletData, WalletData) {
    let source_wallet_pin = "112233";
    let destination_wallet_pin = "332211";

    let source_tempdir = TempDir::new().unwrap();
    let destination_tempdir = TempDir::new().unwrap();

    let (config_server_config, mock_device_config, wallet_config, issuance_url, _) = setup_env_default().await;

    let mut source = setup_file_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.google_key_holder(),
        source_tempdir.path().to_path_buf(),
    )
    .await;
    source = do_wallet_registration(source, source_wallet_pin).await;
    source = do_pid_issuance(source, String::from(source_wallet_pin)).await;
    source
        .start_disclosure(
            &universal_link(&issuance_url, CredentialFormat::SdJwt),
            DisclosureUriSource::Link,
        )
        .await
        .unwrap();
    source
        .continue_disclosure_based_issuance(&[0], String::from(source_wallet_pin))
        .await
        .unwrap();
    source.accept_issuance(String::from(source_wallet_pin)).await.unwrap();

    let mut destination = setup_file_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
        destination_tempdir.path().to_path_buf(),
    )
    .await;
    destination = do_wallet_registration(destination, destination_wallet_pin).await;
    destination = do_pid_issuance(destination, String::from(destination_wallet_pin)).await;

    (
        WalletData {
            wallet: source,
            pin: String::from(source_wallet_pin),
            tempdir: source_tempdir,
        },
        WalletData {
            wallet: destination,
            pin: String::from(destination_wallet_pin),
            tempdir: destination_tempdir,
        },
    )
}

#[tokio::test]
#[serial(hsm)]
async fn test_wallet_transfer() {
    let (
        WalletData {
            wallet: mut source,
            pin: source_wallet_pin,
            tempdir: _source_tempdir,
        },
        WalletData {
            wallet: mut destination,
            pin: destination_wallet_pin,
            tempdir: _destination_tempdir,
        },
    ) = init_wallets().await;

    let url = destination.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination).await;

    source.confirm_transfer(url).await.unwrap();

    assert_states(TransferSessionState::ReadyForTransfer, &mut destination, &mut source).await;

    // Other instructions are not allowed during transfer
    let err = destination
        .begin_change_pin(destination_wallet_pin.clone(), String::from("565656"))
        .await
        .expect_err("should fail during transfer");
    assert_matches!(
        err,
        ChangePinError::Instruction(InstructionError::InstructionValidation)
    );
    let err = source
        .begin_change_pin(source_wallet_pin.clone(), String::from("565656"))
        .await
        .expect_err("should fail during transfer");
    assert_matches!(
        err,
        ChangePinError::Instruction(InstructionError::InstructionValidation)
    );

    source
        .prepare_send_wallet_payload(source_wallet_pin.clone())
        .await
        .unwrap();

    assert_states(
        TransferSessionState::ReadyForTransferConfirmed,
        &mut destination,
        &mut source,
    )
    .await;

    source.send_wallet_payload().await.unwrap();

    assert_states(TransferSessionState::ReadyForDownload, &mut destination, &mut source).await;

    destination.receive_wallet_payload().await.unwrap();

    assert_state(TransferSessionState::Success, &mut source).await;

    // Check if the content of the destination wallet
    let attestations = wallet_attestations(&mut destination).await;
    assert_eq!(4, attestations.len());

    // Check that the source wallet is empty
    let attestations = wallet_attestations(&mut source).await;
    assert!(attestations.is_empty());
}

#[tokio::test]
#[serial(hsm)]
async fn test_wallet_transfer_canceled_from_source() {
    let (mut source_data, mut destination_data) = init_wallets().await;

    let url = destination_data.wallet.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination_data.wallet).await;

    source_data.wallet.confirm_transfer(url).await.unwrap();

    assert_states(
        TransferSessionState::ReadyForTransfer,
        &mut destination_data.wallet,
        &mut source_data.wallet,
    )
    .await;

    source_data.wallet.cancel_transfer(false).await.unwrap();

    assert_state(TransferSessionState::Canceled, &mut destination_data.wallet).await;
}

#[tokio::test]
#[serial(hsm)]
async fn test_wallet_transfer_canceled_from_destination() {
    let (
        WalletData {
            wallet: mut source,
            pin: source_wallet_pin,
            tempdir: _source_tempdir,
        },
        WalletData {
            wallet: mut destination,
            tempdir: _destination_tempdir,
            ..
        },
    ) = init_wallets().await;

    let url = destination.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination).await;

    source.confirm_transfer(url).await.unwrap();

    assert_states(TransferSessionState::ReadyForTransfer, &mut destination, &mut source).await;

    source
        .prepare_send_wallet_payload(source_wallet_pin.clone())
        .await
        .unwrap();

    assert_states(
        TransferSessionState::ReadyForTransferConfirmed,
        &mut destination,
        &mut source,
    )
    .await;

    source.send_wallet_payload().await.unwrap();

    assert_state(TransferSessionState::ReadyForDownload, &mut source).await;

    destination.cancel_transfer(false).await.unwrap();

    assert_state(TransferSessionState::Canceled, &mut source).await;
}

#[tokio::test]
#[serial(hsm)]
async fn test_retry_transfer_after_canceled() {
    let (mut source_data, mut destination_data) = init_wallets().await;

    let url = destination_data.wallet.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination_data.wallet).await;

    source_data.wallet.confirm_transfer(url).await.unwrap();

    assert_states(
        TransferSessionState::ReadyForTransfer,
        &mut destination_data.wallet,
        &mut source_data.wallet,
    )
    .await;

    source_data.wallet.cancel_transfer(false).await.unwrap();

    assert_state(TransferSessionState::Canceled, &mut destination_data.wallet).await;

    let _url = destination_data.wallet.init_transfer().await.unwrap();

    assert_state(TransferSessionState::Created, &mut destination_data.wallet).await;
}
