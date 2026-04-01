use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use serial_test::serial;

use crypto::p256_der::DerVerifyingKey;
use db_test::DbSetup;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
use server_utils::keys::SecretKeyVariant;
use server_utils::settings::SecretKey;
use tests_integration::common::*;
use tests_integration::fake_digid::fake_digid_auth;
use wallet::test::default_wallet_config;
use wscd::mock_remote::MockRemoteWscd;

/// Test the full PID issuance flow, i.e. including OIDC with nl-rdo-max and retrieving the PID from BRP
/// (Haal-Centraal). This test depends on part of the internal API of the DigiD bridge, so it may break when nl-rdo-max
/// is updated.
///
/// Before running this, ensure that you have nl-rdo-max and brpproxy properly configured and running locally:
/// - Run `setup-devenv.sh` if not recently done,
/// - Run `start-devenv.sh digid brpproxy`, or else `docker compose up` in your nl-rdo-max checkout, and `docker compose
///   up brpproxy` in /scripts.
///
/// Run the test itself with `cargo test --package tests_integration --features=digid_test`
///
/// See also
/// - `test_pid_ok()`, which uses the WP but mocks the OIDC part,
/// - `accept_issuance()` in the `openid4vc` integration tests, which also mocks the HTTP server and client.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc1_test_pid_issuance_digid_bridge() {
    let db_setup = DbSetup::create_clean().await;
    let (mut settings, _) = pid_issuer_settings(db_setup.pid_issuer_url());

    // Generate a WUA key pair so MockRemoteWscd can sign WUAs accepted by the test pid issuer.
    let wua_signing_key = SigningKey::random(&mut OsRng);
    settings.wua_issuer_pubkey = DerVerifyingKey::from(*wua_signing_key.verifying_key());

    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();

    let issuer_url = start_pid_issuer_server(
        settings.clone(),
        hsm,
        BrpPidAttributeService::try_new(
            HttpBrpClient::new(settings.brp_server.clone()),
            &settings.digid.bsn_privkey,
            settings.digid.client_id.clone(),
            &settings.digid.http_config,
            SecretKeyVariant::from_settings(
                SecretKey::Software {
                    secret_key: (0..32).collect::<Vec<_>>().try_into().unwrap(),
                },
                None,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .await;

    start_gba_hc_converter(gba_hc_converter_settings()).await;

    let wallet_config = default_wallet_config();

    // Discover the credential issuer and start authorization code flow
    let http_client = HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap();
    let credential_issuer_discovery = HttpIssuanceDiscovery::new(http_client);
    let redirect_uri = urls::issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap())
        .as_ref()
        .clone();

    let authorization_session = credential_issuer_discovery
        .start_authorization_code_flow(
            &issuer_url.public,
            wallet_config.pid_issuance.client_id.clone(),
            redirect_uri,
        )
        .await
        .unwrap();

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(
        authorization_session.auth_url().clone(),
        settings.digid.http_config.base_url().clone(),
        "999991772",
    )
    .await;

    // Start issuance by exchanging the authorization code for the attestation previews
    let issuance_session = authorization_session
        .start_issuance(&redirect_url, &wallet_config.issuer_trust_anchors())
        .await
        .unwrap();

    let wscd = MockRemoteWscd::new_with_wua_signing_key(wua_signing_key);
    let credential_with_metadata = issuance_session
        .accept_issuance(&wallet_config.issuer_trust_anchors(), &wscd, true)
        .await
        .unwrap();

    assert_eq!(1, credential_with_metadata.len());
    assert_eq!(8, credential_with_metadata[0].copies.as_ref().as_slice().len());
}
