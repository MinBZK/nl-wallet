use std::assert_matches;

use crypto::kid::VerifyingKeyWithKid;
use http_utils::client::TlsPinningConfig;
use http_utils::reqwest::default_reqwest_client_builder;
use jwt::SignedJwt;
use jwt::error::JwtVerifyError;
use regex::regex;
use reqwest::StatusCode;
use reqwest::header;
use reqwest::header::HeaderValue;
use tests_integration::common::*;
use tokio::fs;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use wallet::errors::ConfigurationError;
use wallet::errors::HttpClientError;
use wallet::test::ConfigSigningKey;
use wallet::test::HttpConfigurationRepository;
use wallet::test::Repository;
use wallet::test::RepositoryUpdateState;
use wallet::test::UpdateableRepository;
use wallet::test::default_config_server_config;
use wallet::test::default_wallet_config;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_provider::settings::Settings as WpSettings;

#[tokio::test]
async fn test_wrprc_status_list() {
    let (mut static_settings, _) = static_server_settings();
    let expected = fs::read_to_string(static_settings.wrprc_publish_dir.as_ref().join("1.jwt"))
        .await
        .unwrap();
    static_settings.crl_port = 0;
    let port = start_static_crl_server(&static_settings).await;

    let response = default_reqwest_client_builder()
        .build()
        .unwrap()
        .get(local_http_base_url(port).join("wrprc/1"))
        .header(header::ACCEPT, "application/statuslist+jwt")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/statuslist+jwt"
    );
    assert_eq!(response.text().await.unwrap(), expected);
}

