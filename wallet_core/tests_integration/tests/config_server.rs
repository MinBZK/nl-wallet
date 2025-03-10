use std::env;

use assert_matches::assert_matches;
use jsonwebtoken::Algorithm;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePrivateKey;
use rand_core::OsRng;
use regex::Regex;
use reqwest::header::HeaderValue;
use tokio::fs;

use jwt::error::JwtError;
use tests_integration::common::*;
use wallet::errors::ConfigurationError;
use wallet::wallet_deps::default_config_server_config;
use wallet::wallet_deps::default_wallet_config;
use wallet::wallet_deps::HttpConfigurationRepository;
use wallet::wallet_deps::Repository;
use wallet::wallet_deps::RepositoryUpdateState;
use wallet::wallet_deps::UpdateableRepository;
use wallet_common::config::config_server_config::ConfigServerConfiguration;
use wallet_common::config::http::TlsPinningConfig;

#[tokio::test]
async fn test_wallet_config() {
    let mut served_wallet_config = default_wallet_config();
    served_wallet_config.lock_timeouts.inactive_timeout = 1;
    served_wallet_config.lock_timeouts.background_timeout = 1;
    served_wallet_config.version = 2;

    let (mut cs_settings, cs_root_ca) = config_server_settings();
    cs_settings.wallet_config_jwt = config_jwt(&served_wallet_config);
    let port = cs_settings.port;
    start_config_server(cs_settings, cs_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_config_base_url(&port),
            trust_anchors: vec![cs_root_ca],
        },
        ..default_config_server_config()
    };

    let storage_path = env::temp_dir();
    let etag_file = storage_path.join("wallet-config.etag");
    // make sure there are no storage files from previous test runs
    let _ = fs::remove_file(etag_file.as_path()).await;

    let http_config = HttpConfigurationRepository::new(
        config_server_config.signing_public_key.as_inner().into(),
        storage_path.clone(),
        default_wallet_config(),
    )
    .await
    .unwrap();

    let before = http_config.get();
    let result = http_config.fetch(&config_server_config.http_config).await.unwrap();
    let after = http_config.get();

    assert_matches!(result, RepositoryUpdateState::Updated { .. });
    assert_ne!(before.lock_timeouts, after.lock_timeouts);

    let content = fs::read(etag_file.as_path()).await.unwrap();
    let header_value = HeaderValue::from_bytes(&content).unwrap();

    let quoted_hash_regex = Regex::new(r#"".+""#).unwrap();
    assert!(quoted_hash_regex.is_match(header_value.to_str().unwrap()));

    // Second fetch should use earlier etag
    let result = http_config.fetch(&config_server_config.http_config).await.unwrap();
    assert_matches!(result, RepositoryUpdateState::Unmodified(_));
}

#[tokio::test]
async fn test_wallet_config_stale() {
    let (settings, _) = wallet_provider_settings();

    let mut served_wallet_config = default_wallet_config();
    served_wallet_config.account_server.http_config.base_url = local_wp_base_url(&settings.webserver.port);

    let (mut cs_settings, cs_root_ca) = config_server_settings();
    cs_settings.wallet_config_jwt = config_jwt(&served_wallet_config);
    let port = cs_settings.port;
    start_config_server(cs_settings, cs_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_config_base_url(&port),
            trust_anchors: vec![cs_root_ca],
        },
        ..default_config_server_config()
    };

    let http_config = HttpConfigurationRepository::new(
        config_server_config.signing_public_key.as_inner().into(),
        env::temp_dir(),
        default_wallet_config(),
    )
    .await
    .unwrap();

    let before = http_config.get();
    let result = http_config.fetch(&config_server_config.http_config).await.unwrap();
    let after = http_config.get();

    assert_matches!(result, RepositoryUpdateState::Unmodified(_));
    assert_eq!(before.version, after.version);
}

#[tokio::test]
async fn test_wallet_config_signature_verification_failed() {
    let (settings, _) = wallet_provider_settings();

    let mut served_wallet_config = default_wallet_config();
    served_wallet_config.account_server.http_config.base_url = local_wp_base_url(&settings.webserver.port);
    // set the wallet_config that will be return from the config server to a lower version number than
    // we already have in the default configuration
    served_wallet_config.version = 0;

    let (mut cs_settings, cs_root_ca) = config_server_settings();
    let signing_key = SigningKey::random(&mut OsRng);
    let pkcs8_der = signing_key.to_pkcs8_der().unwrap();
    let jwt = jsonwebtoken::encode(
        &Header {
            alg: Algorithm::ES256,
            ..Default::default()
        },
        &served_wallet_config,
        &EncodingKey::from_ec_der(pkcs8_der.as_bytes()),
    )
    .unwrap();
    // Serve a wallet configuration as JWT signed by a random key
    cs_settings.wallet_config_jwt = jwt;
    let port = cs_settings.port;
    start_config_server(cs_settings, cs_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_config_base_url(&port),
            trust_anchors: vec![cs_root_ca],
        },
        ..default_config_server_config()
    };

    let http_config = HttpConfigurationRepository::new(
        config_server_config.signing_public_key.as_inner().into(),
        env::temp_dir(),
        default_wallet_config(),
    )
    .await
    .unwrap();

    let result = http_config
        .fetch(&config_server_config.http_config)
        .await
        .expect_err("Expecting invalid signature error");

    assert_matches!(result, ConfigurationError::Jwt(JwtError::Validation(e))
        if *e.kind() == jsonwebtoken::errors::ErrorKind::InvalidSignature);
}
