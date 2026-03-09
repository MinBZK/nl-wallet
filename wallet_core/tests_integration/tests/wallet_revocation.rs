use std::iter;

use assert_matches::assert_matches;
use axum::http::StatusCode;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryOrder;
use serde_json::json;
use serial_test::serial;
use tempfile::TempDir;

use audit_log::entity;
use db_test::DbSetup;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::trusted_reqwest_client_builder;
use openid4vc::oidc::MockOidcClient;
use wallet::AccountRevokedData;
use wallet::BlockedReason;
use wallet::PidIssuancePurpose;
use wallet::RevocationReason;
use wallet::WalletState;
use wallet::errors::AccountProviderError;
use wallet::errors::AccountProviderResponseError;
use wallet::errors::InstructionError;
use wallet::errors::IssuanceError;
use wallet::errors::WalletRegistrationError;
use wallet::errors::WalletUnlockError;
use wallet_account::messages::errors::AccountError;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;
use wallet_provider_persistence::test::clear_flags_dropper;

use tests_integration::common::*;

/// Revoke a wallet via the wallet provider's internal endpoint and assert
/// that the wallet wipes itself (UserRequest revocation).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm, MockOidcClient)]
async fn test_revoke_wallet_by_revocation_code() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (config_server_config, mock_device_config, wallet_config, wp_port, wp_root_ca, _, audit_log_connection) =
        setup_revocation_env(&db_setup, false).await;

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

    assert_audit_log_entry(
        &audit_log_connection,
        "revoke_wallet_by_revocation_code",
        json!({"revocation_code": revocation_code}),
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
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm, MockOidcClient)]
async fn test_revoke_wallets_by_id() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (
        config_server_config,
        mock_device_config,
        wallet_config,
        wp_port,
        wp_root_ca,
        connection,
        audit_log_connection,
    ) = setup_revocation_env(&db_setup, false).await;

    let wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let wallet_ids = get_all_wallet_ids(&connection).await;
    let wallet_id = wallet_ids.first().expect("should have registered a wallet").to_string();

    call_wp_revocation_endpoint(wp_root_ca, wp_port, "/internal/revoke-wallets-by-id/", &[&wallet_id]).await;

    assert_audit_log_entry(
        &audit_log_connection,
        "revoke_wallets_by_wallet_id",
        json!({"wallet_ids": [wallet_id]}),
    )
    .await;

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
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm, MockOidcClient)]
async fn test_revoke_wallets_by_recovery_code() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (
        config_server_config,
        mock_device_config,
        wallet_config,
        wp_port,
        wp_root_ca,
        connection,
        audit_log_connection,
    ) = setup_revocation_env(&db_setup, false).await;

    let wallet = setup_in_memory_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let wallet_ids = get_all_wallet_ids(&connection).await;
    let wallet_id = wallet_ids.first().expect("should have registered a wallet").to_string();

    let recovery_code = get_wallet_recovery_code(&connection, &wallet_id).await;

    call_wp_revocation_endpoint(
        wp_root_ca,
        wp_port,
        "/internal/revoke-wallet-by-recovery-code/",
        &recovery_code,
    )
    .await;

    assert_audit_log_entry(
        &audit_log_connection,
        "revoke_wallets_by_recovery_code",
        json!({"recovery_code": recovery_code}),
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

    // TODO: remove `start_context` and `#[serial(MockOidcClient)]` when implementing ACF (PVW-5575)
    let ctx = MockOidcClient::start_context();
    ctx.expect().return_once(|_, _, _| Ok(mock_oidc_start_result()));

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

/// Revoke the wallet solution and assert that the wallet is blocked (AdminRequest revocation)
/// and no new account can be registered.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_revoke_wallet_solution() {
    let db_setup = DbSetup::create_clean().await;
    let _clear_flags = clear_flags_dropper(&db_setup);
    let pin = "112233";

    // Use a random recovery code to prevent breaking other tests that use a constant recovery code
    let (
        config_server_config,
        mock_device_config,
        wallet_config,
        wp_port,
        wp_root_ca,
        _connection,
        audit_log_connection,
    ) = setup_revocation_env(&db_setup, true).await;

    let wallet = setup_in_memory_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    call_revoke_solution_endpoint(wp_root_ca, wp_port, StatusCode::OK).await;

    assert_audit_log_entry(&audit_log_connection, "revoke_solution", json!({})).await;

    assert_wallet_revoked(
        &mut wallet,
        pin,
        true,
        AccountRevokedData {
            revocation_reason: RevocationReason::WalletSolutionCompromised,
            can_register_new_account: false,
        },
    )
    .await;

    // Try to register a new wallet and see it won't succeed
    let mut wallet = setup_in_memory_wallet(
        config_server_config,
        wallet_config,
        mock_device_config.apple_key_holder(),
    )
    .await;
    let err = wallet.register(pin).await.expect_err("Could still register");

    assert_matches!(
        err,
        WalletRegistrationError::ChallengeRequest(AccountProviderError::Response(
            AccountProviderResponseError::Account(
                AccountError::AccountRevoked(AccountRevokedData {
                    revocation_reason: RevocationReason::WalletSolutionCompromised,
                    can_register_new_account: false,
                }),
                _
            )
        ))
    );
}