#[tokio::test]
async fn test_wallet_config() {
    let mut served_wallet_config = default_wallet_config();
    served_wallet_config.lock_timeouts.warning_timeout = 1;
    served_wallet_config.lock_timeouts.inactive_timeout = 1;
    served_wallet_config.lock_timeouts.background_timeout = 1;
    served_wallet_config.version = 2;

    let (mut static_settings, static_root_ca) = static_server_settings();
    static_settings.wallet_config_jwt = config_jwt(&served_wallet_config).await.into();
    let config_url = start_static_server(static_settings, static_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig::try_new(config_url, vec_nonempty![static_root_ca]).unwrap(),
        ..default_config_server_config()
    };

    let storage_dir = tempfile::tempdir().unwrap();
    let etag_path = storage_dir.path().join("config.etag");
    let http_config = HttpConfigurationRepository::new(
        VerifyingKeyWithKid {
            kid: config_server_config.signing_key_id,
            public_key: config_server_config.signing_public_key,
        },
        etag_path.clone(),
        default_wallet_config(),
    );

    let before = http_config.get();
    let result = http_config.fetch(&config_server_config.http_config).await.unwrap();
    let after = http_config.get();

    assert_matches!(result, RepositoryUpdateState::Updated { .. });
    assert_ne!(before.lock_timeouts, after.lock_timeouts);

    let content = fs::read(&etag_path).await.unwrap();
    let header_value = HeaderValue::from_bytes(&content).unwrap();

    let quoted_hash_regex = regex!(r#"".+""#);
    assert!(quoted_hash_regex.is_match(header_value.to_str().unwrap()));

    // Second fetch should use earlier etag
    let result = http_config.fetch(&config_server_config.http_config).await.unwrap();
    assert_matches!(result, RepositoryUpdateState::Unmodified(_));
}

#[tokio::test]
async fn test_wallet_config_stale() {
    let served_wallet_config = default_wallet_config();

    let (mut static_settings, static_root_ca) = static_server_settings();
    static_settings.wallet_config_jwt = config_jwt(&served_wallet_config).await.into();
    let config_url = start_static_server(static_settings, static_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig::try_new(config_url, vec_nonempty![static_root_ca]).unwrap(),
        ..default_config_server_config()
    };

    let storage_dir = tempfile::tempdir().unwrap();
    let etag_path = storage_dir.path().join("config.etag");
    let http_config = HttpConfigurationRepository::new(
        VerifyingKeyWithKid {
            kid: config_server_config.signing_key_id,
            public_key: config_server_config.signing_public_key,
        },
        etag_path,
        default_wallet_config(),
    );

    let before = http_config.get();
    let result = http_config.fetch(&config_server_config.http_config).await.unwrap();
    let after = http_config.get();

    assert_matches!(result, RepositoryUpdateState::Unmodified(_));
    assert_eq!(before.version, after.version);
}

#[tokio::test]
async fn test_wallet_config_incorrect_kid() {
    let served_wallet_config = default_wallet_config();

    let (mut static_settings, static_root_ca) = static_server_settings();
    static_settings.wallet_config_jwt = config_jwt(&served_wallet_config).await.into();
    let config_url = start_static_server(static_settings, static_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig::try_new(config_url, vec_nonempty![static_root_ca]).unwrap(),
        ..default_config_server_config()
    };

    let storage_dir = tempfile::tempdir().unwrap();
    let etag_path = storage_dir.path().join("config.etag");
    let http_config = HttpConfigurationRepository::new(
        VerifyingKeyWithKid {
            kid: "different".try_into().unwrap(),
            public_key: config_server_config.signing_public_key,
        },
        etag_path,
        default_wallet_config(),
    );

    let result = http_config
        .fetch(&config_server_config.http_config)
        .await
        .expect_err("Expecting http client error");

    assert_matches!(result, ConfigurationError::HttpClient(HttpClientError::Response(e, _))
        if e.status() == Some(StatusCode::NOT_FOUND));
}

#[tokio::test]
async fn test_wallet_config_signature_verification_failed() {
    let settings = WpSettings::new().expect("Could not read settings");

    let mut served_wallet_config = default_wallet_config();
    served_wallet_config.account_server.http_config = TlsPinningConfig::try_new(
        local_wp_base_url(settings.webserver.port),
        VecNonEmpty::try_from(served_wallet_config.account_server.http_config.trust_anchors().to_vec()).unwrap(),
    )
    .unwrap();
    // set the wallet_config that will be return from the config server to a lower version number than
    // we already have in the default configuration
    served_wallet_config.version = 0;

    let (mut static_settings, static_root_ca) = static_server_settings();
    let signing_key = ConfigSigningKey::new_with_kid("0");
    let jwt = SignedJwt::sign_with_kid(&served_wallet_config, &signing_key)
        .await
        .unwrap();
    // Serve a wallet configuration as JWT signed by a random key
    static_settings.wallet_config_jwt = jwt.into();
    let config_url = start_static_server(static_settings, static_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig::try_new(config_url, vec_nonempty![static_root_ca]).unwrap(),
        ..default_config_server_config()
    };

    let storage_dir = tempfile::tempdir().unwrap();
    let etag_path = storage_dir.path().join("config.etag");
    let http_config = HttpConfigurationRepository::new(
        VerifyingKeyWithKid {
            kid: config_server_config.signing_key_id,
            public_key: config_server_config.signing_public_key,
        },
        etag_path,
        default_wallet_config(),
    );

    let result = http_config
        .fetch(&config_server_config.http_config)
        .await
        .expect_err("Expecting invalid signature error");

    assert_matches!(result, ConfigurationError::JwtVerify(JwtVerifyError::Validation(e))
        if *e.kind() == jsonwebtoken::errors::ErrorKind::InvalidSignature);
}

/// A wallet configuration that the wallet rejects should not cause its `ETag` to be stored. Otherwise, the wallet keeps
/// receiving `304 Not Modified` for that configuration and never gets to verify it again.
#[tokio::test]
async fn test_wallet_config_rejected_etag_not_stored() {
    let mut served_wallet_config = default_wallet_config();
    served_wallet_config.version = default_wallet_config().version + 1;

    // Sign the served configuration with a key the current app does not know, so that it gets rejected.
    let signing_key = ConfigSigningKey::new_with_kid("0");
    let (mut static_settings, static_root_ca) = static_server_settings();
    static_settings.wallet_config_jwt = SignedJwt::sign_with_kid(&served_wallet_config, &signing_key)
        .await
        .unwrap()
        .into();
    let config_url = start_static_server(static_settings, static_root_ca.clone()).await;

    let config_server_config = ConfigServerConfiguration {
        http_config: TlsPinningConfig::try_new(config_url, vec_nonempty![static_root_ca]).unwrap(),
        ..default_config_server_config()
    };

    let storage_dir = tempfile::tempdir().unwrap();
    let etag_path = storage_dir.path().join("config.etag");
    let rejecting_http_config = HttpConfigurationRepository::new(
        VerifyingKeyWithKid {
            kid: config_server_config.signing_key_id.clone(),
            public_key: config_server_config.signing_public_key,
        },
        etag_path.clone(),
        default_wallet_config(),
    );

    rejecting_http_config
        .fetch(&config_server_config.http_config)
        .await
        .expect_err("fetching a wallet configuration signed by an unknown key should fail");

    // Fetching again should result in the same configuration being received and rejected again, instead of the
    // wallet being told that nothing changed.
    let result = rejecting_http_config.fetch(&config_server_config.http_config).await;
    assert_matches!(result, Err(ConfigurationError::JwtVerify(_)));
    assert_eq!(rejecting_http_config.get().version, default_wallet_config().version);

    // An app update that accepts the served configuration starts out with the same storage, and should therefore
    // receive and apply that configuration.
    let accepting_http_config = HttpConfigurationRepository::new(
        VerifyingKeyWithKid {
            kid: config_server_config.signing_key_id,
            public_key: (*signing_key.key.verifying_key()).into(),
        },
        etag_path.clone(),
        default_wallet_config(),
    );

    let result = accepting_http_config
        .fetch(&config_server_config.http_config)
        .await
        .unwrap();
    assert_matches!(result, RepositoryUpdateState::Updated { .. });
    assert_eq!(accepting_http_config.get().version, served_wallet_config.version);
}
