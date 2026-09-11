use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use crypto::p256_der::DerVerifyingKey;
use db_test::DbSetup;
use hsm::model::Hsm as _;
use hsm::service::Pkcs11Hsm;
use hsm::test::HsmSetup;
use http_utils::client::TlsPinningConfig;
use http_utils::reqwest::ReqwestTrustAnchor;
use serial_test::serial;
use tests_integration::common::*;
use utils::vec_at_least::VecNonEmpty;
use wallet::Pin;
use wallet::errors::ChangePinError;
use wallet::errors::InstructionError;
use wallet::errors::WalletUnlockError;
use wallet_configuration::wallet_config::CertificatePublicKey;
use wallet_provider::settings::Settings as WpSettings;
use wallet_provider_service::keys::Kid;

/// Sets up the shared test infrastructure for a key rollover test: a database, an HSM, a mock
/// device, the update policy server (needed for the wallet to work) and the wallet provider
/// settings (not yet started, so that the current signing key kid can still be adjusted).
async fn setup_key_rollover_env() -> (
    DbSetup,
    HsmSetup,
    MockDeviceConfig,
    u16,
    ReqwestTrustAnchor,
    WpSettings,
    ReqwestTrustAnchor,
    Pkcs11Hsm,
) {
    let db_setup = DbSetup::create().await;
    let hsm_setup = HsmSetup::new();
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

    (
        db_setup,
        hsm_setup,
        mock_device_config,
        ups_port,
        ups_root_ca,
        wp_settings,
        wp_root_ca,
        hsm,
    )
}

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
    // Don't drop `_db_setup` and `_hsm_setup`
    let (_db_setup, _hsm_setup, mock_device_config, ups_port, ups_root_ca, mut wp_settings, wp_root_ca, hsm) =
        setup_key_rollover_env().await;

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

    let pin: Pin = "112234".into();
    wallet = do_wallet_registration(wallet, pin.clone()).await;

    // Obtain a handle to the wallet's config repository for direct in-memory updates
    let config_repo = Arc::clone(wallet.config_repository());

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

fn certificate_public_key(key: DerVerifyingKey) -> CertificatePublicKey {
    CertificatePublicKey {
        key,
        created_at: Utc::now().into(),
    }
}

