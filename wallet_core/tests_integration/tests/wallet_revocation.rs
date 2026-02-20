use std::collections::HashSet;
use std::iter;

use assert_matches::assert_matches;
use serial_test::serial;

use crypto::utils::random_string;
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

/// Revoke a wallet via the wallet provider's internal endpoint and assert
/// that the wallet wipes itself (UserRequest revocation).
#[tokio::test]
#[serial(hsm)]
async fn test_revoke_wallet_by_revocation_code() {
    let pin = "112233";

    let (wp_settings, wp_root_ca) = wallet_provider_settings();

    let (config_server_config, mock_device_config, wallet_config, _, _) = setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        (wp_settings, wp_root_ca.clone()),
        verification_server_settings(),
        pid_issuer_settings("123".to_string()),
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

    let mut wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let revocation_code = wallet
        .get_revocation_code_with_pin(pin.to_string())
        .await
        .unwrap()
        .to_string();

    // Revoke the wallet via the internal endpoint.
    let client = trusted_reqwest_client_builder(iter::once(wp_root_ca.into_certificate()))
        .build()
        .unwrap();
    let response = client
        .post(format!(
            "https://localhost:{wp_port}/internal/revoke-wallet-by-revocation-code/"
        ))
        .json(&revocation_code)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    // Checking the PIN sends a CheckPin instruction; the wallet provider informs us we are blocked.
    assert_matches!(
        wallet.check_pin(pin.to_string()).await,
        Err(WalletUnlockError::Instruction(InstructionError::AccountRevoked(
            AccountRevokedData {
                revocation_reason: RevocationReason::UserRequest,
                ..
            }
        )))
    );

    assert!(
        !wallet.has_registration(),
        "wallet should be wiped after UserRequest revocation"
    );
}

/// Revoke a wallet via the wallet provider's internal endpoint and assert that
/// the wallet is blocked (AdminRequest revocation), not wiped.
#[tokio::test]
#[serial(hsm)]
async fn test_revoke_wallets_by_id() {
    let pin = "112233";

    let (wp_settings, wp_root_ca) = wallet_provider_settings();
    let connection = new_connection(wp_settings.database.url.clone()).await.unwrap();

    let (config_server_config, mock_device_config, wallet_config, _, _) = setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        (wp_settings, wp_root_ca.clone()),
        verification_server_settings(),
        pid_issuer_settings("123".to_string()),
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

    let wallet_ids_before: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();

    let mut wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let wallet_ids_after: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();
    let wallet_id = wallet_ids_after
        .difference(&wallet_ids_before)
        .next()
        .expect("should have registered exactly one new wallet")
        .to_string();

    // Revoke the wallet by its ID via the internal endpoint.
    let client = trusted_reqwest_client_builder(std::iter::once(wp_root_ca.into_certificate()))
        .build()
        .unwrap();
    let response = client
        .post(format!("https://localhost:{wp_port}/internal/revoke-wallets-by-id/"))
        .json(&[&wallet_id])
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    // Checking the PIN sends a CheckPin instruction; the wallet provider informs us we are blocked.
    assert_matches!(
        wallet.check_pin(pin.to_string()).await,
        Err(WalletUnlockError::Instruction(InstructionError::AccountRevoked(
            AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: true
            }
        )))
    );

    assert!(
        wallet.has_registration(),
        "wallet should not be wiped after AdminRequest revocation"
    );
    assert_matches!(
        wallet.get_state().await.unwrap(),
        WalletState::Blocked {
            reason: BlockedReason::BlockedByWalletProvider
        }
    );
}

/// Revoke a wallet via the wallet provider's internal endpoint using a PIN recovery code
/// and assert that the wallet is blocked (AdminRequest revocation) and cannot register
/// a new account.
#[tokio::test]
#[serial(hsm)]
async fn test_revoke_wallets_by_recovery_code() {
    let pin = "112233";

    let (wp_settings, wp_root_ca) = wallet_provider_settings();
    let connection = new_connection(wp_settings.database.url.clone()).await.unwrap();

    let (config_server_config, mock_device_config, wallet_config, _, _) = setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        (wp_settings, wp_root_ca.clone()),
        verification_server_settings(),
        pid_issuer_settings(random_string(32)), // Prevent breaking other tests that use a constant revocation code
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

    let wallet_ids_before: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();

    let mut wallet = setup_in_memory_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let wallet_ids_after: HashSet<String> = get_all_wallet_ids(&connection).await.into_iter().collect();
    let wallet_id = wallet_ids_after
        .difference(&wallet_ids_before)
        .next()
        .expect("should have registered exactly one new wallet")
        .to_string();

    let recovery_code = get_wallet_recovery_code(&connection, &wallet_id).await;

    // Revoke the wallet by its recovery code via the internal endpoint.
    let client = trusted_reqwest_client_builder(iter::once(wp_root_ca.into_certificate()))
        .build()
        .unwrap();
    let response = client
        .post(format!(
            "https://localhost:{wp_port}/internal/revoke-wallet-by-recovery-code/"
        ))
        .json(&recovery_code)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    // Checking the PIN sends a CheckPin instruction; the wallet provider informs us we are blocked.
    assert_matches!(
        wallet.check_pin(pin.to_string()).await,
        Err(WalletUnlockError::Instruction(InstructionError::AccountRevoked(
            AccountRevokedData {
                revocation_reason: RevocationReason::AdminRequest,
                can_register_new_account: false,
            }
        )))
    );

    assert!(
        wallet.has_registration(),
        "wallet should not be wiped after AdminRequest revocation"
    );
    assert_matches!(
        wallet.get_state().await.unwrap(),
        WalletState::Blocked {
            reason: BlockedReason::BlockedByWalletProvider
        }
    );

    // Try to setup a new wallet; this will use the same recovery code so the WP will
    // immediately revoke it during PID issuance
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
