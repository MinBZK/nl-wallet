use std::env;

use assert_matches::assert_matches;
use serde_json::json;

use tests_integration::common::*;
use tests_integration::utils::read_file;
use update_policy_server::config::UpdatePolicyConfig;
use wallet::wallet_deps::HttpUpdatePolicyRepository;
use wallet::wallet_deps::Repository;
use wallet::wallet_deps::RepositoryUpdateState;
use wallet::wallet_deps::UpdateableRepository;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::config::wallet_config::UpdatePolicyServerConfiguration;

#[tokio::test]
async fn test_wallet_update_policy() {
    let (mut ups_settings, root_ca) = update_policy_server_settings();
    ups_settings.update_policy =
        serde_json::from_value::<UpdatePolicyConfig>(json!({ env!("CARGO_PKG_VERSION"): "Block" })).unwrap();

    start_update_policy_server(ups_settings.clone(), root_ca).await;

    let root_ca = read_file("ups.ca.crt.der").try_into().unwrap();
    let update_policy_server_config = UpdatePolicyServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_ups_base_url(&ups_settings.port),
            trust_anchors: vec![root_ca],
        },
    };

    let update_policy = HttpUpdatePolicyRepository::new();

    let before = update_policy.get();
    let result = update_policy
        .fetch(&update_policy_server_config.http_config)
        .await
        .unwrap();
    let after = update_policy.get();

    assert_matches!(result, RepositoryUpdateState::Updated { .. });
    assert_ne!(before, after);
}

#[tokio::test]
async fn test_wallet_update_policy_stale() {
    let (ups_settings, root_ca) = update_policy_server_settings();
    start_update_policy_server(ups_settings.clone(), root_ca).await;

    let root_ca = read_file("ups.ca.crt.der").try_into().unwrap();
    let update_policy_server_config = UpdatePolicyServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_ups_base_url(&ups_settings.port),
            trust_anchors: vec![root_ca],
        },
    };

    let update_policy = HttpUpdatePolicyRepository::new();
    let before = update_policy.get();
    let result = update_policy
        .fetch(&update_policy_server_config.http_config)
        .await
        .unwrap();
    let after = update_policy.get();

    assert_matches!(result, RepositoryUpdateState::Unmodified(_));
    assert_eq!(before, after);
}

#[tokio::test]
async fn test_wallet_update_policy_server_tls_pinning() {
    let (ups_settings, root_ca) = update_policy_server_settings();
    start_update_policy_server(ups_settings.clone(), root_ca).await;

    // Use the wrong root CA
    let root_ca = read_file("cs.ca.crt.der").try_into().unwrap();
    let update_policy_server_config = UpdatePolicyServerConfiguration {
        http_config: TlsPinningConfig {
            base_url: local_ups_base_url(&ups_settings.port),
            trust_anchors: vec![root_ca],
        },
    };

    let update_policy = HttpUpdatePolicyRepository::new();
    let before = update_policy.get();
    let result = update_policy.fetch(&update_policy_server_config.http_config).await;
    let after = update_policy.get();

    assert_matches!(result, Err(_));
    assert_eq!(before, after);
}