/// Tests the full wallet certificate signing key rollover lifecycle with a real wallet and WP.
///
/// In practice, a rollover proceeds as follows:
/// 1. A new key is added to the wallet app's configuration and distributed to wallets.
/// 2. The wallet provider rolls over to the new key.
/// 3. The old key is removed from the wallet configuration, after when the rollover is complete.
///
/// Unlike the instruction result signing key, the wallet certificate signing key is only used (and
/// therefore only verified by the wallet) when a new wallet certificate is issued, i.e. during
/// registration and PIN change. This test therefore drives the PIN change flow to completion at each
/// stage, instead of merely unlocking the wallet.
///
/// The HSM must have both wallet certificate signing keys pre-provisioned:
/// - `wallet_certificate_signing_0` (kid = "0") — the current key
/// - `wallet_certificate_signing_1` (kid = "1") — the new key (added by setup-devenv.sh)
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_certificate_key_rollover() {
    // Don't drop `_db_setup` and `_hsm_setup`
    let (_db_setup, _hsm_setup, mock_device_config, ups_port, ups_root_ca, mut wp_settings, wp_root_ca, hsm) =
        setup_key_rollover_env().await;

    // Retrieve both certificate public keys from the HSM
    let cert_pubkey_0: DerVerifyingKey = hsm
        .get_verifying_key("wallet_certificate_signing_0")
        .await
        .unwrap()
        .into();
    let cert_pubkey_1: DerVerifyingKey = hsm
        .get_verifying_key("wallet_certificate_signing_1")
        .await
        .unwrap()
        .into();

    // Start the WP with the initial certificate signing key (kid = "0")
    let (wp_port, wp_abort) =
        start_wallet_provider_with_abort_handle(wp_settings.clone(), hsm.clone(), wp_root_ca.clone()).await;

    // Set up the wallet with the initial configuration and register it, so it holds a wallet
    // certificate signed with kid = "0".
    let (config_server_config, mut wallet_config) = build_wallet_environment_with_certificate_public_keys(
        ups_port,
        ups_root_ca.clone(),
        wp_port,
        HashMap::from([("0".to_string(), certificate_public_key(cert_pubkey_0.clone()))]),
    )
    .await;
    let mut wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;

    let pin_a: Pin = "112234".into();
    wallet = do_wallet_registration(wallet, pin_a.clone()).await;

    let config_repo = Arc::clone(wallet.config_repository());

    // Stage 1: Before rollover — WP uses old key, wallet has old config only. PIN change completes
    // fully, so the stored wallet certificate remains signed with kid = "0".
    wallet_config.account_server.certificate_public_keys =
        HashMap::from([("0".to_string(), certificate_public_key(cert_pubkey_0.clone()))]);
    config_repo.update_config(Arc::new(wallet_config.clone()));

    let pin_b: Pin = "223345".into();
    wallet
        .begin_change_pin(pin_a, pin_b.clone())
        .await
        .expect("stage 1: old key accepted by wallet with old config");
    wallet
        .continue_change_pin(&pin_b)
        .await
        .expect("stage 1: PIN change should complete");

    // Stage 2: New key added to wallet config, WP still uses old key.
    wallet_config.account_server.certificate_public_keys = HashMap::from([
        ("0".to_string(), certificate_public_key(cert_pubkey_0.clone())),
        ("1".to_string(), certificate_public_key(cert_pubkey_1.clone())),
    ]);
    config_repo.update_config(Arc::new(wallet_config.clone()));

    let pin_c: Pin = "334456".into();
    wallet
        .begin_change_pin(pin_b, pin_c.clone())
        .await
        .expect("stage 2: old key accepted by wallet with both keys configured");
    wallet
        .continue_change_pin(&pin_c)
        .await
        .expect("stage 2: PIN change should complete");

    // Failure: old key removed from wallet config before WP has rolled over (premature cleanup).
    // The stored certificate is still signed with kid = "0", so it can no longer be validated.
    wallet_config.account_server.certificate_public_keys =
        HashMap::from([("1".to_string(), certificate_public_key(cert_pubkey_1.clone()))]);
    config_repo.update_config(Arc::new(wallet_config.clone()));

    let pin_d: Pin = "445567".into();
    let error = wallet
        .begin_change_pin(pin_c.clone(), pin_d.clone())
        .await
        .expect_err("failure: stored certificate rejected by wallet configured with only the new key");
    assert!(matches!(error, ChangePinError::CertificateValidation(_)));

    // Restore both keys before the WP itself rolls over
    wallet_config.account_server.certificate_public_keys = HashMap::from([
        ("0".to_string(), certificate_public_key(cert_pubkey_0.clone())),
        ("1".to_string(), certificate_public_key(cert_pubkey_1.clone())),
    ]);
    config_repo.update_config(Arc::new(wallet_config.clone()));

    // Roll over: stop the old WP and start a new one with kid = "1". The wallet is still holding a
    // certificate signed with kid = "0" at this point, and will send it along with the instruction
    // that starts the PIN change below (i.e. before it receives its new, kid = "1" signed,
    // certificate). The new WP must therefore still accept kid = "0" via `previous_certificate_kids`.
    drop(wp_abort);
    wp_settings.current_certificate_kid = Kid::try_new("1".to_owned()).unwrap();
    wp_settings.previous_certificate_kids = Some(HashMap::from([(
        Kid::try_new("0".to_owned()).unwrap(),
        Utc::now() + chrono::Duration::hours(1),
    )]));
    let wp_port = start_wallet_provider(wp_settings, hsm.clone(), wp_root_ca.clone()).await;

    // Point the wallet at the new WP by updating the account server URL in the config
    wallet_config.account_server.http_config = TlsPinningConfig::try_new(
        local_wp_base_url(wp_port),
        VecNonEmpty::try_from(wallet_config.account_server.http_config.trust_anchors().to_vec()).unwrap(),
    )
    .unwrap();
    config_repo.update_config(Arc::new(wallet_config.clone()));

    // Stage 3: WP rolled over to new key, wallet config has both keys. The stored certificate is
    // still signed with kid = "0" (validated using the old key), while the new certificate that is
    // issued during this PIN change is signed with kid = "1" (validated using the new key).
    wallet
        .begin_change_pin(pin_c, pin_d.clone())
        .await
        .expect("stage 3: both old (stored certificate) and new (issued certificate) keys are needed");
    wallet
        .continue_change_pin(&pin_d)
        .await
        .expect("stage 3: PIN change should complete");

    // Stage 4: Old key removed from wallet config, WP uses new key. The stored certificate is now
    // signed with kid = "1", so only the new key is needed.
    wallet_config.account_server.certificate_public_keys =
        HashMap::from([("1".to_string(), certificate_public_key(cert_pubkey_1.clone()))]);
    config_repo.update_config(Arc::new(wallet_config.clone()));
    let pin_e: Pin = "556678".into();
    wallet
        .begin_change_pin(pin_d, pin_e.clone())
        .await
        .expect("stage 4: new key accepted by wallet with new config only");
    wallet
        .continue_change_pin(&pin_e)
        .await
        .expect("stage 4: PIN change should complete");

    // Failure case: wallet has not yet fetched the updated config when WP rolls over, so the stored
    // (kid = "1") wallet certificate cannot be validated.
    wallet_config.account_server.certificate_public_keys =
        HashMap::from([("0".to_string(), certificate_public_key(cert_pubkey_0))]);
    config_repo.update_config(Arc::new(wallet_config));
    let pin_f: Pin = "667789".into();
    let error = wallet
        .begin_change_pin(pin_e, pin_f)
        .await
        .expect_err("failure: stored certificate rejected by wallet with stale config");
    assert!(matches!(error, ChangePinError::CertificateValidation(_)));
}
