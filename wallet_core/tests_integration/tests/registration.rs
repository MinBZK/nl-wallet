use tests_integration::common::*;

#[tokio::test]
async fn test_wallet_registration() {
    let settings_and_ca = wallet_provider_settings();
    let connection = database_connection(&settings_and_ca.0).await;

    let wallet = setup_wallet_and_env(config_server_settings(), settings_and_ca, wallet_server_settings()).await;

    let before = wallet_user_count(&connection).await;
    do_wallet_registration(wallet, String::from("123344")).await;
    let after = wallet_user_count(&connection).await;

    assert_eq!(before + 1, after);
}
