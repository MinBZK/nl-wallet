use std::env;

use assert_matches::assert_matches;
use regex::Regex;
use reqwest::header::HeaderValue;
use tokio::fs;

use wallet::{
    mock::default_configuration,
    wallet_deps::{
        ConfigurationRepository, ConfigurationUpdateState, HttpConfigurationRepository,
        UpdateableConfigurationRepository,
    },
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

    let storage_path = env::temp_dir();
    let etag_file = storage_path.join("latest-configuration-etag.txt");
    // make sure there are no storage files from previous test runs
    let _ = fs::remove_file(etag_file.as_path()).await;

    let http_config = HttpConfigurationRepository::new(
        local_config_base_url(&settings.webserver.port),
        storage_path.clone(),
        wallet_config,
    )
    .await
    .unwrap();

    let before = http_config.config();
    let result = http_config.fetch().await.unwrap();
    let after = http_config.config();

    assert_matches!(result, ConfigurationUpdateState::Updated);
    assert_ne!(before.lock_timeouts, after.lock_timeouts);

    let content = fs::read(etag_file.as_path()).await.unwrap();
    let header_value = HeaderValue::from_bytes(&content).unwrap();

    let quoted_hash_regex = Regex::new(r#""\d+""#).unwrap();
    assert!(quoted_hash_regex.is_match(header_value.to_str().unwrap()));

    // Second fetch should use earlier etag
    let result = http_config.fetch().await.unwrap();
    assert_matches!(result, ConfigurationUpdateState::Unmodified);
}
