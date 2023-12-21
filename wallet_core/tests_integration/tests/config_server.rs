use wallet::{
    mock::default_configuration,
    wallet_deps::{ConfigurationRepository, HttpConfigurationRepository, UpdateableConfigurationRepository},
};

use crate::common::*;

pub mod common;

#[tokio::test]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_config() {
    let mut wallet_config = default_configuration();
    wallet_config.lock_timeouts.inactive_timeout = 1;
    wallet_config.lock_timeouts.background_timeout = 1;

    let settings = wallet_provider_settings();
    start_wallet_provider(settings.clone(), wallet_config).await;

    let mut wallet_config = default_configuration();
    wallet_config.account_server.base_url = local_wp_base_url(&settings.webserver.port);

    let http_config = HttpConfigurationRepository::new(local_config_base_url(&settings.webserver.port), wallet_config);

    let before = http_config.config();
    http_config.fetch().await.unwrap();
    let after = http_config.config();

    assert_ne!(before.lock_timeouts, after.lock_timeouts)
}
