use rstest::rstest;
use sea_orm::Database;
use serde_json::json;
use serial_test::serial;

use db_test::DbSetup;
use tests_integration::common::*;
use update_policy_server::config::UpdatePolicyConfig;
use wallet::errors::WalletRegistrationError;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
#[serial(hsm)]
async fn ltc51_test_wallet_registration(
    #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
) {
    let db_setup = DbSetup::create().await;
    let (wallet, _, _) = setup_wallet_and_default_env(&db_setup, vendor).await;

    let connection = Database::connect(db_setup.wallet_provider_url())
        .await
        .expect("Could not open database connection");

    let before = wallet_user_count(&connection).await;
    do_wallet_registration(wallet, "123344").await;
    let after = wallet_user_count(&connection).await;

    assert_eq!(before + 1, after);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc43_test_registration_blocked() {
    let db_setup = DbSetup::create().await;

    let (mut settings, root_ca) = update_policy_server_settings();
    settings.update_policy =
        serde_json::from_value::<UpdatePolicyConfig>(json!({ env!("CARGO_PKG_VERSION"): "Block" })).unwrap();

    let (mut wallet, _, _) = setup_wallet_and_env(
        &db_setup,
        WalletDeviceVendor::Apple,
        (settings, root_ca),
        wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url()),
        pid_issuer_settings(db_setup.pid_issuer_url(), "123".to_string()),
        issuance_server_settings(db_setup.issuance_server_url()),
    )
    .await;

    let result = wallet.register("123344").await;
    assert!(wallet.is_blocked());

    assert!(matches!(result, Err(WalletRegistrationError::VersionBlocked)));
}