/// Test if the wallet solution cannot be revoked if the option is not enabled
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_revoke_wallet_solution_not_enabled() {
    let db_setup = DbSetup::create_clean().await;
    let _clear_flags = clear_flags_dropper(&db_setup);
    let pin = "112233";

    // Use a random recovery code to prevent breaking other tests that use a constant recovery code
    let (
        config_server_config,
        mock_device_config,
        wallet_config,
        wp_port,
        wp_root_ca,
        _connection,
        audit_log_connection,
    ) = setup_revocation_env(&db_setup, false).await;

    let wallet = setup_in_memory_wallet(
        config_server_config.clone(),
        wallet_config.clone(),
        mock_device_config.apple_key_holder(),
    )
    .await;
    let wallet = do_wallet_registration(wallet, pin).await;
    let mut wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    call_revoke_solution_endpoint(wp_root_ca, wp_port, StatusCode::NOT_FOUND).await;

    // Check for no audit logs
    assert_eq!(
        entity::audit_log::Entity::find()
            .count(&audit_log_connection)
            .await
            .unwrap(),
        0
    );

    // Test if wallet still works
    assert!(wallet.check_pin(pin.to_string()).await.is_ok());
}

async fn setup_revocation_env(
    db_setup: &DbSetup,
    revoke_solution_enabled: bool,
) -> (
    ConfigServerConfiguration,
    MockDeviceConfig,
    WalletConfiguration,
    u16,
    ReqwestTrustAnchor,
    sea_orm::DatabaseConnection,
    sea_orm::DatabaseConnection,
) {
    let (mut wp_settings, wp_root_ca) =
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url());
    wp_settings.revoke_solution_enabled = revoke_solution_enabled;
    let connection = new_connection(wp_settings.database.url.clone()).await.unwrap();
    let audit_log_url = wp_settings.audit_log.url.clone();

    let (config_server_config, mock_device_config, wallet_config, _, _) = setup_env(
        static_server_settings(),
        update_policy_server_settings(),
        (wp_settings, wp_root_ca.clone()),
        verification_server_settings(db_setup.verification_server_url()),
        pid_issuer_settings(db_setup.pid_issuer_url()),
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await;

    let audit_log_connection = new_connection(audit_log_url).await.unwrap();

    let wp_port = wallet_config
        .account_server
        .http_config
        .base_url()
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
        audit_log_connection,
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

/// Revoke wallet solution by calling admin endpoint
///
/// Separate revoke solution endpoint to ensure both tests call the same.
async fn call_revoke_solution_endpoint(wp_root_ca: ReqwestTrustAnchor, wp_port: u16, status_code: StatusCode) {
    let client = trusted_reqwest_client_builder(iter::once(wp_root_ca.into_certificate()))
        .build()
        .unwrap();
    let response = client
        .post(format!("https://localhost:{wp_port}/internal/revoke-solution/"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), status_code);
}

async fn assert_audit_log_entry(
    connection: &sea_orm::DatabaseConnection,
    expected_operation: &str,
    expected_params: serde_json::Value,
) {
    let records = entity::audit_log::Entity::find()
        .order_by_asc(entity::audit_log::Column::Id)
        .all(connection)
        .await
        .unwrap();

    assert_eq!(records.len(), 2);

    let start_record = &records[0];
    assert_eq!(start_record.operation.as_deref(), Some(expected_operation));
    assert_eq!(start_record.params.as_ref(), Some(&expected_params));
    assert!(start_record.is_success.is_none());

    let result_record = &records[1];
    assert!(result_record.operation.is_none());
    assert!(result_record.params.is_none());
    assert_eq!(result_record.is_success, Some(true));
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

    assert_eq!(wallet.has_registration(), has_registration);

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
