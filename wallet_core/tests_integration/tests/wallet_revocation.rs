use std::collections::HashSet;
use std::iter;

use assert_matches::assert_matches;
use serial_test::serial;
use tempfile::TempDir;

use crypto::utils::random_string;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::trusted_reqwest_client_builder;
use tests_integration::common::*;
use wallet::AccountRevokedData;
use wallet::BlockedReason;
use wallet::PidIssuancePurpose;
use wallet::RevocationReason;
use wallet::WalletState;
use wallet::errors::InstructionError;
use wallet::errors::IssuanceError;
use wallet::errors::WalletUnlockError;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

/// Revoke a wallet via the wallet provider's internal endpoint and assert
/// that the wallet wipes itself (UserRequest revocation).
#[tokio::test]
#[serial(hsm)]
async fn test_revoke_wallet_by_revocation_code() {
    let pin = "112233";

    let (config_server_config, mock_device_config, wallet_config, wp_port, wp_root_ca, _) =
        setup_revocation_env("123".to_string()).await;

    let dir = TempDir::new().unwrap();
    let wallet = setup_file_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
        dir.path().to_path_buf(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let revocation_code = wallet
        .get_revocation_code_with_pin(pin.to_string())
        .await
        .unwrap()
        .to_string();

    call_wp_revocation_endpoint(
        wp_root_ca,
        wp_port,
        "/internal/revoke-wallet-by-revocation-code/",
        &revocation_code,
    )
    .await;

    assert_wallet_revoked(
        &mut wallet,
        pin,
        false,
        AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true,
        },
    )
    .await;
}

/// Revoke a wallet via the wallet provider's internal endpoint and assert that
/// the wallet is blocked (AdminRequest revocation), not wiped.
#[tokio::test]
#[serial(hsm)]
async fn test_revoke_wallets_by_id() {
    let pin = "112233";

    let (config_server_config, mock_device_config, wallet_config, wp_port, wp_root_ca, connection) =
        setup_revocation_env("123".to_string()).await;

    let wallet_ids_before: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();

    let wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let wallet_ids_after: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();
    let wallet_id = wallet_ids_after
        .difference(&wallet_ids_before)
        .next()
        .expect("should have registered exactly one new wallet")
        .to_string();

    call_wp_revocation_endpoint(wp_root_ca, wp_port, "/internal/revoke-wallets-by-id/", &[&wallet_id]).await;

    assert_wallet_revoked(
        &mut wallet,
        pin,
        true,
        AccountRevokedData {
            revocation_reason: RevocationReason::AdminRequest,
            can_register_new_account: true,
        },
    )
    .await;
}

/// Revoke a wallet via the wallet provider's internal endpoint using a recovery code
/// and assert that the wallet is blocked (AdminRequest revocation) and cannot register
/// a new account.
#[tokio::test]
#[serial(hsm)]
async fn test_revoke_wallets_by_recovery_code() {
    let pin = "112233";

    // Use a random recovery code to prevent breaking other tests that use a constant recovery code
    let (config_server_config, mock_device_config, wallet_config, wp_port, wp_root_ca, connection) =
        setup_revocation_env(random_string(32)).await;

    let wallet_ids_before: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();

    let wallet = setup_in_memory_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let wallet_ids_after: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();
    let wallet_id = wallet_ids_after
        .difference(&wallet_ids_before)
        .next()
        .expect("should have registered exactly one new wallet")
        .to_string();

    let recovery_code = get_wallet_recovery_code(&connection, &wallet_id).await;
    call_wp_revocation_endpoint(
        wp_root_ca,
        wp_port,
        "/internal/revoke-wallet-by-recovery-code/",
        &recovery_code,
    )
    .await;

    assert_wallet_revoked(
        &mut wallet,
        pin,
        true,
        AccountRevokedData {
            revocation_reason: RevocationReason::AdminRequest,
            can_register_new_account: false,
        },
    )
    .await;

    // Try to setup a new wallet; this will use the same recovery code so the WP will
    // inform it that it is revoked during PID issuance.
    let mut wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    let redirect_url = wallet
        .create_pid_issuance_auth_url(PidIssuancePurpose::Enrollment)
        .await
        .expect("Could not create pid issuance auth url");
    let _attestations = wallet
        .continue_pid_issuance(redirect_url)
        .await
        .expect("Could not continue pid issuance");
    let err = wallet
        .accept_issuance(pin.to_string())
        .await
        .expect_err("PID issuance of a revoked wallet should not have succeeded");

    assert_matches!(
        err,
        IssuanceError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::AdminRequest,
            can_register_new_account: false
        }))
    );
}

async fn setup_revocation_env(
    recovery_code: String,
) -> (
    ConfigServerConfiguration,
    MockDeviceConfig,
    WalletConfiguration,
    u16,
    ReqwestTrustAnchor,
    sea_orm::DatabaseConnection,
) {
    let (wp_settings, wp_root_ca) = wallet_provider_settings();
    let connection = new_connection(wp_settings.database.url.clone()).await.unwrap();

    let (config_server_config, mock_device_config, wallet_config, _, _) = setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        (wp_settings, wp_root_ca.clone()),
        verification_server_settings(),
        pid_issuer_settings(recovery_code),
        issuance_server_settings(),
    )
    .await;

    let wp_port = wallet_config
        .account_server
        .http_config
        .base_url
        .as_ref()
        .port()
        .unwrap();

    (
        config_server_config,
        mock_device_config,
        wallet_config,
        wp_port,
        wp_root_ca,
        connection,
    )
}

async fn call_wp_revocation_endpoint(
    wp_root_ca: ReqwestTrustAnchor,
    wp_port: u16,
    path: &str,
    body: impl serde::Serialize,
) {
    let client = trusted_reqwest_client_builder(iter::once(wp_root_ca.into_certificate()))
        .build()
        .unwrap();
    let response = client
        .post(format!("https://localhost:{wp_port}{path}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
}

async fn assert_wallet_revoked(
    wallet: &mut WalletWithStorage,
    pin: &str,
    has_registration: bool,
    revocation_data: AccountRevokedData,
) {
    // Checking the PIN sends a CheckPin instruction; the wallet provider informs us we are revoked.
    assert_matches!(
        wallet.check_pin(pin.to_string()).await,
        Err(WalletUnlockError::Instruction(InstructionError::AccountRevoked(actual))) if actual == revocation_data
    );

    assert!(wallet.has_registration() == has_registration);

    if has_registration {
        assert_eq!(
            wallet.get_state().await.unwrap(),
            WalletState::Blocked {
                reason: BlockedReason::BlockedByWalletProvider,
                can_register_new_account: revocation_data.can_register_new_account
                    && revocation_data.revocation_reason == RevocationReason::AdminRequest,
            }
        );
    }
}
