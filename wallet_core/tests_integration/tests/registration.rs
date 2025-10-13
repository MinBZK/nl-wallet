use rstest::rstest;
use serde_json::json;
use serial_test::serial;

use tests_integration::common::*;
use update_policy_server::config::UpdatePolicyConfig;
use wallet::errors::WalletRegistrationError;

#[tokio::test]
#[rstest]
#[serial(hsm)]
async fn test_wallet_registration(
    #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
) {
    let settings_and_ca = wallet_provider_settings();
    let connection = database_connection(&settings_and_ca.0).await;

    let (wallet, _, _) = setup_wallet_and_default_env(vendor).await;

    let before = wallet_user_count(&connection).await;
    do_wallet_registration(wallet, "123344").await;
    let after = wallet_user_count(&connection).await;

    assert_eq!(before + 1, after);
}

#[tokio::test]
#[serial(hsm)]
async fn test_registration_blocked() {
    let (mut settings, root_ca) = update_policy_server_settings();
    settings.update_policy =
        serde_json::from_value::<UpdatePolicyConfig>(json!({ env!("CARGO_PKG_VERSION"): "Block" })).unwrap();

    let (mut wallet, _, _) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        (settings, root_ca),
        wallet_provider_settings(),
        pid_issuer_settings(),
        issuance_server_settings(),
    )
    .await;

    let result = wallet.register("123344").await;
    assert!(wallet.is_blocked());

    assert!(matches!(result, Err(WalletRegistrationError::VersionBlocked)));
}
