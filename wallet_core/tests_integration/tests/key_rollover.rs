use std::collections::HashMap;
use std::sync::Arc;

use crypto::p256_der::DerVerifyingKey;
use db_test::DbSetup;
use hsm::model::Hsm as _;
use hsm::test::HsmSetup;
use http_utils::client::TlsPinningConfig;
use serial_test::serial;
use tests_integration::common::*;
use utils::vec_at_least::VecNonEmpty;
use wallet::Pin;
use wallet::errors::InstructionError;
use wallet::errors::WalletUnlockError;
use wallet_provider_service::keys::Kid;

/// Tests the full instruction result signing key rollover lifecycle with a real wallet and WP.
///
/// In practice, a rollover proceeds as follows:
/// 1. A new key is added to the wallet app's configuration and distributed to wallets.
/// 2. The wallet provider rolls over to the new key.
/// 3. The old key is removed from the wallet configuration, after when the rollover is complete.
///
/// A single registered wallet is used for the test. The wallet's configuration is updated directly
/// in memory between stages, no config server is involved in the updates. The wallet provider
/// is restarted once when the signing key rolls over.
///
/// The HSM must have both instruction result signing keys pre-provisioned:
/// - `instruction_result_signing_0` (kid = "0") — the current key
/// - `instruction_result_signing_1` (kid = "1") — the new key (added by setup-devenv.sh)
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_instruction_result_key_rollover() {
    let db_setup = DbSetup::create().await;
    let hsm_setup = HsmSetup::new();
    let pin: Pin = "112234".into();
    let mock_device_config = MockDeviceConfig::generate();

    // Start the update policy server (needed for the wallet to work)
    let (ups_settings, ups_root_ca) = update_policy_server_settings();
    let ups_port = start_update_policy_server(ups_settings, ups_root_ca.clone()).await;

    let (mut wp_settings, wp_root_ca) =
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url());
    wp_settings.ios = mock_device_config.ios_wp_settings();
    wp_settings.android.root_public_keys = mock_device_config.android_root_public_keys();

    let hsm = hsm_setup
        .pkcs11_hsm(wp_settings.hsm.clone())
        .expect("Could not initialize HSM");

    // Retrieve both instruction result public keys from the HSM
    let ir_pubkey_0: DerVerifyingKey = hsm
        .get_verifying_key("instruction_result_signing_0")
        .await
        .unwrap()
        .into();
    let ir_pubkey_1: DerVerifyingKey = hsm
        .get_verifying_key("instruction_result_signing_1")
        .await
        .unwrap()
        .into();

    // Start the WP with the initial instruction result signing key (kid = "0")
    let (wp_port, wp_abort) =
        start_wallet_provider_with_abort_handle(wp_settings.clone(), hsm.clone(), wp_root_ca.clone()).await;

    // Set up the wallet with the initial configuration
    let (config_server_config, mut wallet_config) = build_wallet_environment_with_instruction_result_keys(
        ups_port,
        ups_root_ca.clone(),
        wp_port,
        HashMap::from([("0".to_string(), ir_pubkey_0.clone())]),
    )
    .await;
    let mut wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin.clone()).await;

    // Obtain a handle to the wallet's config repository for direct in-memory updates
    let config_repo = wallet.config_repository();

    // Stage 1: Before rollover — WP uses old key, wallet has old config only
    wallet.lock();
    wallet
        .unlock(pin.clone())
        .await
        .expect("stage 1: old key accepted by wallet with old config");

    // Stage 2: New key added to wallet config, WP still uses old key
    wallet_config.account_server.instruction_result_public_keys = HashMap::from([
        ("0".to_string(), ir_pubkey_0.clone()),
        ("1".to_string(), ir_pubkey_1.clone()),
    ]);
    config_repo.update_config(Arc::new(wallet_config.clone()));
    wallet.lock();
    wallet
        .unlock(pin.clone())
        .await
        .expect("stage 2: old key accepted by wallet with both keys configured");

    // Failure: old key removed from wallet config before WP has rolled over (premature cleanup)
    wallet_config.account_server.instruction_result_public_keys =
        HashMap::from([("1".to_string(), ir_pubkey_1.clone())]);
    config_repo.update_config(Arc::new(wallet_config.clone()));
    wallet.lock();
    let error = wallet
        .unlock(pin.clone())
        .await
        .expect_err("failure: old key rejected by wallet configured with only the new key");
    assert!(matches!(
        error,
        WalletUnlockError::Instruction(InstructionError::InstructionResultValidation(_))
    ));

    // Roll over: stop the old WP and start a new one with kid = "1"
    drop(wp_abort);
    wp_settings.current_instruction_result_kid = Kid::try_new("1".to_owned()).unwrap();
    let wp_port = start_wallet_provider(wp_settings, hsm.clone(), wp_root_ca.clone()).await;

    // Point the wallet at the new WP by updating the account server URL in the config
    wallet_config.account_server.http_config = TlsPinningConfig::try_new(
        local_wp_base_url(wp_port),
        VecNonEmpty::try_from(wallet_config.account_server.http_config.trust_anchors().to_vec()).unwrap(),
    )
    .unwrap();

    // Stage 3: WP rolled over to new key, wallet config has both keys
    config_repo.update_config(Arc::new(wallet_config.clone()));
    wallet.lock();
    wallet
        .unlock(pin.clone())
        .await
        .expect("stage 3: new key accepted by wallet with both keys configured");

    // Stage 4: Old key removed from wallet config, WP uses new key
    wallet_config.account_server.instruction_result_public_keys =
        HashMap::from([("1".to_string(), ir_pubkey_1.clone())]);
    config_repo.update_config(Arc::new(wallet_config.clone()));
    wallet.lock();
    wallet
        .unlock(pin.clone())
        .await
        .expect("stage 4: new key accepted by wallet with new config only");

    // Failure case: wallet has not yet fetched the updated config when WP rolls over
    wallet_config.account_server.instruction_result_public_keys = HashMap::from([("0".to_string(), ir_pubkey_0)]);
    config_repo.update_config(Arc::new(wallet_config.clone()));
    wallet.lock();
    let error = wallet
        .unlock(pin)
        .await
        .expect_err("failure: new key rejected by wallet with stale config");
    assert!(matches!(
        error,
        WalletUnlockError::Instruction(InstructionError::InstructionResultValidation(_))
    ));
}
